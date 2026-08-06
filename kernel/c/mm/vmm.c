#include "vmm.h"
#include "pmm.h"
#include "paging.h"
#include "../drivers/serial.h"

VirtualMemoryManager g_vmm;

#if defined(ARCH_AARCH64)
// aarch64: MMU is not enabled yet (identity/stub paging), so the VMM "virtual"
// window is never actually mapped. Heap memory must come from real identity
// RAM, which starts right after the kernel image. The kernel stack sits at the
// top of RAM (0x47FFF000, grows down), so cap the heap below it.
#define KERNEL_HEAP_START 0x40510000
#define KERNEL_HEAP_END   0x47F00000
#else
#define KERNEL_HEAP_START 0x02000000
#define KERNEL_HEAP_END   0xC0000000
#endif

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

#if defined(ARCH_AARCH64)
    // aarch64: MMU is disabled and paging_map_page is a no-op, so virtual
    // addresses equal physical addresses. The old heap window (0x02000000)
    // aliases the pflash/NOR region (not RAM), which silently dropped every
    // heap write and returned 0xFF on reads. Allocate real RAM frames from the
    // PMM and hand out their physical addresses directly.
    if (g_vmm.next_virt_addr + size > KERNEL_HEAP_END) {
        serial_print("VMM: ERROR - Out of heap space\n");
        return 0;
    }

    uintptr_t first_addr = 0;
    for (uint32_t i = 0; i < num_pages; i++) {
        void* phys_frame = pmm_alloc_frame();
        if (!phys_frame) {
            serial_print("VMM: ERROR - Failed to allocate physical frame\n");
            return 0;
        }
        if (i == 0) {
            first_addr = (uintptr_t)phys_frame;
            serial_print("VMM: aarch64 heap page at 0x");
            serial_print_hex(first_addr);
            serial_print("\n");
        }
        g_vmm.allocated_pages++;
    }
    g_vmm.next_virt_addr += size;
    return (void*)first_addr;
#else
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
#endif
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
            pmm_free_frame((void*)(phys & ~(uintptr_t)(PAGE_SIZE - 1)));
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