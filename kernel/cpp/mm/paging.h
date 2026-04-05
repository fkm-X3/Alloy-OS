#ifndef ALLOY_PAGING_H
#define ALLOY_PAGING_H

#include "../boot/types.h"

// Page directory and page table entry flags
#define PAGE_PRESENT    0x001  // Page is present in memory
#define PAGE_WRITE      0x002  // Page is writable
#define PAGE_USER       0x004  // Page is accessible in user mode
#define PAGE_WRITETHROUGH 0x008 // Write-through caching
#define PAGE_CACHE_DISABLE 0x010 // Cache disabled
#define PAGE_ACCESSED   0x020  // Page has been accessed
#define PAGE_DIRTY      0x040  // Page has been written to
#define PAGE_SIZE_FLAG  0x080  // 4MB pages (in PDE)
#define PAGE_GLOBAL     0x100  // Global page (not flushed from TLB)

// Page directory entry (PDE) and page table entry (PTE)
typedef uint32_t page_dir_entry_t;
typedef uint32_t page_table_entry_t;

// Page directory (1024 entries, each covering 4MB)
struct page_directory {
    page_dir_entry_t entries[1024];
} __attribute__((aligned(4096)));

// Page table (1024 entries, each covering 4KB)
struct page_table {
    page_table_entry_t entries[1024];
} __attribute__((aligned(4096)));

class Paging {
public:
    void init();
    
    // Map a virtual address to a physical address
    bool map_page(uint32_t virt_addr, uint32_t phys_addr, uint32_t flags);
    
    // Unmap a virtual address
    void unmap_page(uint32_t virt_addr);
    
    // Get the physical address mapped to a virtual address
    uint32_t get_physical_address(uint32_t virt_addr);
    
    // Enable paging
    void enable();
    
    // Get kernel page directory
    page_directory* get_kernel_directory();
    
private:
    page_directory* kernel_directory;
    page_table* kernel_tables[1024];
    
    uint32_t* get_page_entry(uint32_t virt_addr, bool create);
    void invalidate_page(uint32_t virt_addr);
};

extern Paging g_paging;

#endif // ALLOY_PAGING_H
