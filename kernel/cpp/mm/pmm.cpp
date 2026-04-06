#include "pmm.h"
#include "../boot/multiboot2.h"

extern "C" void serial_print(const char* str);
extern "C" void serial_print_hex(uint32_t value);

// Global instance
PhysicalMemoryManager g_pmm;

// Memory type constants
#define MULTIBOOT_MEMORY_AVAILABLE 1
#define MULTIBOOT_MEMORY_RESERVED 2
#define MULTIBOOT_MEMORY_ACPI_RECLAIMABLE 3
#define MULTIBOOT_MEMORY_NVS 4
#define MULTIBOOT_MEMORY_BADRAM 5

// Static bitmap storage (placed in BSS section)
// Support up to 4GB of RAM (1M frames * 4KB = 4GB)
static uint32_t frame_bitmap[1024 * 1024 / 32]; // 32KB bitmap

void PhysicalMemoryManager::init(uint32_t multiboot_addr) {
    serial_print("PMM: Initializing physical memory manager...\n");
    
    // Initialize fields
    bitmap = frame_bitmap;
    total_frames = 0;
    used_frames = 0;
    total_memory = 0;
    available_memory = 0;
    
    // Mark all frames as used initially
    for (uint32_t i = 0; i < sizeof(frame_bitmap) / sizeof(uint32_t); i++) {
        bitmap[i] = 0xFFFFFFFF;
    }
    
    // Parse multiboot2 info structure
    struct multiboot_tag* tag = (struct multiboot_tag*)(multiboot_addr + 8);
    
    while (tag->type != MULTIBOOT_TAG_TYPE_END) {
        if (tag->type == MULTIBOOT_TAG_TYPE_BASIC_MEMINFO) {
            struct multiboot_tag_basic_meminfo* meminfo = 
                (struct multiboot_tag_basic_meminfo*)tag;
            serial_print("PMM: Basic memory info:\n");
            serial_print("  Lower memory: ");
            serial_print_hex(meminfo->mem_lower);
            serial_print(" KB\n");
            serial_print("  Upper memory: ");
            serial_print_hex(meminfo->mem_upper);
            serial_print(" KB\n");
        }
        else if (tag->type == MULTIBOOT_TAG_TYPE_MMAP) {
            struct multiboot_tag_mmap* mmap = (struct multiboot_tag_mmap*)tag;
            serial_print("PMM: Memory map:\n");
            
            // Parse memory map entries
            for (uint8_t* entry_ptr = (uint8_t*)mmap->entries;
                 entry_ptr < (uint8_t*)tag + tag->size;
                 entry_ptr += mmap->entry_size) {
                
                struct multiboot_mmap_entry* entry = 
                    (struct multiboot_mmap_entry*)entry_ptr;
                
                serial_print("  Region: addr=0x");
                serial_print_hex((uint32_t)entry->addr);
                serial_print(", len=0x");
                serial_print_hex((uint32_t)entry->len);
                serial_print(", type=");
                serial_print_hex(entry->type);
                serial_print("\n");
                
                total_memory += entry->len;
                
                // Mark available regions as free
                if (entry->type == MULTIBOOT_MEMORY_AVAILABLE) {
                    available_memory += entry->len;
                    
                    // Calculate frame range
                    uint64_t base = entry->addr;
                    uint64_t length = entry->len;
                    
                    // Align base up to page boundary
                    if (base % PAGE_SIZE != 0) {
                        uint64_t offset = PAGE_SIZE - (base % PAGE_SIZE);
                        base += offset;
                        if (length > offset) {
                            length -= offset;
                        } else {
                            length = 0;
                        }
                    }
                    
                    // Align length down to page boundary
                    length = (length / PAGE_SIZE) * PAGE_SIZE;
                    
                    // Mark frames as free
                    uint32_t start_frame = base / PAGE_SIZE;
                    uint32_t num_frames = length / PAGE_SIZE;
                    
                    for (uint32_t i = 0; i < num_frames; i++) {
                        uint32_t frame = start_frame + i;
                        if (frame < 1024 * 1024) { // Limit to 4GB
                            clear_frame(frame);
                            if (frame >= total_frames) {
                                total_frames = frame + 1;
                            }
                        }
                    }
                }
            }
        }
        
        // Move to next tag (tags are 8-byte aligned)
        tag = (struct multiboot_tag*)((uint8_t*)tag + ((tag->size + 7) & ~7));
    }
    
    // Reserve first 1MB (for kernel and BIOS)
    for (uint32_t frame = 0; frame < 256; frame++) {
        set_frame(frame);
        used_frames++;
    }
    
    serial_print("PMM: Initialization complete\n");
    serial_print("  Total memory: ");
    serial_print_hex((uint32_t)(total_memory / 1024 / 1024));
    serial_print(" MB\n");
    serial_print("  Available memory: ");
    serial_print_hex((uint32_t)(available_memory / 1024 / 1024));
    serial_print(" MB\n");
    serial_print("  Total frames: ");
    serial_print_hex(total_frames);
    serial_print("\n");
    serial_print("  Used frames: ");
    serial_print_hex(used_frames);
    serial_print("\n");
}

