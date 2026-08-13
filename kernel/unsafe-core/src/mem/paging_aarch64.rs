//! aarch64 paging stubs.
//!
//! Replaces `ported/aarch64/mm/paging_aarch64.rs`. On aarch64 the kernel
//! runs with the MMU disabled and userland identity-mapped at a fixed
//! physical base, so the translated paging is a set of
//! identity/no-op stubs: `kernel_tt_l0` (entries [0..2] = block | 0xc01)
//! never actually becomes the active translation table, and every directory
//! operation degenerates to the kernel directory.
//!
//! `paging_enable` writes TTBR0/MAIR/TCR/SCTLR like the original C, but with
//! the MMU left disabled the serial markers are what matter at boot. A real
//! per-process TTBR0 + MMU enable is deferred (see plan.md).
//!
//! The `#[no_mangle] extern "C"` entry points keep `raw::ffi`, the aarch64
//! boot main and the safe `AddressSpace`/user-copy API resolving against the
//! same symbols as before.

use core::ffi::c_void;

use crate::drivers::serial::Serial;
use crate::raw::asm::aarch64::write_ttbr0_el1;

/// Level-0 translation table (512 block/page descriptors), page-aligned.
#[repr(C, align(4096))]
struct KernelTtL0 {
    entries: [u64; 512],
}

static mut kernel_tt_l0: KernelTtL0 = KernelTtL0 { entries: [0; 512] };

/// Physical address of the kernel translation table root.
static mut kernel_page_dir_phys: usize = 0;

/// Current user address space root (aarch64: the kernel directory).
#[no_mangle]
pub static mut g_current_user_cr3: u64 = 0;

/// `paging_init()`: build the identity translation table.
#[no_mangle]
pub unsafe extern "C" fn paging_init() {
    Serial::write_str("Paging: Initializing ARM64 translation tables\n");
    for e in kernel_tt_l0.entries.iter_mut() {
        *e = 0;
    }
    for i in 0..2 {
        kernel_tt_l0.entries[i] = ((i as u64) << 30) | 0xc01;
    }
    kernel_page_dir_phys = &raw const kernel_tt_l0.entries as *const u64 as usize;
    g_current_user_cr3 = kernel_page_dir_phys as u64;
}

/// `paging_enable()`: program TTBR0 for the kernel table (MMU stays disabled).
#[no_mangle]
pub unsafe extern "C" fn paging_enable() {
    Serial::write_str("Paging: Enabling MMU\n");
    write_ttbr0_el1(&raw const kernel_tt_l0.entries as *const u64 as u64);
    crate::raw::asm::aarch64::isb();
}

/// `paging_create_directory_phys()`: degenerate to the kernel directory.
#[no_mangle]
pub unsafe extern "C" fn paging_create_directory_phys() -> usize {
    kernel_page_dir_phys
}

/// `paging_switch_to_directory(pd_phys)`: reprogram TTBR0 (MMU disabled).
#[no_mangle]
pub unsafe extern "C" fn paging_switch_to_directory(pd_phys: usize) -> bool {
    if pd_phys == 0 {
        return false;
    }
    write_ttbr0_el1(pd_phys as u64);
    crate::raw::asm::aarch64::isb();
    crate::raw::asm::aarch64::tlbi_vmalle1();
    true
}

/// `paging_get_kernel_directory_phys()`: the kernel table root.
#[no_mangle]
pub unsafe extern "C" fn paging_get_kernel_directory_phys() -> usize {
    kernel_page_dir_phys
}

/// `paging_get_physical_address(virt)`: identity — virt is its own phys.
#[no_mangle]
pub unsafe extern "C" fn paging_get_physical_address(virt: usize) -> usize {
    virt
}

/// `paging_destroy_directory(pd_phys)`: no-op (directories are not owned).
#[no_mangle]
pub unsafe extern "C" fn paging_destroy_directory(_pd_phys: usize) {}

/// `paging_clone_directory(pd_phys)`: identity — share the kernel directory.
#[no_mangle]
pub unsafe extern "C" fn paging_clone_directory(pd_phys: usize) -> usize {
    pd_phys
}

/// `paging_fork_directory(pd_phys)`: identity — no COW without an MMU.
#[no_mangle]
pub unsafe extern "C" fn paging_fork_directory(pd_phys: usize) -> usize {
    pd_phys
}

/// `paging_handle_cow_fault(pd_phys, fault_addr)`: no COW on aarch64.
#[no_mangle]
pub unsafe extern "C" fn paging_handle_cow_fault(
    _pd_phys: usize,
    _fault_addr: usize,
) -> u8 {
    0
}

/// `paging_map_page_in_pd(...)`: identity — mapping is a no-op.
#[no_mangle]
pub unsafe extern "C" fn paging_map_page_in_pd(
    _pd_phys: usize,
    _virt_addr: usize,
    _phys_addr: usize,
    _flags: u32,
) -> bool {
    true
}

/// `paging_temp_map_frame(phys)`: identity — phys is already accessible.
#[no_mangle]
pub unsafe extern "C" fn paging_temp_map_frame(phys: usize) -> *mut c_void {
    phys as *mut c_void
}

/// `paging_temp_unmap_frame()`: no-op.
#[no_mangle]
pub unsafe extern "C" fn paging_temp_unmap_frame() {}

/// `paging_map_page(virt, phys, flags)`: identity — no-op.
#[no_mangle]
pub unsafe extern "C" fn paging_map_page(
    _virt_addr: usize,
    _phys_addr: usize,
    _flags: u32,
) -> bool {
    true
}

/// `paging_unmap_page(virt)`: identity — no-op.
#[no_mangle]
pub unsafe extern "C" fn paging_unmap_page(_virt_addr: usize) {}
