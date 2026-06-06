#include "paging.h"
#include "pmm.h"
#include "../drivers/serial.h"

Paging g_paging;

extern uint32_t _kernel_start;
extern uint32_t _kernel_end;

static struct page_directory kernel_dir __attribute__((aligned(4096)));
static struct page_table kernel_pts[4] __attribute__((aligned(4096)));
static uint32_t kernel_directory_phys;

#define PT_VIRT_BASE 0x00C00000
#define PT_VIRT_MAP_TABLE_INDEX 3

static inline void invalidate_page_local(uint32_t virt_addr) {
    asm volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
}

static struct page_table* map_page_table_window(uint32_t dir_index, uint32_t pt_phys) {
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

bool paging_map_page(uint32_t virt_addr, uint32_t phys_addr, uint32_t flags) {
    uint32_t* page_entry = get_page_entry(virt_addr, true);
    if (!page_entry) {
        return false;
    }

    *page_entry = (phys_addr & 0xFFFFF000) | (flags & 0xFFF) | PAGE_PRESENT;
    invalidate_page(virt_addr);

    return true;
}

void paging_unmap_page(uint32_t virt_addr) {
    uint32_t* page_entry = get_page_entry(virt_addr, false);
    if (page_entry) {
        *page_entry = 0;
        invalidate_page(virt_addr);
    }
}

uint32_t paging_get_physical_address(uint32_t virt_addr) {
    uint32_t* page_entry = get_page_entry(virt_addr, false);
    if (!page_entry || !(*page_entry & PAGE_PRESENT)) {
        return 0;
    }

    return (*page_entry & 0xFFFFF000) | (virt_addr & 0xFFF);
}

uint32_t paging_create_directory_phys() {
    void* pd_phys = pmm_alloc_frame();
    if (!pd_phys) {
        serial_print("Paging: ERROR - Failed to allocate page directory frame\n");
        return 0;
    }

    const uint32_t TEMP_INDEX = 100;
    struct page_table* tmp = map_page_table_window(TEMP_INDEX, (uint32_t)pd_phys);
    if (!tmp) {
        serial_print("Paging: ERROR - Failed to map temporary page directory frame\n");
        pmm_free_frame(pd_phys);
        return 0;
    }

    struct page_directory* new_pd = (struct page_directory*)((uint32_t)PT_VIRT_BASE + (TEMP_INDEX * PAGE_SIZE));
    for (int i = 0; i < 1024; i++) {
        new_pd->entries[i] = 0;
    }

    for (int i = 0; i < 4; i++) {
        new_pd->entries[i] = g_paging.kernel_directory->entries[i];
    }

    return (uint32_t)pd_phys;
}

void paging_destroy_directory(uint32_t pd_phys) {
    if (!pd_phys) return;

    serial_print("Paging: Destroying page directory\n");

    const uint32_t TEMP_INDEX = 101;
    struct page_table* tmp_pd_map = map_page_table_window(TEMP_INDEX, pd_phys);
    (void)tmp_pd_map;
    struct page_directory* pd = (struct page_directory*)((uint32_t)PT_VIRT_BASE + (TEMP_INDEX * PAGE_SIZE));

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
            pmm_free_frame((void*)frame_phys);
            pt->entries[i] = 0;
        }

        pmm_free_frame((void*)pt_phys);
        pd->entries[dir] = 0;
    }

    pmm_free_frame((void*)pd_phys);
}

bool paging_switch_to_directory(uint32_t pd_phys) {
    if (!pd_phys) return false;

    const uint32_t SWITCH_INDEX = 200;
    struct page_table* mapped = map_page_table_window(SWITCH_INDEX, pd_phys);
    if (!mapped) {
        serial_print("Paging: ERROR - Failed to map page directory for switch\n");
        return false;
    }

    g_paging.kernel_directory = (struct page_directory*)((uint32_t)PT_VIRT_BASE + (SWITCH_INDEX * PAGE_SIZE));
    g_paging.kernel_tables[SWITCH_INDEX] = mapped;

    asm volatile ("mov %0, %%cr3" :: "r"(pd_phys));

    return true;
}

uint32_t paging_get_kernel_directory_phys() {
    return kernel_directory_phys;
}

uint32_t paging_clone_directory(uint32_t pd_phys) {
    if (!pd_phys) return 0;

    void* new_pd_phys = pmm_alloc_frame();
    if (!new_pd_phys) {
        serial_print("Paging: ERROR - Failed to allocate new directory frame for clone\n");
        return 0;
    }

    const uint32_t SRC_DIR_IDX = 100;
    const uint32_t DST_DIR_IDX = 101;

    struct page_table* src_map = map_page_table_window(SRC_DIR_IDX, pd_phys);
    struct page_table* dst_map = map_page_table_window(DST_DIR_IDX, (uint32_t)new_pd_phys);
    if (!src_map || !dst_map) {
        pmm_free_frame(new_pd_phys);
        return 0;
    }

    struct page_directory* src_pd = (struct page_directory*)((uint32_t)PT_VIRT_BASE + (SRC_DIR_IDX * PAGE_SIZE));
    struct page_directory* dst_pd = (struct page_directory*)((uint32_t)PT_VIRT_BASE + (DST_DIR_IDX * PAGE_SIZE));

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
            paging_destroy_directory((uint32_t)new_pd_phys);
            pmm_free_frame(new_pd_phys);
            return 0;
        }

        struct page_table* src_pt = map_page_table_window(dir, src_pt_phys);
        if (!src_pt) {
            pmm_free_frame(dst_pt_phys);
            paging_destroy_directory((uint32_t)new_pd_phys);
            return 0;
        }

        struct page_table* dst_pt = map_page_table_window(dir + 512, (uint32_t)dst_pt_phys);
        if (!dst_pt) {
            pmm_free_frame(dst_pt_phys);
            paging_destroy_directory((uint32_t)new_pd_phys);
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
                paging_destroy_directory((uint32_t)new_pd_phys);
                return 0;
            }

            __builtin_memcpy(new_frame, (void*)src_frame, PAGE_SIZE);

            dst_pt->entries[i] = ((uint32_t)new_frame & 0xFFFFF000) | flags;
        }

        dst_pd->entries[dir] = ((uint32_t)dst_pt_phys & 0xFFFFF000) | (pde & 0xFFF);
    }

    return (uint32_t)new_pd_phys;
}