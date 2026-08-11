extern "C" {
    fn pmm_alloc_frame() -> *mut ::core::ffi::c_void;
    fn pmm_free_frame(addr: *mut ::core::ffi::c_void);
    fn paging_map_page(virt_addr: uintptr_t, phys_addr: uintptr_t, flags: uint32_t) -> bool_0;
    fn paging_unmap_page(virt_addr: uintptr_t);
    fn paging_get_physical_address(virt_addr: uintptr_t) -> uintptr_t;
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
}
pub type uint32_t = u32;
pub type uintptr_t = usize;
pub type bool_0 = bool;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtualMemoryManager {
    pub next_virt_addr: uintptr_t,
    pub allocated_pages: uint32_t,
}
pub const PAGE_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_vmm: VirtualMemoryManager = VirtualMemoryManager {
    next_virt_addr: 0,
    allocated_pages: 0,
};
#[cfg(target_arch = "x86_64")]
pub const KERNEL_HEAP_START: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
#[cfg(target_arch = "x86_64")]
pub const KERNEL_HEAP_END: ::core::ffi::c_uint = 0xc0000000 as ::core::ffi::c_uint;
#[cfg(target_arch = "aarch64")]
pub const KERNEL_HEAP_START: ::core::ffi::c_int = 0x40510000 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const KERNEL_HEAP_END: ::core::ffi::c_int = 0x47f00000 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn vmm_init() {
    serial_print(
        b"VMM: Initializing virtual memory manager...\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    g_vmm.next_virt_addr = KERNEL_HEAP_START as uintptr_t;
    g_vmm.allocated_pages = 0 as uint32_t;
    serial_print(b"VMM: Initialization complete\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Heap start: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(KERNEL_HEAP_START as uint32_t);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Heap end: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(KERNEL_HEAP_END as uint32_t);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn vmm_alloc_region(
    mut size: uintptr_t,
    mut flags: uint32_t,
) -> *mut ::core::ffi::c_void {
    if size.wrapping_rem(PAGE_SIZE as uintptr_t) != 0 as uintptr_t {
        size = size
            .wrapping_div(PAGE_SIZE as uintptr_t)
            .wrapping_add(1 as ::core::ffi::c_int as uintptr_t)
            .wrapping_mul(PAGE_SIZE as uintptr_t);
    }
    let mut num_pages: uintptr_t = size.wrapping_div(PAGE_SIZE as uintptr_t);
    if g_vmm.next_virt_addr.wrapping_add(size) > KERNEL_HEAP_END as uintptr_t {
        serial_print(
            b"VMM: ERROR - Out of virtual address space\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    let mut virt_start: *mut ::core::ffi::c_void = g_vmm.next_virt_addr as *mut ::core::ffi::c_void;
    let mut i: uint32_t = 0 as uint32_t;
    while (i as uintptr_t) < num_pages {
        let mut phys_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if phys_frame.is_null() {
            serial_print(
                b"VMM: ERROR - Failed to allocate physical frame\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_void>();
        }
        let mut virt: uintptr_t = g_vmm
            .next_virt_addr
            .wrapping_add(i.wrapping_mul(PAGE_SIZE as uint32_t) as uintptr_t);
        if !paging_map_page(virt, phys_frame as uintptr_t, flags) {
            serial_print(
                b"VMM: ERROR - Failed to map page\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            pmm_free_frame(phys_frame);
            return ::core::ptr::null_mut::<::core::ffi::c_void>();
        }
        g_vmm.allocated_pages = g_vmm.allocated_pages.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    g_vmm.next_virt_addr = g_vmm.next_virt_addr.wrapping_add(size);
    return virt_start;
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn vmm_alloc_region(
    mut size: uintptr_t,
    mut flags: uint32_t,
) -> *mut ::core::ffi::c_void {
    if size.wrapping_rem(PAGE_SIZE as uintptr_t) != 0 as uintptr_t {
        size = size
            .wrapping_div(PAGE_SIZE as uintptr_t)
            .wrapping_add(1 as ::core::ffi::c_int as uintptr_t)
            .wrapping_mul(PAGE_SIZE as uintptr_t);
    }
    let mut num_pages: uintptr_t = size.wrapping_div(PAGE_SIZE as uintptr_t);
    if g_vmm.next_virt_addr.wrapping_add(size) > KERNEL_HEAP_END as uintptr_t {
        serial_print(
            b"VMM: ERROR - Out of heap space\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    let mut first_addr: uintptr_t = 0 as uintptr_t;
    let mut i: uint32_t = 0 as uint32_t;
    while (i as uintptr_t) < num_pages {
        let mut phys_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if phys_frame.is_null() {
            serial_print(
                b"VMM: ERROR - Failed to allocate physical frame\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_void>();
        }
        if i == 0 as uint32_t {
            first_addr = phys_frame as uintptr_t;
            serial_print(
                b"VMM: aarch64 heap page at 0x\0" as *const u8 as *const ::core::ffi::c_char,
            );
            serial_print_hex(first_addr as uint32_t);
            serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
        }
        g_vmm.allocated_pages = g_vmm.allocated_pages.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    g_vmm.next_virt_addr = g_vmm.next_virt_addr.wrapping_add(size);
    return first_addr as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn vmm_free_region(
    mut virt_addr: *mut ::core::ffi::c_void,
    mut size: uintptr_t,
) {
    if virt_addr.is_null() {
        return;
    }
    if size.wrapping_rem(PAGE_SIZE as uintptr_t) != 0 as uintptr_t {
        size = size
            .wrapping_div(PAGE_SIZE as uintptr_t)
            .wrapping_add(1 as ::core::ffi::c_int as uintptr_t)
            .wrapping_mul(PAGE_SIZE as uintptr_t);
    }
    let mut num_pages: uintptr_t = size.wrapping_div(PAGE_SIZE as uintptr_t);
    let mut virt: uintptr_t = virt_addr as uintptr_t;
    let mut i: uint32_t = 0 as uint32_t;
    while (i as uintptr_t) < num_pages {
        let mut page_virt: uintptr_t =
            virt.wrapping_add(i.wrapping_mul(PAGE_SIZE as uint32_t) as uintptr_t);
        let mut phys: uintptr_t = paging_get_physical_address(page_virt);
        if phys != 0 as uintptr_t {
            pmm_free_frame(
                (phys & !((PAGE_SIZE - 1 as ::core::ffi::c_int) as uintptr_t))
                    as *mut ::core::ffi::c_void,
            );
            paging_unmap_page(page_virt);
            g_vmm.allocated_pages = g_vmm.allocated_pages.wrapping_sub(1);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vmm_map(
    mut virt_addr: *mut ::core::ffi::c_void,
    mut phys_addr: *mut ::core::ffi::c_void,
    mut flags: uint32_t,
) -> bool_0 {
    return paging_map_page(virt_addr as uintptr_t, phys_addr as uintptr_t, flags);
}
#[no_mangle]
pub unsafe extern "C" fn vmm_unmap(mut virt_addr: *mut ::core::ffi::c_void) {
    paging_unmap_page(virt_addr as uintptr_t);
}
#[no_mangle]
pub unsafe extern "C" fn vmm_get_allocated_pages() -> uint32_t {
    return g_vmm.allocated_pages;
}
#[no_mangle]
pub unsafe extern "C" fn vmm_get_heap_start() -> uintptr_t {
    return KERNEL_HEAP_START as uintptr_t;
}
#[no_mangle]
pub unsafe extern "C" fn vmm_get_heap_size() -> uintptr_t {
    return g_vmm
        .next_virt_addr
        .wrapping_sub(KERNEL_HEAP_START as uintptr_t);
}
#[no_mangle]
pub unsafe extern "C" fn vmm_get_next_virt_addr() -> uintptr_t {
    return g_vmm.next_virt_addr;
}
