#include "paging.h"
#include "pmm.h"
#include "../drivers/serial.h"

Paging g_paging;

extern uintptr_t _kernel_start;
extern uintptr_t _kernel_end;

// ============================================================================
// x86_64 implementation — 4-level PML4 paging, 64-bit entries
// ============================================================================
#ifdef ARCH_X86_64

/* Boot code sets up identity-mapped 2MB huge pages via:
 *   PML4 at phys 0x1000 → PDPT at phys 0x2000 → PD at phys 0x3000
 * All three tables are identity-mapped (within first 4MB).              */

#define X86_64_PML4_PHYS    0x1000ULL
#define X86_64_PDPT_PHYS    0x2000ULL
#define X86_64_PD_PHYS      0x3000ULL

#define X86_64_PML4_VIRT    ((struct page_directory*)X86_64_PML4_PHYS)
#define X86_64_PDPT0_VIRT   ((struct page_table*)X86_64_PDPT_PHYS)
#define X86_64_PD_VIRT      ((struct page_table*)X86_64_PD_PHYS)

/* Window for temporary 4KB page-table mappings.
 * PD[PD_WIN_IDX] points to an allocated PT page; virtual address:
 *   PML4[0] → PDPT[0] → PD[PD_WIN_IDX] → PT[pt_idx]
 *   VA = PD_WIN_IDX << 21 | pt_idx << 12                                 */
#define PD_WIN_IDX   8
#define PT_WIN_BASE  ((uint64_t)PD_WIN_IDX << 21)   /* 0x1000000 = 16 MB */
#define PT_TEMP_IDX  0
#define PT_TEMP_VA   (PT_WIN_BASE + (PT_TEMP_IDX << 12))

/* Second window slot for map_page_in_pd / destroy_directory etc. */
#define PD_WIN2_IDX  9
#define PT_WIN2_BASE  ((uint64_t)PD_WIN2_IDX << 21)  /* 0x1200000 = 18 MB */

static uint64_t kernel_pml4_phys = X86_64_PML4_PHYS;

/* Allocated PT page for the PD_WIN window — allocated once in paging_init. */
static struct page_table* g_win_pt = 0;

static inline void invlpg(uint64_t virt) {
    asm volatile("invlpg (%0)" : : "r"(virt) : "memory");
}

/* Map a physical 4KB frame at the PD_WIN window (returns virtual address). */
static void* win_map(uint64_t phys, int pt_idx) {
    struct page_table* pd = X86_64_PD_VIRT;
    uint64_t old = pd->entries[PD_WIN_IDX];
    if ((old & 1) == 0 || (old & 0x80) != 0) {
        /* PD_WIN_IDX entry not yet a 4KB PT pointer — set it up once. */
        if (!g_win_pt) {
            /* Allocate on first call */
            void* pt_frame = pmm_alloc_frame();
            if (!pt_frame) return 0;
            g_win_pt = (struct page_table*)((uint64_t)(uintptr_t)pt_frame);
            /* Zero it */
            __builtin_memset(g_win_pt, 0, 4096);
        }
        uint64_t pt_phys = (uint64_t)(uintptr_t)g_win_pt;
        pd->entries[PD_WIN_IDX] = (pt_phys & 0xFFFFFFFFF000ULL) | 0x03;
        invlpg(PT_WIN_BASE);
    }
    uint64_t va = PT_WIN_BASE + (uint64_t)pt_idx * 4096;
    /* Write the PTE in the window PT */
    struct page_table* win_pt = (struct page_table*)(uintptr_t)(PT_WIN_BASE);
    win_pt->entries[pt_idx] = (phys & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(va);
    return (void*)(uintptr_t)va;
}

static void win_unmap(int pt_idx) {
    uint64_t va = PT_WIN_BASE + (uint64_t)pt_idx * 4096;
    struct page_table* win_pt = (struct page_table*)(uintptr_t)(PT_WIN_BASE);
    win_pt->entries[pt_idx] = 0;
    invlpg(va);
}

/* Helper: same for PD_WIN2 */
static void* win2_map(uint64_t phys, int pt_idx) {
    struct page_table* pd = X86_64_PD_VIRT;
    uint64_t pt_phys = (uint64_t)(uintptr_t)g_win_pt;
    uint64_t win2_pt_phys = pt_phys + 0x1000; /* one page after win_pt */
    pd->entries[PD_WIN2_IDX] = (win2_pt_phys & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(PT_WIN2_BASE);
    uint64_t va = PT_WIN2_BASE + (uint64_t)pt_idx * 4096;
    struct page_table* win2_pt = (struct page_table*)(uintptr_t)(PT_WIN2_BASE);
    win2_pt->entries[pt_idx] = (phys & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(va);
    return (void*)(uintptr_t)va;
}

static void win2_unmap(int pt_idx) {
    uint64_t va = PT_WIN2_BASE + (uint64_t)pt_idx * 4096;
    struct page_table* win2_pt = (struct page_table*)(uintptr_t)(PT_WIN2_BASE);
    win2_pt->entries[pt_idx] = 0;
    invlpg(va);
}

void paging_init() {
    serial_print("Paging: Initializing x86_64 paging...\n");

    uint64_t cr3;
    asm volatile("mov %%cr3, %0" : "=r"(cr3));
    kernel_pml4_phys = cr3 & 0xFFFFFFFFF000ULL;

    serial_print("  PML4 at phys 0x");
    serial_print_hex64(kernel_pml4_phys);
    serial_print("\n");

    g_paging.kernel_directory = X86_64_PML4_VIRT;

    /* Extend identity map to 16 MB: PD entries 2-7 as 2MB huge pages */
    struct page_table* pd = X86_64_PD_VIRT;
    for (int i = 2; i < 8; i++) {
        pd->entries[i] = ((uint64_t)i << 21) | 0x83ULL;
    }

    /* Pre-allocate the window page table */
    void* win_frame = pmm_alloc_frame();
    if (win_frame) {
        g_win_pt = (struct page_table*)(uintptr_t)win_frame;
        __builtin_memset(g_win_pt, 0, 4096);
        /* Also zero the page after win_pt for win2 */
        __builtin_memset((void*)((uintptr_t)g_win_pt + 0x1000), 0, 4096);
    }

    serial_print("  Identity map extended to 16 MB\n");
    serial_print("  Window PT at phys 0x");
    serial_print_hex64((uint64_t)(uintptr_t)g_win_pt);
    serial_print("\n");
}

void paging_enable() {
    serial_print("Paging: Already enabled (set up by boot code)\n");
}

/* 4-level page walk: PML4 → PDPT → PD → PT.
 * For virtual addresses in the first 16 MB (PML4[0], PDPT[0]),
 * the tables are identity-mapped and directly accessible.       */
static uint64_t* get_page_entry(uint64_t virt_addr, bool create) {
    uint64_t pml4_idx = (virt_addr >> 39) & 0x1FF;
    uint64_t pdpt_idx = (virt_addr >> 30) & 0x1FF;
    uint64_t pd_idx   = (virt_addr >> 21) & 0x1FF;
    uint64_t pt_idx   = (virt_addr >> 12) & 0x1FF;

    struct page_directory* pml4 = X86_64_PML4_VIRT;

    /* PML4[0] is identity-mapped. For simplicity, require PML4[0] for now. */
    if (pml4_idx != 0) {
        if (!create) return 0;
        /* Allocate new PDPT for this PML4 entry */
        void* new_pdpt = pmm_alloc_frame();
        if (!new_pdpt) return 0;
        uint64_t pdpt_phys = (uint64_t)(uintptr_t)new_pdpt;
        __builtin_memset(new_pdpt, 0, 4096);
        pml4->entries[pml4_idx] = (pdpt_phys & 0xFFFFFFFFF000ULL) | 0x03;
    }

    uint64_t pdpt_entry = pml4->entries[pml4_idx];
    uint64_t pdpt_phys = pdpt_entry & 0xFFFFFFFFF000ULL;
    struct page_table* pdpt;
    if (pdpt_phys == X86_64_PDPT_PHYS) {
        pdpt = X86_64_PDPT0_VIRT;
    } else {
        pdpt = (struct page_table*)win_map(pdpt_phys, PT_TEMP_IDX);
    }

    /* PDPT entry: must point to a PD or be a 1GB page */
    uint64_t pdpde = pdpt->entries[pdpt_idx];
    if (!(pdpde & 1)) {
        if (!create) {
            if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);
            return 0;
        }
        void* new_pd = pmm_alloc_frame();
        if (!new_pd) {
            if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);
            return 0;
        }
        uint64_t pd_phys = (uint64_t)(uintptr_t)new_pd;
        __builtin_memset(new_pd, 0, 4096);
        pdpt->entries[pdpt_idx] = (pd_phys & 0xFFFFFFFFF000ULL) | 0x03;
        pdpde = pdpt->entries[pdpt_idx];
    }

    uint64_t pd_phys = pdpde & 0xFFFFFFFFF000ULL;
    struct page_table* pd_tbl;
    if (pd_phys == X86_64_PD_PHYS) {
        pd_tbl = X86_64_PD_VIRT;
    } else {
        /* Map PD through window (need a second slot since pdpt may be using it) */
        pd_tbl = (struct page_table*)win2_map(pd_phys, PT_TEMP_IDX);
    }

    /* PD entry: could be a 2MB huge page or a PT pointer */
    uint64_t pde = pd_tbl->entries[pd_idx];
    if (!(pde & 1)) {
        if (!create) {
            if (pd_phys != X86_64_PD_PHYS) win2_unmap(PT_TEMP_IDX);
            if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);
            return 0;
        }
        void* new_pt = pmm_alloc_frame();
        if (!new_pt) {
            if (pd_phys != X86_64_PD_PHYS) win2_unmap(PT_TEMP_IDX);
            if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);
            return 0;
        }
        uint64_t pt_phys = (uint64_t)(uintptr_t)new_pt;
        __builtin_memset(new_pt, 0, 4096);
        pd_tbl->entries[pd_idx] = (pt_phys & 0xFFFFFFFFF000ULL) | 0x03;
        pde = pd_tbl->entries[pd_idx];
    }

    uint64_t pt_phys = pde & 0xFFFFFFFFF000ULL;
    /* Map the 4KB PT into the window so we can access individual PTEs */
    if (pd_phys != X86_64_PD_PHYS) win2_unmap(PT_TEMP_IDX);
    if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);

    void* win_va = win_map(pt_phys, PT_TEMP_IDX);
    if (!win_va) return 0;

    struct page_table* pt = (struct page_table*)win_va;
    return &pt->entries[pt_idx];
}

