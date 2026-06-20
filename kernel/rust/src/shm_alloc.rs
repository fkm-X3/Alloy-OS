use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::sync::SpinLock;

struct ShmRegion {
    width: u32,
    height: u32,
    bpp: u32,
    size: u32,
    pages: Vec<usize>,
}

static SHM_REGIONS: SpinLock<Option<BTreeMap<i32, ShmRegion>>> = SpinLock::new(None);
static NEXT_FD: SpinLock<i32> = SpinLock::new(1);

pub fn shm_alloc(width: u32, height: u32, bpp: u32) -> i32 {
    let bpp_bytes = (bpp / 8).max(1);
    let stride = width.saturating_mul(bpp_bytes);
    let size = height.saturating_mul(stride);
    if size == 0 || size > 256 * 1024 * 1024 { return -1; }

    let num_pages = (size as usize + 4095) / 4096;
    let mut pages = Vec::with_capacity(num_pages);

    for _ in 0..num_pages {
        let phys = unsafe { crate::ffi::pmm_alloc_frame() };
        if phys.is_null() {
            for &p in &pages {
                unsafe { crate::ffi::pmm_free_frame(p as *mut _); }
            }
            return -1;
        }
        pages.push(phys as usize);
    }

    let mut fd_guard = NEXT_FD.lock();
    let fd = *fd_guard;
    *fd_guard = fd.wrapping_add(1);

    let mut guard = SHM_REGIONS.lock();
    if guard.is_none() {
        *guard = Some(BTreeMap::new());
    }
    if let Some(ref mut map) = *guard {
        map.insert(fd, ShmRegion { width, height, bpp, size, pages });
    }

    fd
}

/// Base address for SHM virtual allocations
const SHM_USER_BASE: u32 = 0x02000000;

static SHM_NEXT_VADDR: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(SHM_USER_BASE);

pub fn shm_user_vaddr(fd: i32) -> u32 {
    let guard = SHM_REGIONS.lock();
    let map = match guard.as_ref() {
        Some(m) => m,
        None => return 0,
    };
    let region = match map.get(&fd) {
        Some(r) => r,
        None => return 0,
    };

    let num_pages = region.pages.len();
    let total_size = num_pages * 4096;

    let vaddr = SHM_NEXT_VADDR.fetch_add(total_size as u32, core::sync::atomic::Ordering::Relaxed);
    if vaddr == 0 { return 0; }

    for (i, &phys) in region.pages.iter().enumerate() {
        let page_vaddr = vaddr + (i as u32 * 4096);
        let ok = unsafe { crate::ffi::vmm_map(
            page_vaddr as *mut core::ffi::c_void,
            phys as *mut core::ffi::c_void,
            crate::ffi::PAGE_PRESENT | crate::ffi::PAGE_WRITE | crate::ffi::PAGE_USER,
        )};
        if !ok { return 0; }
    }

    unsafe { core::ptr::write_bytes(vaddr as *mut u8, 0, region.size as usize); }
    vaddr
}
