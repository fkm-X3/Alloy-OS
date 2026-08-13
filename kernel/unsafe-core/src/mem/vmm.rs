//! Hand-written virtual memory manager.
//!
//! Replaces `ported/common/mm_vmm.rs`. The bump allocator over the kernel
//! heap window keeps the exact translated behavior per arch:
//!
//! - x86_64 allocates a physical frame per page and maps it into the current
//!   (kernel) address space via `paging_map_page`.
//! - aarch64 (MMU disabled, identity) returns the first physical frame of the
//!   region and only advances `next_virt_addr` — the pages are identity-mapped
//!   by the hardware.
//!
//! The `#[no_mangle] extern "C"` entry points keep `raw::ffi`, the boot main
//! and the surviving ported modules (ahci, ...) resolving as before.

use core::ffi::c_void;

use crate::drivers::serial::Serial;
#[cfg(feature = "x86_64")]
use crate::mem::paging;
#[cfg(feature = "aarch64")]
use crate::mem::paging_aarch64 as paging;
use crate::mem::pmm;

#[repr(C)]
pub struct VirtualMemoryManager {
    pub next_virt_addr: usize,
    pub allocated_pages: u32,
}

#[no_mangle]
pub static mut g_vmm: VirtualMemoryManager = VirtualMemoryManager {
    next_virt_addr: 0,
    allocated_pages: 0,
};

#[cfg(feature = "x86_64")]
pub const KERNEL_HEAP_START: usize = 0x0200_0000;
#[cfg(feature = "x86_64")]
pub const KERNEL_HEAP_END: usize = 0xc000_0000;

#[cfg(feature = "aarch64")]
pub const KERNEL_HEAP_START: usize = 0x4051_0000;
#[cfg(feature = "aarch64")]
pub const KERNEL_HEAP_END: usize = 0x47f0_0000;

const PAGE_SIZE: usize = 4096;

#[inline]
fn align_up(size: usize) -> usize {
    (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

/// `vmm_init()`: reset the bump allocator.
#[no_mangle]
pub unsafe extern "C" fn vmm_init() {
    Serial::write_str("VMM: Initializing virtual memory manager...\n");
    g_vmm.next_virt_addr = KERNEL_HEAP_START;
    g_vmm.allocated_pages = 0;
    Serial::write_str("VMM: Initialization complete\n");
    Serial::write_str("  Heap start: 0x");
    Serial::write_hex(KERNEL_HEAP_START as u32);
    Serial::write_str("\n");
    Serial::write_str("  Heap end: 0x");
    Serial::write_hex(KERNEL_HEAP_END as u32);
    Serial::write_str("\n");
}

/// `vmm_alloc_region(size, flags)`: allocate a contiguous virtual region.
///
/// x86_64: each page is backed by a fresh frame mapped with `flags`.
/// aarch64: identity — the first frame address is returned, pages unmapped.
#[cfg(feature = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn vmm_alloc_region(
    size: usize,
    flags: u32,
) -> *mut c_void {
    let size = align_up(size);
    let num_pages = size / PAGE_SIZE;
    if g_vmm.next_virt_addr.wrapping_add(size) > KERNEL_HEAP_END {
        Serial::write_str("VMM: ERROR - Out of virtual address space\n");
        return core::ptr::null_mut();
    }
    let virt_start = g_vmm.next_virt_addr as *mut c_void;
    for i in 0..num_pages {
        let phys = pmm::pmm_alloc_frame();
        if phys.is_null() {
            Serial::write_str("VMM: ERROR - Failed to allocate physical frame\n");
            return core::ptr::null_mut();
        }
        let virt = g_vmm.next_virt_addr + i * PAGE_SIZE;
        if !paging::paging_map_page(virt, phys as usize, flags) {
            Serial::write_str("VMM: ERROR - Failed to map page\n");
            pmm::pmm_free_frame(phys);
            return core::ptr::null_mut();
        }
        g_vmm.allocated_pages += 1;
    }
    g_vmm.next_virt_addr += size;
    virt_start
}

/// `vmm_alloc_region(size, flags)`: allocate a contiguous virtual region.
///
/// aarch64 (identity): no page tables to populate; the first allocated frame
/// is the region's address and only the bump pointer advances.
#[cfg(feature = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn vmm_alloc_region(
    size: usize,
    flags: u32,
) -> *mut c_void {
    let size = align_up(size);
    let num_pages = size / PAGE_SIZE;
    if g_vmm.next_virt_addr.wrapping_add(size) > KERNEL_HEAP_END {
        Serial::write_str("VMM: ERROR - Out of heap space\n");
        return core::ptr::null_mut();
    }
    let mut first_addr: usize = 0;
    for i in 0..num_pages {
        let phys = pmm::pmm_alloc_frame();
        if phys.is_null() {
            Serial::write_str("VMM: ERROR - Failed to allocate physical frame\n");
            return core::ptr::null_mut();
        }
        if i == 0 {
            first_addr = phys as usize;
            Serial::write_str("VMM: aarch64 heap page at 0x");
            Serial::write_hex(first_addr as u32);
            Serial::write_str("\n");
        }
        g_vmm.allocated_pages += 1;
    }
    g_vmm.next_virt_addr += size;
    first_addr as *mut c_void
}

/// `vmm_free_region(virt_addr, size)`: unmap a region and free its frames.
#[no_mangle]
pub unsafe extern "C" fn vmm_free_region(virt_addr: *mut c_void, size: usize) {
    if virt_addr.is_null() {
        return;
    }
    let size = align_up(size);
    let num_pages = size / PAGE_SIZE;
    for i in 0..num_pages {
        let page_virt = virt_addr as usize + i * PAGE_SIZE;
        let phys = paging::paging_get_physical_address(page_virt);
        if phys != 0 {
            pmm::pmm_free_frame((phys & !(PAGE_SIZE - 1)) as *mut c_void);
            paging::paging_unmap_page(page_virt);
            g_vmm.allocated_pages -= 1;
        }
    }
}

/// `vmm_map(virt_addr, phys_addr, flags)`: install a single mapping.
#[no_mangle]
pub unsafe extern "C" fn vmm_map(
    virt_addr: *mut c_void,
    phys_addr: *mut c_void,
    flags: u32,
) -> bool {
    paging::paging_map_page(virt_addr as usize, phys_addr as usize, flags)
}

/// `vmm_unmap(virt_addr)`: remove a single mapping.
#[no_mangle]
pub unsafe extern "C" fn vmm_unmap(virt_addr: *mut c_void) {
    paging::paging_unmap_page(virt_addr as usize);
}

/// `vmm_get_allocated_pages()`: number of frames owned by VMM regions.
#[no_mangle]
pub unsafe extern "C" fn vmm_get_allocated_pages() -> u32 {
    g_vmm.allocated_pages
}

/// `vmm_get_heap_start()`: first byte of the kernel heap window.
#[no_mangle]
pub unsafe extern "C" fn vmm_get_heap_start() -> usize {
    KERNEL_HEAP_START
}

/// `vmm_get_heap_size()`: bytes consumed from the heap window so far.
#[no_mangle]
pub unsafe extern "C" fn vmm_get_heap_size() -> usize {
    g_vmm.next_virt_addr.wrapping_sub(KERNEL_HEAP_START)
}

/// `vmm_get_next_virt_addr()`: next free byte of the heap window.
#[no_mangle]
pub unsafe extern "C" fn vmm_get_next_virt_addr() -> usize {
    g_vmm.next_virt_addr
}
