use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
}
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type bool_0 = bool;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut kernel_page_dir_phys: uintptr_t = 0 as uintptr_t;
#[no_mangle]
pub static mut g_current_user_cr3: uint64_t = 0 as uint64_t;
#[derive(Copy, Clone)]
#[repr(C, align(4096))]
pub struct kernel_tt_l0_T(pub [uint64_t; 512]);
static mut kernel_tt_l0: kernel_tt_l0_T = kernel_tt_l0_T([0 as uint64_t; 512]);
#[no_mangle]
pub unsafe extern "C" fn paging_init() {
    serial_print(
        b"Paging: Initializing ARM64 translation tables\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 512 as ::core::ffi::c_int {
        kernel_tt_l0.0[i as usize] = 0 as uint64_t;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 2 as ::core::ffi::c_int {
        let mut block_addr: uint64_t = (i_0 as uint64_t) << 30 as ::core::ffi::c_int;
        kernel_tt_l0.0[i_0 as usize] = block_addr | 0xc01 as uint64_t;
        i_0 += 1;
    }
    kernel_page_dir_phys = (&raw mut kernel_tt_l0.0 as *mut uint64_t)
        .offset(0 as ::core::ffi::c_int as isize) as *mut uint64_t
        as uintptr_t;
    g_current_user_cr3 = kernel_page_dir_phys as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn paging_enable() {
    serial_print(b"Paging: Enabling MMU\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut ttbr0: uint64_t = &raw mut kernel_tt_l0.0 as *mut uint64_t as uintptr_t as uint64_t;
    asm!("msr ttbr0_el1, {0}\n", inlateout(reg) ttbr0 => _, options(preserves_flags));
    asm!("isb\n", options(preserves_flags));
}
#[no_mangle]
pub unsafe extern "C" fn paging_create_directory_phys() -> uintptr_t {
    return kernel_page_dir_phys;
}
#[no_mangle]
pub unsafe extern "C" fn paging_switch_to_directory(mut pd_phys: uintptr_t) -> bool_0 {
    if pd_phys == 0 as uintptr_t {
        return false_0 != 0;
    }
    asm!(
        "msr ttbr0_el1, {0}\n", inlateout(reg) pd_phys as uint64_t => _,
        options(preserves_flags)
    );
    asm!("isb; tlbi vmalle1; dsb sy; isb\n", options(preserves_flags));
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paging_get_kernel_directory_phys() -> uintptr_t {
    return kernel_page_dir_phys;
}
#[no_mangle]
pub unsafe extern "C" fn paging_get_physical_address(mut virt_addr: uintptr_t) -> uintptr_t {
    return virt_addr;
}
#[no_mangle]
pub unsafe extern "C" fn paging_destroy_directory(mut pd_phys: uintptr_t) {}
#[no_mangle]
pub unsafe extern "C" fn paging_clone_directory(mut pd_phys: uintptr_t) -> uintptr_t {
    return pd_phys;
}
#[no_mangle]
pub unsafe extern "C" fn paging_fork_directory(mut pd_phys: uintptr_t) -> uintptr_t {
    return pd_phys;
}
#[no_mangle]
pub unsafe extern "C" fn paging_handle_cow_fault(mut fault_addr: uintptr_t) -> uint8_t {
    return 0 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn paging_map_page_in_pd(
    mut pd_phys: uintptr_t,
    mut virt_addr: uintptr_t,
    mut phys_addr: uintptr_t,
    mut flags: uint32_t,
) -> bool_0 {
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paging_temp_map_frame(
    mut phys_addr: uintptr_t,
) -> *mut ::core::ffi::c_void {
    return phys_addr as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn paging_temp_unmap_frame() {}
#[no_mangle]
pub unsafe extern "C" fn paging_map_page(
    mut virt_addr: uintptr_t,
    mut phys_addr: uintptr_t,
    mut flags: uint32_t,
) -> bool_0 {
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paging_unmap_page(mut virt_addr: uintptr_t) {}
