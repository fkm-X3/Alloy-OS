use ::core::arch::asm;
extern "C" {
    fn pmm_alloc_frame() -> *mut ::core::ffi::c_void;
    fn pmm_free_frame(addr: *mut ::core::ffi::c_void);
    fn pmm_refcount_inc(addr: *mut ::core::ffi::c_void);
    fn pmm_refcount_dec(addr: *mut ::core::ffi::c_void);
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
    fn serial_print_hex64(value: uint64_t);
}
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type bool_0 = bool;
pub type page_dir_entry_t = uint64_t;
pub type page_table_entry_t = uint64_t;
#[derive(Copy, Clone)]
#[repr(C, align(4096))]
pub struct page_directory(pub page_directory_Inner);
#[derive(Copy, Clone)]
#[repr(C)]
pub struct page_directory_Inner {
    pub entries: [page_dir_entry_t; 512],
}
#[allow(dead_code, non_upper_case_globals)]
const page_directory_PADDING: usize =
    ::core::mem::size_of::<page_directory>() - ::core::mem::size_of::<page_directory_Inner>();
#[derive(Copy, Clone)]
#[repr(C, align(4096))]
pub struct page_table(pub page_table_Inner);
#[derive(Copy, Clone)]
#[repr(C)]
pub struct page_table_Inner {
    pub entries: [page_table_entry_t; 512],
}
#[allow(dead_code, non_upper_case_globals)]
const page_table_PADDING: usize =
    ::core::mem::size_of::<page_table>() - ::core::mem::size_of::<page_table_Inner>();
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Paging {
    pub kernel_directory: *mut page_directory,
    pub kernel_tables: [*mut page_table; 512],
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PAGE_PRESENT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PAGE_WRITE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PAGE_USER: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PAGE_COW: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PAGE_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_paging: Paging = Paging {
    kernel_directory: ::core::ptr::null::<page_directory>() as *mut page_directory,
    kernel_tables: [::core::ptr::null::<page_table>() as *mut page_table; 512],
};
pub const X86_64_PML4_PHYS: ::core::ffi::c_ulonglong = 0x1000 as ::core::ffi::c_ulonglong;
pub const X86_64_PDPT_PHYS: ::core::ffi::c_ulonglong = 0x2000 as ::core::ffi::c_ulonglong;
pub const X86_64_PD_PHYS: ::core::ffi::c_ulonglong = 0x3000 as ::core::ffi::c_ulonglong;
pub const X86_64_PML4_VIRT: *mut page_directory = X86_64_PML4_PHYS as *mut page_directory;
pub const X86_64_PDPT0_VIRT: *mut page_table = X86_64_PDPT_PHYS as *mut page_table;
pub const X86_64_PD_VIRT: *mut page_table = X86_64_PD_PHYS as *mut page_table;
pub const PD_WIN_IDX: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const PT_WIN_BASE: uint64_t = (PD_WIN_IDX as uint64_t) << 21 as ::core::ffi::c_int;
pub const PT_TEMP_IDX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PD_WIN2_IDX: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const PT_WIN2_BASE: uint64_t = (PD_WIN2_IDX as uint64_t) << 21 as ::core::ffi::c_int;
pub const PD_GWIN_IDX: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const G_WIN_PT_VA: *mut page_table =
    ((PD_GWIN_IDX as uint64_t) << 21 as ::core::ffi::c_int) as *mut page_table;
pub const G_WIN2_PT_VA: *mut page_table = ((PD_GWIN_IDX as uint64_t) << 21 as ::core::ffi::c_int)
    .wrapping_add(0x1000 as uint64_t) as *mut page_table;
#[no_mangle]
pub static mut kernel_pml4_phys: uint64_t = X86_64_PML4_PHYS as uint64_t;
#[no_mangle]
pub static mut g_current_user_cr3: uint64_t = X86_64_PML4_PHYS as uint64_t;
#[no_mangle]
pub static mut g_saved_user_cr3: uint64_t = X86_64_PML4_PHYS as uint64_t;
static mut g_win_pt_phys_addr: uint64_t = 0 as uint64_t;
static mut g_win2_pt_phys_addr: uint64_t = 0 as uint64_t;
#[inline]
unsafe extern "C" fn invlpg(mut virt: uint64_t) {
    asm!(
        "invlpg ({0})\n", inlateout(reg) virt => _, options(preserves_flags, att_syntax)
    );
}
unsafe extern "C" fn win_map(
    mut phys: uint64_t,
    mut pt_idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    let mut pd: *mut page_table = X86_64_PD_VIRT;
    let mut old: uint64_t = (*pd).0.entries[PD_WIN_IDX as usize];
    if old & 1 as uint64_t == 0 as uint64_t || old & 0x80 as uint64_t != 0 as uint64_t {
        (*pd).0.entries[PD_WIN_IDX as usize] = (g_win_pt_phys_addr & 0xfffffffff000 as uint64_t
            | 0x3 as uint64_t) as page_table_entry_t;
        invlpg(PT_WIN_BASE);
    }
    let mut va: uint64_t =
        PT_WIN_BASE.wrapping_add((pt_idx as uint64_t).wrapping_mul(4096 as uint64_t));
    (*G_WIN_PT_VA).0.entries[pt_idx as usize] =
        (phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
    invlpg(va);
    return va as uintptr_t as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn win_unmap(mut pt_idx: ::core::ffi::c_int) {
    let mut va: uint64_t =
        PT_WIN_BASE.wrapping_add((pt_idx as uint64_t).wrapping_mul(4096 as uint64_t));
    (*G_WIN_PT_VA).0.entries[pt_idx as usize] =
        (va & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
    invlpg(va);
}
unsafe extern "C" fn win2_map(
    mut phys: uint64_t,
    mut pt_idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    let mut pd: *mut page_table = X86_64_PD_VIRT;
    (*pd).0.entries[PD_WIN2_IDX as usize] =
        (g_win2_pt_phys_addr & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
    invlpg(PT_WIN2_BASE);
    let mut va: uint64_t =
        PT_WIN2_BASE.wrapping_add((pt_idx as uint64_t).wrapping_mul(4096 as uint64_t));
    (*G_WIN2_PT_VA).0.entries[pt_idx as usize] =
        (phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
    invlpg(va);
    return va as uintptr_t as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn win2_unmap(mut pt_idx: ::core::ffi::c_int) {
    let mut va: uint64_t =
        PT_WIN2_BASE.wrapping_add((pt_idx as uint64_t).wrapping_mul(4096 as uint64_t));
    (*G_WIN2_PT_VA).0.entries[pt_idx as usize] =
        (va & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
    invlpg(va);
}
#[no_mangle]
pub unsafe extern "C" fn paging_init() {
    serial_print(
        b"Paging: Initializing x86_64 paging...\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut cr3: uint64_t = 0;
    asm!("mov %cr3, {0}\n", lateout(reg) cr3, options(preserves_flags, att_syntax));
    kernel_pml4_phys = cr3 & 0xfffffffff000 as uint64_t;
    serial_print(b"  PML4 at phys 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(kernel_pml4_phys);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    g_paging.kernel_directory = X86_64_PML4_VIRT;
    let mut pd: *mut page_table = X86_64_PD_VIRT;
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        (*pd).0.entries[i as usize] =
            ((i as uint64_t) << 21 as ::core::ffi::c_int | 0x83 as uint64_t) as page_table_entry_t;
        i += 1;
    }
    let mut win_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if !win_frame.is_null() {
        g_win_pt_phys_addr = win_frame as uintptr_t as uint64_t;
        let mut win_pt: *mut page_table = g_win_pt_phys_addr as uintptr_t as *mut page_table;
        let mut base_phys: uint64_t = (PD_WIN_IDX as uint64_t) << 21 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < 512 as ::core::ffi::c_int {
            (*win_pt).0.entries[i_0 as usize] =
                (base_phys.wrapping_add((i_0 as uint64_t).wrapping_mul(4096 as uint64_t))
                    | 0x3 as uint64_t) as page_table_entry_t;
            i_0 += 1;
        }
    }
    let mut win2_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if !win2_frame.is_null() {
        g_win2_pt_phys_addr = win2_frame as uintptr_t as uint64_t;
        let mut win2_pt: *mut page_table = g_win2_pt_phys_addr as uintptr_t as *mut page_table;
        let mut base_phys_0: uint64_t = (PD_WIN2_IDX as uint64_t) << 21 as ::core::ffi::c_int;
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < 512 as ::core::ffi::c_int {
            (*win2_pt).0.entries[i_1 as usize] =
                (base_phys_0.wrapping_add((i_1 as uint64_t).wrapping_mul(4096 as uint64_t))
                    | 0x3 as uint64_t) as page_table_entry_t;
            i_1 += 1;
        }
    }
    let mut gwin_acc_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if !gwin_acc_frame.is_null() {
        let mut acc_phys: uint64_t = gwin_acc_frame as uintptr_t as uint64_t;
        let mut acc_pt: *mut page_table = acc_phys as uintptr_t as *mut page_table;
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < 512 as ::core::ffi::c_int {
            (*acc_pt).0.entries[i_2 as usize] =
                (((PD_GWIN_IDX << 21 as ::core::ffi::c_int) as uint64_t)
                    .wrapping_add((i_2 as uint64_t).wrapping_mul(4096 as uint64_t))
                    | 0x3 as uint64_t) as page_table_entry_t;
            i_2 += 1;
        }
        (*acc_pt).0.entries[0 as ::core::ffi::c_int as usize] =
            (g_win_pt_phys_addr & 0xfffffffff000 as uint64_t | 0x3 as uint64_t)
                as page_table_entry_t;
        (*acc_pt).0.entries[1 as ::core::ffi::c_int as usize] =
            (g_win2_pt_phys_addr & 0xfffffffff000 as uint64_t | 0x3 as uint64_t)
                as page_table_entry_t;
        (*pd).0.entries[PD_GWIN_IDX as usize] =
            (acc_phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
        invlpg((PD_GWIN_IDX as uint64_t) << 21 as ::core::ffi::c_int);
    }
    serial_print(
        b"  Identity map extended to 32 MB (PD[2..15])\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    serial_print(b"  Window PT at phys 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(g_win_pt_phys_addr);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Window2 PT at phys 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(g_win2_pt_phys_addr);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  g_win_pt VA (PD[12]) = 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(G_WIN_PT_VA as uint64_t);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn paging_enable() {
    serial_print(
        b"Paging: Already enabled (set up by boot code)\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn get_page_entry(mut virt_addr: uint64_t, mut create: bool_0) -> *mut uint64_t {
    let mut pml4_idx: uint64_t = virt_addr >> 39 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pdpt_idx: uint64_t = virt_addr >> 30 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pd_idx: uint64_t = virt_addr >> 21 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pt_idx: uint64_t = virt_addr >> 12 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pml4: *mut page_directory = X86_64_PML4_VIRT;
    if pml4_idx != 0 as uint64_t {
        if !create {
            return ::core::ptr::null_mut::<uint64_t>();
        }
        let mut new_pdpt: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if new_pdpt.is_null() {
            return ::core::ptr::null_mut::<uint64_t>();
        }
        let mut new_pdpt_phys: uint64_t = new_pdpt as uintptr_t as uint64_t;
        let mut zero_va: *mut ::core::ffi::c_void = win_map(new_pdpt_phys, PT_TEMP_IDX);
        if zero_va.is_null() {
            pmm_free_frame(new_pdpt);
            return ::core::ptr::null_mut::<uint64_t>();
        }
        crate::raw::string::memset(
            zero_va,
            0 as ::core::ffi::c_int,
            4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
        );
        (*pml4).0.entries[pml4_idx as usize] =
            (new_pdpt_phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_dir_entry_t;
    }
    let mut pdpt_entry: uint64_t = (*pml4).0.entries[pml4_idx as usize];
    let mut pdpt_phys: uint64_t = pdpt_entry & 0xfffffffff000 as uint64_t;
    let mut pdpt: *mut page_table = ::core::ptr::null_mut::<page_table>();
    if pdpt_phys == X86_64_PDPT_PHYS as uint64_t {
        pdpt = X86_64_PDPT0_VIRT;
    } else {
        pdpt = win_map(pdpt_phys, PT_TEMP_IDX) as *mut page_table;
    }
    let mut pdpde: uint64_t = (*pdpt).0.entries[pdpt_idx as usize];
    if pdpde & 1 as uint64_t == 0 {
        if !create {
            if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
                win_unmap(PT_TEMP_IDX);
            }
            return ::core::ptr::null_mut::<uint64_t>();
        }
        let mut new_pd: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if new_pd.is_null() {
            if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
                win_unmap(PT_TEMP_IDX);
            }
            return ::core::ptr::null_mut::<uint64_t>();
        }
        let mut new_pd_phys: uint64_t = new_pd as uintptr_t as uint64_t;
        let mut zero_va_0: *mut ::core::ffi::c_void =
            ::core::ptr::null_mut::<::core::ffi::c_void>();
        if pdpt_phys == X86_64_PDPT_PHYS as uint64_t {
            zero_va_0 = win_map(new_pd_phys, PT_TEMP_IDX);
        } else {
            zero_va_0 = win2_map(new_pd_phys, PT_TEMP_IDX);
        }
        if zero_va_0.is_null() {
            pmm_free_frame(new_pd);
            if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
                win_unmap(PT_TEMP_IDX);
            }
            return ::core::ptr::null_mut::<uint64_t>();
        }
        crate::raw::string::memset(
            zero_va_0,
            0 as ::core::ffi::c_int,
            4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
        );
        (*pdpt).0.entries[pdpt_idx as usize] =
            (new_pd_phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
        pdpde = (*pdpt).0.entries[pdpt_idx as usize] as uint64_t;
    }
    let mut pd_phys: uint64_t = pdpde & 0xfffffffff000 as uint64_t;
    let mut pd_tbl: *mut page_table = ::core::ptr::null_mut::<page_table>();
    if pd_phys == X86_64_PD_PHYS as uint64_t {
        pd_tbl = X86_64_PD_VIRT;
    } else {
        pd_tbl = win2_map(pd_phys, PT_TEMP_IDX) as *mut page_table;
    }
    let mut pde: uint64_t = (*pd_tbl).0.entries[pd_idx as usize];
    if pde & 1 as uint64_t == 0 {
        if !create {
            if pd_phys != X86_64_PD_PHYS as uint64_t {
                win2_unmap(PT_TEMP_IDX);
            }
            if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
                win_unmap(PT_TEMP_IDX);
            }
            return ::core::ptr::null_mut::<uint64_t>();
        }
        let mut new_pt: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if new_pt.is_null() {
            if pd_phys != X86_64_PD_PHYS as uint64_t {
                win2_unmap(PT_TEMP_IDX);
            }
            if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
                win_unmap(PT_TEMP_IDX);
            }
            return ::core::ptr::null_mut::<uint64_t>();
        }
        let mut new_pt_phys: uint64_t = new_pt as uintptr_t as uint64_t;
        let mut zero_va_1: *mut ::core::ffi::c_void = win_map(new_pt_phys, PT_TEMP_IDX);
        if zero_va_1.is_null() {
            pmm_free_frame(new_pt);
            if pd_phys != X86_64_PD_PHYS as uint64_t {
                win2_unmap(PT_TEMP_IDX);
            }
            if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
                win_unmap(PT_TEMP_IDX);
            }
            return ::core::ptr::null_mut::<uint64_t>();
        }
        crate::raw::string::memset(
            zero_va_1,
            0 as ::core::ffi::c_int,
            4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
        );
        (*pd_tbl).0.entries[pd_idx as usize] =
            (new_pt_phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
        pde = (*pd_tbl).0.entries[pd_idx as usize] as uint64_t;
    }
    let mut pt_phys: uint64_t = pde & 0xfffffffff000 as uint64_t;
    if pd_phys != X86_64_PD_PHYS as uint64_t {
        win2_unmap(PT_TEMP_IDX);
    }
    if pdpt_phys != X86_64_PDPT_PHYS as uint64_t {
        win_unmap(PT_TEMP_IDX);
    }
    let mut win_va: *mut ::core::ffi::c_void = win_map(pt_phys, PT_TEMP_IDX);
    if win_va.is_null() {
        return ::core::ptr::null_mut::<uint64_t>();
    }
    let mut pt: *mut page_table = win_va as *mut page_table;
    return (&raw mut (*pt).0.entries as *mut page_table_entry_t).offset(pt_idx as isize)
        as *mut uint64_t;
}
unsafe extern "C" fn invalidate_page(mut virt_addr: uint64_t) {
    asm!(
        "invlpg ({0})\n", inlateout(reg) virt_addr => _, options(preserves_flags,
        att_syntax)
    );
}
#[no_mangle]
pub unsafe extern "C" fn paging_map_page(
    mut virt_addr: uintptr_t,
    mut phys_addr: uintptr_t,
    mut flags: uint32_t,
) -> bool_0 {
    let mut pte: *mut uint64_t = get_page_entry(virt_addr as uint64_t, true_0 != 0);
    if pte.is_null() {
        return false_0 != 0;
    }
    *pte = phys_addr as uint64_t & 0xfffffffff000 as uint64_t
        | flags as uint64_t & 0xfff as uint64_t
        | 1 as uint64_t;
    invalidate_page(virt_addr as uint64_t);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paging_unmap_page(mut virt_addr: uintptr_t) {
    let mut pte: *mut uint64_t = get_page_entry(virt_addr as uint64_t, false_0 != 0);
    if !pte.is_null() {
        *pte = 0 as uint64_t;
        invalidate_page(virt_addr as uint64_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn paging_get_physical_address(mut virt_addr: uintptr_t) -> uintptr_t {
    let mut pte: *mut uint64_t = get_page_entry(virt_addr as uint64_t, false_0 != 0);
    if pte.is_null() || *pte & 1 as uint64_t == 0 {
        return 0 as uintptr_t;
    }
    return (*pte & 0xfffffffff000 as uint64_t | virt_addr as uint64_t & 0xfff as uint64_t)
        as uintptr_t;
}
#[no_mangle]
pub unsafe extern "C" fn paging_create_directory_phys() -> uintptr_t {
    let mut pml4_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if pml4_frame.is_null() {
        serial_print(
            b"Paging: ERROR - Failed to allocate PML4 frame\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return 0 as uintptr_t;
    }
    let mut new_pml4_phys: uint64_t = pml4_frame as uintptr_t as uint64_t;
    let mut pdpt_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if pdpt_frame.is_null() {
        pmm_free_frame(pml4_frame);
        serial_print(
            b"Paging: ERROR - Failed to allocate PDPT frame\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return 0 as uintptr_t;
    }
    let mut new_pdpt_phys: uint64_t = pdpt_frame as uintptr_t as uint64_t;
    let mut pd_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if pd_frame.is_null() {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        serial_print(
            b"Paging: ERROR - Failed to allocate PD frame\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return 0 as uintptr_t;
    }
    let mut new_pd_phys: uint64_t = pd_frame as uintptr_t as uint64_t;
    let mut pd_va: *mut ::core::ffi::c_void = win_map(new_pd_phys, PT_TEMP_IDX);
    if pd_va.is_null() {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        pmm_free_frame(pd_frame);
        return 0 as uintptr_t;
    }
    let mut new_pd: *mut page_directory = pd_va as *mut page_directory;
    let mut kern_pd: *mut page_directory = X86_64_PD_VIRT as *mut page_directory;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 512 as ::core::ffi::c_int {
        (*new_pd).0.entries[i as usize] = (*kern_pd).0.entries[i as usize];
        i += 1;
    }
    win_unmap(PT_TEMP_IDX);
    let mut pdpt_va: *mut ::core::ffi::c_void = win_map(new_pdpt_phys, PT_TEMP_IDX);
    if pdpt_va.is_null() {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        pmm_free_frame(pd_frame);
        return 0 as uintptr_t;
    }
    let mut new_pdpt: *mut page_directory = pdpt_va as *mut page_directory;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 512 as ::core::ffi::c_int {
        (*new_pdpt).0.entries[i_0 as usize] = 0 as page_dir_entry_t;
        i_0 += 1;
    }
    (*new_pdpt).0.entries[0 as ::core::ffi::c_int as usize] =
        (new_pd_phys & 0xfffffffff000 as uint64_t | 0x7 as uint64_t) as page_dir_entry_t;
    win_unmap(PT_TEMP_IDX);
    let mut pml4_va: *mut ::core::ffi::c_void = win_map(new_pml4_phys, PT_TEMP_IDX);
    if pml4_va.is_null() {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        pmm_free_frame(pd_frame);
        return 0 as uintptr_t;
    }
    let mut new_pml4: *mut page_directory = pml4_va as *mut page_directory;
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < 512 as ::core::ffi::c_int {
        (*new_pml4).0.entries[i_1 as usize] = 0 as page_dir_entry_t;
        i_1 += 1;
    }
    (*new_pml4).0.entries[0 as ::core::ffi::c_int as usize] =
        (new_pdpt_phys & 0xfffffffff000 as uint64_t | 0x7 as uint64_t) as page_dir_entry_t;
    win_unmap(PT_TEMP_IDX);
    return new_pml4_phys as uintptr_t;
}
#[no_mangle]
pub unsafe extern "C" fn paging_destroy_directory(mut pd_phys: uintptr_t) {
    if pd_phys == 0 {
        return;
    }
    serial_print(
        b"Paging: Destroying page directory (x86_64)\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    let mut win_va: *mut ::core::ffi::c_void = win_map(pd_phys as uint64_t, PT_TEMP_IDX);
    if win_va.is_null() {
        return;
    }
    let mut pml4: *mut page_directory = win_va as *mut page_directory;
    let mut pml4_i: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
    while pml4_i < 512 as ::core::ffi::c_int {
        let mut pml4e: uint64_t = (*pml4).0.entries[pml4_i as usize];
        if !(pml4e & 1 as uint64_t == 0) {
            let mut pdpt_phys: uint64_t = pml4e & 0xfffffffff000 as uint64_t;
            let mut pdpt_va: *mut ::core::ffi::c_void = win_map(pdpt_phys, PT_TEMP_IDX);
            let mut pdpt_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while pdpt_i < 512 as ::core::ffi::c_int {
                let mut pdpde: uint64_t = (*(win_va as *mut page_table)).0.entries[pdpt_i as usize];
                if !(pdpde & 1 as uint64_t == 0) {
                    let mut pd_phys_entry: uint64_t = pdpde & 0xfffffffff000 as uint64_t;
                    let mut pd_tbl: *mut page_table =
                        win_map(pd_phys_entry, PT_TEMP_IDX) as *mut page_table;
                    if !pd_tbl.is_null() {
                        let mut pd_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while pd_i < 512 as ::core::ffi::c_int {
                            let mut pd_entry: uint64_t = (*pd_tbl).0.entries[pd_i as usize];
                            if !(pd_entry & 1 as uint64_t == 0) {
                                if pd_entry & 0x80 as uint64_t != 0 {
                                    let mut frame: uint64_t = pd_entry & 0xfffffffff000 as uint64_t;
                                    pmm_free_frame(frame as uintptr_t as *mut ::core::ffi::c_void);
                                    (*pd_tbl).0.entries[pd_i as usize] = 0 as page_table_entry_t;
                                } else {
                                    let mut pt_phys: uint64_t =
                                        pd_entry & 0xfffffffff000 as uint64_t;
                                    let mut pt: *mut page_table =
                                        win_map(pt_phys, PT_TEMP_IDX) as *mut page_table;
                                    if !pt.is_null() {
                                        let mut pt_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while pt_i < 512 as ::core::ffi::c_int {
                                            let mut pte: uint64_t = (*pt).0.entries[pt_i as usize];
                                            if !(pte & 1 as uint64_t == 0) {
                                                let mut frame_0: uint64_t =
                                                    pte & 0xfffffffff000 as uint64_t;
                                                if pte & PAGE_COW as uint64_t != 0 {
                                                    pmm_refcount_dec(
                                                        frame_0 as uintptr_t
                                                            as *mut ::core::ffi::c_void,
                                                    );
                                                } else {
                                                    pmm_free_frame(
                                                        frame_0 as uintptr_t
                                                            as *mut ::core::ffi::c_void,
                                                    );
                                                }
                                                (*pt).0.entries[pt_i as usize] =
                                                    0 as page_table_entry_t;
                                            }
                                            pt_i += 1;
                                        }
                                        pmm_free_frame(
                                            pt_phys as uintptr_t as *mut ::core::ffi::c_void,
                                        );
                                        (*pd_tbl).0.entries[pd_i as usize] =
                                            0 as page_table_entry_t;
                                    }
                                }
                            }
                            pd_i += 1;
                        }
                        pmm_free_frame(pd_phys_entry as uintptr_t as *mut ::core::ffi::c_void);
                        (*(win_va as *mut page_table)).0.entries[pdpt_i as usize] =
                            0 as page_table_entry_t;
                    }
                }
                pdpt_i += 1;
            }
            pmm_free_frame(pdpt_phys as uintptr_t as *mut ::core::ffi::c_void);
            (*pml4).0.entries[pml4_i as usize] = 0 as page_dir_entry_t;
        }
        pml4_i += 1;
    }
    win_unmap(PT_TEMP_IDX);
    pmm_free_frame(pd_phys as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn paging_switch_to_directory(mut pd_phys: uintptr_t) -> bool_0 {
    if pd_phys == 0 {
        return false_0 != 0;
    }
    asm!(
        "mov {0}, %cr3\n", inlateout(reg) pd_phys as uint64_t => _,
        options(preserves_flags, att_syntax)
    );
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paging_get_kernel_directory_phys() -> uintptr_t {
    return kernel_pml4_phys as uintptr_t;
}
unsafe extern "C" fn clone_page_table(mut src_pt: *mut page_table, mut dst_pt: *mut page_table) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 512 as ::core::ffi::c_int {
        let mut pte: uint64_t = (*src_pt).0.entries[i as usize];
        if !(pte & 1 as uint64_t == 0) {
            let mut src_frame: uint64_t = pte & 0xfffffffff000 as uint64_t;
            let mut new_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
            if new_frame.is_null() {
                serial_print(
                    b"Paging: clone - OOM during page copy\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            } else {
                let mut new_frame_phys: uint64_t = new_frame as uintptr_t as uint64_t;
                let mut src_va: *mut ::core::ffi::c_void =
                    win2_map(src_frame, 1 as ::core::ffi::c_int);
                let mut dst_va: *mut ::core::ffi::c_void =
                    win_map(new_frame_phys, 1 as ::core::ffi::c_int);
                if !src_va.is_null() && !dst_va.is_null() {
                    crate::raw::string::memcpy(
                        dst_va,
                        src_va,
                        PAGE_SIZE as ::core::ffi::c_ulong as crate::raw::string::size_t,
                    );
                }
                win_unmap(1 as ::core::ffi::c_int);
                win2_unmap(1 as ::core::ffi::c_int);
                (*dst_pt).0.entries[i as usize] = (new_frame_phys & 0xfffffffff000 as uint64_t
                    | pte & 0xfff as uint64_t
                    | 1 as uint64_t)
                    as page_table_entry_t;
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn paging_clone_directory(mut pd_phys: uintptr_t) -> uintptr_t {
    let mut current_block: u64;
    serial_print(
        b"Paging: Cloning page directory (x86_64)\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut dst_pml4_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if dst_pml4_frame.is_null() {
        return 0 as uintptr_t;
    }
    let mut dst_pml4_phys: uint64_t = dst_pml4_frame as uintptr_t as uint64_t;
    let mut zero_va: *mut ::core::ffi::c_void = win_map(dst_pml4_phys, PT_TEMP_IDX);
    if zero_va.is_null() {
        pmm_free_frame(dst_pml4_frame);
        return 0 as uintptr_t;
    }
    crate::raw::string::memset(
        zero_va,
        0 as ::core::ffi::c_int,
        4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
    );
    win_unmap(PT_TEMP_IDX);
    let mut src_pml4: *mut page_directory =
        win_map(pd_phys as uint64_t, 0 as ::core::ffi::c_int) as *mut page_directory;
    let mut dst_pml4: *mut page_directory =
        win2_map(dst_pml4_phys, 0 as ::core::ffi::c_int) as *mut page_directory;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        (*dst_pml4).0.entries[i as usize] = (*src_pml4).0.entries[i as usize];
        i += 1;
    }
    let mut pml4_i: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
    's_54: loop {
        if !(pml4_i < 512 as ::core::ffi::c_int) {
            current_block = 6367734732029634840;
            break;
        }
        let mut pml4e: uint64_t = (*src_pml4).0.entries[pml4_i as usize];
        if !(pml4e & 1 as uint64_t == 0) {
            let mut src_pdpt_phys: uint64_t = pml4e & 0xfffffffff000 as uint64_t;
            let mut pdpt_flags: uint64_t = pml4e & 0xfff as uint64_t;
            let mut dst_pdpt_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
            if dst_pdpt_frame.is_null() {
                current_block = 18189442286432478671;
                break;
            }
            let mut dst_pdpt_phys: uint64_t = dst_pdpt_frame as uintptr_t as uint64_t;
            let mut zero_va2: *mut ::core::ffi::c_void =
                win2_map(dst_pdpt_phys, 1 as ::core::ffi::c_int);
            if zero_va2.is_null() {
                current_block = 18189442286432478671;
                break;
            }
            crate::raw::string::memset(
                zero_va2,
                0 as ::core::ffi::c_int,
                4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
            win2_unmap(1 as ::core::ffi::c_int);
            (*dst_pml4).0.entries[pml4_i as usize] =
                (dst_pdpt_phys & 0xfffffffff000 as uint64_t | pdpt_flags | 1 as uint64_t)
                    as page_dir_entry_t;
            win2_unmap(0 as ::core::ffi::c_int);
            win_unmap(0 as ::core::ffi::c_int);
            let mut src_pdpt: *mut page_table =
                win_map(src_pdpt_phys, 0 as ::core::ffi::c_int) as *mut page_table;
            let mut dst_pdpt: *mut page_table =
                win2_map(dst_pdpt_phys, 0 as ::core::ffi::c_int) as *mut page_table;
            let mut pdpt_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while pdpt_i < 512 as ::core::ffi::c_int {
                let mut pdpde: uint64_t = (*src_pdpt).0.entries[pdpt_i as usize];
                if !(pdpde & 1 as uint64_t == 0) {
                    let mut src_pd_phys: uint64_t = pdpde & 0xfffffffff000 as uint64_t;
                    let mut pd_flags: uint64_t = pdpde & 0xfff as uint64_t;
                    let mut dst_pd_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
                    if dst_pd_frame.is_null() {
                        current_block = 18189442286432478671;
                        break 's_54;
                    }
                    let mut dst_pd_phys: uint64_t = dst_pd_frame as uintptr_t as uint64_t;
                    let mut zero_va3: *mut ::core::ffi::c_void =
                        win2_map(dst_pd_phys, 1 as ::core::ffi::c_int);
                    if zero_va3.is_null() {
                        current_block = 18189442286432478671;
                        break 's_54;
                    }
                    crate::raw::string::memset(
                        zero_va3,
                        0 as ::core::ffi::c_int,
                        4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
                    );
                    win2_unmap(1 as ::core::ffi::c_int);
                    (*dst_pdpt).0.entries[pdpt_i as usize] =
                        (dst_pd_phys & 0xfffffffff000 as uint64_t | pd_flags | 1 as uint64_t)
                            as page_table_entry_t;
                    win2_unmap(0 as ::core::ffi::c_int);
                    win_unmap(0 as ::core::ffi::c_int);
                    let mut src_pd: *mut page_table =
                        win_map(src_pd_phys, 0 as ::core::ffi::c_int) as *mut page_table;
                    let mut dst_pd: *mut page_table =
                        win2_map(dst_pd_phys, 0 as ::core::ffi::c_int) as *mut page_table;
                    let mut pd_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while pd_i < 512 as ::core::ffi::c_int {
                        let mut pde: uint64_t = (*src_pd).0.entries[pd_i as usize];
                        if !(pde & 1 as uint64_t == 0) {
                            if pde & 0x80 as uint64_t != 0 {
                                let mut huge_base: uint64_t = pde & 0xfffffffff000 as uint64_t;
                                let mut src_flags: uint64_t = pde & 0xfff as uint64_t;
                                let mut dst_pt_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
                                if dst_pt_frame.is_null() {
                                    current_block = 18189442286432478671;
                                    break 's_54;
                                }
                                let mut dst_pt_phys: uint64_t =
                                    dst_pt_frame as uintptr_t as uint64_t;
                                let mut zero_va_pt: *mut ::core::ffi::c_void =
                                    win2_map(dst_pt_phys, 1 as ::core::ffi::c_int);
                                if zero_va_pt.is_null() {
                                    current_block = 18189442286432478671;
                                    break 's_54;
                                }
                                crate::raw::string::memset(
                                    zero_va_pt,
                                    0 as ::core::ffi::c_int,
                                    4096 as ::core::ffi::c_int as ::core::ffi::c_ulong
                                        as crate::raw::string::size_t,
                                );
                                win2_unmap(1 as ::core::ffi::c_int);
                                (*dst_pd).0.entries[pd_i as usize] = (dst_pt_phys
                                    & 0xfffffffff000 as uint64_t
                                    | src_flags & !(0x80 as uint64_t)
                                    | 1 as uint64_t)
                                    as page_table_entry_t;
                                win2_unmap(0 as ::core::ffi::c_int);
                                win_unmap(0 as ::core::ffi::c_int);
                                let mut dst_pt: *mut page_table =
                                    win_map(dst_pt_phys, 0 as ::core::ffi::c_int)
                                        as *mut page_table;
                                let mut pt_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while pt_i < 512 as ::core::ffi::c_int {
                                    let mut phys: uint64_t = huge_base.wrapping_add(
                                        (pt_i as uint64_t) << 12 as ::core::ffi::c_int,
                                    );
                                    let mut new_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
                                    if !new_frame.is_null() {
                                        let mut new_frame_phys: uint64_t =
                                            new_frame as uintptr_t as uint64_t;
                                        let mut src_va: *mut ::core::ffi::c_void =
                                            win_map(phys, 1 as ::core::ffi::c_int);
                                        let mut dst_va: *mut ::core::ffi::c_void =
                                            win2_map(new_frame_phys, 1 as ::core::ffi::c_int);
                                        if !src_va.is_null() && !dst_va.is_null() {
                                            crate::raw::string::memcpy(
                                                dst_va,
                                                src_va,
                                                PAGE_SIZE as ::core::ffi::c_ulong as crate::raw::string::size_t,
                                            );
                                        }
                                        win_unmap(1 as ::core::ffi::c_int);
                                        win2_unmap(1 as ::core::ffi::c_int);
                                        (*dst_pt).0.entries[pt_i as usize] = (new_frame_phys
                                            & 0xfffffffff000 as uint64_t
                                            | src_flags & (0xfff as uint64_t & !(0x80 as uint64_t))
                                            | 1 as uint64_t)
                                            as page_table_entry_t;
                                    }
                                    pt_i += 1;
                                }
                                win_unmap(0 as ::core::ffi::c_int);
                                win_map(src_pd_phys, 0 as ::core::ffi::c_int);
                                win2_map(dst_pd_phys, 0 as ::core::ffi::c_int);
                            } else {
                                let mut src_pt_phys: uint64_t = pde & 0xfffffffff000 as uint64_t;
                                let mut pt_flags: uint64_t = pde & 0xfff as uint64_t;
                                let mut dst_pt_frame_0: *mut ::core::ffi::c_void =
                                    pmm_alloc_frame();
                                if dst_pt_frame_0.is_null() {
                                    current_block = 18189442286432478671;
                                    break 's_54;
                                }
                                let mut dst_pt_phys_0: uint64_t =
                                    dst_pt_frame_0 as uintptr_t as uint64_t;
                                let mut zero_va_pt2: *mut ::core::ffi::c_void =
                                    win2_map(dst_pt_phys_0, 1 as ::core::ffi::c_int);
                                if zero_va_pt2.is_null() {
                                    current_block = 18189442286432478671;
                                    break 's_54;
                                }
                                crate::raw::string::memset(
                                    zero_va_pt2,
                                    0 as ::core::ffi::c_int,
                                    4096 as ::core::ffi::c_int as ::core::ffi::c_ulong
                                        as crate::raw::string::size_t,
                                );
                                win2_unmap(1 as ::core::ffi::c_int);
                                (*dst_pd).0.entries[pd_i as usize] = (dst_pt_phys_0
                                    & 0xfffffffff000 as uint64_t
                                    | pt_flags
                                    | 1 as uint64_t)
                                    as page_table_entry_t;
                                win2_unmap(0 as ::core::ffi::c_int);
                                win_unmap(0 as ::core::ffi::c_int);
                                let mut src_pt: *mut page_table =
                                    win_map(src_pt_phys, 0 as ::core::ffi::c_int)
                                        as *mut page_table;
                                let mut dst_pt_0: *mut page_table =
                                    win2_map(dst_pt_phys_0, 0 as ::core::ffi::c_int)
                                        as *mut page_table;
                                clone_page_table(src_pt, dst_pt_0);
                                win_unmap(0 as ::core::ffi::c_int);
                                win2_unmap(0 as ::core::ffi::c_int);
                                win_map(src_pd_phys, 0 as ::core::ffi::c_int);
                                win2_map(dst_pd_phys, 0 as ::core::ffi::c_int);
                            }
                        }
                        pd_i += 1;
                    }
                    win_unmap(0 as ::core::ffi::c_int);
                    win2_unmap(0 as ::core::ffi::c_int);
                    win_map(src_pdpt_phys, 0 as ::core::ffi::c_int);
                    win2_map(dst_pdpt_phys, 0 as ::core::ffi::c_int);
                }
                pdpt_i += 1;
            }
            win_unmap(0 as ::core::ffi::c_int);
            win2_unmap(0 as ::core::ffi::c_int);
            win_map(pd_phys as uint64_t, 0 as ::core::ffi::c_int);
            win2_map(dst_pml4_phys, 0 as ::core::ffi::c_int);
        }
        pml4_i += 1;
    }
    match current_block {
        18189442286432478671 => {
            serial_print(
                b"Paging: Clone failed - out of memory\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            pmm_free_frame(dst_pml4_phys as uintptr_t as *mut ::core::ffi::c_void);
            win_unmap(0 as ::core::ffi::c_int);
            win2_unmap(0 as ::core::ffi::c_int);
            return 0 as uintptr_t;
        }
        _ => {
            win_unmap(0 as ::core::ffi::c_int);
            win2_unmap(0 as ::core::ffi::c_int);
            serial_print(b"Paging: Clone complete\n\0" as *const u8 as *const ::core::ffi::c_char);
            return dst_pml4_phys as uintptr_t;
        }
    };
}
unsafe extern "C" fn fork_fork_pt(mut src_pt: *mut page_table, mut dst_pt: *mut page_table) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 512 as ::core::ffi::c_int {
        let mut pte: uint64_t = (*src_pt).0.entries[i as usize];
        if pte & 1 as uint64_t == 0 {
            (*dst_pt).0.entries[i as usize] = 0 as page_table_entry_t;
        } else {
            let mut frame_phys: uint64_t = pte & 0xfffffffff000 as uint64_t;
            let mut flags: uint64_t = pte & 0xfff as uint64_t;
            if flags & PAGE_WRITE as uint64_t != 0 {
                flags &= !PAGE_WRITE as uint64_t;
                flags |= PAGE_COW as uint64_t;
                pmm_refcount_inc(frame_phys as uintptr_t as *mut ::core::ffi::c_void);
                (*src_pt).0.entries[i as usize] = (frame_phys | flags) as page_table_entry_t;
                (*dst_pt).0.entries[i as usize] = (frame_phys | flags) as page_table_entry_t;
            } else {
                flags |= PAGE_COW as uint64_t;
                pmm_refcount_inc(frame_phys as uintptr_t as *mut ::core::ffi::c_void);
                (*src_pt).0.entries[i as usize] = (frame_phys | flags) as page_table_entry_t;
                (*dst_pt).0.entries[i as usize] = (frame_phys | flags) as page_table_entry_t;
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn paging_fork_directory(mut pd_phys: uintptr_t) -> uintptr_t {
    let mut current_block: u64;
    serial_print(
        b"Paging: Forking page directory (x86_64, COW)\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    let mut dst_pml4_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if dst_pml4_frame.is_null() {
        return 0 as uintptr_t;
    }
    let mut dst_pml4_phys: uint64_t = dst_pml4_frame as uintptr_t as uint64_t;
    let mut zero_va: *mut ::core::ffi::c_void = win_map(dst_pml4_phys, PT_TEMP_IDX);
    if zero_va.is_null() {
        pmm_free_frame(dst_pml4_frame);
        return 0 as uintptr_t;
    }
    crate::raw::string::memset(
        zero_va,
        0 as ::core::ffi::c_int,
        4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
    );
    win_unmap(PT_TEMP_IDX);
    let mut src_pml4: *mut page_directory =
        win_map(pd_phys as uint64_t, 0 as ::core::ffi::c_int) as *mut page_directory;
    let mut dst_pml4: *mut page_directory =
        win2_map(dst_pml4_phys, 0 as ::core::ffi::c_int) as *mut page_directory;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        (*dst_pml4).0.entries[i as usize] = (*src_pml4).0.entries[i as usize];
        i += 1;
    }
    let mut pml4_i: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
    's_54: loop {
        if !(pml4_i < 512 as ::core::ffi::c_int) {
            current_block = 10753070352654377903;
            break;
        }
        let mut pml4e: uint64_t = (*src_pml4).0.entries[pml4_i as usize];
        if !(pml4e & 1 as uint64_t == 0) {
            let mut src_pdpt_phys: uint64_t = pml4e & 0xfffffffff000 as uint64_t;
            let mut pdpt_flags: uint64_t = pml4e & 0xfff as uint64_t;
            let mut dst_pdpt_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
            if dst_pdpt_frame.is_null() {
                current_block = 15725482718582867333;
                break;
            }
            let mut dst_pdpt_phys: uint64_t = dst_pdpt_frame as uintptr_t as uint64_t;
            let mut zero_va2: *mut ::core::ffi::c_void =
                win2_map(dst_pdpt_phys, 1 as ::core::ffi::c_int);
            if zero_va2.is_null() {
                current_block = 15725482718582867333;
                break;
            }
            crate::raw::string::memset(
                zero_va2,
                0 as ::core::ffi::c_int,
                4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
            win2_unmap(1 as ::core::ffi::c_int);
            (*dst_pml4).0.entries[pml4_i as usize] =
                (dst_pdpt_phys & 0xfffffffff000 as uint64_t | pdpt_flags | 1 as uint64_t)
                    as page_dir_entry_t;
            win2_unmap(0 as ::core::ffi::c_int);
            win_unmap(0 as ::core::ffi::c_int);
            let mut src_pdpt: *mut page_table =
                win_map(src_pdpt_phys, 0 as ::core::ffi::c_int) as *mut page_table;
            let mut dst_pdpt: *mut page_table =
                win2_map(dst_pdpt_phys, 0 as ::core::ffi::c_int) as *mut page_table;
            let mut pdpt_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while pdpt_i < 512 as ::core::ffi::c_int {
                let mut pdpde: uint64_t = (*src_pdpt).0.entries[pdpt_i as usize];
                if !(pdpde & 1 as uint64_t == 0) {
                    let mut src_pd_phys: uint64_t = pdpde & 0xfffffffff000 as uint64_t;
                    let mut pd_flags: uint64_t = pdpde & 0xfff as uint64_t;
                    let mut dst_pd_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
                    if dst_pd_frame.is_null() {
                        current_block = 15725482718582867333;
                        break 's_54;
                    }
                    let mut dst_pd_phys: uint64_t = dst_pd_frame as uintptr_t as uint64_t;
                    let mut zero_va3: *mut ::core::ffi::c_void =
                        win2_map(dst_pd_phys, 1 as ::core::ffi::c_int);
                    if zero_va3.is_null() {
                        current_block = 15725482718582867333;
                        break 's_54;
                    }
                    crate::raw::string::memset(
                        zero_va3,
                        0 as ::core::ffi::c_int,
                        4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
                    );
                    win2_unmap(1 as ::core::ffi::c_int);
                    (*dst_pdpt).0.entries[pdpt_i as usize] =
                        (dst_pd_phys & 0xfffffffff000 as uint64_t | pd_flags | 1 as uint64_t)
                            as page_table_entry_t;
                    win2_unmap(0 as ::core::ffi::c_int);
                    win_unmap(0 as ::core::ffi::c_int);
                    let mut src_pd: *mut page_table =
                        win_map(src_pd_phys, 0 as ::core::ffi::c_int) as *mut page_table;
                    let mut dst_pd: *mut page_table =
                        win2_map(dst_pd_phys, 0 as ::core::ffi::c_int) as *mut page_table;
                    let mut pd_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while pd_i < 512 as ::core::ffi::c_int {
                        let mut pde: uint64_t = (*src_pd).0.entries[pd_i as usize];
                        if !(pde & 1 as uint64_t == 0) {
                            if pde & 0x80 as uint64_t != 0 {
                                (*dst_pd).0.entries[pd_i as usize] = pde as page_table_entry_t;
                            } else {
                                let mut src_pt_phys: uint64_t = pde & 0xfffffffff000 as uint64_t;
                                let mut pt_flags: uint64_t = pde & 0xfff as uint64_t;
                                let mut dst_pt_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
                                if dst_pt_frame.is_null() {
                                    current_block = 15725482718582867333;
                                    break 's_54;
                                }
                                let mut dst_pt_phys: uint64_t =
                                    dst_pt_frame as uintptr_t as uint64_t;
                                let mut zero_va_pt: *mut ::core::ffi::c_void =
                                    win2_map(dst_pt_phys, 1 as ::core::ffi::c_int);
                                if zero_va_pt.is_null() {
                                    current_block = 15725482718582867333;
                                    break 's_54;
                                }
                                crate::raw::string::memset(
                                    zero_va_pt,
                                    0 as ::core::ffi::c_int,
                                    4096 as ::core::ffi::c_int as ::core::ffi::c_ulong
                                        as crate::raw::string::size_t,
                                );
                                win2_unmap(1 as ::core::ffi::c_int);
                                (*dst_pd).0.entries[pd_i as usize] = (dst_pt_phys
                                    & 0xfffffffff000 as uint64_t
                                    | pt_flags
                                    | 1 as uint64_t)
                                    as page_table_entry_t;
                                win2_unmap(0 as ::core::ffi::c_int);
                                win_unmap(0 as ::core::ffi::c_int);
                                let mut src_pt: *mut page_table =
                                    win_map(src_pt_phys, 0 as ::core::ffi::c_int)
                                        as *mut page_table;
                                let mut dst_pt: *mut page_table =
                                    win2_map(dst_pt_phys, 0 as ::core::ffi::c_int)
                                        as *mut page_table;
                                fork_fork_pt(src_pt, dst_pt);
                                win_unmap(0 as ::core::ffi::c_int);
                                win2_unmap(0 as ::core::ffi::c_int);
                                win_map(src_pd_phys, 0 as ::core::ffi::c_int);
                                win2_map(dst_pd_phys, 0 as ::core::ffi::c_int);
                            }
                        }
                        pd_i += 1;
                    }
                    win_unmap(0 as ::core::ffi::c_int);
                    win2_unmap(0 as ::core::ffi::c_int);
                    win_map(src_pdpt_phys, 0 as ::core::ffi::c_int);
                    win2_map(dst_pdpt_phys, 0 as ::core::ffi::c_int);
                }
                pdpt_i += 1;
            }
            win_unmap(0 as ::core::ffi::c_int);
            win2_unmap(0 as ::core::ffi::c_int);
            win_map(pd_phys as uint64_t, 0 as ::core::ffi::c_int);
            win2_map(dst_pml4_phys, 0 as ::core::ffi::c_int);
        }
        pml4_i += 1;
    }
    match current_block {
        15725482718582867333 => {
            serial_print(
                b"Paging: Fork failed - out of memory\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            pmm_free_frame(dst_pml4_phys as uintptr_t as *mut ::core::ffi::c_void);
            win_unmap(0 as ::core::ffi::c_int);
            win2_unmap(0 as ::core::ffi::c_int);
            return 0 as uintptr_t;
        }
        _ => {
            win_unmap(0 as ::core::ffi::c_int);
            win2_unmap(0 as ::core::ffi::c_int);
            serial_print(
                b"Paging: Fork complete (COW)\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return dst_pml4_phys as uintptr_t;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn paging_handle_cow_fault(mut fault_addr: uintptr_t) -> uint8_t {
    let mut pte: *mut uint64_t = get_page_entry(fault_addr as uint64_t, false_0 != 0);
    if pte.is_null() || *pte & 1 as uint64_t == 0 {
        return 0 as uint8_t;
    }
    let mut flags: uint64_t = *pte & 0xfff as uint64_t;
    if flags & PAGE_COW as uint64_t == 0 {
        return 0 as uint8_t;
    }
    let mut old_frame: uint64_t = *pte & 0xfffffffff000 as uint64_t;
    let mut new_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if new_frame.is_null() {
        serial_print(
            b"Paging: COW - out of memory!\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as uint8_t;
    }
    let mut new_frame_phys: uint64_t = new_frame as uintptr_t as uint64_t;
    let mut cow_src: *mut ::core::ffi::c_void = win_map(old_frame, 1 as ::core::ffi::c_int);
    let mut cow_dst: *mut ::core::ffi::c_void = win2_map(new_frame_phys, 1 as ::core::ffi::c_int);
    if !cow_src.is_null() && !cow_dst.is_null() {
        crate::raw::string::memcpy(
            cow_dst,
            cow_src,
            PAGE_SIZE as ::core::ffi::c_ulong as crate::raw::string::size_t,
        );
    }
    win_unmap(1 as ::core::ffi::c_int);
    win2_unmap(1 as ::core::ffi::c_int);
    pmm_refcount_dec(old_frame as uintptr_t as *mut ::core::ffi::c_void);
    flags &= !PAGE_COW as uint64_t;
    flags |= PAGE_WRITE as uint64_t;
    *pte = new_frame as uintptr_t as uint64_t & 0xfffffffff000 as uint64_t | flags;
    invalidate_page(fault_addr as uint64_t);
    return 1 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn paging_map_page_in_pd(
    mut pd_phys: uintptr_t,
    mut virt_addr: uintptr_t,
    mut phys_addr: uintptr_t,
    mut flags: uint32_t,
) -> bool_0 {
    let mut pml4: *mut page_directory = ::core::ptr::null_mut::<page_directory>();
    let mut vaddr: uint64_t = 0;
    let mut pml4_idx: uint64_t = 0;
    let mut pdpt_idx: uint64_t = 0;
    let mut pd_idx: uint64_t = 0;
    let mut pt_idx: uint64_t = 0;
    let mut pdpt_phys: uint64_t = 0;
    let mut pdpt_va: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pdpt: *mut page_table = ::core::ptr::null_mut::<page_table>();
    let mut pd_phys_tbl: uint64_t = 0;
    let mut pd_va: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pd_tbl: *mut page_table = ::core::ptr::null_mut::<page_table>();
    let mut pt_phys: uint64_t = 0;
    let mut pt_va: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pt: *mut page_table = ::core::ptr::null_mut::<page_table>();
    let mut current_block: u64;
    let mut result: bool_0 = false_0 != 0;
    let mut win_va: *mut ::core::ffi::c_void = win_map(pd_phys as uint64_t, PT_TEMP_IDX);
    if !win_va.is_null() {
        pml4 = win_va as *mut page_directory;
        vaddr = virt_addr as uint64_t;
        pml4_idx = vaddr >> 39 as ::core::ffi::c_int & 0x1ff as uint64_t;
        pdpt_idx = vaddr >> 30 as ::core::ffi::c_int & 0x1ff as uint64_t;
        pd_idx = vaddr >> 21 as ::core::ffi::c_int & 0x1ff as uint64_t;
        pt_idx = vaddr >> 12 as ::core::ffi::c_int & 0x1ff as uint64_t;
        if (*pml4).0.entries[pml4_idx as usize] & 1 as page_dir_entry_t == 0 {
            let mut new_pdpt: *mut ::core::ffi::c_void = pmm_alloc_frame();
            if new_pdpt.is_null() {
                win_unmap(PT_TEMP_IDX);
                current_block = 6138325815142479848;
            } else {
                let mut new_pdpt_phys: uint64_t = new_pdpt as uintptr_t as uint64_t;
                win_unmap(PT_TEMP_IDX);
                let mut zero_va: *mut ::core::ffi::c_void = win_map(new_pdpt_phys, PT_TEMP_IDX);
                if zero_va.is_null() {
                    pmm_free_frame(new_pdpt);
                    current_block = 6138325815142479848;
                } else {
                    crate::raw::string::memset(
                        zero_va,
                        0 as ::core::ffi::c_int,
                        4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
                    );
                    win_unmap(PT_TEMP_IDX);
                    pml4 = win_map(pd_phys as uint64_t, PT_TEMP_IDX) as *mut page_directory;
                    if pml4.is_null() {
                        pmm_free_frame(new_pdpt);
                        current_block = 6138325815142479848;
                    } else {
                        (*pml4).0.entries[pml4_idx as usize] = (new_pdpt_phys
                            & 0xfffffffff000 as uint64_t
                            | 0x3 as uint64_t
                            | (flags & PAGE_USER as uint32_t) as uint64_t)
                            as page_dir_entry_t;
                        current_block = 17407779659766490442;
                    }
                }
            }
        } else {
            current_block = 17407779659766490442;
        }
        match current_block {
            6138325815142479848 => {}
            _ => {
                pdpt_phys = (*pml4).0.entries[pml4_idx as usize] & 0xfffffffff000 as uint64_t;
                win_unmap(PT_TEMP_IDX);
                pdpt_va = win_map(pdpt_phys, PT_TEMP_IDX);
                if !pdpt_va.is_null() {
                    pdpt = pdpt_va as *mut page_table;
                    if (*pdpt).0.entries[pdpt_idx as usize] & 1 as page_table_entry_t == 0 {
                        let mut new_pd: *mut ::core::ffi::c_void = pmm_alloc_frame();
                        if new_pd.is_null() {
                            win_unmap(PT_TEMP_IDX);
                            current_block = 6138325815142479848;
                        } else {
                            let mut new_pd_phys: uint64_t = new_pd as uintptr_t as uint64_t;
                            win_unmap(PT_TEMP_IDX);
                            let mut zero_va_0: *mut ::core::ffi::c_void =
                                win_map(new_pd_phys, PT_TEMP_IDX);
                            if zero_va_0.is_null() {
                                pmm_free_frame(new_pd);
                                current_block = 6138325815142479848;
                            } else {
                                crate::raw::string::memset(
                                    zero_va_0,
                                    0 as ::core::ffi::c_int,
                                    4096 as ::core::ffi::c_int as ::core::ffi::c_ulong
                                        as crate::raw::string::size_t,
                                );
                                win_unmap(PT_TEMP_IDX);
                                pdpt = win_map(pdpt_phys, PT_TEMP_IDX) as *mut page_table;
                                if pdpt.is_null() {
                                    pmm_free_frame(new_pd);
                                    current_block = 6138325815142479848;
                                } else {
                                    (*pdpt).0.entries[pdpt_idx as usize] = (new_pd_phys
                                        & 0xfffffffff000 as uint64_t
                                        | 0x3 as uint64_t
                                        | (flags & PAGE_USER as uint32_t) as uint64_t)
                                        as page_table_entry_t;
                                    current_block = 7205609094909031804;
                                }
                            }
                        }
                    } else {
                        current_block = 7205609094909031804;
                    }
                    match current_block {
                        6138325815142479848 => {}
                        _ => {
                            pd_phys_tbl =
                                (*pdpt).0.entries[pdpt_idx as usize] & 0xfffffffff000 as uint64_t;
                            win_unmap(PT_TEMP_IDX);
                            pd_va = win_map(pd_phys_tbl, PT_TEMP_IDX);
                            if !pd_va.is_null() {
                                pd_tbl = pd_va as *mut page_table;
                                if (*pd_tbl).0.entries[pd_idx as usize] & 1 as page_table_entry_t
                                    == 0
                                {
                                    let mut new_pt: *mut ::core::ffi::c_void = pmm_alloc_frame();
                                    if new_pt.is_null() {
                                        win_unmap(PT_TEMP_IDX);
                                        current_block = 6138325815142479848;
                                    } else {
                                        let mut new_pt_phys: uint64_t =
                                            new_pt as uintptr_t as uint64_t;
                                        win_unmap(PT_TEMP_IDX);
                                        let mut zero_va_1: *mut ::core::ffi::c_void =
                                            win_map(new_pt_phys, PT_TEMP_IDX);
                                        if zero_va_1.is_null() {
                                            pmm_free_frame(new_pt);
                                            current_block = 6138325815142479848;
                                        } else {
                                            crate::raw::string::memset(
                                                zero_va_1,
                                                0 as ::core::ffi::c_int,
                                                4096 as ::core::ffi::c_int as ::core::ffi::c_ulong
                                                    as crate::raw::string::size_t,
                                            );
                                            win_unmap(PT_TEMP_IDX);
                                            pd_tbl = win_map(pd_phys_tbl, PT_TEMP_IDX)
                                                as *mut page_table;
                                            if pd_tbl.is_null() {
                                                pmm_free_frame(new_pt);
                                                current_block = 6138325815142479848;
                                            } else {
                                                (*pd_tbl).0.entries[pd_idx as usize] = (new_pt_phys
                                                    & 0xfffffffff000 as uint64_t
                                                    | 0x3 as uint64_t
                                                    | (flags & PAGE_USER as uint32_t) as uint64_t)
                                                    as page_table_entry_t;
                                                current_block = 3546145585875536353;
                                            }
                                        }
                                    }
                                } else if (*pd_tbl).0.entries[pd_idx as usize]
                                    & 0x80 as page_table_entry_t
                                    != 0
                                {
                                    let mut huge_phys_base: uint64_t = (*pd_tbl).0.entries
                                        [pd_idx as usize]
                                        & 0xfffffffffe00 as uint64_t;
                                    let mut huge_flags: uint64_t =
                                        (*pd_tbl).0.entries[pd_idx as usize] & 0xfff as uint64_t;
                                    let mut pt_flags: uint64_t = huge_flags & !(0x80 as uint64_t);
                                    let mut new_pt_0: *mut ::core::ffi::c_void = pmm_alloc_frame();
                                    if new_pt_0.is_null() {
                                        win_unmap(PT_TEMP_IDX);
                                        current_block = 6138325815142479848;
                                    } else {
                                        let mut new_pt_phys_0: uint64_t =
                                            new_pt_0 as uintptr_t as uint64_t;
                                        win_unmap(PT_TEMP_IDX);
                                        let mut pt_fill_va: *mut ::core::ffi::c_void =
                                            win_map(new_pt_phys_0, PT_TEMP_IDX);
                                        if pt_fill_va.is_null() {
                                            pmm_free_frame(new_pt_0);
                                            current_block = 6138325815142479848;
                                        } else {
                                            let mut new_pt_tbl: *mut page_table =
                                                pt_fill_va as *mut page_table;
                                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                            while i < 512 as ::core::ffi::c_int {
                                                (*new_pt_tbl).0.entries[i as usize] =
                                                    0 as page_table_entry_t;
                                                i += 1;
                                            }
                                            win_unmap(PT_TEMP_IDX);
                                            pd_tbl = win_map(pd_phys_tbl, PT_TEMP_IDX)
                                                as *mut page_table;
                                            if pd_tbl.is_null() {
                                                pmm_free_frame(new_pt_0);
                                                current_block = 6138325815142479848;
                                            } else {
                                                (*pd_tbl).0.entries[pd_idx as usize] =
                                                    (new_pt_phys_0 & 0xfffffffff000 as uint64_t
                                                        | pt_flags & !(0x80 as uint64_t)
                                                        | (flags & PAGE_USER as uint32_t)
                                                            as uint64_t)
                                                        as page_table_entry_t;
                                                asm!(
                                                    "mov %cr3, %rax; mov %rax, %cr3\n", out("rax") _,
                                                    options(preserves_flags, att_syntax)
                                                );
                                                current_block = 3546145585875536353;
                                            }
                                        }
                                    }
                                } else {
                                    current_block = 3546145585875536353;
                                }
                                match current_block {
                                    6138325815142479848 => {}
                                    _ => {
                                        pt_phys = (*pd_tbl).0.entries[pd_idx as usize]
                                            & 0xfffffffff000 as uint64_t;
                                        win_unmap(PT_TEMP_IDX);
                                        pt_va = win_map(pt_phys, PT_TEMP_IDX);
                                        if !pt_va.is_null() {
                                            pt = pt_va as *mut page_table;
                                            (*pt).0.entries[pt_idx as usize] =
                                                (phys_addr as uint64_t & 0xfffffffff000 as uint64_t
                                                    | flags as uint64_t & 0xfff as uint64_t
                                                    | 1 as uint64_t)
                                                    as page_table_entry_t;
                                            win_unmap(PT_TEMP_IDX);
                                            invlpg(virt_addr as uint64_t);
                                            result = true_0 != 0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn paging_dump_user_pt(mut cr3: uint64_t, mut fault_addr: uint64_t) {
    let mut pml4_phys: uint64_t = cr3;
    let mut pml4: *mut uint64_t = win_map(pml4_phys, PT_TEMP_IDX) as *mut uint64_t;
    if pml4.is_null() {
        serial_print(
            b"  [dump] Cannot map user PML4\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut pml4e0: uint64_t = *pml4.offset(0 as ::core::ffi::c_int as isize);
    asm!("\n", options(preserves_flags, att_syntax));
    win_unmap(PT_TEMP_IDX);
    let mut pdpt_phys: uint64_t = pml4e0 & 0xffffffffffff0 as uint64_t;
    let mut pdpt: *mut uint64_t = win_map(pdpt_phys, PT_TEMP_IDX) as *mut uint64_t;
    if pdpt.is_null() {
        serial_print(
            b"  [dump] Cannot map user PDPT\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut pdpte0: uint64_t = *pdpt.offset(0 as ::core::ffi::c_int as isize);
    asm!("\n", options(preserves_flags, att_syntax));
    win_unmap(PT_TEMP_IDX);
    let mut pd_phys: uint64_t = pdpte0 & 0xffffffffffff0 as uint64_t;
    let mut pd: *mut uint64_t = win_map(pd_phys, PT_TEMP_IDX) as *mut uint64_t;
    if pd.is_null() {
        serial_print(b"  [dump] Cannot map user PD\n\0" as *const u8 as *const ::core::ffi::c_char);
        return;
    }
    let mut pd_idx: uint64_t = fault_addr >> 21 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pde_val: uint64_t = *pd.offset(pd_idx as isize);
    asm!("\n", options(preserves_flags, att_syntax));
    win_unmap(PT_TEMP_IDX);
    serial_print(b"  PML4[0]=0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(pml4e0);
    serial_print(b" PDPT[0]=0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(pdpte0);
    serial_print(b" PD[\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(pd_idx as uint32_t);
    serial_print(b"]=0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(pde_val);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn paging_demand_map_kernel_page(
    mut fault_addr: uint64_t,
    mut user_cr3: uint64_t,
) -> bool_0 {
    let mut pml4_idx: uint64_t = fault_addr >> 39 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pdpt_idx: uint64_t = fault_addr >> 30 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pd_idx: uint64_t = fault_addr >> 21 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pt_idx: uint64_t = fault_addr >> 12 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pml4: *mut uint64_t = X86_64_PML4_PHYS as uintptr_t as *mut uint64_t;
    let mut pml4e: uint64_t = *pml4.offset(pml4_idx as isize);
    if pml4e & 1 as uint64_t == 0 {
        return false_0 != 0;
    }
    let mut pdpt_phys: uint64_t = pml4e & 0xfffffffff000 as uint64_t;
    let mut pdpt: *mut uint64_t = pdpt_phys as uintptr_t as *mut uint64_t;
    let mut pdpte: uint64_t = *pdpt.offset(pdpt_idx as isize);
    if pdpte & 1 as uint64_t == 0 {
        return false_0 != 0;
    }
    if pdpte & 0x80 as uint64_t != 0 {
        let mut phys_frame: uint64_t = pdpte & 0xffffffffc00000 as uint64_t;
        let mut page_addr: uint64_t = fault_addr & !(0xfff as uint64_t);
        return paging_map_page_in_pd(
            user_cr3 as uintptr_t,
            page_addr as uintptr_t,
            (phys_frame as uintptr_t)
                .wrapping_add(fault_addr as uintptr_t & 0x3fffffff as uintptr_t),
            (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
        );
    }
    let mut pd_phys_addr: uint64_t = pdpte & 0xfffffffff000 as uint64_t;
    let mut pd: *mut uint64_t = pd_phys_addr as uintptr_t as *mut uint64_t;
    let mut pde: uint64_t = *pd.offset(pd_idx as isize);
    if pde & 1 as uint64_t == 0 {
        return false_0 != 0;
    }
    if pde & 0x80 as uint64_t != 0 {
        let mut phys_frame_0: uint64_t = pde & 0xfffffffffe00 as uint64_t;
        let mut page_addr_0: uint64_t = fault_addr & !(0xfff as uint64_t);
        return paging_map_page_in_pd(
            user_cr3 as uintptr_t,
            page_addr_0 as uintptr_t,
            (phys_frame_0 as uintptr_t)
                .wrapping_add(fault_addr as uintptr_t & 0x1fffff as uintptr_t),
            (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
        );
    }
    let mut pt_phys: uint64_t = pde & 0xfffffffff000 as uint64_t;
    let mut pt: *mut uint64_t = win_map(pt_phys, PT_TEMP_IDX) as *mut uint64_t;
    if pt.is_null() {
        return false_0 != 0;
    }
    let mut pte: uint64_t = *pt.offset(pt_idx as isize);
    asm!("\n", options(preserves_flags, att_syntax));
    win_unmap(PT_TEMP_IDX);
    if pte & 1 as uint64_t == 0 {
        return false_0 != 0;
    }
    let mut phys_frame_1: uint64_t = pte & 0xfffffffff000 as uint64_t;
    let mut page_addr_1: uint64_t = fault_addr & !(0xfff as uint64_t);
    return paging_map_page_in_pd(
        user_cr3 as uintptr_t,
        page_addr_1 as uintptr_t,
        phys_frame_1 as uintptr_t,
        (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn paging_demand_alloc_kernel_page(mut fault_addr: uint64_t) -> bool_0 {
    let mut page_addr: uint64_t = fault_addr & !(0xfff as uint64_t);
    let mut pd_idx: uint64_t = page_addr >> 21 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut pt_idx: uint64_t = page_addr >> 12 as ::core::ffi::c_int & 0x1ff as uint64_t;
    let mut frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if frame.is_null() {
        return false_0 != 0;
    }
    let mut phys: uint64_t = frame as uintptr_t as uint64_t;
    let mut zva: *mut ::core::ffi::c_void = win_map(phys, PT_TEMP_IDX);
    if !zva.is_null() {
        crate::raw::string::memset(
            zva,
            0 as ::core::ffi::c_int,
            4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
        );
        win_unmap(PT_TEMP_IDX);
    }
    let mut pd: *mut page_table = X86_64_PD_PHYS as uintptr_t as *mut page_table;
    let mut pde: uint64_t = (*pd).0.entries[pd_idx as usize];
    if pde & 1 as uint64_t == 0 as uint64_t {
        let mut pt_frame: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if pt_frame.is_null() {
            pmm_free_frame(frame);
            return false_0 != 0;
        }
        let mut pt_phys: uint64_t = pt_frame as uintptr_t as uint64_t;
        let mut zva2: *mut ::core::ffi::c_void = win_map(pt_phys, PT_TEMP_IDX);
        if zva2.is_null() {
            pmm_free_frame(frame);
            pmm_free_frame(pt_frame);
            return false_0 != 0;
        }
        crate::raw::string::memset(
            zva2,
            0 as ::core::ffi::c_int,
            4096 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
        );
        win_unmap(PT_TEMP_IDX);
        (*pd).0.entries[pd_idx as usize] =
            (pt_phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
        asm!(
            "mov %cr3, %rax; mov %rax, %cr3\n", out("rax") _, options(preserves_flags,
            att_syntax)
        );
        pde = (*pd).0.entries[pd_idx as usize] as uint64_t;
    } else if pde & 0x80 as uint64_t != 0 {
        let mut huge_base: uint64_t = pde & 0xfffffffffe00 as uint64_t;
        let mut huge_flags: uint64_t = pde & 0xfff as uint64_t;
        let mut pt_flags: uint64_t = huge_flags & !(0x80 as uint64_t);
        let mut pt_frame_0: *mut ::core::ffi::c_void = pmm_alloc_frame();
        if pt_frame_0.is_null() {
            pmm_free_frame(frame);
            return false_0 != 0;
        }
        let mut pt_phys_0: uint64_t = pt_frame_0 as uintptr_t as uint64_t;
        let mut pt_fill_va: *mut ::core::ffi::c_void = win_map(pt_phys_0, PT_TEMP_IDX);
        if pt_fill_va.is_null() {
            pmm_free_frame(frame);
            pmm_free_frame(pt_frame_0);
            return false_0 != 0;
        }
        let mut pt_tbl: *mut page_table = pt_fill_va as *mut page_table;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 512 as ::core::ffi::c_int {
            (*pt_tbl).0.entries[i as usize] =
                (huge_base.wrapping_add((i as uint64_t) << 12 as ::core::ffi::c_int)
                    | pt_flags & !(0x4 as uint64_t)
                    | 1 as uint64_t) as page_table_entry_t;
            i += 1;
        }
        win_unmap(PT_TEMP_IDX);
        (*pd).0.entries[pd_idx as usize] = (pt_phys_0 & 0xfffffffff000 as uint64_t
            | pt_flags & !(0x4 as uint64_t))
            as page_table_entry_t;
        asm!(
            "mov %cr3, %rax; mov %rax, %cr3\n", out("rax") _, options(preserves_flags,
            att_syntax)
        );
        pde = (*pd).0.entries[pd_idx as usize] as uint64_t;
    }
    let mut pt_phys_1: uint64_t = pde & 0xfffffffff000 as uint64_t;
    if pt_phys_1 == 0 {
        pmm_free_frame(frame);
        return false_0 != 0;
    }
    let mut pt_va: *mut ::core::ffi::c_void = win_map(pt_phys_1, PT_TEMP_IDX);
    if pt_va.is_null() {
        pmm_free_frame(frame);
        return false_0 != 0;
    }
    let mut pt: *mut page_table = pt_va as *mut page_table;
    (*pt).0.entries[pt_idx as usize] =
        (phys & 0xfffffffff000 as uint64_t | 0x3 as uint64_t) as page_table_entry_t;
    win_unmap(PT_TEMP_IDX);
    invlpg(page_addr);
    asm!(
        "mov %cr3, %rax; mov %rax, %cr3\n", out("rax") _, options(preserves_flags,
        att_syntax)
    );
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paging_temp_map_frame(
    mut phys_addr: uintptr_t,
) -> *mut ::core::ffi::c_void {
    return win_map(phys_addr as uint64_t, 511 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn paging_temp_unmap_frame() {
    win_unmap(511 as ::core::ffi::c_int);
}
