#ifndef ALLOY_PMM_H
#define ALLOY_PMM_H

#include "../boot/types.h"

#define PAGE_SIZE 4096
#define FRAMES_PER_BYTE 8

struct memory_region {
    uint64_t base;
    uint64_t length;
    uint32_t type;
};

typedef struct {
    uint32_t* bitmap;
    uint32_t total_frames;
    uint32_t used_frames;
    uint64_t total_memory;
    uint64_t available_memory;
} PhysicalMemoryManager;

extern PhysicalMemoryManager g_pmm;

#ifdef __cplusplus
extern "C" {
#endif

void pmm_init(uint32_t multiboot_addr);
void* pmm_alloc_frame();
void pmm_free_frame(void* addr);
uint64_t pmm_get_total_memory();
uint64_t pmm_get_available_memory();
uint32_t pmm_get_total_frames();
uint32_t pmm_get_used_frames();

void pmm_refcount_inc(void* addr);
void pmm_refcount_dec(void* addr);
uint32_t pmm_refcount_get(void* addr);

#ifdef __cplusplus
}
#endif

#endif // ALLOY_PMM_H