static void invalidate_page(uint64_t virt_addr) {
    asm volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
}

bool paging_map_page(uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags) {
    uint64_t* pte = get_page_entry((uint64_t)virt_addr, true);
    if (!pte) return false;
    *pte = ((uint64_t)phys_addr & 0xFFFFFFFFF000ULL) | (flags & 0xFFFULL) | 1;
    invalidate_page(virt_addr);
    return true;
}

void paging_unmap_page(uintptr_t virt_addr) {
    uint64_t* pte = get_page_entry((uint64_t)virt_addr, false);
    if (pte) {
        *pte = 0;
        invalidate_page(virt_addr);
    }
}

uintptr_t paging_get_physical_address(uintptr_t virt_addr) {
    uint64_t* pte = get_page_entry((uint64_t)virt_addr, false);
    if (!pte || !(*pte & 1)) return 0;
    return (uintptr_t)((*pte & 0xFFFFFFFFF000ULL) | (virt_addr & 0xFFF));
}

uintptr_t paging_create_directory_phys() {
    /* Allocate a new PML4 frame */
    void* pd_frame = pmm_alloc_frame();
    if (!pd_frame) {
        serial_print("Paging: ERROR - Failed to allocate PML4 frame\n");
        return 0;
    }
    uint64_t new_pml4_phys = (uint64_t)(uintptr_t)pd_frame;

    /* Map it through the window and zero it */
    void* win_va = win_map(new_pml4_phys, PT_TEMP_IDX);
    if (!win_va) {
        pmm_free_frame(pd_frame);
        return 0;
    }
    struct page_directory* new_pml4 = (struct page_directory*)win_va;
    for (int i = 0; i < 512; i++) {
        new_pml4->entries[i] = 0;
    }

    /* Copy kernel PML4 entries (only index 0 for now) */
    struct page_directory* cur_pml4 = X86_64_PML4_VIRT;
    new_pml4->entries[0] = cur_pml4->entries[0];

    win_unmap(PT_TEMP_IDX);
    return (uintptr_t)new_pml4_phys;
}

