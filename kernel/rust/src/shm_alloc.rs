use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloy_kernel_hal::PhysFrame;
use alloy_kernel_hal::sync::SpinLock;

struct ShmRegion {
    width: u32,
    height: u32,
    bpp: u32,
    size: u32,
    pages: Vec<PhysFrame>,
}

static SHM_REGIONS: SpinLock<Option<BTreeMap<i32, ShmRegion>>> = SpinLock::new(None);
static NEXT_FD: SpinLock<i32> = SpinLock::new(1);

pub fn shm_alloc(width: u32, height: u32, bpp: u32) -> i32 {
    let bpp_bytes = (bpp / 8).max(1);
    let stride = width.saturating_mul(bpp_bytes);
    let size = height.saturating_mul(stride);
    if size == 0 || size > 256 * 1024 * 1024 {
        return -1;
    }

    let num_pages = (size as usize + 4095) / 4096;
    let mut pages = Vec::with_capacity(num_pages);

    // Frames are RAII (`PhysFrame`): the `pages` Vec frees them on drop,
    // so the error path below needs no manual cleanup.
    for _ in 0..num_pages {
        let Some(frame) = PhysFrame::alloc() else {
            return -1;
        };
        pages.push(frame);
    }

    let mut fd_guard = NEXT_FD.lock();
    let fd = *fd_guard;
    *fd_guard = fd.wrapping_add(1);

    let mut guard = SHM_REGIONS.lock();
    if guard.is_none() {
        *guard = Some(BTreeMap::new());
    }
    if let Some(ref mut map) = *guard {
        map.insert(
            fd,
            ShmRegion {
                width,
                height,
                bpp,
                size,
                pages,
            },
        );
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
    if vaddr == 0 {
        return 0;
    }

    for (i, frame) in region.pages.iter().enumerate() {
        let page_vaddr = vaddr + (i as u32 * 4096);
        let ok = alloy_kernel_hal::mem::map_page(
            page_vaddr as usize,
            frame.addr(),
            alloy_kernel_hal::PageFlags::user_write(),
        );
        if !ok {
            return 0;
        }
    }

    alloy_kernel_hal::mem::zero_phys_bytes(vaddr as usize, region.size as usize);
    vaddr
}
