#include "paging.h"
#include "pmm.h"

extern "C" void serial_print(const char* str);
extern "C" void serial_print_hex(uint32_t value);

// Global instance
Paging g_paging;

// External symbols from linker script
extern "C" uint32_t _kernel_start;
extern "C" uint32_t _kernel_end;

// Static storage for kernel page directory and initial page tables
static page_directory kernel_dir __attribute__((aligned(4096)));
static page_table kernel_pts[4] __attribute__((aligned(4096))); // First 16MB

// Virtual window (12MB-16MB) used to access arbitrary page-table frames.
// We keep a stable virtual mapping for each PDE index: PT_VIRT_BASE + index*4KB.
#define PT_VIRT_BASE 0x00C00000
#define PT_VIRT_MAP_TABLE_INDEX 3 // kernel_pts[3] covers 12MB-16MB

static inline void invalidate_page_local(uint32_t virt_addr) {
    asm volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
}

static page_table* map_page_table_window(uint32_t dir_index, uint32_t pt_phys) {
    uint32_t pt_virt = PT_VIRT_BASE + (dir_index * PAGE_SIZE);
    kernel_pts[PT_VIRT_MAP_TABLE_INDEX].entries[dir_index] =
        (pt_phys & 0xFFFFF000) | PAGE_PRESENT | PAGE_WRITE;
    invalidate_page_local(pt_virt);
    return (page_table*)pt_virt;
}

void Paging::init() {
    serial_print("Paging: Initializing paging...\n");
    
    kernel_directory = &kernel_dir;
    
    // Clear page directory
    for (int i = 0; i < 1024; i++) {
        kernel_directory->entries[i] = 0;
        kernel_tables[i] = nullptr;
    }
    
    // Identity map the first 16MB (4 page tables)
    // This covers kernel code/data and VGA memory
    for (int pt = 0; pt < 4; pt++) {
        kernel_tables[pt] = &kernel_pts[pt];
        
        // Clear page table
        for (int i = 0; i < 1024; i++) {
            kernel_pts[pt].entries[i] = 0;
        }
        
        // Map 4MB (1024 pages * 4KB)
        for (int i = 0; i < 1024; i++) {
            uint32_t phys = (pt * 1024 + i) * 0x1000; // Physical address
            kernel_pts[pt].entries[i] = phys | PAGE_PRESENT | PAGE_WRITE;
        }
        
        // Add page table to directory
        uint32_t pt_phys = (uint32_t)&kernel_pts[pt];
        kernel_directory->entries[pt] = pt_phys | PAGE_PRESENT | PAGE_WRITE;
    }
    
    serial_print("Paging: Identity mapped first 16MB\n");
    serial_print("  Page directory at: 0x");
    serial_print_hex((uint32_t)kernel_directory);
    serial_print("\n");
    
    // Log kernel boundaries
    serial_print("  Kernel start: 0x");
    serial_print_hex((uint32_t)&_kernel_start);
    serial_print("\n");
    serial_print("  Kernel end: 0x");
    serial_print_hex((uint32_t)&_kernel_end);
    serial_print("\n");
}

void Paging::enable() {
    serial_print("Paging: Enabling paging...\n");
    
    // Load page directory into CR3
    uint32_t pd_phys = (uint32_t)kernel_directory;
    asm volatile("mov %0, %%cr3" :: "r"(pd_phys));
    
    // Enable paging by setting bit 31 of CR0
    uint32_t cr0;
    asm volatile("mov %%cr0, %0" : "=r"(cr0));
    cr0 |= 0x80000000; // Set PG bit
    asm volatile("mov %0, %%cr0" :: "r"(cr0));
    
    serial_print("Paging: Paging enabled successfully\n");
}

bool Paging::map_page(uint32_t virt_addr, uint32_t phys_addr, uint32_t flags) {
    uint32_t* page_entry = get_page_entry(virt_addr, true);
    if (!page_entry) {
        return false;
    }
    
    *page_entry = (phys_addr & 0xFFFFF000) | (flags & 0xFFF) | PAGE_PRESENT;
    invalidate_page(virt_addr);
    
    return true;
}

void Paging::unmap_page(uint32_t virt_addr) {
    uint32_t* page_entry = get_page_entry(virt_addr, false);
    if (page_entry) {
        *page_entry = 0;
        invalidate_page(virt_addr);
    }
}

uint32_t Paging::get_physical_address(uint32_t virt_addr) {
    uint32_t* page_entry = get_page_entry(virt_addr, false);
    if (!page_entry || !(*page_entry & PAGE_PRESENT)) {
        return 0;
    }
    
    return (*page_entry & 0xFFFFF000) | (virt_addr & 0xFFF);
}

page_directory* Paging::get_kernel_directory() {
    return kernel_directory;
}

// Create a new page directory and initialize it with kernel mappings for the first 4 PDEs
uint32_t Paging::create_page_directory_phys() {
    // Allocate a physical frame for the new page directory
    void* pd_phys = g_pmm.alloc_frame();
    if (!pd_phys) {
        serial_print("Paging: ERROR - Failed to allocate page directory frame\n");
        return 0;
    }

    // Map the new page-directory frame into the stable PT window at a safe index (e.g., 100)
    const uint32_t TEMP_INDEX = 100;
    page_table* tmp = map_page_table_window(TEMP_INDEX, (uint32_t)pd_phys);
    if (!tmp) {
        serial_print("Paging: ERROR - Failed to map temporary page directory frame\n");
        g_pmm.free_frame(pd_phys);
        return 0;
    }

    // Zero the new directory
    page_directory* new_pd = (page_directory*)((uint32_t)PT_VIRT_BASE + (TEMP_INDEX * PAGE_SIZE));
    for (int i = 0; i < 1024; i++) {
        new_pd->entries[i] = 0;
    }

    // Copy kernel mappings for the first 4 entries (identity mapped region)
    for (int i = 0; i < 4; i++) {
        new_pd->entries[i] = kernel_directory->entries[i];
    }

    // leave the temporary mapping in place so the kernel can reference this page directory via a stable virtual address
    return (uint32_t)pd_phys;
}