void paging_destroy_directory(uintptr_t pd_phys) {
    if (!pd_phys) return;
    serial_print("Paging: Destroying page directory (x86_64)\n");

    void* win_va = win_map((uint64_t)pd_phys, PT_TEMP_IDX);
    if (!win_va) return;
    struct page_directory* pml4 = (struct page_directory*)win_va;

    /* Walk PML4 entries 4..511 (skip kernel entries 0..3). */
    for (int pml4_i = 4; pml4_i < 512; pml4_i++) {
        uint64_t pml4e = pml4->entries[pml4_i];
        if (!(pml4e & 1)) continue;

        uint64_t pdpt_phys = pml4e & 0xFFFFFFFFF000ULL;
        void* pdpt_va = win_map(pdpt_phys, PT_TEMP_IDX);
        (void)pdpt_va;

        /* For each PDPT entry, walk PD, then PT */
        for (int pdpt_i = 0; pdpt_i < 512; pdpt_i++) {
            uint64_t pdpde = ((struct page_table*)win_va)->entries[pdpt_i];
            if (!(pdpde & 1)) continue;

            uint64_t pd_phys_entry = pdpde & 0xFFFFFFFFF000ULL;
            struct page_table* pd_tbl = (struct page_table*)win_map(pd_phys_entry, PT_TEMP_IDX);
            if (!pd_tbl) continue;

            for (int pd_i = 0; pd_i < 512; pd_i++) {
                uint64_t pd_entry = pd_tbl->entries[pd_i];
                if (!(pd_entry & 1)) continue;
                /* 2MB huge page? Free the frame, skip PT walk */
                if (pd_entry & 0x80) {
                    uint64_t frame = pd_entry & 0xFFFFFFFFF000ULL;
                    pmm_free_frame((void*)(uintptr_t)frame);
                    pd_tbl->entries[pd_i] = 0;
                    continue;
                }
                /* 4KB PT pointer */
                uint64_t pt_phys = pd_entry & 0xFFFFFFFFF000ULL;
                struct page_table* pt = (struct page_table*)win_map(pt_phys, PT_TEMP_IDX);
                if (!pt) continue;

                for (int pt_i = 0; pt_i < 512; pt_i++) {
                    uint64_t pte = pt->entries[pt_i];
                    if (!(pte & 1)) continue;
                    uint64_t frame = pte & 0xFFFFFFFFF000ULL;
                    if (pte & PAGE_COW) {
                        pmm_refcount_dec((void*)(uintptr_t)frame);
                    } else {
                        pmm_free_frame((void*)(uintptr_t)frame);
                    }
                    pt->entries[pt_i] = 0;
                }
                pmm_free_frame((void*)(uintptr_t)pt_phys);
                pd_tbl->entries[pd_i] = 0;
            }
            pmm_free_frame((void*)(uintptr_t)pd_phys_entry);
            ((struct page_table*)win_va)->entries[pdpt_i] = 0;
        }
        pmm_free_frame((void*)(uintptr_t)pdpt_phys);
        pml4->entries[pml4_i] = 0;
    }

    win_unmap(PT_TEMP_IDX);
    pmm_free_frame((void*)(uintptr_t)pd_phys);
}

bool paging_switch_to_directory(uintptr_t pd_phys) {
    if (!pd_phys) return false;
    asm volatile("mov %0, %%cr3" : : "r"((uint64_t)pd_phys) : "memory");
    return true;
}

uintptr_t paging_get_kernel_directory_phys() {
    return (uintptr_t)kernel_pml4_phys;
}

/* Helper: deep-copy a single 4KB page table. src/dst PTs must be mapped
 * through win/win2 respectively before calling.                          */
static void clone_page_table(struct page_table* src_pt,
                              struct page_table* dst_pt) {
    for (int i = 0; i < 512; i++) {
        uint64_t pte = src_pt->entries[i];
        if (!(pte & 1)) continue;

        uint64_t src_frame = pte & 0xFFFFFFFFF000ULL;
        void* new_frame = pmm_alloc_frame();
        if (!new_frame) {
            serial_print("Paging: clone - OOM during page copy\n");
            continue;
        }
        __builtin_memcpy(new_frame, (void*)(uintptr_t)src_frame, PAGE_SIZE);
        dst_pt->entries[i] = ((uint64_t)(uintptr_t)new_frame & 0xFFFFFFFFF000ULL)
                           | (pte & 0xFFFULL) | 1;
    }
}

uintptr_t paging_clone_directory(uintptr_t pd_phys) {
    serial_print("Paging: Cloning page directory (x86_64)\n");

    void* dst_pml4_frame = pmm_alloc_frame();
    if (!dst_pml4_frame) return 0;
    uint64_t dst_pml4_phys = (uint64_t)(uintptr_t)dst_pml4_frame;
    __builtin_memset(dst_pml4_frame, 0, 4096);

    /* Map source & dest PML4s simultaneously */
    struct page_directory* src_pml4 = (struct page_directory*)win_map(pd_phys, 0);
    struct page_directory* dst_pml4 = (struct page_directory*)win2_map(dst_pml4_phys, 0);

    /* Share kernel entries (0-3) — these point into the identity-mapped
     * PML4→PDPT→PD hierarchy used by all kernel code.                  */
    for (int i = 0; i < 4; i++)
        dst_pml4->entries[i] = src_pml4->entries[i];

    for (int pml4_i = 4; pml4_i < 512; pml4_i++) {
        uint64_t pml4e = src_pml4->entries[pml4_i];
        if (!(pml4e & 1)) continue;

        uint64_t src_pdpt_phys = pml4e & 0xFFFFFFFFF000ULL;
        uint64_t pdpt_flags    = pml4e & 0xFFFULL;

        void* dst_pdpt_frame = pmm_alloc_frame();
        if (!dst_pdpt_frame) goto fail;
        uint64_t dst_pdpt_phys = (uint64_t)(uintptr_t)dst_pdpt_frame;
        __builtin_memset(dst_pdpt_frame, 0, 4096);
        dst_pml4->entries[pml4_i] = (dst_pdpt_phys & 0xFFFFFFFFF000ULL)
                                  | pdpt_flags | 1;

        /* Switch windows to src/dst PDPT */
        win2_unmap(0);
        win_unmap(0);
        struct page_table* src_pdpt = (struct page_table*)win_map(src_pdpt_phys, 0);
        struct page_table* dst_pdpt = (struct page_table*)win2_map(dst_pdpt_phys, 0);

        for (int pdpt_i = 0; pdpt_i < 512; pdpt_i++) {
            uint64_t pdpde = src_pdpt->entries[pdpt_i];
            if (!(pdpde & 1)) continue;

            uint64_t src_pd_phys = pdpde & 0xFFFFFFFFF000ULL;
            uint64_t pd_flags    = pdpde & 0xFFFULL;

            void* dst_pd_frame = pmm_alloc_frame();
            if (!dst_pd_frame) goto fail;
            uint64_t dst_pd_phys = (uint64_t)(uintptr_t)dst_pd_frame;
            __builtin_memset(dst_pd_frame, 0, 4096);
            dst_pdpt->entries[pdpt_i] = (dst_pd_phys & 0xFFFFFFFFF000ULL)
                                      | pd_flags | 1;

            /* Switch windows to src/dst PD */
            win2_unmap(0);
            win_unmap(0);
            struct page_table* src_pd = (struct page_table*)win_map(src_pd_phys, 0);
            struct page_table* dst_pd = (struct page_table*)win2_map(dst_pd_phys, 0);

            for (int pd_i = 0; pd_i < 512; pd_i++) {
                uint64_t pde = src_pd->entries[pd_i];
                if (!(pde & 1)) continue;

                if (pde & 0x80) {
                    /* 2MB huge page — break into 512 4KB pages for the clone.
                     * Source frame_addr is the base of the 2MB region.       */
                    uint64_t huge_base = pde & 0xFFFFFFFFF000ULL;
                    uint64_t src_flags = pde & 0xFFFULL;

                    void* dst_pt_frame = pmm_alloc_frame();
                    if (!dst_pt_frame) goto fail;
                    uint64_t dst_pt_phys = (uint64_t)(uintptr_t)dst_pt_frame;
                    __builtin_memset(dst_pt_frame, 0, 4096);

                    dst_pd->entries[pd_i] = (dst_pt_phys & 0xFFFFFFFFF000ULL)
                                          | (src_flags & ~0x80ULL) | 1;

                    win2_unmap(0);
                    win_unmap(0);
                    struct page_table* dst_pt = (struct page_table*)win_map(dst_pt_phys, 0);

                    for (int pt_i = 0; pt_i < 512; pt_i++) {
                        uint64_t phys = huge_base + ((uint64_t)pt_i << 12);
                        void* new_frame = pmm_alloc_frame();
                        if (!new_frame) continue;
                        __builtin_memcpy(new_frame, (void*)(uintptr_t)phys, PAGE_SIZE);
                        dst_pt->entries[pt_i] =
                            ((uint64_t)(uintptr_t)new_frame & 0xFFFFFFFFF000ULL)
                            | (src_flags & (0xFFFULL & ~0x80ULL)) | 1;
                    }

                    win_unmap(0);
                    /* Re-map both src and dst PD for next PD iteration */
                    win_map(src_pd_phys, 0);
                    win2_map(dst_pd_phys, 0);
                } else {
                    /* 4KB page table */
                    uint64_t src_pt_phys = pde & 0xFFFFFFFFF000ULL;
                    uint64_t pt_flags    = pde & 0xFFFULL;

                    void* dst_pt_frame = pmm_alloc_frame();
                    if (!dst_pt_frame) goto fail;
                    uint64_t dst_pt_phys = (uint64_t)(uintptr_t)dst_pt_frame;
                    __builtin_memset(dst_pt_frame, 0, 4096);

                    dst_pd->entries[pd_i] = (dst_pt_phys & 0xFFFFFFFFF000ULL)
                                          | pt_flags | 1;

                    win2_unmap(0);
                    win_unmap(0);
                    struct page_table* src_pt = (struct page_table*)win_map(src_pt_phys, 0);
                    struct page_table* dst_pt = (struct page_table*)win2_map(dst_pt_phys, 0);

                    clone_page_table(src_pt, dst_pt);

                    win_unmap(0);
                    win2_unmap(0);
                    /* Re-map both src and dst PD for next PD iteration */
                    win_map(src_pd_phys, 0);
                    win2_map(dst_pd_phys, 0);
                }
            }

            win_unmap(0);
            win2_unmap(0);
            /* Re-map both src and dst PDPT for next PDPT iteration */
            win_map(src_pdpt_phys, 0);
            win2_map(dst_pdpt_phys, 0);
        }

        win_unmap(0);
        win2_unmap(0);
        /* Re-map both src and dst PML4 for next PML4 iteration */
        win_map(pd_phys, 0);
        win2_map(dst_pml4_phys, 0);
    }

    win_unmap(0);
    win2_unmap(0);
    serial_print("Paging: Clone complete\n");
    return (uintptr_t)dst_pml4_phys;

fail:
    serial_print("Paging: Clone failed - out of memory\n");
    /* Best-effort cleanup: free dst_pml4 (user data is leaked but
     * the kernel can continue).                                      */
    pmm_free_frame((void*)(uintptr_t)dst_pml4_phys);
    win_unmap(0);
    win2_unmap(0);
    return 0;
}

