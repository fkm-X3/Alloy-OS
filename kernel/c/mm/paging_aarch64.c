#include "paging.h"
#include "pmm.h"
#include "../drivers/serial.h"
#include <stdbool.h>

// aarch64 paging using 4KB granule (L0→L1→L2→L3).
// With 4KB granule and 48-bit address space (T0SZ=16):
//   L0: 512 × 512GB (must be table descriptor, not block)
//   L1: 512 × 1GB    (can be table or block)
//   L2: 512 × 2MB    (can be table or block)
//   L3: 512 × 4KB    (page table)
//
// Current setup: L0 → L1 table, L1 uses 1GB block entries for identity map.
// MMU is NOT enabled yet (SCTLR_EL1.M=0), so all physical addresses
// are accessed directly as virtual addresses.

#define L0_SHIFT  39
#define L1_SHIFT  30
#define L2_SHIFT  21
#define L3_SHIFT  12
#define Ln_MASK   0x1FF

#define PD_TABLE  0x3ULL
#define PG_BLOCK  0xC01ULL
#define PG_PAGE   0xC03ULL
#define PG_AF     0x400ULL

static uintptr_t kernel_page_dir_phys = 0;
static uint64_t kernel_tt_l0[512] __attribute__((aligned(4096)));
static uint64_t kernel_tt_l1_0[512] __attribute__((aligned(4096)));
static uint64_t kernel_tt_l1_1[512] __attribute__((aligned(4096)));

void paging_init() {
    serial_print("Paging: Initializing ARM64 translation tables\n");

    for (int i = 0; i < 512; i++) {
        kernel_tt_l0[i] = 0;
        kernel_tt_l1_0[i] = 0;
        kernel_tt_l1_1[i] = 0;
    }

    // L0[0] → L1 table covering [0, 1GB), L0[1] → L1 table covering [1GB, 2GB)
    kernel_tt_l0[0] = ((uint64_t)(uintptr_t)kernel_tt_l1_0) | PD_TABLE;
    kernel_tt_l0[1] = ((uint64_t)(uintptr_t)kernel_tt_l1_1) | PD_TABLE;

    // L1 entries: 1GB block descriptors for identity map
    for (int i = 0; i < 512; i++) {
        uint64_t block_addr = (uint64_t)i << L1_SHIFT;  // 1GB-aligned
        kernel_tt_l1_0[i] = block_addr | PG_BLOCK | PG_AF;
        kernel_tt_l1_1[i] = (block_addr + ((uint64_t)1 << L1_SHIFT)) | PG_BLOCK | PG_AF;
    }

    kernel_page_dir_phys = (uintptr_t)&kernel_tt_l0[0];
}

void paging_enable() {
    serial_print("Paging: Enabling MMU\n");
    // MMU already enabled in boot_aarch64.S
    // Set TTBR0_EL1 to point to L0 table
    uint64_t ttbr0 = (uint64_t)(uintptr_t)kernel_tt_l0;
    asm volatile("msr ttbr0_el1, %0" : : "r"(ttbr0));
    asm volatile("isb");
}

uintptr_t paging_create_directory_phys() {
    return kernel_page_dir_phys;
}

bool paging_switch_to_directory(uintptr_t pd_phys) {
    if (pd_phys == 0) return false;
    asm volatile("msr ttbr0_el1, %0" : : "r"((uint64_t)pd_phys));
    asm volatile("isb; tlbi vmalle1; dsb sy; isb");
    return true;
}

uintptr_t paging_get_kernel_directory_phys() {
    return kernel_page_dir_phys;
}

uintptr_t paging_get_physical_address(uintptr_t virt_addr) {
    return virt_addr;
}

void paging_destroy_directory(uintptr_t pd_phys) {
    (void)pd_phys;
}

uintptr_t paging_clone_directory(uintptr_t pd_phys) {
    return pd_phys;
}

uintptr_t paging_fork_directory(uintptr_t pd_phys) {
    return pd_phys;
}

uint8_t paging_handle_cow_fault(uintptr_t fault_addr) {
    (void)fault_addr;
    return 0;
}

bool paging_map_page_in_pd(uintptr_t pd_phys, uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags) {
    (void)pd_phys;
    (void)virt_addr;
    (void)phys_addr;
    (void)flags;
    return true;
}

