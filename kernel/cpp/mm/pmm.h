#ifndef ALLOY_PMM_H
#define ALLOY_PMM_H

#include "../boot/types.h"

// Physical Memory Manager (PMM)
// Manages physical memory using a bitmap allocator

#define PAGE_SIZE 4096
#define FRAMES_PER_BYTE 8

struct memory_region {
    uint64_t base;
    uint64_t length;
    uint32_t type;
};

class PhysicalMemoryManager {
public:
    void init(uint32_t multiboot_addr);
    
    // Allocate a physical frame (4KB page)
    void* alloc_frame();
    
    // Free a physical frame
    void free_frame(void* addr);
    
    // Get total memory size
    uint64_t get_total_memory() const;
    
    // Get available memory size
    uint64_t get_available_memory() const;
    
    // Get number of frames
    uint32_t get_total_frames() const;
    
    // Get number of used frames
    uint32_t get_used_frames() const;
    
private:
    uint32_t* bitmap;          // Bitmap for frame allocation
    uint32_t total_frames;     // Total number of frames
    uint32_t used_frames;      // Number of used frames
    uint64_t total_memory;     // Total memory in bytes
    uint64_t available_memory; // Available memory in bytes
    
    void set_frame(uint32_t frame_number);
    void clear_frame(uint32_t frame_number);
    bool test_frame(uint32_t frame_number);
    int32_t find_free_frame();
};

extern PhysicalMemoryManager g_pmm;

#endif // ALLOY_PMM_H