static void fork_fork_pt(struct page_table* src_pt,
                          struct page_table* dst_pt) {
    for (int i = 0; i < 512; i++) {
        uint64_t pte = src_pt->entries[i];
        if (!(pte & 1)) {
            dst_pt->entries[i] = 0;
            continue;
        }

        uint64_t frame_phys = pte & 0xFFFFFFFFF000ULL;
        uint64_t flags      = pte & 0xFFFULL;

        if (flags & PAGE_WRITE) {
            /* Writable → make both read-only + COW */
            flags &= ~PAGE_WRITE;
            flags |= PAGE_COW;
            pmm_refcount_inc((void*)(uintptr_t)frame_phys);

            src_pt->entries[i] = frame_phys | flags;
            dst_pt->entries[i] = frame_phys | flags;
        } else {
            /* Already read-only — share with COW (if not already) */
            flags |= PAGE_COW;
            pmm_refcount_inc((void*)(uintptr_t)frame_phys);

            src_pt->entries[i] = frame_phys | flags;
            dst_pt->entries[i] = frame_phys | flags;
        }
    }
}

uintptr_t paging_fork_directory(uintptr_t pd_phys) {
    serial_print("Paging: Forking page directory (x86_64, COW)\n");

    void* dst_pml4_frame = pmm_alloc_frame();
    if (!dst_pml4_frame) return 0;
    uint64_t dst_pml4_phys = (uint64_t)(uintptr_t)dst_pml4_frame;
    __builtin_memset(dst_pml4_frame, 0, 4096);

    struct page_directory* src_pml4 = (struct page_directory*)win_map(pd_phys, 0);
    struct page_directory* dst_pml4 = (struct page_directory*)win2_map(dst_pml4_phys, 0);

    for (int i = 0; i < 4; i++)
        dst_pml4->entries[i] = src_pml4->entries[i];

    for (int pml4_i = 4; pml4_i < 512; pml4_i++) {
        uint64_t pml4e = src_pml4->entries[pml4_i];
        if (!(pml4e & 1)) continue;

        uint64_t src_pdpt_phys = pml4e & 0xFFFFFFFFF000ULL;
        uint64_t pdpt_flags    = pml4e & 0xFFFULL;

        void* dst_pdpt_frame = pmm_alloc_frame();
        if (!dst_pdpt_frame) goto fork_fail;
        uint64_t dst_pdpt_phys = (uint64_t)(uintptr_t)dst_pdpt_frame;
        __builtin_memset(dst_pdpt_frame, 0, 4096);
        dst_pml4->entries[pml4_i] = (dst_pdpt_phys & 0xFFFFFFFFF000ULL)
                                  | pdpt_flags | 1;

        win2_unmap(0);
        win_unmap(0);
        struct page_table* src_pdpt = (struct page_table*)win_map(src_pdpt_phys, 0);
        struct page_table* dst_pdpt = (struct page_table*)win2_map(dst_pdpt_phys, 0);

        for (int pdpt_i = 0; pdpt_i < 512; pdpt_i++) {
            uint64_t pdpde = src_pdpt->entries[pdpt_i];
            if (!(pdpde & 1)) continue;

            uint64_t src_pd_phys = pdpde & 0xFFFFFFFFF000ULL;
            uint64_t pd_flags    = pdpde & 0xFFFULL;

            void* dst_pd_frame = pmm_alloc_frame();
            if (!dst_pd_frame) goto fork_fail;
            uint64_t dst_pd_phys = (uint64_t)(uintptr_t)dst_pd_frame;
            __builtin_memset(dst_pd_frame, 0, 4096);
            dst_pdpt->entries[pdpt_i] = (dst_pd_phys & 0xFFFFFFFFF000ULL)
                                      | pd_flags | 1;

            win2_unmap(0);
            win_unmap(0);
            struct page_table* src_pd = (struct page_table*)win_map(src_pd_phys, 0);
            struct page_table* dst_pd = (struct page_table*)win2_map(dst_pd_phys, 0);

            for (int pd_i = 0; pd_i < 512; pd_i++) {
                uint64_t pde = src_pd->entries[pd_i];
                if (!(pde & 1)) continue;

                if (pde & 0x80) {
                    /* 2MB huge page — share as-is (kernel identity pages) */
                    dst_pd->entries[pd_i] = pde;
                } else {
                    uint64_t src_pt_phys = pde & 0xFFFFFFFFF000ULL;
                    uint64_t pt_flags    = pde & 0xFFFULL;

                    void* dst_pt_frame = pmm_alloc_frame();
                    if (!dst_pt_frame) goto fork_fail;
                    uint64_t dst_pt_phys = (uint64_t)(uintptr_t)dst_pt_frame;
                    __builtin_memset(dst_pt_frame, 0, 4096);

                    dst_pd->entries[pd_i] = (dst_pt_phys & 0xFFFFFFFFF000ULL)
                                          | pt_flags | 1;

                    win2_unmap(0);
                    win_unmap(0);
                    struct page_table* src_pt = (struct page_table*)win_map(src_pt_phys, 0);
                    struct page_table* dst_pt = (struct page_table*)win2_map(dst_pt_phys, 0);

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
        win_map(pd_phys, 0);
        win2_map(dst_pml4_phys, 0);
    }

    win_unmap(0);
    win2_unmap(0);
    serial_print("Paging: Fork complete (COW)\n");
    return (uintptr_t)dst_pml4_phys;

fork_fail:
    serial_print("Paging: Fork failed - out of memory\n");
    pmm_free_frame((void*)(uintptr_t)dst_pml4_phys);
    win_unmap(0);
    win2_unmap(0);
    return 0;
}

uint8_t paging_handle_cow_fault(uintptr_t fault_addr) {
    uint64_t* pte = get_page_entry((uint64_t)fault_addr, false);
    if (!pte || !(*pte & 1)) return 0;

    uint64_t flags = *pte & 0xFFFULL;
    if (!(flags & PAGE_COW)) return 0;

    uint64_t old_frame = *pte & 0xFFFFFFFFF000ULL;

    void* new_frame = pmm_alloc_frame();
    if (!new_frame) {
        serial_print("Paging: COW - out of memory!\n");
        return 0;
    }

    __builtin_memcpy(new_frame, (void*)(uintptr_t)old_frame, PAGE_SIZE);

    pmm_refcount_dec((void*)(uintptr_t)old_frame);

    flags &= ~PAGE_COW;
    flags |= PAGE_WRITE;
    *pte = ((uint64_t)(uintptr_t)new_frame & 0xFFFFFFFFF000ULL) | flags;

    invalidate_page((uint64_t)fault_addr);

    return 1;
}

bool paging_map_page_in_pd(uintptr_t pd_phys, uintptr_t virt_addr,
                            uintptr_t phys_addr, uint32_t flags) {
    /* Map source PD through window */
    void* win_va = win_map((uint64_t)pd_phys, PT_TEMP_IDX);
    if (!win_va) return false;
    struct page_directory* pml4 = (struct page_directory*)win_va;

    uint64_t vaddr = (uint64_t)virt_addr;
    uint64_t pml4_idx = (vaddr >> 39) & 0x1FF;
    uint64_t pdpt_idx = (vaddr >> 30) & 0x1FF;
    uint64_t pd_idx   = (vaddr >> 21) & 0x1FF;
    uint64_t pt_idx   = (vaddr >> 12) & 0x1FF;

    /* First 16 MB is shared kernel space — use kernel_pts */
    if (pml4_idx == 0 && pdpt_idx == 0 && pd_idx < 8) {
        /* For the identity-mapped PD entries, need a proper 4KB PT window.
         * Since PD entries 0-7 are 2MB huge pages, we can't do 4KB mapping there.
         * Use the dedicated window PT instead. */
        win_unmap(PT_TEMP_IDX);
        /* Share kernel page tables by cloning entries into the target PML4 */
        pml4->entries[0] = X86_64_PML4_VIRT->entries[0];
        /* Map through window page table at PD_WIN_IDX */
        struct page_table* win_pt = (struct page_table*)(uintptr_t)PT_WIN_BASE;
        win_pt->entries[pt_idx] = ((uint64_t)phys_addr & 0xFFFFFFFFF000ULL) | (flags & 0xFFFULL) | 1;
        invlpg(virt_addr);
        return true;
    }

    /* Walk/create page tables in the target PML4 for non-shared addresses */
    /* --- PML4 entry --- */
    if (!(pml4->entries[pml4_idx] & 1)) {
        void* new_pdpt = pmm_alloc_frame();
        if (!new_pdpt) { win_unmap(PT_TEMP_IDX); return false; }
        __builtin_memset(new_pdpt, 0, 4096);
        pml4->entries[pml4_idx] = ((uint64_t)(uintptr_t)new_pdpt & 0xFFFFFFFFF000ULL) | 0x03;
    }

    /* --- PDPT entry --- */
    uint64_t pdpt_phys = pml4->entries[pml4_idx] & 0xFFFFFFFFF000ULL;
    win_unmap(PT_TEMP_IDX);
    void* pdpt_va = win_map(pdpt_phys, PT_TEMP_IDX);
    if (!pdpt_va) return false;
    struct page_table* pdpt = (struct page_table*)pdpt_va;

    if (!(pdpt->entries[pdpt_idx] & 1)) {
        void* new_pd = pmm_alloc_frame();
        if (!new_pd) { win_unmap(PT_TEMP_IDX); return false; }
        __builtin_memset(new_pd, 0, 4096);
        pdpt->entries[pdpt_idx] = ((uint64_t)(uintptr_t)new_pd & 0xFFFFFFFFF000ULL) | 0x03;
    }

    /* --- PD entry --- */
    uint64_t pd_phys_tbl = pdpt->entries[pdpt_idx] & 0xFFFFFFFFF000ULL;
    win_unmap(PT_TEMP_IDX);
    void* pd_va = win_map(pd_phys_tbl, PT_TEMP_IDX);
    if (!pd_va) return false;
    struct page_table* pd_tbl = (struct page_table*)pd_va;

    if (!(pd_tbl->entries[pd_idx] & 1)) {
        void* new_pt = pmm_alloc_frame();
        if (!new_pt) { win_unmap(PT_TEMP_IDX); return false; }
        __builtin_memset(new_pt, 0, 4096);
        pd_tbl->entries[pd_idx] = ((uint64_t)(uintptr_t)new_pt & 0xFFFFFFFFF000ULL) | 0x03;
    }

    /* --- PT entry --- */
    uint64_t pt_phys = pd_tbl->entries[pd_idx] & 0xFFFFFFFFF000ULL;
    win_unmap(PT_TEMP_IDX);
    void* pt_va = win_map(pt_phys, PT_TEMP_IDX);
    if (!pt_va) return false;
    struct page_table* pt = (struct page_table*)pt_va;

    pt->entries[pt_idx] = ((uint64_t)phys_addr & 0xFFFFFFFFF000ULL) | (flags & 0xFFFULL) | 1;

    win_unmap(PT_TEMP_IDX);
    return true;
}

void* paging_temp_map_frame(uintptr_t phys_addr) {
    struct page_table* win_pt = (struct page_table*)(uintptr_t)(PT_WIN_BASE);
    win_pt->entries[PT_TEMP_IDX] = ((uint64_t)phys_addr & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(PT_TEMP_VA);
    return (void*)(uintptr_t)PT_TEMP_VA;
}

void paging_temp_unmap_frame(void) {
    struct page_table* win_pt = (struct page_table*)(uintptr_t)(PT_WIN_BASE);
    win_pt->entries[PT_TEMP_IDX] = 0;
    invlpg(PT_TEMP_VA);
}

// ============================================================================
// i686 implementation — 2-level paging, 32-bit entries
// ============================================================================
#else

static struct page_directory kernel_dir __attribute__((aligned(4096)));
static struct page_table kernel_pts[4] __attribute__((aligned(4096)));
static uintptr_t kernel_directory_phys;

#define PT_VIRT_BASE 0x00C00000
#define PT_VIRT_MAP_TABLE_INDEX 3

static inline void invalidate_page_local(uint32_t virt_addr) {
    asm volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
}

static struct page_table* map_page_table_window(uint32_t dir_index, uintptr_t pt_phys) {
    uint32_t pt_virt = PT_VIRT_BASE + (dir_index * PAGE_SIZE);
    kernel_pts[PT_VIRT_MAP_TABLE_INDEX].entries[dir_index] =
        (pt_phys & 0xFFFFF000) | PAGE_PRESENT | PAGE_WRITE;
    invalidate_page_local(pt_virt);
    return (struct page_table*)pt_virt;
}

void paging_init() {
    serial_print("Paging: Initializing paging...\n");

    g_paging.kernel_directory = &kernel_dir;
    kernel_directory_phys = (uint32_t)&kernel_dir;

    for (int i = 0; i < 1024; i++) {
        g_paging.kernel_directory->entries[i] = 0;
        g_paging.kernel_tables[i] = 0;
    }

    for (int pt = 0; pt < 4; pt++) {
        g_paging.kernel_tables[pt] = &kernel_pts[pt];

        for (int i = 0; i < 1024; i++) {
            kernel_pts[pt].entries[i] = 0;
        }

        for (int i = 0; i < 1024; i++) {
            uint32_t phys = (pt * 1024 + i) * 0x1000;
            kernel_pts[pt].entries[i] = phys | PAGE_PRESENT | PAGE_WRITE;
        }

        uint32_t pt_phys = (uint32_t)&kernel_pts[pt];
        g_paging.kernel_directory->entries[pt] = pt_phys | PAGE_PRESENT | PAGE_WRITE;
    }

    serial_print("Paging: Identity mapped first 16MB\n");
    serial_print("  Page directory at: 0x");
    serial_print_hex((uint32_t)g_paging.kernel_directory);
    serial_print("\n");
    serial_print("  Kernel start: 0x");
    serial_print_hex((uint32_t)&_kernel_start);
    serial_print("\n");
    serial_print("  Kernel end: 0x");
    serial_print_hex((uint32_t)&_kernel_end);
    serial_print("\n");
}

void paging_enable() {
    serial_print("Paging: Enabling paging...\n");

    uint32_t pd_phys = (uint32_t)g_paging.kernel_directory;
    asm volatile("mov %0, %%cr3" :: "r"(pd_phys));

    uint32_t cr0;
    asm volatile("mov %%cr0, %0" : "=r"(cr0));
    cr0 |= 0x80000000;
    asm volatile("mov %0, %%cr0" :: "r"(cr0));

    serial_print("Paging: Paging enabled successfully\n");
}

static uint32_t* get_page_entry(uint32_t virt_addr, bool create) {
    uint32_t dir_index = virt_addr >> 22;
    uint32_t table_index = (virt_addr >> 12) & 0x3FF;

    if (dir_index < 4) {
        return &kernel_pts[dir_index].entries[table_index];
    }

    if (!(g_paging.kernel_directory->entries[dir_index] & PAGE_PRESENT)) {
        if (!create) {
            return 0;
        }

        void* pt_phys = pmm_alloc_frame();
        if (!pt_phys) {
            serial_print("Paging: ERROR - Failed to allocate page table\n");
            return 0;
        }

        struct page_table* pt = map_page_table_window(dir_index, (uint32_t)pt_phys);
        for (int i = 0; i < 1024; i++) {
            pt->entries[i] = 0;
        }

        g_paging.kernel_directory->entries[dir_index] = (uint32_t)pt_phys | PAGE_PRESENT | PAGE_WRITE;
        g_paging.kernel_tables[dir_index] = pt;
    }

    struct page_table* pt = g_paging.kernel_tables[dir_index];
    if (!pt) {
        uint32_t pt_phys = g_paging.kernel_directory->entries[dir_index] & 0xFFFFF000;
        pt = map_page_table_window(dir_index, pt_phys);
        g_paging.kernel_tables[dir_index] = pt;
    }

    return &pt->entries[table_index];
}

static void invalidate_page(uint32_t virt_addr) {
    asm volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
}

bool paging_map_page(uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags) {
    uint32_t* page_entry = get_page_entry(virt_addr, true);
    if (!page_entry) {
        return false;
    }

    *page_entry = (phys_addr & 0xFFFFF000) | (flags & 0xFFF) | PAGE_PRESENT;
    invalidate_page(virt_addr);

    return true;
}

void paging_unmap_page(uintptr_t virt_addr) {
    uint32_t* page_entry = get_page_entry(virt_addr, false);
    if (page_entry) {
        *page_entry = 0;
        invalidate_page(virt_addr);
    }
}

uintptr_t paging_get_physical_address(uintptr_t virt_addr) {
    uint32_t* page_entry = get_page_entry(virt_addr, false);
    if (!page_entry || !(*page_entry & PAGE_PRESENT)) {
        return 0;
    }

    return (*page_entry & 0xFFFFF000) | (virt_addr & 0xFFF);
}

uintptr_t paging_create_directory_phys() {
    void* pd_phys = pmm_alloc_frame();
    if (!pd_phys) {
        serial_print("Paging: ERROR - Failed to allocate page directory frame\n");
        return 0;
    }

    const uint32_t TEMP_INDEX = 100;
    struct page_table* tmp = map_page_table_window(TEMP_INDEX, (uint32_t)(uintptr_t)pd_phys);
    if (!tmp) {
        serial_print("Paging: ERROR - Failed to map temporary page directory frame\n");
        pmm_free_frame(pd_phys);
        return 0;
    }

    struct page_directory* new_pd = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (TEMP_INDEX * PAGE_SIZE));
    for (int i = 0; i < 1024; i++) {
        new_pd->entries[i] = 0;
    }

    for (int i = 0; i < 4; i++) {
        new_pd->entries[i] = g_paging.kernel_directory->entries[i];
    }

    return (uintptr_t)pd_phys;
}

void paging_destroy_directory(uintptr_t pd_phys) {
    if (!pd_phys) return;

    serial_print("Paging: Destroying page directory\n");

    const uint32_t TEMP_INDEX = 101;
    struct page_table* tmp_pd_map = map_page_table_window(TEMP_INDEX, pd_phys);
    (void)tmp_pd_map;
    struct page_directory* pd = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (TEMP_INDEX * PAGE_SIZE));

    for (int dir = 4; dir < 1024; dir++) {
        uint32_t pde = pd->entries[dir];
        if (!(pde & PAGE_PRESENT)) continue;

        uint32_t pt_phys = pde & 0xFFFFF000;
        struct page_table* pt = map_page_table_window(dir, pt_phys);
        if (!pt) continue;

        for (int i = 0; i < 1024; i++) {
            uint32_t pte = pt->entries[i];
            if (!(pte & PAGE_PRESENT)) continue;
            uint32_t frame_phys = pte & 0xFFFFF000;
            if (pte & PAGE_COW) {
                pmm_refcount_dec((void*)(uintptr_t)frame_phys);
            } else {
                pmm_free_frame((void*)(uintptr_t)frame_phys);
            }
            pt->entries[i] = 0;
        }

        pmm_free_frame((void*)(uintptr_t)pt_phys);
        pd->entries[dir] = 0;
    }

    pmm_free_frame((void*)(uintptr_t)pd_phys);
}

bool paging_switch_to_directory(uintptr_t pd_phys) {
    if (!pd_phys) return false;

    const uint32_t SWITCH_INDEX = 200;
    struct page_table* mapped = map_page_table_window(SWITCH_INDEX, pd_phys);
    if (!mapped) {
        serial_print("Paging: ERROR - Failed to map page directory for switch\n");
        return false;
    }

    g_paging.kernel_directory = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (SWITCH_INDEX * PAGE_SIZE));
    g_paging.kernel_tables[SWITCH_INDEX] = mapped;

    asm volatile ("mov %0, %%cr3" :: "r"(pd_phys));

    return true;
}

uintptr_t paging_get_kernel_directory_phys() {
    return kernel_directory_phys;
}

uintptr_t paging_clone_directory(uintptr_t pd_phys) {
    if (!pd_phys) return 0;

    void* new_pd_phys = pmm_alloc_frame();
    if (!new_pd_phys) {
        serial_print("Paging: ERROR - Failed to allocate new directory frame for clone\n");
        return 0;
    }

    const uint32_t SRC_DIR_IDX = 100;
    const uint32_t DST_DIR_IDX = 101;

    struct page_table* src_map = map_page_table_window(SRC_DIR_IDX, pd_phys);
    struct page_table* dst_map = map_page_table_window(DST_DIR_IDX, (uintptr_t)new_pd_phys);
    if (!src_map || !dst_map) {
        pmm_free_frame(new_pd_phys);
        return 0;
    }

    struct page_directory* src_pd = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (SRC_DIR_IDX * PAGE_SIZE));
    struct page_directory* dst_pd = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (DST_DIR_IDX * PAGE_SIZE));

    for (int i = 0; i < 4; i++) {
        dst_pd->entries[i] = src_pd->entries[i];
    }

    for (int i = 4; i < 1024; i++) {
        dst_pd->entries[i] = 0;
    }

    for (int dir = 4; dir < 1024; dir++) {
        uint32_t pde = src_pd->entries[dir];
        if (!(pde & PAGE_PRESENT)) continue;

        uint32_t src_pt_phys = pde & 0xFFFFF000;

        void* dst_pt_phys = pmm_alloc_frame();
        if (!dst_pt_phys) {
            paging_destroy_directory((uintptr_t)new_pd_phys);
            pmm_free_frame(new_pd_phys);
            return 0;
        }

        struct page_table* src_pt = map_page_table_window(dir, src_pt_phys);
        if (!src_pt) {
            pmm_free_frame(dst_pt_phys);
            paging_destroy_directory((uintptr_t)new_pd_phys);
            return 0;
        }

        struct page_table* dst_pt = map_page_table_window(dir + 512, (uintptr_t)dst_pt_phys);
        if (!dst_pt) {
            pmm_free_frame(dst_pt_phys);
            paging_destroy_directory((uintptr_t)new_pd_phys);
            return 0;
        }

        for (int i = 0; i < 1024; i++) {
            uint32_t pte = src_pt->entries[i];
            if (!(pte & PAGE_PRESENT)) {
                dst_pt->entries[i] = 0;
                continue;
            }

            uint32_t src_frame = pte & 0xFFFFF000;
            uint32_t flags = pte & 0xFFF;

            void* new_frame = pmm_alloc_frame();
            if (!new_frame) {
                paging_destroy_directory((uintptr_t)new_pd_phys);
                return 0;
            }

            __builtin_memcpy(new_frame, (void*)(uintptr_t)src_frame, PAGE_SIZE);

            dst_pt->entries[i] = ((uintptr_t)new_frame & 0xFFFFF000) | flags;
        }

        dst_pd->entries[dir] = ((uintptr_t)dst_pt_phys & 0xFFFFF000) | (pde & 0xFFF);
    }

    return (uintptr_t)new_pd_phys;
}

