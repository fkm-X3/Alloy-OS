pub mod vnode;
pub mod tmpfs;
pub mod fat32;
pub mod mount;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use crate::sync::SpinLock;
use crate::utils::{copy_from_user, copy_to_user};
use mount::{MountTable, FsType};
use crate::block::BlockDevice;

struct FsState {
    next_id: u64,
    path_to_id: BTreeMap<String, u64>,
    data: BTreeMap<u64, Vec<u8>>,
    mount_table: MountTable,
    block_devices: Vec<Option<Box<dyn BlockDevice>>>,
    fat32_filesystems: BTreeMap<u64, fat32::Fat32Fs>,
}

impl FsState {
    fn new() -> Self {
        FsState {
            next_id: 1,
            path_to_id: BTreeMap::new(),
            data: BTreeMap::new(),
            mount_table: MountTable::new(),
            block_devices: Vec::new(),
            fat32_filesystems: BTreeMap::new(),
        }
    }

    fn register_block_device(&mut self, dev: Box<dyn BlockDevice>) -> usize {
        let id = self.block_devices.len();
        let ns = dev.num_sectors();
        self.block_devices.push(Some(dev));
        unsafe {
            crate::ffi::serial_print(c"[VFS] Block device ".as_ptr() as *const u8);
            crate::ffi::serial_print_hex(id as u32);
            crate::ffi::serial_print(c": ".as_ptr() as *const u8);
            crate::ffi::serial_print_hex(ns as u32);
            crate::ffi::serial_print(c" sectors\n".as_ptr() as *const u8);
        }
        id
    }

    fn mount_fat32(&mut self, dev_id: usize, mount_path: &str) -> Result<(), ()> {
        let dev = self.block_devices[dev_id].as_mut().ok_or(())?;
        let mut fs = fat32::Fat32Fs::new(dev_id, dev.as_mut())?;
        let key = 1000 + dev_id as u64;

        if let Ok(root_entries) = fs.root_entries(dev.as_mut()) {
            unsafe {
                crate::ffi::serial_print(c"[FAT32] Mounting at ".as_ptr() as *const u8);
                crate::ffi::serial_print(mount_path.as_ptr());
                crate::ffi::serial_print(c"\n".as_ptr() as *const u8);
                for entry in &root_entries {
                    let name_str = core::str::from_utf8(&entry.name[..entry.name_len]).unwrap_or("?");
                    crate::ffi::serial_print(c"  ".as_ptr() as *const u8);
                    if entry.is_dir { crate::ffi::serial_print(c"[DIR]  ".as_ptr() as *const u8); }
                    else { crate::ffi::serial_print(c"[FILE] ".as_ptr() as *const u8); }
                    crate::ffi::serial_print(name_str.as_ptr());
                    crate::ffi::serial_print(c"\n".as_ptr() as *const u8);
                }
            }
        }

        self.fat32_filesystems.insert(key, fs);
        self.mount_table.mount(mount_path, FsType::Fat32, Some(dev_id)).map_err(|_| ())?;

        let vnode_id = self.next_id;
        self.next_id += 1;
        self.path_to_id.insert(normalize_path(mount_path), vnode_id);
        self.data.insert(vnode_id, Vec::new());

        Ok(())
    }
}

static VFS_STATE: SpinLock<Option<FsState>> = SpinLock::new(None);

fn normalize_path(path: &str) -> String {
    let mut out = String::new();
    let mut prev_slash = false;
    for b in path.as_bytes() {
        if *b == b'/' {
            if !prev_slash {
                out.push('/');
                prev_slash = true;
            }
        } else {
            out.push(*b as char);
            prev_slash = false;
        }
    }
    if out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    if out.is_empty() { out.push('/') }
    out
}

pub fn vfs_init() {
    {
        let mut guard = VFS_STATE.lock();
        *guard = Some(FsState::new());
    }

    if let Ok(_id) = vfs_open("/dev/console", 0, 0) {
        unsafe { crate::ffi::serial_print(c"[VFS] /dev/console created\n".as_ptr() as *const u8); }
    }

    let hello_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../hello"));
    if !hello_bytes.is_empty() {
        if let Ok(id) = vfs_open("/hello", 0, 0) {
            let mut g = VFS_STATE.lock();
            if let Some(state) = g.as_mut() {
                state.data.insert(id, hello_bytes.to_vec());
                unsafe { crate::ffi::serial_print(c"[VFS] /hello embedded into VFS\n".as_ptr() as *const u8); }
            }
        }
        if let Ok(id2) = vfs_open("/bin/hello", 0, 0) {
            let mut g2 = VFS_STATE.lock();
            if let Some(state2) = g2.as_mut() {
                state2.data.insert(id2, hello_bytes.to_vec());
            }
        }
    }

    unsafe { crate::ffi::serial_print(c"[VFS] Initializing block devices...\n".as_ptr() as *const u8); }

    let devices = crate::block::init_block_devices();
    let mut guard = VFS_STATE.lock();
    if let Some(state) = guard.as_mut() {
        for dev in devices {
            state.register_block_device(dev);
        }
    }
}

