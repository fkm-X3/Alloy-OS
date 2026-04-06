#include "boot/types.h"

// GDT Entry structure
struct gdt_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t base_middle;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
} __attribute__((packed));

// GDT Pointer structure
struct gdt_ptr {
    uint16_t limit;
    uint32_t base;  // 32-bit address in protected mode
} __attribute__((packed));

// GDT with 5 entries: null, kernel code, kernel data, user code, user data
struct gdt_entry gdt[5];
struct gdt_ptr gdtp;

// External assembly function to load GDT
extern "C" void gdt_flush(uint32_t gdt_ptr);

// Set a GDT entry
static void gdt_set_gate(int num, uint32_t base, uint32_t limit, uint8_t access, uint8_t gran) {
    gdt[num].base_low = (base & 0xFFFF);
    gdt[num].base_middle = (base >> 16) & 0xFF;
    gdt[num].base_high = (base >> 24) & 0xFF;
    
    gdt[num].limit_low = (limit & 0xFFFF);
    gdt[num].granularity = ((limit >> 16) & 0x0F) | (gran & 0xF0);
    gdt[num].access = access;
}

extern "C" void init_gdt() {
    gdtp.limit = (sizeof(struct gdt_entry) * 5) - 1;
    gdtp.base = (uint32_t)&gdt;
    
    // NULL descriptor
    gdt_set_gate(0, 0, 0, 0, 0);
    
    // Kernel code segment (0x08)
    // Base=0, Limit=0xFFFFF, Access=0x9A (present, ring 0, executable, readable)
    // Granularity=0xCF (4KB granularity, 32-bit)
    gdt_set_gate(1, 0, 0xFFFFFFFF, 0x9A, 0xCF);
    
    // Kernel data segment (0x10)
    // Base=0, Limit=0xFFFFF, Access=0x92 (present, ring 0, writable)
    // Granularity=0xCF (4KB granularity, 32-bit)
    gdt_set_gate(2, 0, 0xFFFFFFFF, 0x92, 0xCF);
    
    // User code segment (0x18)
    // Base=0, Limit=0xFFFFF, Access=0xFA (present, ring 3, executable, readable)
    // Granularity=0xCF (4KB granularity, 32-bit)
    gdt_set_gate(3, 0, 0xFFFFFFFF, 0xFA, 0xCF);
    
    // User data segment (0x20)
    // Base=0, Limit=0xFFFFF, Access=0xF2 (present, ring 3, writable)
    // Granularity=0xCF (4KB granularity, 32-bit)
    gdt_set_gate(4, 0, 0xFFFFFFFF, 0xF2, 0xCF);
    
    // Load the GDT
    gdt_flush((uint32_t)&gdtp);
}