uintptr_t paging_fork_directory(uintptr_t pd_phys) {
    if (!pd_phys) return 0;

    void* new_pd_phys = pmm_alloc_frame();
    if (!new_pd_phys) {
        serial_print("Paging: ERROR - Failed to allocate new directory frame for fork\n");
        return 0;
    }

    const uint32_t SRC_DIR_IDX = 100;
    const uint32_t DST_DIR_IDX = 101;

    struct page_table* src_map = map_page_table_window(SRC_DIR_IDX, pd_phys);
    struct page_table* dst_map = map_page_table_window(DST_DIR_IDX, (uintptr_t)new_pd_phys);
    if (!src_map || !dst_map) {
        pmm_free_frame(new_pd_phys);
        return 0;
    }

    struct page_directory* src_pd = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (SRC_DIR_IDX * PAGE_SIZE));
    struct page_directory* dst_pd = (struct page_directory*)((uintptr_t)PT_VIRT_BASE + (DST_DIR_IDX * PAGE_SIZE));

    for (int i = 0; i < 4; i++) {
        dst_pd->entries[i] = src_pd->entries[i];
    }

    for (int i = 4; i < 1024; i++) {
        dst_pd->entries[i] = 0;
    }

    for (int dir = 4; dir < 1024; dir++) {
        uint32_t pde = src_pd->entries[dir];
        if (!(pde & PAGE_PRESENT)) continue;

        uint32_t src_pt_phys = pde & 0xFFFFF000;

        void* dst_pt_phys = pmm_alloc_frame();
        if (!dst_pt_phys) {
            paging_destroy_directory((uintptr_t)new_pd_phys);
            pmm_free_frame(new_pd_phys);
            return 0;
        }

        struct page_table* src_pt = map_page_table_window(dir, src_pt_phys);
        if (!src_pt) {
            pmm_free_frame(dst_pt_phys);
            paging_destroy_directory((uintptr_t)new_pd_phys);
            return 0;
        }

        struct page_table* dst_pt = map_page_table_window(dir + 512, (uintptr_t)dst_pt_phys);
        if (!dst_pt) {
            pmm_free_frame(dst_pt_phys);
            paging_destroy_directory((uintptr_t)new_pd_phys);
            return 0;
        }

        for (int i = 0; i < 1024; i++) {
            uint32_t pte = src_pt->entries[i];
            if (!(pte & PAGE_PRESENT)) {
                dst_pt->entries[i] = 0;
                continue;
            }

            uint32_t frame_phys = pte & 0xFFFFF000;
            uint32_t flags = pte & 0xFFF;

            if (flags & PAGE_WRITE) {
                flags &= ~PAGE_WRITE;
                flags |= PAGE_COW;

                src_pt->entries[i] = frame_phys | flags;

                pmm_refcount_inc((void*)(uintptr_t)frame_phys);

                dst_pt->entries[i] = frame_phys | flags;
            } else {
                flags |= PAGE_COW;
                pmm_refcount_inc((void*)(uintptr_t)frame_phys);

                src_pt->entries[i] = frame_phys | flags;

                dst_pt->entries[i] = frame_phys | flags;
            }
        }

        dst_pd->entries[dir] = ((uintptr_t)dst_pt_phys & 0xFFFFF000) | (pde & 0xFFF);
    }

    return (uintptr_t)new_pd_phys;
}

