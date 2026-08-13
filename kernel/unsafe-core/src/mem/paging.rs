//! Hand-written x86_64 paging.
//!
//! Replaces `ported/x86_64/mm/paging.rs`. This is a faithful re-port of the
//! translated C: the fixed PML4/PDPT/PD frames at phys 0x1000/0x2000/0x3000,
//! the identity map of PD[2..15], the two shared temp windows (PD[10]/PD[11])
//! with their global-access PTs (PD[12]), page-table creation/destruction,
//! deep clone and COW fork, and the demand-map fault paths all keep their
//! exact machine behavior — the boot asm, the ported IDT fault handler and
//! `raw::ffi` resolve against the same symbols as before.
//!
//! Only the `kernel_tt_l0`-style table wrappers are written by hand; the
//! memory layout and the temp-window tricks are the C contract.

#![allow(non_upper_case_globals)]

use core::ffi::c_void;

use crate::drivers::serial::Serial;
use crate::raw::asm::x86_64::{invlpg, read_cr3, write_cr3};
use crate::raw::string;
use crate::mem::pmm;

pub const PAGE_PRESENT: u64 = 0x1;
pub const PAGE_WRITE: u64 = 0x2;
pub const PAGE_USER: u64 = 0x4;
pub const PAGE_COW: u64 = 0x200;
pub const PAGE_SIZE: usize = 4096;

const PTE_ADDR_MASK: u64 = 0x000f_ffff_f000; // 40-bit physical frame mask

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

#[repr(C)]
pub struct Paging {
    pub kernel_directory: *mut PageTable,
    pub kernel_tables: [*mut PageTable; 512],
}

#[no_mangle]
pub static mut g_paging: Paging = Paging {
    kernel_directory: core::ptr::null_mut(),
    kernel_tables: [core::ptr::null_mut(); 512],
};

// Fixed frames laid down by the boot code / C boot page tables.
pub const X86_64_PML4_PHYS: u64 = 0x1000;
pub const X86_64_PDPT_PHYS: u64 = 0x2000;
pub const X86_64_PD_PHYS: u64 = 0x3000;

// Temp-window indexes in the kernel PD.
pub const PD_WIN_IDX: usize = 10;
pub const PT_WIN_BASE: u64 = (PD_WIN_IDX as u64) << 21;
pub const PT_TEMP_IDX: usize = 0;
pub const PD_WIN2_IDX: usize = 11;
pub const PT_WIN2_BASE: u64 = (PD_WIN2_IDX as u64) << 21;
pub const PD_GWIN_IDX: usize = 12;
pub const G_WIN_PT_VA: u64 = (PD_GWIN_IDX as u64) << 21;
pub const G_WIN2_PT_VA: u64 = G_WIN_PT_VA + 0x1000;

#[no_mangle]
pub static mut kernel_pml4_phys: u64 = X86_64_PML4_PHYS;
#[no_mangle]
pub static mut g_current_user_cr3: u64 = X86_64_PML4_PHYS;
#[no_mangle]
pub static mut g_saved_user_cr3: u64 = X86_64_PML4_PHYS;

static mut g_win_pt_phys_addr: u64 = 0;
static mut g_win2_pt_phys_addr: u64 = 0;

// ----------------------------------------------------------------------------
// Fixed-window helpers
// ----------------------------------------------------------------------------

#[inline]
unsafe fn pml4() -> &'static mut PageTable {
    &mut *(X86_64_PML4_PHYS as *mut PageTable)
}

#[inline]
unsafe fn pdpt0() -> &'static mut PageTable {
    &mut *(X86_64_PDPT_PHYS as *mut PageTable)
}

#[inline]
unsafe fn pd() -> &'static mut PageTable {
    &mut *(X86_64_PD_PHYS as *mut PageTable)
}

#[inline]
unsafe fn g_win_pt() -> &'static mut PageTable {
    &mut *(G_WIN_PT_VA as *mut PageTable)
}

#[inline]
unsafe fn g_win2_pt() -> &'static mut PageTable {
    &mut *(G_WIN2_PT_VA as *mut PageTable)
}

#[inline]
unsafe fn flush_tlb() {
    // Reload CR3 to itself to flush the whole TLB.
    write_cr3(read_cr3());
}

/// Map a physical frame into window 1 (PD[10]) at `pt_idx`, returning its VA.
unsafe fn win_map(phys: u64, pt_idx: usize) -> *mut c_void {
    // User directories copy the kernel PD half, so PD[10] already points at
    // the window PT. Only (re)install it while the kernel PML4 is active —
    // `pd()` (VA 0x3000) is identity-mapped solely under the kernel CR3, and
    // `copy_from_user`/`copy_to_user` walk the page tables under the user CR3.
    if read_cr3() & PTE_ADDR_MASK == kernel_pml4_phys {
        let pd = pd();
        let old = pd.entries[PD_WIN_IDX];
        if old & PAGE_PRESENT == 0 || old & 0x80 != 0 {
            pd.entries[PD_WIN_IDX] = (g_win_pt_phys_addr & PTE_ADDR_MASK) | 0x3;
            invlpg(PT_WIN_BASE as usize);
        }
    }
    let va = PT_WIN_BASE + (pt_idx as u64) * 4096;
    g_win_pt().entries[pt_idx] = (phys & PTE_ADDR_MASK) | 0x3;
    invlpg(va as usize);
    va as usize as *mut c_void
}

/// Unmap window 1 slot `pt_idx`, leaving it identity-mapped (unused).
unsafe fn win_unmap(pt_idx: usize) {
    let va = PT_WIN_BASE + (pt_idx as u64) * 4096;
    g_win_pt().entries[pt_idx] = (va & PTE_ADDR_MASK) | 0x3;
    invlpg(va as usize);
}

/// Map a physical frame into window 2 (PD[11]) at `pt_idx`, returning its VA.
unsafe fn win2_map(phys: u64, pt_idx: usize) -> *mut c_void {
    let pd = pd();
    pd.entries[PD_WIN2_IDX] = (g_win2_pt_phys_addr & PTE_ADDR_MASK) | 0x3;
    invlpg(PT_WIN2_BASE as usize);
    let va = PT_WIN2_BASE + (pt_idx as u64) * 4096;
    g_win2_pt().entries[pt_idx] = (phys & PTE_ADDR_MASK) | 0x3;
    invlpg(va as usize);
    va as usize as *mut c_void
}

/// Unmap window 2 slot `pt_idx`, leaving it identity-mapped (unused).
unsafe fn win2_unmap(pt_idx: usize) {
    let va = PT_WIN2_BASE + (pt_idx as u64) * 4096;
    g_win2_pt().entries[pt_idx] = (va & PTE_ADDR_MASK) | 0x3;
    invlpg(va as usize);
}

// ----------------------------------------------------------------------------
// Init / enable
// ----------------------------------------------------------------------------

