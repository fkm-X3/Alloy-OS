#include "vmm.h"
#include "pmm.h"
#include "paging.h"
#include "../drivers/serial.h"

VirtualMemoryManager g_vmm;

#define KERNEL_HEAP_START 0x01000000
#define KERNEL_HEAP_END   0xC0000000

void vmm_init() {
    serial_print("VMM: Initializing virtual memory manager...\n");

    g_vmm.next_virt_addr = KERNEL_HEAP_START;
    g_vmm.allocated_pages = 0;

    serial_print("VMM: Initialization complete\n");
    serial_print("  Heap start: 0x");
    serial_print_hex(KERNEL_HEAP_START);
    serial_print("\n");
    serial_print("  Heap end: 0x");
    serial_print_hex(KERNEL_HEAP_END);
    serial_print("\n");
}

void* vmm_alloc_region(uintptr_t size, uint32_t flags) {
    if (size % PAGE_SIZE != 0) {
        size = ((size / PAGE_SIZE) + 1) * PAGE_SIZE;
    }

    uintptr_t num_pages = size / PAGE_SIZE;

    if (g_vmm.next_virt_addr + size > KERNEL_HEAP_END) {
        serial_print("VMM: ERROR - Out of virtual address space\n");
        return 0;
    }

    void* virt_start = (void*)(uintptr_t)g_vmm.next_virt_addr;

    for (uint32_t i = 0; i < num_pages; i++) {
        void* phys_frame = pmm_alloc_frame();
        if (!phys_frame) {
            serial_print("VMM: ERROR - Failed to allocate physical frame\n");
            return 0;
        }

        uintptr_t virt = g_vmm.next_virt_addr + (i * PAGE_SIZE);
        if (!paging_map_page(virt, (uintptr_t)phys_frame, flags)) {
            serial_print("VMM: ERROR - Failed to map page\n");
            pmm_free_frame(phys_frame);
            return 0;
        }

        g_vmm.allocated_pages++;
    }

    g_vmm.next_virt_addr += size;

    return virt_start;
}

void vmm_free_region(void* virt_addr, uintptr_t size) {
    if (!virt_addr) {
        return;
    }

    if (size % PAGE_SIZE != 0) {
        size = ((size / PAGE_SIZE) + 1) * PAGE_SIZE;
    }

    uintptr_t num_pages = size / PAGE_SIZE;
    uintptr_t virt = (uintptr_t)virt_addr;

    for (uint32_t i = 0; i < num_pages; i++) {
        uintptr_t page_virt = virt + (i * PAGE_SIZE);
        uintptr_t phys = paging_get_physical_address(page_virt);

        if (phys != 0) {
            pmm_free_frame((void*)(phys & 0xFFFFF000));
            paging_unmap_page(page_virt);
            g_vmm.allocated_pages--;
        }
    }
}

bool vmm_map(void* virt_addr, void* phys_addr, uint32_t flags) {
    return paging_map_page((uintptr_t)virt_addr, (uintptr_t)phys_addr, flags);
}

void vmm_unmap(void* virt_addr) {
    paging_unmap_page((uintptr_t)virt_addr);
}

uint32_t vmm_get_allocated_pages() {
    return g_vmm.allocated_pages;
}

uintptr_t vmm_get_heap_start() {
    return KERNEL_HEAP_START;
}

uintptr_t vmm_get_heap_size() {
    return g_vmm.next_virt_addr - KERNEL_HEAP_START;
}

uintptr_t vmm_get_next_virt_addr() {
    return g_vmm.next_virt_addr;
}