#define PD_WIN  1010
#define PT_WIN  1011

bool paging_map_page_in_pd(uintptr_t pd_phys, uintptr_t virt_addr,
                            uintptr_t phys_addr, uint32_t flags) {
    kernel_pts[3].entries[PD_WIN] = (pd_phys & 0xFFFFF000) | PAGE_PRESENT | PAGE_WRITE;
    invalidate_page_local(PT_VIRT_BASE + PD_WIN * PAGE_SIZE);
    struct page_directory* target_pd =
        (struct page_directory*)(PT_VIRT_BASE + PD_WIN * PAGE_SIZE);

    uint32_t dir_index = virt_addr >> 22;
    uint32_t table_index = (virt_addr >> 12) & 0x3FF;

    if (dir_index < 4) {
        kernel_pts[dir_index].entries[table_index] =
            (phys_addr & 0xFFFFF000) | (flags & 0xFFF) | PAGE_PRESENT;
        invalidate_page_local(virt_addr);

        kernel_pts[3].entries[PD_WIN] = 0;
        invalidate_page_local(PT_VIRT_BASE + PD_WIN * PAGE_SIZE);
        return true;
    }

    uint32_t pde = target_pd->entries[dir_index];
    uint32_t pt_phys;

    if (!(pde & PAGE_PRESENT)) {
        void* new_pt = pmm_alloc_frame();
        if (!new_pt) {
            kernel_pts[3].entries[PD_WIN] = 0;
            invalidate_page_local(PT_VIRT_BASE + PD_WIN * PAGE_SIZE);
            return false;
        }
        pt_phys = (uint32_t)(uintptr_t)new_pt;

        kernel_pts[3].entries[PT_WIN] = pt_phys | PAGE_PRESENT | PAGE_WRITE;
        invalidate_page_local(PT_VIRT_BASE + PT_WIN * PAGE_SIZE);
        struct page_table* pt = (struct page_table*)(PT_VIRT_BASE + PT_WIN * PAGE_SIZE);
        for (int i = 0; i < 1024; i++)
            pt->entries[i] = 0;

        target_pd->entries[dir_index] = pt_phys | PAGE_PRESENT | PAGE_WRITE;
    } else {
        pt_phys = pde & 0xFFFFF000;
        kernel_pts[3].entries[PT_WIN] = pt_phys | PAGE_PRESENT | PAGE_WRITE;
        invalidate_page_local(PT_VIRT_BASE + PT_WIN * PAGE_SIZE);
    }

    struct page_table* pt = (struct page_table*)(PT_VIRT_BASE + PT_WIN * PAGE_SIZE);
    pt->entries[table_index] =
        (phys_addr & 0xFFFFF000) | (flags & 0xFFF) | PAGE_PRESENT;

    kernel_pts[3].entries[PD_WIN] = 0;
    kernel_pts[3].entries[PT_WIN] = 0;
    invalidate_page_local(PT_VIRT_BASE + PD_WIN * PAGE_SIZE);
    invalidate_page_local(PT_VIRT_BASE + PT_WIN * PAGE_SIZE);

    return true;
}

