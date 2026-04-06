#ifndef ALLOY_VMM_H
#define ALLOY_VMM_H

#include "../boot/types.h"

// Virtual Memory Manager (VMM)
// Provides high-level interface for virtual memory management

class VirtualMemoryManager {
public:
    void init();
    
    // Allocate virtual memory region
    void* alloc_region(uint32_t size, uint32_t flags);
    
    // Free virtual memory region
    void free_region(void* virt_addr, uint32_t size);
    
    // Map virtual to physical address
    bool map(void* virt_addr, void* phys_addr, uint32_t flags);
    
    // Unmap virtual address
    void unmap(void* virt_addr);
    
    // Get statistics
    uint32_t get_allocated_pages() const;
    uint32_t get_heap_start() const;
    uint32_t get_heap_size() const;
    uint32_t get_next_virt_addr() const;
    
private:
    uint32_t next_virt_addr;  // Next available virtual address for allocation
    uint32_t allocated_pages; // Number of allocated pages
};

extern VirtualMemoryManager g_vmm;

#endif // ALLOY_VMM_H