// Walk page tables to find L3 PTE for a virtual address.
// Creates intermediate tables if 'create' is true.
// Returns a pointer to the L3 PTE, or NULL on failure.
static uint64_t* walk_pte(uintptr_t virt_addr, bool create) {
    int l0_idx = (virt_addr >> L0_SHIFT) & Ln_MASK;
    int l1_idx = (virt_addr >> L1_SHIFT) & Ln_MASK;
    int l2_idx = (virt_addr >> L2_SHIFT) & Ln_MASK;
    int l3_idx = (virt_addr >> L3_SHIFT) & Ln_MASK;

    uint64_t* l0 = (uint64_t*)(uintptr_t)kernel_tt_l0;

    // L0 → L1 table
    uint64_t l0e = l0[l0_idx];
    if (!(l0e & 0x3)) return 0;
    uint64_t* l1 = (uint64_t*)(uintptr_t)(l0e & 0xFFFFFFFFF000ULL);

    // L1 → L2 table (split 1GB block if needed)
    uint64_t l1e = l1[l1_idx];
    if ((l1e & 0x3) == 0x1) {
        if (!create) return 0;
        uint64_t block_base = l1e & 0xFFFFFFC000000000ULL;
        uint64_t* l2_new = pmm_alloc_frame();
        if (!l2_new) return 0;
        for (int i = 0; i < 512; i++) {
            l2_new[i] = (block_base + ((uint64_t)i << L2_SHIFT)) | PG_BLOCK | PG_AF;
        }
        l1[l1_idx] = ((uint64_t)(uintptr_t)l2_new) | PD_TABLE;
        l1e = l1[l1_idx];
    } else if (!(l1e & 0x1)) {
        if (!create) return 0;
        uint64_t* l2_new = pmm_alloc_frame();
        if (!l2_new) return 0;
        for (int i = 0; i < 512; i++) l2_new[i] = 0;
        l1[l1_idx] = ((uint64_t)(uintptr_t)l2_new) | PD_TABLE;
        l1e = l1[l1_idx];
    }

    uint64_t* l2 = (uint64_t*)(uintptr_t)(l1e & 0xFFFFFFFFF000ULL);

    // L2 → L3 table (split 2MB block if needed)
    uint64_t l2e = l2[l2_idx];
    if ((l2e & 0x3) == 0x1) {
        if (!create) return 0;
        uint64_t block_base = l2e & 0xFFFFFFFE00000ULL;
        uint64_t* l3_new = pmm_alloc_frame();
        if (!l3_new) return 0;
        for (int i = 0; i < 512; i++) {
            l3_new[i] = (block_base + ((uint64_t)i << L3_SHIFT)) | PG_PAGE | PG_AF;
        }
        l2[l2_idx] = ((uint64_t)(uintptr_t)l3_new) | PD_TABLE;
        l2e = l2[l2_idx];
    } else if (!(l2e & 0x1)) {
        if (!create) return 0;
        uint64_t* l3_new = pmm_alloc_frame();
        if (!l3_new) return 0;
        for (int i = 0; i < 512; i++) l3_new[i] = 0;
        l2[l2_idx] = ((uint64_t)(uintptr_t)l3_new) | PD_TABLE;
        l2e = l2[l2_idx];
    }

    uint64_t* l3 = (uint64_t*)(uintptr_t)(l2e & 0xFFFFFFFFF000ULL);
    return &l3[l3_idx];
}

bool paging_map_page(uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags) {
    uint64_t* pte = walk_pte(virt_addr, true);
    if (!pte) return false;

    uint64_t attr = PG_PAGE | PG_AF;
    if (flags & PAGE_WRITE) attr |= 0x80;
    if (flags & PAGE_USER)  attr |= 0x40;

    *pte = ((uint64_t)phys_addr & 0xFFFFFFFFF000ULL) | attr;
    asm volatile("dsb sy; tlbi vae1is, %0; dsb sy; isb" : : "r"(virt_addr));
    return true;
}

void paging_unmap_page(uintptr_t virt_addr) {
    uint64_t* pte = walk_pte(virt_addr, false);
    if (!pte) return;

    *pte = 0;
    asm volatile("dsb sy; tlbi vae1is, %0; dsb sy; isb" : : "r"(virt_addr));
}

void* paging_temp_map_frame(uintptr_t phys_addr) {
    return (void*)(uintptr_t)phys_addr;
}

void paging_temp_unmap_frame() {
}