void Paging::destroy_page_directory(uint32_t pd_phys) {
    if (!pd_phys) return;

    serial_print("Paging: Destroying page directory\n");

    // Map the page-directory frame into the stable PT window at a temporary index
    const uint32_t TEMP_INDEX = 101;
    page_table* tmp_pd_map = map_page_table_window(TEMP_INDEX, pd_phys);
    page_directory* pd = (page_directory*)((uint32_t)PT_VIRT_BASE + (TEMP_INDEX * PAGE_SIZE));

    // Iterate PDEs (skip first 4 kernel identity-mapped entries)
    for (int dir = 4; dir < 1024; dir++) {
        uint32_t pde = pd->entries[dir];
        if (!(pde & PAGE_PRESENT)) continue;

        uint32_t pt_phys = pde & 0xFFFFF000;
        // Map the page-table frame into the PT window at this dir index
        page_table* pt = map_page_table_window(dir, pt_phys);
        if (!pt) continue; // should not happen

        // Iterate PTEs and free referenced physical frames
        for (int i = 0; i < 1024; i++) {
            uint32_t pte = pt->entries[i];
            if (!(pte & PAGE_PRESENT)) continue;
            uint32_t frame_phys = pte & 0xFFFFF000;
            // Free the physical frame mapped by this PTE
            g_pmm.free_frame((void*)frame_phys);
            // Clear entry
            pt->entries[i] = 0;
        }

        // Free the page-table frame itself
        g_pmm.free_frame((void*)pt_phys);
        // Clear PDE
        pd->entries[dir] = 0;
    }

    // Finally free the page-directory frame
    g_pmm.free_frame((void*)pd_phys);
}

bool Paging::switch_to_page_directory(uint32_t pd_phys) {
    if (!pd_phys) return false;

    // Map the page-directory frame into the stable PT window at a reserved index
    const uint32_t SWITCH_INDEX = 200;
    page_table* mapped = map_page_table_window(SWITCH_INDEX, pd_phys);
    if (!mapped) {
        serial_print("Paging: ERROR - Failed to map page directory for switch\n");
        return false;
    }

    // Update internal kernel_directory pointer to the new virtual mapping
    kernel_directory = (page_directory*)((uint32_t)PT_VIRT_BASE + (SWITCH_INDEX * PAGE_SIZE));
    kernel_tables[SWITCH_INDEX] = mapped;

    // Load CR3 to switch to this page directory
    asm volatile ("mov %0, %%cr3" :: "r"(pd_phys));

    return true;
}

// C-compatible wrappers for Rust
extern "C" uint32_t paging_create_directory_phys() {
    return g_paging.create_page_directory_phys();
}

extern "C" bool paging_switch_to_directory(uint32_t pd_phys) {
    return g_paging.switch_to_page_directory(pd_phys);
}

extern "C" void paging_destroy_directory(uint32_t pd_phys) {
    g_paging.destroy_page_directory(pd_phys);
}

extern "C" uint32_t paging_get_kernel_directory_phys() {
    return (uint32_t)g_paging.get_kernel_directory();
}

extern "C" uint32_t paging_get_physical_address(uint32_t virt) {
    return g_paging.get_physical_address(virt);
}

uint32_t* Paging::get_page_entry(uint32_t virt_addr, bool create) {
    uint32_t dir_index = virt_addr >> 22;
    uint32_t table_index = (virt_addr >> 12) & 0x3FF;

    // The first 16MB is always backed by the static identity-mapped tables.
    if (dir_index < 4) {
        return &kernel_pts[dir_index].entries[table_index];
    }
    
    // Check if page table exists
    if (!(kernel_directory->entries[dir_index] & PAGE_PRESENT)) {
        if (!create) {
            return nullptr;
        }
        
        // Allocate a new page table
        void* pt_phys = g_pmm.alloc_frame();
        if (!pt_phys) {
            serial_print("Paging: ERROR - Failed to allocate page table\n");
            return nullptr;
        }
        
        // Map this page-table frame into the stable PT window and clear it.
        page_table* pt = map_page_table_window(dir_index, (uint32_t)pt_phys);
        for (int i = 0; i < 1024; i++) {
            pt->entries[i] = 0;
        }
        
        // Add to directory
        kernel_directory->entries[dir_index] = (uint32_t)pt_phys | PAGE_PRESENT | PAGE_WRITE;
        kernel_tables[dir_index] = pt;
    }
    
    // Get page table
    page_table* pt = kernel_tables[dir_index];
    if (!pt) {
        uint32_t pt_phys = kernel_directory->entries[dir_index] & 0xFFFFF000;
        pt = map_page_table_window(dir_index, pt_phys);
        kernel_tables[dir_index] = pt;
    }
    
    return &pt->entries[table_index];
}

void Paging::invalidate_page(uint32_t virt_addr) {
    asm volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
}
