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
 *   VA = PD_WIN_IDX << 21 | pt_idx << 12
 * Use PD index 10 (20 MB) — above 2 MB, below the heap (32 MB).
 * The kernel image may extend past 20 MB with large include_bytes! data,
 * so the window PTs are pre-filled with identity mappings to preserve
 * kernel data access after the huge page is replaced with a PT.          */
#define PD_WIN_IDX   10
#define PT_WIN_BASE  ((uint64_t)PD_WIN_IDX << 21)   /* 0x1400000 = 20 MB */
#define PT_TEMP_IDX  0
#define PT_TEMP_VA   (PT_WIN_BASE + (PT_TEMP_IDX << 12))

/* Second window slot for map_page_in_pd / destroy_directory etc. */
#define PD_WIN2_IDX  11
#define PT_WIN2_BASE  ((uint64_t)PD_WIN2_IDX << 21)  /* 0x1600000 = 22 MB */

/* Permanent accessor for g_win_pt via PD[12] (VA 24 MB).
 * Identity-mapped pages at g_win_pt's physical address get clobbered when
 * user ELF loading maps a page at the same VA via paging_map_page_in_pd.
 * PD[12] is outside any user ELF segment range so this mapping is safe.   */
#define PD_GWIN_IDX  12
#define G_WIN_PT_VA  ((struct page_table*)((uint64_t)PD_GWIN_IDX << 21))
#define G_WIN2_PT_VA ((struct page_table*)(((uint64_t)PD_GWIN_IDX << 21) + 0x1000))

uint64_t kernel_pml4_phys = X86_64_PML4_PHYS;
uint64_t g_current_user_cr3 = X86_64_PML4_PHYS;

/* Saved user CR3, written by ISR/IRQ stubs before switching to kernel CR3.
 * Read by exception_handler to access the faulting task's page tables. */
uint64_t g_saved_user_cr3 = X86_64_PML4_PHYS;

/* Physical addresses of window page tables (immutable after paging_init). */
static uint64_t g_win_pt_phys_addr = 0;
static uint64_t g_win2_pt_phys_addr = 0;

static inline void invlpg(uint64_t virt) {
    asm volatile("invlpg (%0)" : : "r"(virt) : "memory");
}

/* Map a physical 4KB frame at the PD_WIN window (returns virtual address).
 * PTE writes go through the permanent PD[12] accessor (G_WIN_PT_VA) to avoid
 * clobbering identity-mapped pages that overlap user ELF virtual addresses. */