/// `paging_init()`: build the kernel temp windows on top of the boot page
/// tables (PML4/PDPT/PD at phys 0x1000/0x2000/0x3000).
#[no_mangle]
pub unsafe extern "C" fn paging_init() {
    Serial::write_str("Paging: Initializing x86_64 paging...\n");

    kernel_pml4_phys = read_cr3() & PTE_ADDR_MASK;
    Serial::write_str("  PML4 at phys 0x");
    Serial::write_hex64(kernel_pml4_phys);
    Serial::write_str("\n");

    g_paging.kernel_directory = X86_64_PML4_PHYS as *mut PageTable;

    // Identity-map physical 4 MiB..32 MiB (PD[2..15]) as 2 MiB pages.
    let pd_tbl = pd();
    for i in 2..16 {
        pd_tbl.entries[i] = ((i as u64) << 21) | 0x83;
    }

    // Window 1 page table: identity-maps PT_WIN_BASE..PT_WIN_BASE+2MiB.
    let win_frame = pmm::pmm_alloc_frame();
    if !win_frame.is_null() {
        g_win_pt_phys_addr = win_frame as u64;
        let win_pt = &mut *(g_win_pt_phys_addr as *mut PageTable);
        let base_phys = (PD_WIN_IDX as u64) << 21;
        for i in 0..512 {
            win_pt.entries[i] = (base_phys + (i as u64) * 4096) | 0x3;
        }
    }

    // Window 2 page table: identity-maps PT_WIN2_BASE..PT_WIN2_BASE+2MiB.
    let win2_frame = pmm::pmm_alloc_frame();
    if !win2_frame.is_null() {
        g_win2_pt_phys_addr = win2_frame as u64;
        let win2_pt = &mut *(g_win2_pt_phys_addr as *mut PageTable);
        let base_phys = (PD_WIN2_IDX as u64) << 21;
        for i in 0..512 {
            win2_pt.entries[i] = (base_phys + (i as u64) * 4096) | 0x3;
        }
    }

    // Global-access page table (PD[12]): lets us edit the window PTs from any
    // CR3. Entry 0 -> window-1 PT, entry 1 -> window-2 PT, rest identity.
    let gwin_frame = pmm::pmm_alloc_frame();
    if !gwin_frame.is_null() {
        let acc_phys = gwin_frame as u64;
        let acc_pt = &mut *(acc_phys as *mut PageTable);
        for i in 0..512 {
            acc_pt.entries[i] = ((PD_GWIN_IDX as u64) << 21) + (i as u64) * 4096 | 0x3;
        }
        acc_pt.entries[0] = (g_win_pt_phys_addr & PTE_ADDR_MASK) | 0x3;
        acc_pt.entries[1] = (g_win2_pt_phys_addr & PTE_ADDR_MASK) | 0x3;
        pd().entries[PD_GWIN_IDX] = (acc_phys & PTE_ADDR_MASK) | 0x3;
        invlpg(((PD_GWIN_IDX as u64) << 21) as usize);
    }

    Serial::write_str("  Identity map extended to 32 MB (PD[2..15])\n");
    Serial::write_str("  Window PT at phys 0x");
    Serial::write_hex64(g_win_pt_phys_addr);
    Serial::write_str("\n");
    Serial::write_str("  Window2 PT at phys 0x");
    Serial::write_hex64(g_win2_pt_phys_addr);
    Serial::write_str("\n");
    Serial::write_str("  g_win_pt VA (PD[12]) = 0x");
    Serial::write_hex64(G_WIN_PT_VA);
    Serial::write_str("\n");
}

/// `paging_enable()`: paging is already on (set up by the boot asm).
#[no_mangle]
pub unsafe extern "C" fn paging_enable() {
    Serial::write_str("Paging: Already enabled (set up by boot code)\n");
}

// ----------------------------------------------------------------------------
// Current-address-space walk
// ----------------------------------------------------------------------------