pub fn vfs_open(path: &str, _flags: u32, _mode: u32) -> Result<u64, i32> {
    let norm = normalize_path(path);
    let mut guard = VFS_STATE.lock();
    let state = guard.as_mut().ok_or(-1)?;

    if let Some(&id) = state.path_to_id.get(&norm) {
        return Ok(id);
    }

    let id = state.next_id;
    state.next_id += 1;
    state.path_to_id.insert(norm.clone(), id);
    state.data.insert(id, Vec::new());
    Ok(id)
}

pub fn vfs_read_all(vnode_id: u64) -> Option<Vec<u8>> {
    let guard = VFS_STATE.lock();
    let state = guard.as_ref()?;
    state.data.get(&vnode_id).cloned()
}

pub fn vfs_read(vnode_id: u64, offset: &mut usize, user_buf_ptr: u32, len: usize) -> isize {
    let guard = VFS_STATE.lock();
    let state = match guard.as_ref() {
        Some(s) => s,
        None => return -1,
    };
    if let Some(vec) = state.data.get(&vnode_id) {
        if *offset >= vec.len() { return 0; }
        let available = vec.len() - *offset;
        let to_copy = core::cmp::min(available, len);
        unsafe {
            if copy_to_user(user_buf_ptr, &vec[*offset..(*offset+to_copy)]).is_ok() {
                *offset += to_copy;
                return to_copy as isize;
            }
        }
    }
    -1
}

pub fn vfs_write(vnode_id: u64, offset: &mut usize, user_buf_ptr: u32, len: usize) -> isize {
    let mut guard = VFS_STATE.lock();
    let state = match guard.as_mut() {
        Some(s) => s,
        None => return -1,
    };
    if let Some(typ) = state.path_to_id.iter().find_map(|(p,&id)| if id==vnode_id { Some(p.clone()) } else { None }) {
        if typ == "/dev/console" {
            let mut tmp = vec![0u8; len];
            unsafe {
                if copy_from_user(user_buf_ptr, &mut tmp).is_err() {
                    return -1;
                }
                let mut buf = [0u8; 512];
                let cpy = core::cmp::min(len, 511);
                buf[..cpy].copy_from_slice(&tmp[..cpy]);
                buf[cpy] = 0;
                crate::ffi::serial_print(buf.as_ptr());
                *offset += len;
                return len as isize;
            }
        }
    }
    if let Some(vec) = state.data.get_mut(&vnode_id) {
        if *offset > vec.len() {
            vec.resize(*offset, 0);
        }
        let mut tmp = vec![0u8; len];
        unsafe {
            if copy_from_user(user_buf_ptr, &mut tmp).is_err() {
                return -1;
            }
        }
        if *offset + len > vec.len() {
            vec.resize(*offset + len, 0);
        }
        vec[*offset..*offset+len].copy_from_slice(&tmp[..len]);
        *offset += len;
        return len as isize;
    }
    -1
}

pub fn vfs_close(_vnode_id: u64) -> i32 {
    0
}

pub fn vfs_create_pipe() -> Result<u64, i32> {
    let mut guard = VFS_STATE.lock();
    let state = guard.as_mut().ok_or(-1)?;
    let id = state.next_id;
    state.next_id += 1;
    state.data.insert(id, Vec::new());
    Ok(id)
}

pub fn vfs_lseek(vnode_id: u64, offset: &mut usize, off: i32, whence: u32) -> isize {
    let guard = VFS_STATE.lock();
    let state = match guard.as_ref() {
        Some(s) => s,
        None => return -1,
    };
    if let Some(vec) = state.data.get(&vnode_id) {
        let newpos: isize = match whence {
            0 => off as isize,
            1 => (*offset as isize) + (off as isize),
            2 => (vec.len() as isize) + (off as isize),
            _ => return -1,
        };
        if newpos < 0 { return -1; }
        *offset = newpos as usize;
        return *offset as isize;
    }
    -1
}

pub fn vfs_mount_fat32(dev_id: usize, mount_path: &str) -> Result<(), ()> {
    let mut guard = VFS_STATE.lock();
    let state = guard.as_mut().ok_or(())?;
    state.mount_fat32(dev_id, mount_path)
}

pub fn vfs_block_device_count() -> usize {
    let guard = VFS_STATE.lock();
    match guard.as_ref() {
        Some(state) => state.block_devices.len(),
        None => 0,
    }
}

pub fn vfs_block_device_sectors(dev_id: usize) -> u64 {
    let guard = VFS_STATE.lock();
    match guard.as_ref() {
        Some(state) => {
            if let Some(Some(dev)) = state.block_devices.get(dev_id) {
                dev.num_sectors()
            } else { 0 }
        }
        None => 0,
    }
}

pub fn vfs_list_fat32(dev_id: usize) -> Result<alloc::vec::Vec<crate::fs::fat32::Fat32File>, ()> {
    let mut guard = VFS_STATE.lock();
    let state = guard.as_mut().ok_or(())?;
    let dev = state.block_devices[dev_id].as_mut().ok_or(())?;
    let key = 1000 + dev_id as u64;
    if let Some(fs) = state.fat32_filesystems.get_mut(&key) {
        fs.root_entries(dev.as_mut())
    } else {
        let mut fs = fat32::Fat32Fs::new(dev_id, dev.as_mut())?;
        let entries = fs.root_entries(dev.as_mut())?;
        state.fat32_filesystems.insert(key, fs);
        Ok(entries)
    }
}