static void* win_map(uint64_t phys, int pt_idx) {
    struct page_table* pd = X86_64_PD_VIRT;
    uint64_t old = pd->entries[PD_WIN_IDX];
    if ((old & 1) == 0 || (old & 0x80) != 0) {
        /* PD_WIN_IDX entry not yet a 4KB PT pointer — set it up once. */
        pd->entries[PD_WIN_IDX] = (g_win_pt_phys_addr & 0xFFFFFFFFF000ULL) | 0x03;
        invlpg(PT_WIN_BASE);
    }
    uint64_t va = PT_WIN_BASE + (uint64_t)pt_idx * 4096;
    G_WIN_PT_VA->entries[pt_idx] = (phys & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(va);
    return (void*)(uintptr_t)va;
}

static void win_unmap(int pt_idx) {
    uint64_t va = PT_WIN_BASE + (uint64_t)pt_idx * 4096;
    /* Restore identity mapping so kernel data in this range stays accessible. */
    G_WIN_PT_VA->entries[pt_idx] = (va & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(va);
}

/* Helper: same for PD_WIN2 */
static void* win2_map(uint64_t phys, int pt_idx) {
    struct page_table* pd = X86_64_PD_VIRT;
    pd->entries[PD_WIN2_IDX] = (g_win2_pt_phys_addr & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(PT_WIN2_BASE);
    uint64_t va = PT_WIN2_BASE + (uint64_t)pt_idx * 4096;
    G_WIN2_PT_VA->entries[pt_idx] = (phys & 0xFFFFFFFFF000ULL) | 0x03;
    invlpg(va);
    return (void*)(uintptr_t)va;
}

static void win2_unmap(int pt_idx) {
    uint64_t va = PT_WIN2_BASE + (uint64_t)pt_idx * 4096;
    /* Restore identity mapping so kernel data in this range stays accessible. */
    G_WIN2_PT_VA->entries[pt_idx] = (va & 0xFFFFFFFFF000ULL) | 0x03;
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

    /* Extend identity map to 32 MB: PD entries 2-15 as 2MB huge pages.
     * PD[10..11] (20-24 MB) will be replaced with 4KB page tables by
     * win_map/win2_map, so those huge pages are temporary placeholders.
     * The page tables are pre-filled with identity mappings to preserve
     * kernel data access (e.g. .rodata from large include_bytes!).       */
    struct page_table* pd = X86_64_PD_VIRT;
    for (int i = 2; i < 16; i++) {
        pd->entries[i] = ((uint64_t)i << 21) | 0x83ULL;
    }

    /* Pre-allocate three separate frames:
     *   g_win_pt_phys_addr   — page table for PD_WIN  (window 1)
     *   g_win2_pt_phys_addr  — page table for PD_WIN2 (window 2)
     *   accessor PT          — maps G_WIN_PT_VA / G_WIN2_PT_VA
     * Each must be a distinct physical frame to avoid aliasing.
     * g_win_pt and g_win2_pt are pre-filled with identity mappings so that
     * replacing the huge pages doesn't lose access to kernel data in these
     * ranges (e.g. .rodata from large include_bytes! in 20-24 MB).        */
    void* win_frame = pmm_alloc_frame();
    if (win_frame) {
        g_win_pt_phys_addr = (uint64_t)(uintptr_t)win_frame;
        struct page_table* win_pt = (struct page_table*)(uintptr_t)g_win_pt_phys_addr;
        uint64_t base_phys = (uint64_t)PD_WIN_IDX << 21;
        for (int i = 0; i < 512; i++) {
            win_pt->entries[i] = (base_phys + (uint64_t)i * 4096) | 0x03;
        }
    }
    void* win2_frame = pmm_alloc_frame();
    if (win2_frame) {
        g_win2_pt_phys_addr = (uint64_t)(uintptr_t)win2_frame;
        struct page_table* win2_pt = (struct page_table*)(uintptr_t)g_win2_pt_phys_addr;
        uint64_t base_phys = (uint64_t)PD_WIN2_IDX << 21;
        for (int i = 0; i < 512; i++) {
            win2_pt->entries[i] = (base_phys + (uint64_t)i * 4096) | 0x03;
        }
    }

    /* Set up permanent g_win_pt accessor via PD[12] (VA 24 MB).
     * User ELF loading overwrites identity-mapped PTEs that overlap with
     * user virtual addresses (e.g. g_win_pt at phys 0x100000 gets clobbered
     * when a user page is mapped at VA 0x100000).  PD[12] is outside any
     * user ELF segment range, so this mapping is never touched.           */
    void* gwin_acc_frame = pmm_alloc_frame();
    if (gwin_acc_frame) {
        uint64_t acc_phys = (uint64_t)(uintptr_t)gwin_acc_frame;
        /* Fill all 512 entries with identity mapping so the entire 2 MB
         * range (24-26 MB) remains accessible after replacing the huge page.
         * The stack and other kernel data may reside within this range.      */
        struct page_table* acc_pt = (struct page_table*)(uintptr_t)acc_phys;
        for (int i = 0; i < 512; i++) {
            acc_pt->entries[i] = ((PD_GWIN_IDX << 21) + (uint64_t)i * 4096) | 0x03;
        }
        /* Overwrite the two window slots to point to the actual window PTs */
        acc_pt->entries[0] =
            (g_win_pt_phys_addr & 0xFFFFFFFFF000ULL) | 0x03;
        acc_pt->entries[1] =
            (g_win2_pt_phys_addr & 0xFFFFFFFFF000ULL) | 0x03;
        /* Install PD[12] entry (replaces the 2MB huge page) */
        pd->entries[PD_GWIN_IDX] = (acc_phys & 0xFFFFFFFFF000ULL) | 0x03;
        invlpg((uint64_t)PD_GWIN_IDX << 21);
    }

    serial_print("  Identity map extended to 32 MB (PD[2..15])\n");
    serial_print("  Window PT at phys 0x");
    serial_print_hex64(g_win_pt_phys_addr);
    serial_print("\n");
    serial_print("  Window2 PT at phys 0x");
    serial_print_hex64(g_win2_pt_phys_addr);
    serial_print("\n");
    serial_print("  g_win_pt VA (PD[12]) = 0x");
    serial_print_hex64((uint64_t)G_WIN_PT_VA);
    serial_print("\n");
}

void paging_enable() {
    serial_print("Paging: Already enabled (set up by boot code)\n");
}

/* 4-level page walk: PML4 → PDPT → PD → PT.
 * For virtual addresses in the first 32 MB (PML4[0], PDPT[0]),
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
        uint64_t new_pdpt_phys = (uint64_t)(uintptr_t)new_pdpt;
        /* Map through window before zeroing — frame may not be identity-mapped */
        void* zero_va = win_map(new_pdpt_phys, PT_TEMP_IDX);
        if (!zero_va) { pmm_free_frame(new_pdpt); return 0; }
        __builtin_memset(zero_va, 0, 4096);
        pml4->entries[pml4_idx] = (new_pdpt_phys & 0xFFFFFFFFF000ULL) | 0x03;
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
        uint64_t new_pd_phys = (uint64_t)(uintptr_t)new_pd;
        /* Map through an available window before zeroing — frame may not be identity-mapped */
        void* zero_va;
        if (pdpt_phys == X86_64_PDPT_PHYS) {
            zero_va = win_map(new_pd_phys, PT_TEMP_IDX);
        } else {
            zero_va = win2_map(new_pd_phys, PT_TEMP_IDX);
        }
        if (!zero_va) {
            pmm_free_frame(new_pd);
            if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);
            return 0;
        }
        __builtin_memset(zero_va, 0, 4096);
        pdpt->entries[pdpt_idx] = (new_pd_phys & 0xFFFFFFFFF000ULL) | 0x03;
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
        uint64_t new_pt_phys = (uint64_t)(uintptr_t)new_pt;
        /* Map through window before zeroing — frame may not be identity-mapped */
        void* zero_va = win_map(new_pt_phys, PT_TEMP_IDX);
        if (!zero_va) {
            pmm_free_frame(new_pt);
            if (pd_phys != X86_64_PD_PHYS) win2_unmap(PT_TEMP_IDX);
            if (pdpt_phys != X86_64_PDPT_PHYS) win_unmap(PT_TEMP_IDX);
            return 0;
        }
        __builtin_memset(zero_va, 0, 4096);
        pd_tbl->entries[pd_idx] = (new_pt_phys & 0xFFFFFFFFF000ULL) | 0x03;
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
    void* pml4_frame = pmm_alloc_frame();
    if (!pml4_frame) {
        serial_print("Paging: ERROR - Failed to allocate PML4 frame\n");
        return 0;
    }
    uint64_t new_pml4_phys = (uint64_t)(uintptr_t)pml4_frame;

    /* Allocate a new PDPT frame (user gets its own copy) */
    void* pdpt_frame = pmm_alloc_frame();
    if (!pdpt_frame) {
        pmm_free_frame(pml4_frame);
        serial_print("Paging: ERROR - Failed to allocate PDPT frame\n");
        return 0;
    }
    uint64_t new_pdpt_phys = (uint64_t)(uintptr_t)pdpt_frame;

    /* Allocate a new PD frame (user gets its own copy of identity map entries).
     * This is critical: if the user shares the kernel's PD, mapping user ELF
     * pages at VA >= 2MB would corrupt the kernel's own page tables.         */
    void* pd_frame = pmm_alloc_frame();
    if (!pd_frame) {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        serial_print("Paging: ERROR - Failed to allocate PD frame\n");
        return 0;
    }
    uint64_t new_pd_phys = (uint64_t)(uintptr_t)pd_frame;

    /* Map and initialize the new PD: copy kernel PD entries so the
     * identity-map page-table structure is reachable from user CR3.
     * This is needed for demand-mapping (paging_demand_map_kernel_page)
     * and page-table walks that resolve physical frames.                */
    void* pd_va = win_map(new_pd_phys, PT_TEMP_IDX);
    if (!pd_va) {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        pmm_free_frame(pd_frame);
        return 0;
    }
    struct page_directory* new_pd = (struct page_directory*)pd_va;

    /* Copy kernel PD entries so the identity-map page tables are reachable
     * from the user PD (needed for demand-mapping and page-table walks).
     * Do NOT set PAGE_USER on these entries — kernel code, data, heap,
     * stacks, and page tables must be ring-0-only.  The ISR/IRQ stubs
     * save the user CR3 and switch to kernel CR3 before touching any
     * kernel state, so the kernel can always reach its own pages.
     * User-mapped pages (ELF, stack, SHM) are added later via
     * paging_map_page_in_pd() with PAGE_USER in their flags.            */
    struct page_directory* kern_pd = X86_64_PD_VIRT;
    for (int i = 0; i < 512; i++) {
        new_pd->entries[i] = kern_pd->entries[i];
    }
    win_unmap(PT_TEMP_IDX);

    /* Map and initialize the new PDPT: only entry[0] → new PD */
    void* pdpt_va = win_map(new_pdpt_phys, PT_TEMP_IDX);
    if (!pdpt_va) {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        pmm_free_frame(pd_frame);
        return 0;
    }
    struct page_directory* new_pdpt = (struct page_directory*)pdpt_va;
    for (int i = 0; i < 512; i++) {
        new_pdpt->entries[i] = 0;
    }
    new_pdpt->entries[0] = (new_pd_phys & 0xFFFFFFFFF000ULL) | 0x07;
    win_unmap(PT_TEMP_IDX);

    /* Map and initialize the new PML4: entry[0] → new PDPT */
    void* pml4_va = win_map(new_pml4_phys, PT_TEMP_IDX);
    if (!pml4_va) {
        pmm_free_frame(pml4_frame);
        pmm_free_frame(pdpt_frame);
        pmm_free_frame(pd_frame);
        return 0;
    }
    struct page_directory* new_pml4 = (struct page_directory*)pml4_va;
    for (int i = 0; i < 512; i++) {
        new_pml4->entries[i] = 0;
    }
    new_pml4->entries[0] = (new_pdpt_phys & 0xFFFFFFFFF000ULL) | 0x07;
    win_unmap(PT_TEMP_IDX);

    return (uintptr_t)new_pml4_phys;
}

void paging_destroy_directory(uintptr_t pd_phys) {
    if (!pd_phys) return;
    serial_print("Paging: Destroying page directory (x86_64)\n");

    void* win_va = win_map((uint64_t)pd_phys, PT_TEMP_IDX);
    if (!win_va) return;
    struct page_directory* pml4 = (struct page_directory*)win_va;

    /* PML4[0] now shares the kernel's PDPT — do NOT free it.
     * Only walk and free user-owned mappings at PML4 entries >= 4.    */
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
        uint64_t new_frame_phys = (uint64_t)(uintptr_t)new_frame;
        /* Map src and dst frames through unused window slots for copying */
        void* src_va = win2_map(src_frame, 1);
        void* dst_va = win_map(new_frame_phys, 1);
        if (src_va && dst_va) {
            __builtin_memcpy(dst_va, src_va, PAGE_SIZE);
        }
        win_unmap(1);
        win2_unmap(1);
        dst_pt->entries[i] = (new_frame_phys & 0xFFFFFFFFF000ULL)
                           | (pte & 0xFFFULL) | 1;
    }
}

