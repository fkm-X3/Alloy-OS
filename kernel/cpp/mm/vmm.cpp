#include "vmm.h"
#include "pmm.h"
#include "paging.h"

extern "C" void serial_print(const char* str);
extern "C" void serial_print_hex(uint32_t value);

// Global instance
VirtualMemoryManager g_vmm;

// Kernel heap starts at 16MB (after identity-mapped region)
#define KERNEL_HEAP_START 0x01000000
#define KERNEL_HEAP_END   0xC0000000  // 3GB (reserve upper 1GB for kernel)

void VirtualMemoryManager::init() {
    serial_print("VMM: Initializing virtual memory manager...\n");
    
    next_virt_addr = KERNEL_HEAP_START;
    allocated_pages = 0;
    
    serial_print("VMM: Initialization complete\n");
    serial_print("  Heap start: 0x");
    serial_print_hex(KERNEL_HEAP_START);
    serial_print("\n");
    serial_print("  Heap end: 0x");
    serial_print_hex(KERNEL_HEAP_END);
    serial_print("\n");
}

void* VirtualMemoryManager::alloc_region(uint32_t size, uint32_t flags) {
    // Align size to page boundary
    if (size % PAGE_SIZE != 0) {
        size = ((size / PAGE_SIZE) + 1) * PAGE_SIZE;
    }
    
    uint32_t num_pages = size / PAGE_SIZE;
    
    // Check if we have enough virtual address space
    if (next_virt_addr + size > KERNEL_HEAP_END) {
        serial_print("VMM: ERROR - Out of virtual address space\n");
        return nullptr;
    }
    
    void* virt_start = (void*)next_virt_addr;
    
    // Allocate and map physical frames
    for (uint32_t i = 0; i < num_pages; i++) {
        void* phys_frame = g_pmm.alloc_frame();
        if (!phys_frame) {
            serial_print("VMM: ERROR - Failed to allocate physical frame\n");
            // TODO: Free previously allocated frames
            return nullptr;
        }
        
        uint32_t virt = next_virt_addr + (i * PAGE_SIZE);
        if (!g_paging.map_page(virt, (uint32_t)phys_frame, flags)) {
            serial_print("VMM: ERROR - Failed to map page\n");
            g_pmm.free_frame(phys_frame);
            // TODO: Free previously allocated frames
            return nullptr;
        }
        
        allocated_pages++;
    }
    
    next_virt_addr += size;
    
    return virt_start;
}

void VirtualMemoryManager::free_region(void* virt_addr, uint32_t size) {
    if (!virt_addr) {
        return;
    }
    
    // Align size to page boundary
    if (size % PAGE_SIZE != 0) {
        size = ((size / PAGE_SIZE) + 1) * PAGE_SIZE;
    }
    
    uint32_t num_pages = size / PAGE_SIZE;
    uint32_t virt = (uint32_t)virt_addr;
    
    // Free physical frames and unmap pages
    for (uint32_t i = 0; i < num_pages; i++) {
        uint32_t page_virt = virt + (i * PAGE_SIZE);
        uint32_t phys = g_paging.get_physical_address(page_virt);
        
        if (phys != 0) {
            g_pmm.free_frame((void*)(phys & 0xFFFFF000));
            g_paging.unmap_page(page_virt);
            allocated_pages--;
        }
    }
}

bool VirtualMemoryManager::map(void* virt_addr, void* phys_addr, uint32_t flags) {
    return g_paging.map_page((uint32_t)virt_addr, (uint32_t)phys_addr, flags);
}

void VirtualMemoryManager::unmap(void* virt_addr) {
    g_paging.unmap_page((uint32_t)virt_addr);
}

uint32_t VirtualMemoryManager::get_allocated_pages() const {
    return allocated_pages;
}

// C FFI wrappers for Rust
extern "C" void* vmm_alloc_region(uint32_t size, uint32_t flags) {
    return g_vmm.alloc_region(size, flags);
}

extern "C" void vmm_free_region(void* addr, uint32_t size) {
    g_vmm.free_region(addr, size);
}

extern "C" bool vmm_map(void* virt_addr, void* phys_addr, uint32_t flags) {
    return g_vmm.map(virt_addr, phys_addr, flags);
}

extern "C" void vmm_unmap(void* virt_addr) {
    g_vmm.unmap(virt_addr);
}