/// Resolve the PTE for `virt_addr`, creating intermediate tables when `create`.
///
/// The returned pointer aliases the final page table mapped into window slot 0,
/// so callers must modify it before any further window mapping.
unsafe fn get_page_entry(virt_addr: u64, create: bool) -> *mut u64 {
    let pml4_idx = (virt_addr >> 39) & 0x1ff;
    let pdpt_idx = (virt_addr >> 30) & 0x1ff;
    let pd_idx = (virt_addr >> 21) & 0x1ff;
    let pt_idx = (virt_addr >> 12) & 0x1ff;

    // Map the ACTIVE PML4 (from CR3) through window slot 0. The walk must be
    // CR3-relative: VA 0x1000 is identity-mapped only under the kernel CR3, and
    // `copy_from_user`/`copy_to_user` walk the page tables while running under
    // the current user CR3.
    let cr3 = read_cr3() & PTE_ADDR_MASK;
    let pml4 = win_map(cr3, PT_TEMP_IDX) as *mut PageTable;
    if pml4.is_null() {
        return core::ptr::null_mut();
    }

    if pml4_idx != 0 {
        if !create {
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        let new_pdpt = pmm::pmm_alloc_frame();
        if new_pdpt.is_null() {
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        let new_pdpt_phys = new_pdpt as u64;
        let zero_va = win_map(new_pdpt_phys, PT_TEMP_IDX + 1);
        if zero_va.is_null() {
            pmm::pmm_free_frame(new_pdpt);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        string::memset(zero_va, 0, PAGE_SIZE);
        (*pml4).entries[pml4_idx as usize] = (new_pdpt_phys & PTE_ADDR_MASK) | 0x3;
    }

    let pml4e = (*pml4).entries[pml4_idx as usize];
    if pml4e & PAGE_PRESENT == 0 {
        win_unmap(PT_TEMP_IDX);
        return core::ptr::null_mut();
    }
    let pdpt_phys = pml4e & PTE_ADDR_MASK;
    let pdpt = win_map(pdpt_phys, PT_TEMP_IDX + 1) as *mut PageTable;
    if pdpt.is_null() {
        win_unmap(PT_TEMP_IDX);
        return core::ptr::null_mut();
    }

    let mut pdpde = (*pdpt).entries[pdpt_idx as usize];
    if pdpde & PAGE_PRESENT == 0 {
        if !create {
            win_unmap(PT_TEMP_IDX + 1);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        let new_pd = pmm::pmm_alloc_frame();
        if new_pd.is_null() {
            win_unmap(PT_TEMP_IDX + 1);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        let new_pd_phys = new_pd as u64;
        let zero_va = win_map(new_pd_phys, PT_TEMP_IDX + 2);
        if zero_va.is_null() {
            pmm::pmm_free_frame(new_pd);
            win_unmap(PT_TEMP_IDX + 1);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        string::memset(zero_va, 0, PAGE_SIZE);
        (*pdpt).entries[pdpt_idx as usize] = (new_pd_phys & PTE_ADDR_MASK) | 0x3;
        pdpde = (*pdpt).entries[pdpt_idx as usize];
    }

    let pd_phys = pdpde & PTE_ADDR_MASK;
    let pd_tbl = win_map(pd_phys, PT_TEMP_IDX + 2) as *mut PageTable;
    if pd_tbl.is_null() {
        win_unmap(PT_TEMP_IDX + 1);
        win_unmap(PT_TEMP_IDX);
        return core::ptr::null_mut();
    }

    let mut pde = (*pd_tbl).entries[pd_idx as usize];
    if pde & PAGE_PRESENT == 0 {
        if !create {
            win_unmap(PT_TEMP_IDX + 2);
            win_unmap(PT_TEMP_IDX + 1);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        let new_pt = pmm::pmm_alloc_frame();
        if new_pt.is_null() {
            win_unmap(PT_TEMP_IDX + 2);
            win_unmap(PT_TEMP_IDX + 1);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        let new_pt_phys = new_pt as u64;
        let zero_va = win_map(new_pt_phys, PT_TEMP_IDX + 3);
        if zero_va.is_null() {
            pmm::pmm_free_frame(new_pt);
            win_unmap(PT_TEMP_IDX + 2);
            win_unmap(PT_TEMP_IDX + 1);
            win_unmap(PT_TEMP_IDX);
            return core::ptr::null_mut();
        }
        string::memset(zero_va, 0, PAGE_SIZE);
        (*pd_tbl).entries[pd_idx as usize] = (new_pt_phys & PTE_ADDR_MASK) | 0x3;
        pde = (*pd_tbl).entries[pd_idx as usize];
    }

    let pt_phys = pde & PTE_ADDR_MASK;
    win_unmap(PT_TEMP_IDX + 2);
    win_unmap(PT_TEMP_IDX + 1);
    let win_va = win_map(pt_phys, PT_TEMP_IDX + 3);
    if win_va.is_null() {
        win_unmap(PT_TEMP_IDX);
        return core::ptr::null_mut();
    }
    let pt = win_va as *mut PageTable;
    (&mut (*pt).entries as *mut [u64; 512]).cast::<u64>().add(pt_idx as usize)
}

/// `paging_map_page(virt_addr, phys_addr, flags)`: install a mapping in the
/// current address space.
#[no_mangle]
pub unsafe extern "C" fn paging_map_page(
    virt_addr: usize,
    phys_addr: usize,
    flags: u32,
) -> bool {
    let pte = get_page_entry(virt_addr as u64, true);
    if pte.is_null() {
        return false;
    }
    *pte = (phys_addr as u64 & PTE_ADDR_MASK) | (flags as u64 & 0xfff) | PAGE_PRESENT;
    invlpg(virt_addr);
    true
}

/// `paging_unmap_page(virt_addr)`: remove a mapping from the current space.
#[no_mangle]
pub unsafe extern "C" fn paging_unmap_page(virt_addr: usize) {
    let pte = get_page_entry(virt_addr as u64, false);
    if !pte.is_null() {
        *pte = 0;
        invlpg(virt_addr);
    }
}

/// `paging_get_physical_address(virt_addr)`: translate in the current space.
#[no_mangle]
pub unsafe extern "C" fn paging_get_physical_address(virt_addr: usize) -> usize {
    let pte = get_page_entry(virt_addr as u64, false);
    if pte.is_null() || *pte & PAGE_PRESENT == 0 {
        return 0;
    }
    ((*pte & PTE_ADDR_MASK) | (virt_addr as u64 & 0xfff)) as usize
}

// ----------------------------------------------------------------------------
// Address-space lifecycle
// ----------------------------------------------------------------------------

/// `paging_create_directory_phys()`: fresh directory whose kernel half mirrors
/// the kernel PD (heap/identity shared), user half empty.
#[no_mangle]
pub unsafe extern "C" fn paging_create_directory_phys() -> usize {
    let pml4_frame = pmm::pmm_alloc_frame();
    if pml4_frame.is_null() {
        Serial::write_str("Paging: ERROR - Failed to allocate PML4 frame\n");
        return 0;
    }
    let new_pml4_phys = pml4_frame as u64;

    let pdpt_frame = pmm::pmm_alloc_frame();
    if pdpt_frame.is_null() {
        pmm::pmm_free_frame(pml4_frame);
        Serial::write_str("Paging: ERROR - Failed to allocate PDPT frame\n");
        return 0;
    }
    let new_pdpt_phys = pdpt_frame as u64;

    let pd_frame = pmm::pmm_alloc_frame();
    if pd_frame.is_null() {
        pmm::pmm_free_frame(pml4_frame);
        pmm::pmm_free_frame(pdpt_frame);
        Serial::write_str("Paging: ERROR - Failed to allocate PD frame\n");
        return 0;
    }
    let new_pd_phys = pd_frame as u64;

    // New PD is a copy of the kernel PD: heap + windows come along for free.
    let pd_va = win_map(new_pd_phys, PT_TEMP_IDX);
    if pd_va.is_null() {
        pmm::pmm_free_frame(pml4_frame);
        pmm::pmm_free_frame(pdpt_frame);
        pmm::pmm_free_frame(pd_frame);
        return 0;
    }
    let new_pd = pd_va as *mut PageTable;
    for i in 0..512 {
        (*new_pd).entries[i] = pd().entries[i];
    }
    win_unmap(PT_TEMP_IDX);

    // PDPT: zeroed except entry 0 -> new PD.
    let pdpt_va = win_map(new_pdpt_phys, PT_TEMP_IDX);
    if pdpt_va.is_null() {
        pmm::pmm_free_frame(pml4_frame);
        pmm::pmm_free_frame(pdpt_frame);
        pmm::pmm_free_frame(pd_frame);
        return 0;
    }
    let new_pdpt = pdpt_va as *mut PageTable;
    for e in (*new_pdpt).entries.iter_mut() {
        *e = 0;
    }
    (*new_pdpt).entries[0] = (new_pd_phys & PTE_ADDR_MASK) | 0x7;
    win_unmap(PT_TEMP_IDX);

    // PML4: zeroed except entry 0 -> new PDPT.
    let pml4_va = win_map(new_pml4_phys, PT_TEMP_IDX);
    if pml4_va.is_null() {
        pmm::pmm_free_frame(pml4_frame);
        pmm::pmm_free_frame(pdpt_frame);
        pmm::pmm_free_frame(pd_frame);
        return 0;
    }
    let new_pml4 = pml4_va as *mut PageTable;
    for e in (*new_pml4).entries.iter_mut() {
        *e = 0;
    }
    (*new_pml4).entries[0] = (new_pdpt_phys & PTE_ADDR_MASK) | 0x7;
    win_unmap(PT_TEMP_IDX);

    new_pml4_phys as usize
}

/// `paging_destroy_directory(pd_phys)`: tear down the kernel half of a
/// directory, freeing every frame it owns (COW pages via refcount).
#[no_mangle]
pub unsafe extern "C" fn paging_destroy_directory(pd_phys: usize) {
    if pd_phys == 0 {
        return;
    }
    Serial::write_str("Paging: Destroying page directory (x86_64)\n");

    let win_va = win_map(pd_phys as u64, PT_TEMP_IDX);
    if win_va.is_null() {
        return;
    }
    let pml4 = win_va as *mut PageTable;

    // Only the kernel half (pml4 idx 4..) is owned by the directory; the user
    // half is reclaimed elsewhere (scheduler leak-not-free).
    for pml4_i in 4..512 {
        let pml4e = (*pml4).entries[pml4_i];
        if pml4e & PAGE_PRESENT == 0 {
            continue;
        }
        let pdpt_phys = pml4e & PTE_ADDR_MASK;
        let pdpt_va = win_map(pdpt_phys, PT_TEMP_IDX + 1) as *mut PageTable;
        for pdpt_i in 0..512 {
            let pdpde = (*pdpt_va).entries[pdpt_i];
            if pdpde & PAGE_PRESENT == 0 {
                continue;
            }
            let pd_phys_entry = pdpde & PTE_ADDR_MASK;
            let pd_tbl = win_map(pd_phys_entry, PT_TEMP_IDX + 2) as *mut PageTable;
            if pd_tbl.is_null() {
                continue;
            }
            for pd_i in 0..512 {
                let pd_entry = (*pd_tbl).entries[pd_i];
                if pd_entry & PAGE_PRESENT == 0 {
                    continue;
                }
                if pd_entry & 0x80 != 0 {
                    // 2 MiB huge page: single frame, not refcounted.
                    pmm::pmm_free_frame((pd_entry & PTE_ADDR_MASK) as *mut c_void);
                    (*pd_tbl).entries[pd_i] = 0;
                } else {
                    let pt_phys = pd_entry & PTE_ADDR_MASK;
                    let pt = win_map(pt_phys, PT_TEMP_IDX + 3) as *mut PageTable;
                    if pt.is_null() {
                        continue;
                    }
                    for pt_i in 0..512 {
                        let pte = (*pt).entries[pt_i];
                        if pte & PAGE_PRESENT == 0 {
                            continue;
                        }
                        let frame = pte & PTE_ADDR_MASK;
                        if pte & PAGE_COW != 0 {
                            pmm::pmm_refcount_dec(frame as *mut c_void);
                        } else {
                            pmm::pmm_free_frame(frame as *mut c_void);
                        }
                        (*pt).entries[pt_i] = 0;
                    }
                    pmm::pmm_free_frame(pt_phys as *mut c_void);
                    (*pd_tbl).entries[pd_i] = 0;
                }
            }
            pmm::pmm_free_frame(pd_phys_entry as *mut c_void);
            (*pdpt_va).entries[pdpt_i] = 0;
        }
        pmm::pmm_free_frame(pdpt_phys as *mut c_void);
        (*pml4).entries[pml4_i] = 0;
    }

    win_unmap(PT_TEMP_IDX + 3);
    win_unmap(PT_TEMP_IDX + 2);
    win_unmap(PT_TEMP_IDX + 1);
    win_unmap(PT_TEMP_IDX);
    pmm::pmm_free_frame(pd_phys as *mut c_void);
}

/// `paging_switch_to_directory(pd_phys)`: make a directory the active one.
#[no_mangle]
pub unsafe extern "C" fn paging_switch_to_directory(pd_phys: usize) -> bool {
    if pd_phys == 0 {
        return false;
    }
    write_cr3(pd_phys as u64);
    true
}

/// `paging_get_kernel_directory_phys()`: physical address of the kernel PML4.
#[no_mangle]
pub unsafe extern "C" fn paging_get_kernel_directory_phys() -> usize {
    kernel_pml4_phys as usize
}

// ----------------------------------------------------------------------------
// Deep clone and COW fork
// ----------------------------------------------------------------------------

/// Deep-copy every present page in `src_pt` into freshly-allocated frames in
/// `dst_pt` (window slots 1 are used for the copy; the tables themselves are
/// mapped in slots 0).
unsafe fn clone_page_table(src_pt: *mut PageTable, dst_pt: *mut PageTable) {
    for i in 0..512 {
        let pte = (*src_pt).entries[i];
        if pte & PAGE_PRESENT == 0 {
            continue;
        }
        let src_frame = pte & PTE_ADDR_MASK;
        let new_frame = pmm::pmm_alloc_frame();
        if new_frame.is_null() {
            Serial::write_str("Paging: clone - OOM during page copy\n");
            continue;
        }
        let new_frame_phys = new_frame as u64;
        let src_va = win2_map(src_frame, 1);
        let dst_va = win_map(new_frame_phys, 1);
        if !src_va.is_null() && !dst_va.is_null() {
            string::memcpy(dst_va, src_va, PAGE_SIZE);
        }
        win_unmap(1);
        win2_unmap(1);
        (*dst_pt).entries[i] = (new_frame_phys & PTE_ADDR_MASK) | (pte & 0xfff) | PAGE_PRESENT;
    }
}

/// `paging_clone_directory(pd_phys)`: deep clone for `clone()`.
///
/// The low 4 PML4 entries (user half) are shared verbatim; the kernel half is
/// deep-copied so the child owns its own (kernel) page tables.
#[no_mangle]
pub unsafe extern "C" fn paging_clone_directory(pd_phys: usize) -> usize {
    Serial::write_str("Paging: Cloning page directory (x86_64)\n");

    let dst_pml4_frame = pmm::pmm_alloc_frame();
    if dst_pml4_frame.is_null() {
        return 0;
    }
    let dst_pml4_phys = dst_pml4_frame as u64;
    let zero_va = win_map(dst_pml4_phys, PT_TEMP_IDX);
    if zero_va.is_null() {
        pmm::pmm_free_frame(dst_pml4_frame);
        return 0;
    }
    string::memset(zero_va, 0, PAGE_SIZE);
    win_unmap(PT_TEMP_IDX);

    let src_pml4 = win_map(pd_phys as u64, 0) as *mut PageTable;
    let dst_pml4 = win2_map(dst_pml4_phys, 0) as *mut PageTable;

    // User half shared verbatim.
    for i in 0..4 {
        (*dst_pml4).entries[i] = (*src_pml4).entries[i];
    }

    let mut ok = true;
    'outer: for pml4_i in 4..512 {
        let pml4e = (*src_pml4).entries[pml4_i];
        if pml4e & PAGE_PRESENT == 0 {
            continue;
        }
        let src_pdpt_phys = pml4e & PTE_ADDR_MASK;
        let pdpt_flags = pml4e & 0xfff;

        let dst_pdpt_frame = pmm::pmm_alloc_frame();
        if dst_pdpt_frame.is_null() {
            ok = false;
            break;
        }
        let dst_pdpt_phys = dst_pdpt_frame as u64;
        let zero_va2 = win2_map(dst_pdpt_phys, 1);
        if zero_va2.is_null() {
            ok = false;
            break;
        }
        string::memset(zero_va2, 0, PAGE_SIZE);
        win2_unmap(1);
        (*dst_pml4).entries[pml4_i] = (dst_pdpt_phys & PTE_ADDR_MASK) | pdpt_flags | PAGE_PRESENT;
        win2_unmap(0);
        win_unmap(0);

        let src_pdpt = win_map(src_pdpt_phys, 0) as *mut PageTable;
        let dst_pdpt = win2_map(dst_pdpt_phys, 0) as *mut PageTable;
        for pdpt_i in 0..512 {
            let pdpde = (*src_pdpt).entries[pdpt_i];
            if pdpde & PAGE_PRESENT == 0 {
                continue;
            }
            let src_pd_phys = pdpde & PTE_ADDR_MASK;
            let pd_flags = pdpde & 0xfff;

            let dst_pd_frame = pmm::pmm_alloc_frame();
            if dst_pd_frame.is_null() {
                ok = false;
                break 'outer;
            }
            let dst_pd_phys = dst_pd_frame as u64;
            let zero_va3 = win2_map(dst_pd_phys, 1);
            if zero_va3.is_null() {
                ok = false;
                break 'outer;
            }
            string::memset(zero_va3, 0, PAGE_SIZE);
            win2_unmap(1);
            (*dst_pdpt).entries[pdpt_i] = (dst_pd_phys & PTE_ADDR_MASK) | pd_flags | PAGE_PRESENT;
            win2_unmap(0);
            win_unmap(0);

            let src_pd = win_map(src_pd_phys, 0) as *mut PageTable;
            let dst_pd = win2_map(dst_pd_phys, 0) as *mut PageTable;
            for pd_i in 0..512 {
                let pde = (*src_pd).entries[pd_i];
                if pde & PAGE_PRESENT == 0 {
                    continue;
                }
                if pde & 0x80 != 0 {
                    // 2 MiB huge page: split into a 4 KiB page table with
                    // per-page deep copies.
                    let huge_base = pde & PTE_ADDR_MASK;
                    let src_flags = pde & 0xfff;
                    let dst_pt_frame = pmm::pmm_alloc_frame();
                    if dst_pt_frame.is_null() {
                        ok = false;
                        break 'outer;
                    }
                    let dst_pt_phys = dst_pt_frame as u64;
                    let zero_va_pt = win2_map(dst_pt_phys, 1);
                    if zero_va_pt.is_null() {
                        ok = false;
                        break 'outer;
                    }
                    string::memset(zero_va_pt, 0, PAGE_SIZE);
                    win2_unmap(1);
                    (*dst_pd).entries[pd_i] =
                        (dst_pt_phys & PTE_ADDR_MASK) | (src_flags & !0x80) | PAGE_PRESENT;
                    win2_unmap(0);
                    win_unmap(0);

                    let dst_pt = win_map(dst_pt_phys, 0) as *mut PageTable;
                    for pt_i in 0..512 {
                        let phys = huge_base + ((pt_i as u64) << 12);
                        let new_frame = pmm::pmm_alloc_frame();
                        if !new_frame.is_null() {
                            let new_frame_phys = new_frame as u64;
                            let src_va = win_map(phys, 1);
                            let dst_va = win2_map(new_frame_phys, 1);
                            if !src_va.is_null() && !dst_va.is_null() {
                                string::memcpy(dst_va, src_va, PAGE_SIZE);
                            }
                            win_unmap(1);
                            win2_unmap(1);
                            (*dst_pt).entries[pt_i] = (new_frame_phys & PTE_ADDR_MASK)
                                | (src_flags & (0xfff & !0x80))
                                | PAGE_PRESENT;
                        }
                    }
                    win_unmap(0);
                    win_map(src_pd_phys, 0);
                    win2_map(dst_pd_phys, 0);
                } else {
                    let src_pt_phys = pde & PTE_ADDR_MASK;
                    let pt_flags = pde & 0xfff;
                    let dst_pt_frame = pmm::pmm_alloc_frame();
                    if dst_pt_frame.is_null() {
                        ok = false;
                        break 'outer;
                    }
                    let dst_pt_phys = dst_pt_frame as u64;
                    let zero_va_pt2 = win2_map(dst_pt_phys, 1);
                    if zero_va_pt2.is_null() {
                        ok = false;
                        break 'outer;
                    }
                    string::memset(zero_va_pt2, 0, PAGE_SIZE);
                    win2_unmap(1);
                    (*dst_pd).entries[pd_i] =
                        (dst_pt_phys & PTE_ADDR_MASK) | pt_flags | PAGE_PRESENT;
                    win2_unmap(0);
                    win_unmap(0);

                    let src_pt = win_map(src_pt_phys, 0) as *mut PageTable;
                    let dst_pt = win2_map(dst_pt_phys, 0) as *mut PageTable;
                    clone_page_table(src_pt, dst_pt);
                    win_unmap(0);
                    win2_unmap(0);
                    win_map(src_pd_phys, 0);
                    win2_map(dst_pd_phys, 0);
                }
            }
            win_unmap(0);
            win2_unmap(0);
            win_map(src_pdpt_phys, 0);
            win2_map(dst_pdpt_phys, 0);
        }
        win_unmap(0);
        win2_unmap(0);
        win_map(pd_phys as u64, 0);
        win2_map(dst_pml4_phys, 0);
    }

    if !ok {
        Serial::write_str("Paging: Clone failed - out of memory\n");
        pmm::pmm_free_frame(dst_pml4_phys as *mut c_void);
        win_unmap(0);
        win2_unmap(0);
        return 0;
    }
    win_unmap(0);
    win2_unmap(0);
    Serial::write_str("Paging: Clone complete\n");
    dst_pml4_phys as usize
}

/// COW-ify `src_pt` into `dst_pt`: writable user pages become shared COW.
unsafe fn fork_fork_pt(src_pt: *mut PageTable, dst_pt: *mut PageTable) {
    for i in 0..512 {
        let pte = (*src_pt).entries[i];
        if pte & PAGE_PRESENT == 0 {
            (*dst_pt).entries[i] = 0;
        } else if pte & PAGE_USER != 0 && pte & PAGE_WRITE != 0 {
            let frame_phys = pte & PTE_ADDR_MASK;
            let flags = (pte & 0xfff) & !PAGE_WRITE | PAGE_COW;
            pmm::pmm_refcount_inc(frame_phys as *mut c_void);
            (*src_pt).entries[i] = frame_phys | flags;
            (*dst_pt).entries[i] = frame_phys | flags;
        } else {
            (*dst_pt).entries[i] = pte;
        }
    }
}

/// `paging_fork_directory(pd_phys)`: COW fork of a whole directory.
#[no_mangle]
pub unsafe extern "C" fn paging_fork_directory(pd_phys: usize) -> usize {
    Serial::write_str("Paging: Forking page directory (x86_64, COW)\n");

    let dst_pml4_frame = pmm::pmm_alloc_frame();
    if dst_pml4_frame.is_null() {
        return 0;
    }
    let dst_pml4_phys = dst_pml4_frame as u64;
    let zero_va = win_map(dst_pml4_phys, PT_TEMP_IDX);
    if zero_va.is_null() {
        pmm::pmm_free_frame(dst_pml4_frame);
        return 0;
    }
    string::memset(zero_va, 0, PAGE_SIZE);
    win_unmap(PT_TEMP_IDX);

    let src_pml4 = win_map(pd_phys as u64, 0) as *mut PageTable;
    let dst_pml4 = win2_map(dst_pml4_phys, 0) as *mut PageTable;

    let mut ok = true;
    'outer: for pml4_i in 0..512 {
        let pml4e = (*src_pml4).entries[pml4_i];
        if pml4e & PAGE_PRESENT == 0 {
            continue;
        }
        let src_pdpt_phys = pml4e & PTE_ADDR_MASK;
        let pdpt_flags = pml4e & 0xfff;

        let dst_pdpt_frame = pmm::pmm_alloc_frame();
        if dst_pdpt_frame.is_null() {
            ok = false;
            break;
        }
        let dst_pdpt_phys = dst_pdpt_frame as u64;
        let zero_va2 = win2_map(dst_pdpt_phys, 1);
        if zero_va2.is_null() {
            ok = false;
            break;
        }
        string::memset(zero_va2, 0, PAGE_SIZE);
        win2_unmap(1);
        (*dst_pml4).entries[pml4_i] = (dst_pdpt_phys & PTE_ADDR_MASK) | pdpt_flags | PAGE_PRESENT;
        win2_unmap(0);
        win_unmap(0);

        let src_pdpt = win_map(src_pdpt_phys, 0) as *mut PageTable;
        let dst_pdpt = win2_map(dst_pdpt_phys, 0) as *mut PageTable;
        for pdpt_i in 0..512 {
            let pdpde = (*src_pdpt).entries[pdpt_i];
            if pdpde & PAGE_PRESENT == 0 {
                continue;
            }
            let src_pd_phys = pdpde & PTE_ADDR_MASK;
            let pd_flags = pdpde & 0xfff;

            let dst_pd_frame = pmm::pmm_alloc_frame();
            if dst_pd_frame.is_null() {
                ok = false;
                break 'outer;
            }
            let dst_pd_phys = dst_pd_frame as u64;
            let zero_va3 = win2_map(dst_pd_phys, 1);
            if zero_va3.is_null() {
                ok = false;
                break 'outer;
            }
            string::memset(zero_va3, 0, PAGE_SIZE);
            win2_unmap(1);
            (*dst_pdpt).entries[pdpt_i] = (dst_pd_phys & PTE_ADDR_MASK) | pd_flags | PAGE_PRESENT;
            win2_unmap(0);
            win_unmap(0);

            let src_pd = win_map(src_pd_phys, 0) as *mut PageTable;
            let dst_pd = win2_map(dst_pd_phys, 0) as *mut PageTable;
            for pd_i in 0..512 {
                let pde = (*src_pd).entries[pd_i];
                if pde & PAGE_PRESENT == 0 {
                    continue;
                }
                if pde & 0x80 != 0 {
                    // 2 MiB huge page shared as-is (kernel identity).
                    (*dst_pd).entries[pd_i] = pde;
                } else {
                    let src_pt_phys = pde & PTE_ADDR_MASK;
                    let pt_flags = pde & 0xfff;
                    let dst_pt_frame = pmm::pmm_alloc_frame();
                    if dst_pt_frame.is_null() {
                        ok = false;
                        break 'outer;
                    }
                    let dst_pt_phys = dst_pt_frame as u64;
                    let zero_va_pt = win2_map(dst_pt_phys, 1);
                    if zero_va_pt.is_null() {
                        ok = false;
                        break 'outer;
                    }
                    string::memset(zero_va_pt, 0, PAGE_SIZE);
                    win2_unmap(1);
                    (*dst_pd).entries[pd_i] =
                        (dst_pt_phys & PTE_ADDR_MASK) | pt_flags | PAGE_PRESENT;
                    win2_unmap(0);
                    win_unmap(0);

                    let src_pt = win_map(src_pt_phys, 0) as *mut PageTable;
                    let dst_pt = win2_map(dst_pt_phys, 0) as *mut PageTable;
                    fork_fork_pt(src_pt, dst_pt);
                    win_unmap(0);
                    win2_unmap(0);
                    win_map(src_pd_phys, 0);
                    win2_map(dst_pd_phys, 0);
                }
            }
            win_unmap(0);
            win2_unmap(0);
            win_map(src_pdpt_phys, 0);
            win2_map(dst_pdpt_phys, 0);
        }
        win_unmap(0);
        win2_unmap(0);
        win_map(pd_phys as u64, 0);
        win2_map(dst_pml4_phys, 0);
    }

    if !ok {
        Serial::write_str("Paging: Fork failed - out of memory\n");
        pmm::pmm_free_frame(dst_pml4_phys as *mut c_void);
        win_unmap(0);
        win2_unmap(0);
        return 0;
    }
    win_unmap(0);
    win2_unmap(0);
    Serial::write_str("Paging: Fork complete (COW)\n");
    dst_pml4_phys as usize
}

// ----------------------------------------------------------------------------
// COW fault handling
// ----------------------------------------------------------------------------

/// Resolve the PTE for `virt_addr` in the directory rooted at `pd_phys`,
/// walking through the temp windows (safe from any CR3). Returns NULL for
/// unmapped or 2 MiB huge-page entries. The result aliases window slot 0.
unsafe fn get_page_entry_in_pd(pd_phys: usize, virt_addr: u64) -> *mut u64 {
    let pml4_idx = (virt_addr >> 39) & 0x1ff;
    let pdpt_idx = (virt_addr >> 30) & 0x1ff;
    let pd_idx = (virt_addr >> 21) & 0x1ff;
    let pt_idx = (virt_addr >> 12) & 0x1ff;

    let pml4 = win_map(pd_phys as u64, PT_TEMP_IDX) as *mut PageTable;
    let pml4e = (*pml4).entries[pml4_idx as usize];
    win_unmap(PT_TEMP_IDX);
    if pml4e & PAGE_PRESENT == 0 {
        return core::ptr::null_mut();
    }

    let pdpt_phys = pml4e & PTE_ADDR_MASK;
    let pdpt = win_map(pdpt_phys, PT_TEMP_IDX) as *mut PageTable;
    let pdpde = (*pdpt).entries[pdpt_idx as usize];
    win_unmap(PT_TEMP_IDX);
    if pdpde & PAGE_PRESENT == 0 {
        return core::ptr::null_mut();
    }

    let pd_tbl_phys = pdpde & PTE_ADDR_MASK;
    let pd_tbl = win_map(pd_tbl_phys, PT_TEMP_IDX) as *mut PageTable;
    let pde = (*pd_tbl).entries[pd_idx as usize];
    win_unmap(PT_TEMP_IDX);
    if pde & PAGE_PRESENT == 0 || pde & 0x80 != 0 {
        return core::ptr::null_mut();
    }

    let pt_phys = pde & PTE_ADDR_MASK;
    let pt = win_map(pt_phys, PT_TEMP_IDX) as *mut PageTable;
    (&mut (*pt).entries as *mut [u64; 512]).cast::<u64>().add(pt_idx as usize)
}

/// `paging_handle_cow_fault(pd_phys, fault_addr)`: break a COW page, giving
/// the faulting space a private writable copy.
#[no_mangle]
pub unsafe extern "C" fn paging_handle_cow_fault(
    pd_phys: usize,
    fault_addr: usize,
) -> u8 {
    let pte = get_page_entry_in_pd(pd_phys, fault_addr as u64);
    if pte.is_null() || *pte & PAGE_PRESENT == 0 {
        return 0;
    }
    let flags = *pte & 0xfff;
    if flags & PAGE_COW == 0 {
        return 0;
    }

    let old_frame = *pte & PTE_ADDR_MASK;
    let new_frame = pmm::pmm_alloc_frame();
    if new_frame.is_null() {
        Serial::write_str("Paging: COW - out of memory!\n");
        return 0;
    }
    let new_frame_phys = new_frame as u64;

    let cow_src = win_map(old_frame, 1);
    let cow_dst = win2_map(new_frame_phys, 1);
    if !cow_src.is_null() && !cow_dst.is_null() {
        string::memcpy(cow_dst, cow_src, PAGE_SIZE);
    }
    win_unmap(1);
    win2_unmap(1);

    pmm::pmm_refcount_dec(old_frame as *mut c_void);
    let flags = flags & !PAGE_COW | PAGE_WRITE;
    *pte = (new_frame_phys & PTE_ADDR_MASK) | flags;
    invlpg(fault_addr);
    1
}

/// `paging_map_page_in_pd(pd_phys, virt_addr, phys_addr, flags)`: map a frame
/// into an arbitrary directory, creating intermediate tables as needed.
#[no_mangle]
pub unsafe extern "C" fn paging_map_page_in_pd(
    pd_phys: usize,
    virt_addr: usize,
    phys_addr: usize,
    flags: u32,
) -> bool {
    let vaddr = virt_addr as u64;
    let pml4_idx = (vaddr >> 39) & 0x1ff;
    let pdpt_idx = (vaddr >> 30) & 0x1ff;
    let pd_idx = (vaddr >> 21) & 0x1ff;
    let pt_idx = (vaddr >> 12) & 0x1ff;
    let user_bit = (flags & PAGE_USER as u32) as u64;

    // --- PML4 entry ---
    let pml4 = win_map(pd_phys as u64, PT_TEMP_IDX) as *mut PageTable;
    if pml4.is_null() {
        return false;
    }
    if (*pml4).entries[pml4_idx as usize] & PAGE_PRESENT == 0 {
        let new_pdpt = pmm::pmm_alloc_frame();
        if new_pdpt.is_null() {
            win_unmap(PT_TEMP_IDX);
            return false;
        }
        let new_pdpt_phys = new_pdpt as u64;
        win_unmap(PT_TEMP_IDX);
        let zero_va = win_map(new_pdpt_phys, PT_TEMP_IDX);
        if zero_va.is_null() {
            pmm::pmm_free_frame(new_pdpt);
            return false;
        }
        string::memset(zero_va, 0, PAGE_SIZE);
        win_unmap(PT_TEMP_IDX);
        let pml4 = win_map(pd_phys as u64, PT_TEMP_IDX) as *mut PageTable;
        if pml4.is_null() {
            pmm::pmm_free_frame(new_pdpt);
            return false;
        }
        (*pml4).entries[pml4_idx as usize] = (new_pdpt_phys & PTE_ADDR_MASK) | 0x3 | user_bit;
    }
    let pdpt_phys = (*pml4).entries[pml4_idx as usize] & PTE_ADDR_MASK;
    win_unmap(PT_TEMP_IDX);

    // --- PDPT entry ---
    let pdpt_va = win_map(pdpt_phys, PT_TEMP_IDX);
    if pdpt_va.is_null() {
        return false;
    }
    let pdpt = pdpt_va as *mut PageTable;
    if (*pdpt).entries[pdpt_idx as usize] & PAGE_PRESENT == 0 {
        let new_pd = pmm::pmm_alloc_frame();
        if new_pd.is_null() {
            win_unmap(PT_TEMP_IDX);
            return false;
        }
        let new_pd_phys = new_pd as u64;
        win_unmap(PT_TEMP_IDX);
        let zero_va = win_map(new_pd_phys, PT_TEMP_IDX);
        if zero_va.is_null() {
            pmm::pmm_free_frame(new_pd);
            return false;
        }
        string::memset(zero_va, 0, PAGE_SIZE);
        win_unmap(PT_TEMP_IDX);
        let pdpt = win_map(pdpt_phys, PT_TEMP_IDX) as *mut PageTable;
        if pdpt.is_null() {
            pmm::pmm_free_frame(new_pd);
            return false;
        }
        (*pdpt).entries[pdpt_idx as usize] = (new_pd_phys & PTE_ADDR_MASK) | 0x3 | user_bit;
    }
    let pd_phys_tbl = (*pdpt).entries[pdpt_idx as usize] & PTE_ADDR_MASK;
    win_unmap(PT_TEMP_IDX);

    // --- PD entry ---
    let pd_va = win_map(pd_phys_tbl, PT_TEMP_IDX);
    if pd_va.is_null() {
        return false;
    }
    let pd_tbl = pd_va as *mut PageTable;
    let pde = (*pd_tbl).entries[pd_idx as usize];
    if pde & PAGE_PRESENT == 0 {
        let new_pt = pmm::pmm_alloc_frame();
        if new_pt.is_null() {
            win_unmap(PT_TEMP_IDX);
            return false;
        }
        let new_pt_phys = new_pt as u64;
        win_unmap(PT_TEMP_IDX);
        let zero_va = win_map(new_pt_phys, PT_TEMP_IDX);
        if zero_va.is_null() {
            pmm::pmm_free_frame(new_pt);
            return false;
        }
        string::memset(zero_va, 0, PAGE_SIZE);
        win_unmap(PT_TEMP_IDX);
        let pd_tbl = win_map(pd_phys_tbl, PT_TEMP_IDX) as *mut PageTable;
        if pd_tbl.is_null() {
            pmm::pmm_free_frame(new_pt);
            return false;
        }
        (*pd_tbl).entries[pd_idx as usize] = (new_pt_phys & PTE_ADDR_MASK) | 0x3 | user_bit;
    } else if pde & 0x80 != 0 {
        // Split a 2 MiB huge page into a page table before mapping.
        let huge_phys_base = (*pd_tbl).entries[pd_idx as usize] & 0xffff_ffff_fe00;
        let huge_flags = (*pd_tbl).entries[pd_idx as usize] & 0xfff;
        let pt_flags = huge_flags & !0x80;
        let new_pt = pmm::pmm_alloc_frame();
        if new_pt.is_null() {
            win_unmap(PT_TEMP_IDX);
            return false;
        }
        let new_pt_phys = new_pt as u64;
        win_unmap(PT_TEMP_IDX);
        let pt_fill_va = win_map(new_pt_phys, PT_TEMP_IDX);
        if pt_fill_va.is_null() {
            pmm::pmm_free_frame(new_pt);
            return false;
        }
        let new_pt_tbl = pt_fill_va as *mut PageTable;
        for e in (*new_pt_tbl).entries.iter_mut() {
            *e = 0;
        }
        win_unmap(PT_TEMP_IDX);
        let pd_tbl = win_map(pd_phys_tbl, PT_TEMP_IDX) as *mut PageTable;
        if pd_tbl.is_null() {
            pmm::pmm_free_frame(new_pt);
            return false;
        }
        (*pd_tbl).entries[pd_idx as usize] =
            (new_pt_phys & PTE_ADDR_MASK) | (pt_flags & !0x80) | user_bit;
        flush_tlb();
    }
    let pt_phys = (*pd_tbl).entries[pd_idx as usize] & PTE_ADDR_MASK;
    win_unmap(PT_TEMP_IDX);

    // --- PTE ---
    let pt_va = win_map(pt_phys, PT_TEMP_IDX);
    if pt_va.is_null() {
        return false;
    }
    let pt = pt_va as *mut PageTable;
    (*pt).entries[pt_idx as usize] =
        (phys_addr as u64 & PTE_ADDR_MASK) | (flags as u64 & 0xfff) | PAGE_PRESENT;
    win_unmap(PT_TEMP_IDX);
    invlpg(virt_addr);
    true
}

// ----------------------------------------------------------------------------
// Diagnostics
// ----------------------------------------------------------------------------

/// `paging_dump_user_pt(cr3, fault_addr)`: dump the user-space walk (diagnostic).
#[no_mangle]
pub unsafe extern "C" fn paging_dump_user_pt(cr3: u64, fault_addr: u64) {
    let pml4_phys = cr3;
    let pml4 = win_map(pml4_phys, PT_TEMP_IDX) as *mut u64;
    if pml4.is_null() {
        Serial::write_str("  [dump] Cannot map user PML4\n");
        return;
    }
    let pml4e0 = *pml4.offset(0);
    win_unmap(PT_TEMP_IDX);

    let pdpt_phys = pml4e0 & 0xffff_ffff_fff0;
    let pdpt = win_map(pdpt_phys, PT_TEMP_IDX) as *mut u64;
    if pdpt.is_null() {
        Serial::write_str("  [dump] Cannot map user PDPT\n");
        return;
    }
    let pdpte0 = *pdpt.offset(0);
    win_unmap(PT_TEMP_IDX);

    let pd_phys = pdpte0 & 0xffff_ffff_fff0;
    let pd = win_map(pd_phys, PT_TEMP_IDX) as *mut u64;
    if pd.is_null() {
        Serial::write_str("  [dump] Cannot map user PD\n");
        return;
    }
    let pd_idx = (fault_addr >> 21) & 0x1ff;
    let pde_val = *pd.offset(pd_idx as isize);
    win_unmap(PT_TEMP_IDX);

    Serial::write_str("  PML4[0]=0x");
    Serial::write_hex64(pml4e0);
    Serial::write_str(" PDPT[0]=0x");
    Serial::write_hex64(pdpte0);
    Serial::write_str(" PD[");
    Serial::write_hex(pd_idx as u32);
    Serial::write_str("]=0x");
    Serial::write_hex64(pde_val);
    Serial::write_str("\n");
}

// ----------------------------------------------------------------------------
// Demand paging (used by the IDT fault path)
// ----------------------------------------------------------------------------

/// `paging_demand_map_kernel_page(fault_addr, user_cr3)`: propagate an
/// existing kernel mapping into a user directory (present-but-unmapped case).
#[no_mangle]
pub unsafe extern "C" fn paging_demand_map_kernel_page(
    fault_addr: u64,
    user_cr3: u64,
) -> bool {
    let pml4_idx = (fault_addr >> 39) & 0x1ff;
    let pdpt_idx = (fault_addr >> 30) & 0x1ff;
    let pd_idx = (fault_addr >> 21) & 0x1ff;
    let pt_idx = (fault_addr >> 12) & 0x1ff;

    let pml4 = X86_64_PML4_PHYS as *mut u64;
    let pml4e = *pml4.offset(pml4_idx as isize);
    if pml4e & PAGE_PRESENT == 0 {
        return false;
    }
    let pdpt_phys = pml4e & PTE_ADDR_MASK;
    let pdpt = pdpt_phys as *mut u64;
    let pdpte = *pdpt.offset(pdpt_idx as isize);
    if pdpte & PAGE_PRESENT == 0 {
        return false;
    }
    if pdpte & 0x80 != 0 {
        // 1 GiB huge page in the kernel: map the 4 KiB slice into the user.
        let phys_frame = pdpte & 0xffff_ffff_c00000;
        let page_addr = fault_addr & !0xfff;
        return paging_map_page_in_pd(
            user_cr3 as usize,
            page_addr as usize,
            (phys_frame as usize).wrapping_add(fault_addr as usize & 0x3fff_ffff),
            (PAGE_PRESENT | PAGE_WRITE) as u32,
        );
    }
    let pd_phys_addr = pdpte & PTE_ADDR_MASK;
    let pd = pd_phys_addr as *mut u64;
    let pde = *pd.offset(pd_idx as isize);
    if pde & PAGE_PRESENT == 0 {
        return false;
    }
    if pde & 0x80 != 0 {
        // 2 MiB huge page in the kernel.
        let phys_frame = pde & 0xffff_ffff_fe00;
        let page_addr = fault_addr & !0xfff;
        return paging_map_page_in_pd(
            user_cr3 as usize,
            page_addr as usize,
            (phys_frame as usize).wrapping_add(fault_addr as usize & 0x1f_ffff),
            (PAGE_PRESENT | PAGE_WRITE) as u32,
        );
    }
    let pt_phys = pde & PTE_ADDR_MASK;
    let pt = win_map(pt_phys, PT_TEMP_IDX) as *mut u64;
    if pt.is_null() {
        return false;
    }
    let pte = *pt.offset(pt_idx as isize);
    win_unmap(PT_TEMP_IDX);
    if pte & PAGE_PRESENT == 0 {
        return false;
    }
    let phys_frame = pte & PTE_ADDR_MASK;
    let page_addr = fault_addr & !0xfff;
    paging_map_page_in_pd(
        user_cr3 as usize,
        page_addr as usize,
        phys_frame as usize,
        (PAGE_PRESENT | PAGE_WRITE) as u32,
    )
}

/// `paging_demand_alloc_kernel_page(fault_addr)`: allocate a fresh zeroed
/// frame for the kernel heap (not-present kernel-space fault).
#[no_mangle]
pub unsafe extern "C" fn paging_demand_alloc_kernel_page(fault_addr: u64) -> bool {
    let page_addr = fault_addr & !0xfff;
    let pd_idx = (page_addr >> 21) & 0x1ff;
    let pt_idx = (page_addr >> 12) & 0x1ff;

    let frame = pmm::pmm_alloc_frame();
    if frame.is_null() {
        return false;
    }
    let phys = frame as u64;

    // Zero the new frame.
    let zva = win_map(phys, PT_TEMP_IDX);
    if !zva.is_null() {
        string::memset(zva, 0, PAGE_SIZE);
        win_unmap(PT_TEMP_IDX);
    }

    let pd = pd();
    let mut pde = pd.entries[pd_idx as usize];
    if pde & PAGE_PRESENT == 0 {
        let pt_frame = pmm::pmm_alloc_frame();
        if pt_frame.is_null() {
            pmm::pmm_free_frame(frame);
            return false;
        }
        let pt_phys = pt_frame as u64;
        let zva2 = win_map(pt_phys, PT_TEMP_IDX);
        if zva2.is_null() {
            pmm::pmm_free_frame(frame);
            pmm::pmm_free_frame(pt_frame);
            return false;
        }
        string::memset(zva2, 0, PAGE_SIZE);
        win_unmap(PT_TEMP_IDX);
        pd.entries[pd_idx as usize] = (pt_phys & PTE_ADDR_MASK) | 0x3;
        flush_tlb();
        pde = pd.entries[pd_idx as usize];
    } else if pde & 0x80 != 0 {
        // Split a 2 MiB huge page.
        let huge_base = pde & 0xffff_ffff_fe00;
        let huge_flags = pde & 0xfff;
        let pt_flags = huge_flags & !0x80;
        let pt_frame = pmm::pmm_alloc_frame();
        if pt_frame.is_null() {
            pmm::pmm_free_frame(frame);
            return false;
        }
        let pt_phys = pt_frame as u64;
        let pt_fill_va = win_map(pt_phys, PT_TEMP_IDX);
        if pt_fill_va.is_null() {
            pmm::pmm_free_frame(frame);
            pmm::pmm_free_frame(pt_frame);
            return false;
        }
        let pt_tbl = pt_fill_va as *mut PageTable;
        for i in 0..512 {
            (*pt_tbl).entries[i] = huge_base + ((i as u64) << 12) | (pt_flags & !0x4) | PAGE_PRESENT;
        }
        win_unmap(PT_TEMP_IDX);
        pd.entries[pd_idx as usize] = (pt_phys & PTE_ADDR_MASK) | (pt_flags & !0x4);
        flush_tlb();
        pde = pd.entries[pd_idx as usize];
    }

    let pt_phys = pde & PTE_ADDR_MASK;
    if pt_phys == 0 {
        pmm::pmm_free_frame(frame);
        return false;
    }
    let pt_va = win_map(pt_phys, PT_TEMP_IDX);
    if pt_va.is_null() {
        pmm::pmm_free_frame(frame);
        return false;
    }
    let pt = pt_va as *mut PageTable;
    (*pt).entries[pt_idx as usize] = (phys & PTE_ADDR_MASK) | 0x3;
    win_unmap(PT_TEMP_IDX);
    invlpg(page_addr as usize);
    flush_tlb();
    true
}

// ----------------------------------------------------------------------------
// Temp frame window (used by the safe `with_temp_frame` helper)
// ----------------------------------------------------------------------------

/// `paging_temp_map_frame(phys_addr)`: map a frame into temp-window slot 511.
#[no_mangle]
pub unsafe extern "C" fn paging_temp_map_frame(phys_addr: usize) -> *mut c_void {
    win_map(phys_addr as u64, 511)
}

/// `paging_temp_unmap_frame()`: release temp-window slot 511.
#[no_mangle]
pub unsafe extern "C" fn paging_temp_unmap_frame() {
    win_unmap(511);
}
