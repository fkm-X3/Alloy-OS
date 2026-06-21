#include "pmm.h"
#include "../boot/multiboot2.h"
#include "../drivers/serial.h"

extern uint32_t _kernel_start;
extern uint32_t _kernel_end;

PhysicalMemoryManager g_pmm;

#define MAX_PHYSICAL_FRAMES (1024 * 1024)
static uint32_t frame_refcounts[MAX_PHYSICAL_FRAMES];

#define MULTIBOOT_MEMORY_AVAILABLE 1
#define MULTIBOOT_MEMORY_RESERVED 2
#define MULTIBOOT_MEMORY_ACPI_RECLAIMABLE 3
#define MULTIBOOT_MEMORY_NVS 4
#define MULTIBOOT_MEMORY_BADRAM 5

static uint32_t frame_bitmap[1024 * 1024 / 32];

static void set_frame(uint32_t frame_number) {
    uint32_t index = frame_number / 32;
    uint32_t bit = frame_number % 32;
    g_pmm.bitmap[index] |= (1 << bit);
}

static void clear_frame(uint32_t frame_number) {
    uint32_t index = frame_number / 32;
    uint32_t bit = frame_number % 32;
    g_pmm.bitmap[index] &= ~(1 << bit);
}

static bool test_frame(uint32_t frame_number) {
    uint32_t index = frame_number / 32;
    uint32_t bit = frame_number % 32;
    return (g_pmm.bitmap[index] & (1 << bit)) != 0;
}

static int32_t find_free_frame() {
    for (uint32_t i = 0; i < g_pmm.total_frames / 32; i++) {
        if (g_pmm.bitmap[i] != 0xFFFFFFFF) {
            for (uint32_t bit = 0; bit < 32; bit++) {
                if ((g_pmm.bitmap[i] & (1 << bit)) == 0) {
                    return i * 32 + bit;
                }
            }
        }
    }
    return -1;
}

void pmm_init(uint32_t multiboot_addr) {
    serial_print("PMM: Initializing physical memory manager...\n");

    g_pmm.bitmap = frame_bitmap;
    g_pmm.total_frames = 0;
    g_pmm.used_frames = 0;
    g_pmm.total_memory = 0;
    g_pmm.available_memory = 0;

    for (uint32_t i = 0; i < sizeof(frame_bitmap) / sizeof(uint32_t); i++) {
        g_pmm.bitmap[i] = 0xFFFFFFFF;
    }

    for (uint32_t i = 0; i < MAX_PHYSICAL_FRAMES; i++) {
        frame_refcounts[i] = 0;
    }

    if (multiboot_addr == 0) {
        serial_print("PMM: No multiboot info (aarch64), using default memory layout\n");
        g_pmm.total_memory = 128 * 1024 * 1024;
        g_pmm.available_memory = 128 * 1024 * 1024;
        /* QEMU virt: RAM at 0x40000000, 128MB */
        uint32_t ram_start_frame = 0x40000000 / PAGE_SIZE;
        uint32_t ram_end_frame = 0x48000000 / PAGE_SIZE;
        g_pmm.total_frames = ram_end_frame;
        for (uint32_t i = ram_start_frame; i < ram_end_frame; i++) {
            clear_frame(i);
        }
    } else {
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

                    g_pmm.total_memory += entry->len;

                    if (entry->type == MULTIBOOT_MEMORY_AVAILABLE) {
                        g_pmm.available_memory += entry->len;

                        uint64_t base = entry->addr;
                        uint64_t length = entry->len;

                        if (base % PAGE_SIZE != 0) {
                            uint64_t offset = PAGE_SIZE - (base % PAGE_SIZE);
                            base += offset;
                            if (length > offset) {
                                length -= offset;
                            } else {
                                length = 0;
                            }
                        }

                        length = (length / PAGE_SIZE) * PAGE_SIZE;

                        uint32_t start_frame = base / PAGE_SIZE;
                        uint32_t num_frames = length / PAGE_SIZE;

                        for (uint32_t i = 0; i < num_frames; i++) {
                            uint32_t frame = start_frame + i;
                            if (frame < 1024 * 1024) {
                                clear_frame(frame);
                                if (frame >= g_pmm.total_frames) {
                                    g_pmm.total_frames = frame + 1;
                                }
                            }
                        }
                    }
                }
            }

            tag = (struct multiboot_tag*)((uint8_t*)tag + ((tag->size + 7) & ~7));
        }
    }

    for (uint32_t frame = 0; frame < 256; frame++) {
        set_frame(frame);
        g_pmm.used_frames++;
    }

    uint32_t kernel_start = (uint32_t)&_kernel_start;
    uint32_t kernel_end = (uint32_t)&_kernel_end;
    serial_print("  Kernel region start: 0x");
    serial_print_hex(kernel_start);
    serial_print("\n");
    serial_print("  Kernel region end: 0x");
    serial_print_hex(kernel_end);
    serial_print("\n");
    uint32_t kernel_start_frame = kernel_start / PAGE_SIZE;
    uint32_t kernel_end_frame = (kernel_end + PAGE_SIZE - 1) / PAGE_SIZE;
    for (uint32_t frame = kernel_start_frame; frame < kernel_end_frame; frame++) {
        if (!test_frame(frame)) {
            set_frame(frame);
            g_pmm.used_frames++;
        }
    }

    serial_print("PMM: Initialization complete\n");
    serial_print("  Total memory: ");
    serial_print_hex((uint32_t)(g_pmm.total_memory / 1024 / 1024));
    serial_print(" MB\n");
    serial_print("  Available memory: ");
    serial_print_hex((uint32_t)(g_pmm.available_memory / 1024 / 1024));
    serial_print(" MB\n");
    serial_print("  Total frames: ");
    serial_print_hex(g_pmm.total_frames);
    serial_print("\n");
    serial_print("  Used frames: ");
    serial_print_hex(g_pmm.used_frames);
    serial_print("\n");
}

