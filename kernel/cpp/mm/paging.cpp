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