#define TEMP_DATA_WIN 1012

void* paging_temp_map_frame(uintptr_t phys_addr) {
    kernel_pts[3].entries[TEMP_DATA_WIN] =
        (phys_addr & 0xFFFFF000) | PAGE_PRESENT | PAGE_WRITE;
    invalidate_page_local(PT_VIRT_BASE + TEMP_DATA_WIN * PAGE_SIZE);
    return (void*)(PT_VIRT_BASE + TEMP_DATA_WIN * PAGE_SIZE);
}

void paging_temp_unmap_frame(void) {
    kernel_pts[3].entries[TEMP_DATA_WIN] = 0;
    invalidate_page_local(PT_VIRT_BASE + TEMP_DATA_WIN * PAGE_SIZE);
}

uint8_t paging_handle_cow_fault(uintptr_t fault_addr) {
    uint32_t* pte = get_page_entry(fault_addr, false);
    if (!pte || !(*pte & PAGE_PRESENT)) {
        return 0;
    }

    uint32_t flags = *pte & 0xFFF;
    if (!(flags & PAGE_COW)) {
        return 0;
    }

    uint32_t old_frame = *pte & 0xFFFFF000;

    void* new_frame = pmm_alloc_frame();
    if (!new_frame) {
        serial_print("Paging: ERROR - COW: out of memory!\n");
        return 0;
    }

    __builtin_memcpy(new_frame, (void*)(uintptr_t)old_frame, PAGE_SIZE);

    pmm_refcount_dec((void*)(uintptr_t)old_frame);

    flags &= ~PAGE_COW;
    flags |= PAGE_WRITE;
    *pte = ((uintptr_t)new_frame & 0xFFFFF000) | flags;

    invalidate_page_local(fault_addr);

    return 1;
}

#endif /* ARCH_X86_64 */
