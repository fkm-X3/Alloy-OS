// Virtual File System (VFS) module - minimal tmpfs-backed implementation

pub mod vnode;
pub mod tmpfs;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use crate::sync::SpinLock;

use crate::utils::{copy_from_user, copy_to_user};

// Simple in-memory mapping: vnode id -> data
struct FsState {
    next_id: u64,
    path_to_id: BTreeMap<String, u64>,
    data: BTreeMap<u64, Vec<u8>>,
}

impl FsState {
    fn new() -> Self {
        FsState {
            next_id: 1,
            path_to_id: BTreeMap::new(),
            data: BTreeMap::new(),
        }
    }
}

static VFS_STATE: SpinLock<Option<FsState>> = SpinLock::new(None);

/// Initialize VFS (call once during early boot)
pub fn vfs_init() {
    // Initialize VFS state (no lock needed during boot)
    {
        let mut guard = VFS_STATE.lock();
        *guard = Some(FsState::new());
    }
    // Create /dev/console vnode for serial output
    if let Ok(_id) = vfs_open("/dev/console", 0, 0) {
        unsafe { crate::ffi::serial_print(b"[VFS] /dev/console created\n\0".as_ptr()); }
    }

    // Embed a built-in hello test binary into the VFS if present at build time.
    // This uses include_bytes! to bundle the prebuilt userland binary located at ../../hello
    // relative to the kernel/rust crate directory. If it doesn't exist, this is a no-op.
    let hello_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../hello"));
    if hello_bytes.len() > 0 {
        if let Ok(id) = vfs_open("/hello", 0, 0) {
            // store the file contents
            let mut g = VFS_STATE.lock();
            if let Some(state) = g.as_mut() {
                state.data.insert(id, hello_bytes.to_vec());
                unsafe { crate::ffi::serial_print(b"[VFS] /hello embedded into VFS\n\0".as_ptr()); }
            }
        }
        if let Ok(id2) = vfs_open("/bin/hello", 0, 0) {
            let mut g2 = VFS_STATE.lock();
            if let Some(state2) = g2.as_mut() {
                state2.data.insert(id2, hello_bytes.to_vec());
            }
        }
    }
}

/// Open a path and return vnode id
fn normalize_path(path: &str) -> String {
    // Simple normalization: collapse multiple slashes and remove trailing slash (except root)
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

/// Return a copy of entire file contents as a Vec<u8>
pub fn vfs_read_all(vnode_id: u64) -> Option<Vec<u8>> {
    let guard = VFS_STATE.lock();
    let state = guard.as_ref()?;
    state.data.get(&vnode_id).map(|v| v.clone())
}

/// Read from vnode into user buffer
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
            if let Ok(_) = copy_to_user(user_buf_ptr, &vec[*offset..(*offset+to_copy)]) {
                *offset += to_copy;
                return to_copy as isize;
            }
        }
    }
    -1
}

/// Write from user buffer into vnode at offset
pub fn vfs_write(vnode_id: u64, offset: &mut usize, user_buf_ptr: u32, len: usize) -> isize {
    let mut guard = VFS_STATE.lock();
    let state = match guard.as_mut() {
        Some(s) => s,
        None => return -1,
    };
    // Device special-case
    if let Some(typ) = state.path_to_id.iter().find_map(|(p,&id)| if id==vnode_id { Some(p.clone()) } else { None }) {
        if typ == "/dev/console" {
            // Copy from user and print to serial
            let mut tmp = vec![0u8; len];
            unsafe {
                if let Err(_) = copy_from_user(user_buf_ptr, &mut tmp) {
                    return -1;
                }
                // Null-terminate for serial_print
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
        // Append or write at offset
        if *offset > vec.len() {
            vec.resize(*offset, 0);
        }
        let mut tmp = vec![0u8; len];
        unsafe {
            if let Err(_) = copy_from_user(user_buf_ptr, &mut tmp) {
                return -1;
            }
        }
        // If writing beyond current length, extend
        if *offset + len > vec.len() {
            vec.resize(*offset + len, 0);
        }
        vec[*offset..*offset+len].copy_from_slice(&tmp[..len]);
        *offset += len;
        return len as isize;
    }
    -1
}

/// Close vnode (no-op for tmpfs)
pub fn vfs_close(_vnode_id: u64) -> i32 {
    0
}

/// Create an anonymous pipe vnode and return its id
pub fn vfs_create_pipe() -> Result<u64, i32> {
    let mut guard = VFS_STATE.lock();
    let state = guard.as_mut().ok_or(-1)?;
    let id = state.next_id;
    state.next_id += 1;
    state.data.insert(id, Vec::new());
    Ok(id)
}

/// Seek helper for vnode
pub fn vfs_lseek(vnode_id: u64, offset: &mut usize, off: i32, whence: u32) -> isize {
    let guard = VFS_STATE.lock();
    let state = match guard.as_ref() {
        Some(s) => s,
        None => return -1,
    };

    if let Some(vec) = state.data.get(&vnode_id) {
        let newpos: isize = match whence {
            0 => off as isize,                 // SEEK_SET
            1 => (*offset as isize) + (off as isize), // SEEK_CUR
            2 => (vec.len() as isize) + (off as isize), // SEEK_END
            _ => return -1,
        };
        if newpos < 0 { return -1; }
        *offset = newpos as usize;
        return *offset as isize;
    }
    -1
}
