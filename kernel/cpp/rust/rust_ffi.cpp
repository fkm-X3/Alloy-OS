/**
 * FFI Bridge - C++ exports for Rust kernel
 * 
 * This file provides C-compatible exports of C++ functions
 * so they can be called from Rust code.
 */

#include "../drivers/vga.h"
#include "../mm/vmm.h"
#include "../mm/pmm.h"

extern "C" {

// Serial output (already declared in serial driver)
extern void serial_print(const char* str);

// VGA functions - these are already extern "C" in vga driver
// Just ensuring they're visible

// Memory management exports

/**
 * Allocate a virtual memory region
 * @param size Size in bytes
 * @param flags Page flags (Present, Write, User, etc.)
 * @return Pointer to allocated region or nullptr on failure
 */
void* vmm_alloc_region(uint32_t size, uint32_t flags) {
    return g_vmm.alloc_region(size, flags);
}

/**
 * Free a virtual memory region
 * @param addr Address of region to free
 * @param size Size of region in bytes
 */
void vmm_free_region(void* addr, uint32_t size) {
    g_vmm.free_region(addr, size);
}

/**
 * Map a virtual address to a physical address
 * @param virt_addr Virtual address (page-aligned)
 * @param phys_addr Physical address (page-aligned)
 * @param flags Page flags
 * @return true on success, false on failure
 */
bool vmm_map(void* virt_addr, void* phys_addr, uint32_t flags) {
    return g_vmm.map(virt_addr, phys_addr, flags);
}

/**
 * Unmap a virtual address
 * @param virt_addr Virtual address to unmap
 */
void vmm_unmap(void* virt_addr) {
    g_vmm.unmap(virt_addr);
}

/**
 * Allocate a physical frame (4KB page)
 * @return Pointer to physical frame or nullptr on failure
 */
void* pmm_alloc_frame() {
    return g_pmm.alloc_frame();
}

/**
 * Free a physical frame
 * @param addr Address of frame to free
 */
void pmm_free_frame(void* addr) {
    g_pmm.free_frame(addr);
}

} // extern "C"