void* PhysicalMemoryManager::alloc_frame() {
    int32_t frame = find_free_frame();
    if (frame == -1) {
        serial_print("PMM: ERROR - Out of memory!\n");
        return nullptr;
    }
    
    set_frame(frame);
    used_frames++;
    
    return (void*)(frame * PAGE_SIZE);
}

void PhysicalMemoryManager::free_frame(void* addr) {
    uint32_t frame = (uint32_t)addr / PAGE_SIZE;
    
    if (frame >= total_frames) {
        serial_print("PMM: ERROR - Invalid frame address\n");
        return;
    }
    
    if (!test_frame(frame)) {
        serial_print("PMM: WARNING - Double free detected\n");
        return;
    }
    
    clear_frame(frame);
    used_frames--;
}

uint64_t PhysicalMemoryManager::get_total_memory() const {
    return total_memory;
}

uint64_t PhysicalMemoryManager::get_available_memory() const {
    return available_memory;
}

uint32_t PhysicalMemoryManager::get_total_frames() const {
    return total_frames;
}

uint32_t PhysicalMemoryManager::get_used_frames() const {
    return used_frames;
}

void PhysicalMemoryManager::set_frame(uint32_t frame_number) {
    uint32_t index = frame_number / 32;
    uint32_t bit = frame_number % 32;
    bitmap[index] |= (1 << bit);
}

void PhysicalMemoryManager::clear_frame(uint32_t frame_number) {
    uint32_t index = frame_number / 32;
    uint32_t bit = frame_number % 32;
    bitmap[index] &= ~(1 << bit);
}

bool PhysicalMemoryManager::test_frame(uint32_t frame_number) {
    uint32_t index = frame_number / 32;
    uint32_t bit = frame_number % 32;
    return (bitmap[index] & (1 << bit)) != 0;
}

int32_t PhysicalMemoryManager::find_free_frame() {
    for (uint32_t i = 0; i < total_frames / 32; i++) {
        if (bitmap[i] != 0xFFFFFFFF) {
            // Found a bitmap entry with free frames
            for (uint32_t bit = 0; bit < 32; bit++) {
                if ((bitmap[i] & (1 << bit)) == 0) {
                    return i * 32 + bit;
                }
            }
        }
    }
    
    return -1; // No free frames
}

// C FFI wrappers for Rust
extern "C" void* pmm_alloc_frame() {
    return g_pmm.alloc_frame();
}

extern "C" void pmm_free_frame(void* addr) {
    g_pmm.free_frame(addr);
}

extern "C" uint32_t pmm_get_total_frames() {
    return g_pmm.get_total_frames();
}

extern "C" uint32_t pmm_get_used_frames() {
    return g_pmm.get_used_frames();
}

extern "C" uint64_t pmm_get_total_memory() {
    return g_pmm.get_total_memory();
}

extern "C" uint64_t pmm_get_available_memory() {
    return g_pmm.get_available_memory();
}
