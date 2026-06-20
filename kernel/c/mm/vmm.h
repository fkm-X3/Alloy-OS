#ifndef ALLOY_VMM_H
#define ALLOY_VMM_H

#include "../boot/types.h"

typedef struct {
    uintptr_t next_virt_addr;
    uint32_t allocated_pages;
} VirtualMemoryManager;

extern VirtualMemoryManager g_vmm;

#ifdef __cplusplus
extern "C" {
#endif

void vmm_init();
void* vmm_alloc_region(uintptr_t size, uint32_t flags);
void vmm_free_region(void* virt_addr, uintptr_t size);
bool vmm_map(void* virt_addr, void* phys_addr, uint32_t flags);
void vmm_unmap(void* virt_addr);
uint32_t vmm_get_allocated_pages();
uintptr_t vmm_get_heap_start();
uintptr_t vmm_get_heap_size();
uintptr_t vmm_get_next_virt_addr();

#ifdef __cplusplus
}
#endif

#endif // ALLOY_VMM_H