void* pmm_alloc_frame() {
    int32_t frame = find_free_frame();
    if (frame == -1) {
        serial_print("PMM: ERROR - Out of memory!\n");
        return 0;
    }

    set_frame(frame);
    g_pmm.used_frames++;

    if (frame < MAX_PHYSICAL_FRAMES) {
        frame_refcounts[frame] = 1;
    }

    return (void*)(frame * PAGE_SIZE);
}

void pmm_free_frame(void* addr) {
    uint32_t frame = (uint32_t)addr / PAGE_SIZE;

    if (frame >= g_pmm.total_frames) {
        serial_print("PMM: ERROR - Invalid frame address\n");
        return;
    }

    if (!test_frame(frame)) {
        serial_print("PMM: WARNING - Double free detected\n");
        return;
    }

    clear_frame(frame);
    g_pmm.used_frames--;
}

uint64_t pmm_get_total_memory() {
    return g_pmm.total_memory;
}

uint64_t pmm_get_available_memory() {
    return g_pmm.available_memory;
}

uint32_t pmm_get_total_frames() {
    return g_pmm.total_frames;
}

uint32_t pmm_get_used_frames() {
    return g_pmm.used_frames;
}

void pmm_refcount_inc(void* addr) {
    uint32_t frame = (uint32_t)addr / PAGE_SIZE;
    if (frame < MAX_PHYSICAL_FRAMES) {
        frame_refcounts[frame]++;
    }
}

void pmm_refcount_dec(void* addr) {
    uint32_t frame = (uint32_t)addr / PAGE_SIZE;
    if (frame < MAX_PHYSICAL_FRAMES) {
        if (frame_refcounts[frame] > 0) {
            frame_refcounts[frame]--;
        }
        if (frame_refcounts[frame] == 0) {
            pmm_free_frame(addr);
        }
    }
}

uint32_t pmm_refcount_get(void* addr) {
    uint32_t frame = (uint32_t)addr / PAGE_SIZE;
    if (frame < MAX_PHYSICAL_FRAMES) {
        return frame_refcounts[frame];
    }
    return 0;
}