uintptr_t paging_clone_directory(uintptr_t pd_phys) {
    serial_print("Paging: Cloning page directory (x86_64)\n");

    void* dst_pml4_frame = pmm_alloc_frame();
    if (!dst_pml4_frame) return 0;
    uint64_t dst_pml4_phys = (uint64_t)(uintptr_t)dst_pml4_frame;
    void* zero_va = win_map(dst_pml4_phys, PT_TEMP_IDX);
    if (!zero_va) { pmm_free_frame(dst_pml4_frame); return 0; }
    __builtin_memset(zero_va, 0, 4096);
    win_unmap(PT_TEMP_IDX);

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
        void* zero_va2 = win2_map(dst_pdpt_phys, 1);
        if (!zero_va2) goto fail;
        __builtin_memset(zero_va2, 0, 4096);
        win2_unmap(1);
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
            void* zero_va3 = win2_map(dst_pd_phys, 1);
            if (!zero_va3) goto fail;
            __builtin_memset(zero_va3, 0, 4096);
            win2_unmap(1);
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
                    void* zero_va_pt = win2_map(dst_pt_phys, 1);
                    if (!zero_va_pt) goto fail;
                    __builtin_memset(zero_va_pt, 0, 4096);
                    win2_unmap(1);

                    dst_pd->entries[pd_i] = (dst_pt_phys & 0xFFFFFFFFF000ULL)
                                          | (src_flags & ~0x80ULL) | 1;

                    win2_unmap(0);
                    win_unmap(0);
                    struct page_table* dst_pt = (struct page_table*)win_map(dst_pt_phys, 0);

                    for (int pt_i = 0; pt_i < 512; pt_i++) {
                        uint64_t phys = huge_base + ((uint64_t)pt_i << 12);
                        void* new_frame = pmm_alloc_frame();
                        if (!new_frame) continue;
                        uint64_t new_frame_phys = (uint64_t)(uintptr_t)new_frame;
                        void* src_va = win_map(phys, 1);
                        void* dst_va = win2_map(new_frame_phys, 1);
                        if (src_va && dst_va)
                            __builtin_memcpy(dst_va, src_va, PAGE_SIZE);
                        win_unmap(1);
                        win2_unmap(1);
                        dst_pt->entries[pt_i] =
                            (new_frame_phys & 0xFFFFFFFFF000ULL)
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
                    void* zero_va_pt2 = win2_map(dst_pt_phys, 1);
                    if (!zero_va_pt2) goto fail;
                    __builtin_memset(zero_va_pt2, 0, 4096);
                    win2_unmap(1);

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
    void* zero_va = win_map(dst_pml4_phys, PT_TEMP_IDX);
    if (!zero_va) { pmm_free_frame(dst_pml4_frame); return 0; }
    __builtin_memset(zero_va, 0, 4096);
    win_unmap(PT_TEMP_IDX);

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
        void* zero_va2 = win2_map(dst_pdpt_phys, 1);
        if (!zero_va2) goto fork_fail;
        __builtin_memset(zero_va2, 0, 4096);
        win2_unmap(1);
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
            void* zero_va3 = win2_map(dst_pd_phys, 1);
            if (!zero_va3) goto fork_fail;
            __builtin_memset(zero_va3, 0, 4096);
            win2_unmap(1);
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
                    void* zero_va_pt = win2_map(dst_pt_phys, 1);
                    if (!zero_va_pt) goto fork_fail;
                    __builtin_memset(zero_va_pt, 0, 4096);
                    win2_unmap(1);

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
    uint64_t new_frame_phys = (uint64_t)(uintptr_t)new_frame;
    void* cow_src = win_map(old_frame, 1);
    void* cow_dst = win2_map(new_frame_phys, 1);
    if (cow_src && cow_dst)
        __builtin_memcpy(cow_dst, cow_src, PAGE_SIZE);
    win_unmap(1);
    win2_unmap(1);

    pmm_refcount_dec((void*)(uintptr_t)old_frame);

    flags &= ~PAGE_COW;
    flags |= PAGE_WRITE;
    *pte = ((uint64_t)(uintptr_t)new_frame & 0xFFFFFFFFF000ULL) | flags;

    invalidate_page((uint64_t)fault_addr);

    return 1;
}

bool paging_map_page_in_pd(uintptr_t pd_phys, uintptr_t virt_addr,
                            uintptr_t phys_addr, uint32_t flags) {
    /* NOTE: caller must disable interrupts — win_map/win_unmap use a shared
     * window slot (PT_TEMP_IDX) that gets clobbered if a context switch
     * fires between win_map and win_unmap. */
    bool result = false;

    void* win_va = win_map((uint64_t)pd_phys, PT_TEMP_IDX);
    if (!win_va) goto out;
    struct page_directory* pml4 = (struct page_directory*)win_va;

    uint64_t vaddr = (uint64_t)virt_addr;
    uint64_t pml4_idx = (vaddr >> 39) & 0x1FF;
    uint64_t pdpt_idx = (vaddr >> 30) & 0x1FF;
    uint64_t pd_idx   = (vaddr >> 21) & 0x1FF;
    uint64_t pt_idx   = (vaddr >> 12) & 0x1FF;

    /* Walk/create page tables in the target PML4 */
    /* --- PML4 entry --- */
    if (!(pml4->entries[pml4_idx] & 1)) {
        void* new_pdpt = pmm_alloc_frame();
        if (!new_pdpt) { win_unmap(PT_TEMP_IDX); goto out; }
        uint64_t new_pdpt_phys = (uint64_t)(uintptr_t)new_pdpt;
        win_unmap(PT_TEMP_IDX);
        void* zero_va = win_map(new_pdpt_phys, PT_TEMP_IDX);
        if (!zero_va) { pmm_free_frame(new_pdpt); goto out; }
        __builtin_memset(zero_va, 0, 4096);
        win_unmap(PT_TEMP_IDX);
        pml4 = (struct page_directory*)win_map((uint64_t)pd_phys, PT_TEMP_IDX);
        if (!pml4) { pmm_free_frame(new_pdpt); goto out; }
        pml4->entries[pml4_idx] = (new_pdpt_phys & 0xFFFFFFFFF000ULL) | 0x03 | (flags & PAGE_USER);
    }

    /* --- PDPT entry --- */
    uint64_t pdpt_phys = pml4->entries[pml4_idx] & 0xFFFFFFFFF000ULL;
    win_unmap(PT_TEMP_IDX);
    void* pdpt_va = win_map(pdpt_phys, PT_TEMP_IDX);
    if (!pdpt_va) goto out;
    struct page_table* pdpt = (struct page_table*)pdpt_va;

    if (!(pdpt->entries[pdpt_idx] & 1)) {
        void* new_pd = pmm_alloc_frame();
        if (!new_pd) { win_unmap(PT_TEMP_IDX); goto out; }
        uint64_t new_pd_phys = (uint64_t)(uintptr_t)new_pd;
        win_unmap(PT_TEMP_IDX);
        void* zero_va = win_map(new_pd_phys, PT_TEMP_IDX);
        if (!zero_va) { pmm_free_frame(new_pd); goto out; }
        __builtin_memset(zero_va, 0, 4096);
        win_unmap(PT_TEMP_IDX);
        pdpt = (struct page_table*)win_map(pdpt_phys, PT_TEMP_IDX);
        if (!pdpt) { pmm_free_frame(new_pd); goto out; }
        pdpt->entries[pdpt_idx] = (new_pd_phys & 0xFFFFFFFFF000ULL) | 0x03 | (flags & PAGE_USER);
    }

    /* --- PD entry --- */
    uint64_t pd_phys_tbl = pdpt->entries[pdpt_idx] & 0xFFFFFFFFF000ULL;
    win_unmap(PT_TEMP_IDX);
    void* pd_va = win_map(pd_phys_tbl, PT_TEMP_IDX);
    if (!pd_va) goto out;
    struct page_table* pd_tbl = (struct page_table*)pd_va;

    if (!(pd_tbl->entries[pd_idx] & 1)) {
        /* Not present — allocate a fresh PT */
        void* new_pt = pmm_alloc_frame();
        if (!new_pt) { win_unmap(PT_TEMP_IDX); goto out; }
        uint64_t new_pt_phys = (uint64_t)(uintptr_t)new_pt;
        win_unmap(PT_TEMP_IDX);
        void* zero_va = win_map(new_pt_phys, PT_TEMP_IDX);
        if (!zero_va) { pmm_free_frame(new_pt); goto out; }
        __builtin_memset(zero_va, 0, 4096);
        win_unmap(PT_TEMP_IDX);
        pd_tbl = (struct page_table*)win_map(pd_phys_tbl, PT_TEMP_IDX);
        if (!pd_tbl) { pmm_free_frame(new_pt); goto out; }
        pd_tbl->entries[pd_idx] = (new_pt_phys & 0xFFFFFFFFF000ULL) | 0x03 | (flags & PAGE_USER);
    } else if (pd_tbl->entries[pd_idx] & 0x80) {
        /* 2MB huge page — split into a 4KB page table with identity-mapped entries. */
        uint64_t huge_phys_base = pd_tbl->entries[pd_idx] & 0xFFFFFFFFFE00ULL;
        uint64_t huge_flags     = pd_tbl->entries[pd_idx] & 0xFFfull;
        uint64_t pt_flags = huge_flags & ~0x80ULL;

        void* new_pt = pmm_alloc_frame();
        if (!new_pt) { win_unmap(PT_TEMP_IDX); goto out; }
        uint64_t new_pt_phys = (uint64_t)(uintptr_t)new_pt;

        win_unmap(PT_TEMP_IDX);
        void* pt_fill_va = win_map(new_pt_phys, PT_TEMP_IDX);
        if (!pt_fill_va) { pmm_free_frame(new_pt); goto out; }
        struct page_table* new_pt_tbl = (struct page_table*)pt_fill_va;
        for (int i = 0; i < 512; i++) {
            new_pt_tbl->entries[i] = 0;
        }
        win_unmap(PT_TEMP_IDX);
        pd_tbl = (struct page_table*)win_map(pd_phys_tbl, PT_TEMP_IDX);
        if (!pd_tbl) { pmm_free_frame(new_pt); goto out; }
        pd_tbl->entries[pd_idx] = (new_pt_phys & 0xFFFFFFFFF000ULL) | (pt_flags & ~0x80ULL) | (flags & PAGE_USER);
        asm volatile("mov %%cr3, %%rax; mov %%rax, %%cr3" ::: "rax", "memory");
    }

    /* --- PT entry --- */
    uint64_t pt_phys = pd_tbl->entries[pd_idx] & 0xFFFFFFFFF000ULL;
    win_unmap(PT_TEMP_IDX);
    void* pt_va = win_map(pt_phys, PT_TEMP_IDX);
    if (!pt_va) goto out;
    struct page_table* pt = (struct page_table*)pt_va;

    pt->entries[pt_idx] = ((uint64_t)phys_addr & 0xFFFFFFFFF000ULL) | (flags & 0xFFFULL) | 1;

    win_unmap(PT_TEMP_IDX);
    invlpg(virt_addr);
    result = true;

out:
    return result;
}

/* Dump user page table entries for a given CR3 and fault address.
 * Called from the page fault handler to show the actual page table entries
 * the CPU used for the faulting access. Uses win_map to safely read the
 * user's page table frames (which may be beyond the identity map). */
void paging_dump_user_pt(uint64_t cr3, uint64_t fault_addr) {
    uint64_t pml4_phys = cr3;
    uint64_t* pml4 = (uint64_t*)win_map(pml4_phys, PT_TEMP_IDX);
    if (!pml4) { serial_print("  [dump] Cannot map user PML4\n"); return; }
    uint64_t pml4e0 = pml4[0];
    /* Compiler barrier: must come AFTER the read and BEFORE win_unmap.
     * Without this, the compiler may move win_unmap's store (to a different
     * address — G_WIN_PT_VA) before the read from the window (PT_WIN_BASE),
     * because it considers the two addresses independent.                    */
    asm volatile("" ::: "memory");
    win_unmap(PT_TEMP_IDX);

    uint64_t pdpt_phys = pml4e0 & 0x000FFFFFFFFFFFF0ULL;
    uint64_t* pdpt = (uint64_t*)win_map(pdpt_phys, PT_TEMP_IDX);
    if (!pdpt) { serial_print("  [dump] Cannot map user PDPT\n"); return; }
    uint64_t pdpte0 = pdpt[0];
    asm volatile("" ::: "memory");
    win_unmap(PT_TEMP_IDX);

    uint64_t pd_phys = pdpte0 & 0x000FFFFFFFFFFFF0ULL;
    uint64_t* pd = (uint64_t*)win_map(pd_phys, PT_TEMP_IDX);
    if (!pd) { serial_print("  [dump] Cannot map user PD\n"); return; }
    uint64_t pd_idx = (fault_addr >> 21) & 0x1FF;
    uint64_t pde_val = pd[pd_idx];
    asm volatile("" ::: "memory");
    win_unmap(PT_TEMP_IDX);

    serial_print("  PML4[0]=0x"); serial_print_hex64(pml4e0);
    serial_print(" PDPT[0]=0x"); serial_print_hex64(pdpte0);
    serial_print(" PD["); serial_print_hex((uint32_t)pd_idx);
    serial_print("]=0x");  serial_print_hex64(pde_val);
    serial_print("\n");
}

/* Demand-map a kernel page into the user's page directory.
 * Called from the page fault handler when the kernel (ring 0) faults on a
 * kernel address while the ISR is running under user CR3 (saved in
 * g_saved_user_cr3).  Kernel pages are ring-0-only (no PAGE_USER), so
 * post-snapshot heap pages are missing from the user PD.  We walk the
 * kernel's own page tables (identity-mapped at fixed phys addrs) to
 * resolve the physical frame, then call paging_map_page_in_pd() to create
 * the mapping in the user PD (without PAGE_USER — ring-0-only).         */
bool paging_demand_map_kernel_page(uint64_t fault_addr, uint64_t user_cr3) {
    uint64_t pml4_idx = (fault_addr >> 39) & 0x1FF;
    uint64_t pdpt_idx = (fault_addr >> 30) & 0x1FF;
    uint64_t pd_idx   = (fault_addr >> 21) & 0x1FF;
    uint64_t pt_idx   = (fault_addr >> 12) & 0x1FF;

    /* Kernel PML4/PDPT/PD are identity-mapped at fixed phys addresses.
     * We are running under kernel CR3 (ISR stub switched), so these
     * identity-mapped kernel page tables are directly accessible.        */
    uint64_t* pml4 = (uint64_t*)(uintptr_t)X86_64_PML4_PHYS;
    uint64_t pml4e = pml4[pml4_idx];
    if (!(pml4e & 1)) return false;

    uint64_t pdpt_phys = pml4e & 0xFFFFFFFFF000ULL;
    uint64_t* pdpt = (uint64_t*)(uintptr_t)pdpt_phys;
    uint64_t pdpte = pdpt[pdpt_idx];
    if (!(pdpte & 1)) return false;

    if (pdpte & 0x80) {
        /* 1 GB huge page — map 4 KB page within it */
        uint64_t phys_frame = pdpte & 0xFFFFFFFFC00000ULL;
        uint64_t page_addr = fault_addr & ~0xFFFULL;
        return paging_map_page_in_pd(user_cr3, page_addr,
                                     phys_frame + (fault_addr & 0x3FFFFFFFULL),
                                     PAGE_PRESENT | PAGE_WRITE);
    }

    uint64_t pd_phys_addr = pdpte & 0xFFFFFFFFF000ULL;
    uint64_t* pd = (uint64_t*)(uintptr_t)pd_phys_addr;
    uint64_t pde = pd[pd_idx];
    if (!(pde & 1)) return false;

    if (pde & 0x80) {
        /* 2 MB huge page — map 4 KB page within it */
        uint64_t phys_frame = pde & 0xFFFFFFFFFE00ULL;
        uint64_t page_addr = fault_addr & ~0xFFFULL;
        return paging_map_page_in_pd(user_cr3, page_addr,
                                     phys_frame + (fault_addr & 0x1FFFFFULL),
                                     PAGE_PRESENT | PAGE_WRITE);
    }

    /* 4 KB page table — walk it via win_map */
    uint64_t pt_phys = pde & 0xFFFFFFFFF000ULL;
    uint64_t* pt = (uint64_t*)win_map(pt_phys, PT_TEMP_IDX);
    if (!pt) return false;
    uint64_t pte = pt[pt_idx];
    asm volatile("" ::: "memory");
    win_unmap(PT_TEMP_IDX);

    if (!(pte & 1)) return false;

    uint64_t phys_frame = pte & 0xFFFFFFFFF000ULL;
    uint64_t page_addr = fault_addr & ~0xFFFULL;
    return paging_map_page_in_pd(user_cr3, page_addr, phys_frame,
                                 PAGE_PRESENT | PAGE_WRITE);
}

/* Demand-map a fresh page in the kernel's own page tables.
 * Used when the kernel faults on its own heap under kernel CR3.
 * Allocates a zeroed frame and maps it at the faulting address.
 *
 * Unlike paging_map_page_in_pd (which uses win_map for all levels), this
 * function accesses the kernel's PML4/PDPT/PD via identity-mapped addresses
 * (0x1000 / 0x2000 / 0x3000) to avoid potential win_map aliasing issues.
 * Only the PT level uses win_map (PT may not be identity-mapped).       */
bool paging_demand_alloc_kernel_page(uint64_t fault_addr) {
    uint64_t page_addr = fault_addr & ~0xFFFULL;
    uint64_t pd_idx   = (page_addr >> 21) & 0x1FF;
    uint64_t pt_idx   = (page_addr >> 12) & 0x1FF;

    void* frame = pmm_alloc_frame();
    if (!frame) return false;
    uint64_t phys = (uint64_t)(uintptr_t)frame;

    void* zva = win_map(phys, PT_TEMP_IDX);
    if (zva) {
        __builtin_memset(zva, 0, 4096);
        win_unmap(PT_TEMP_IDX);
    }

    /* Kernel PML4[0], PDPT[0], PD are identity-mapped at fixed phys addrs.
     * Access PD directly — no win_map needed for level-2 walks.       */
    struct page_table* pd = (struct page_table*)(uintptr_t)X86_64_PD_PHYS;

    /* Get PD entry (the CPU walks PML4[0] → PDPT[0] → PD[pd_idx]). */
    uint64_t pde = pd->entries[pd_idx];

    if ((pde & 1) == 0) {
        /* PD entry not present — allocate a fresh PT and zero it */
        void* pt_frame = pmm_alloc_frame();
        if (!pt_frame) { pmm_free_frame(frame); return false; }
        uint64_t pt_phys = (uint64_t)(uintptr_t)pt_frame;

        void* zva2 = win_map(pt_phys, PT_TEMP_IDX);
        if (!zva2) { pmm_free_frame(frame); pmm_free_frame(pt_frame); return false; }
        __builtin_memset(zva2, 0, 4096);
        win_unmap(PT_TEMP_IDX);

        pd->entries[pd_idx] = (pt_phys & 0xFFFFFFFFF000ULL) | 0x03;
        asm volatile("mov %%cr3, %%rax; mov %%rax, %%cr3" ::: "rax", "memory");
        pde = pd->entries[pd_idx];
    } else if (pde & 0x80) {
        /* 2MB huge page — split into a 4KB page table */
        uint64_t huge_base  = pde & 0xFFFFFFFFFE00ULL;
        uint64_t huge_flags = pde & 0xFFFULL;
        uint64_t pt_flags   = huge_flags & ~0x80ULL;

        void* pt_frame = pmm_alloc_frame();
        if (!pt_frame) { pmm_free_frame(frame); return false; }
        uint64_t pt_phys = (uint64_t)(uintptr_t)pt_frame;

        void* pt_fill_va = win_map(pt_phys, PT_TEMP_IDX);
        if (!pt_fill_va) { pmm_free_frame(frame); pmm_free_frame(pt_frame); return false; }
        struct page_table* pt_tbl = (struct page_table*)pt_fill_va;
        for (int i = 0; i < 512; i++) {
            pt_tbl->entries[i] = (huge_base + ((uint64_t)i << 12)) | (pt_flags & ~0x04ULL) | 1;
        }
        win_unmap(PT_TEMP_IDX);

        pd->entries[pd_idx] = (pt_phys & 0xFFFFFFFFF000ULL) | (pt_flags & ~0x04ULL);
        asm volatile("mov %%cr3, %%rax; mov %%rax, %%cr3" ::: "rax", "memory");
        pde = pd->entries[pd_idx];
    }

    /* Now PD entry must be a present 4KB PT pointer */
    uint64_t pt_phys = pde & 0xFFFFFFFFF000ULL;
    if (!pt_phys) { pmm_free_frame(frame); return false; }

    void* pt_va = win_map(pt_phys, PT_TEMP_IDX);
    if (!pt_va) { pmm_free_frame(frame); return false; }
    struct page_table* pt = (struct page_table*)pt_va;
    pt->entries[pt_idx] = (phys & 0xFFFFFFFFF000ULL) | 0x03;
    win_unmap(PT_TEMP_IDX);

    invlpg(page_addr);
    asm volatile("mov %%cr3, %%rax; mov %%rax, %%cr3" ::: "rax", "memory");
    return true;
}

void* paging_temp_map_frame(uintptr_t phys_addr) {
    /* Use a dedicated window slot (511) via win_map so we don't
     * conflict with the self-map at index 0 or with other win_map
     * callers that use index 0. PTE writes go through PD[12]
     * (G_WIN_PT_VA) so identity-mapped pages are not touched.    */
    return win_map(phys_addr, 511);
}

void paging_temp_unmap_frame(void) {
    win_unmap(511);
}

// ============================================================================
// x86_64 implementation — 2-level paging, 32-bit entries
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
