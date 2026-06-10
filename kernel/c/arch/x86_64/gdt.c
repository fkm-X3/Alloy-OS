#include "boot/types.h"

struct gdt_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t base_middle;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
} __attribute__((packed));

struct gdt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

struct gdt_entry gdt[6];
struct gdt_ptr gdtp;

extern void gdt_flush(uint64_t gdt_ptr);

static void gdt_set_gate(int num, uint64_t base, uint64_t limit, uint8_t access, uint8_t gran) {
    gdt[num].base_low = (uint16_t)(base & 0xFFFF);
    gdt[num].base_middle = (uint8_t)((base >> 16) & 0xFF);
    gdt[num].base_high = (uint8_t)((base >> 24) & 0xFF);

    gdt[num].limit_low = (uint16_t)(limit & 0xFFFF);
    gdt[num].granularity = (uint8_t)(((limit >> 16) & 0x0F) | (gran & 0xF0));
    gdt[num].access = access;

    // x86_64: base bits 32-63 are in granularity byte for long mode
    // But for 64-bit segments, base and limit are mostly ignored
}

void init_gdt() {
    gdtp.limit = (uint16_t)(sizeof(struct gdt_entry) * 6 - 1);
    gdtp.base = (uint64_t)&gdt;

    // Null descriptor
    gdt_set_gate(0, 0, 0, 0, 0);

    // Kernel code (64-bit): DPL=0, L-bit set, present, executable, readable
    // Access: P=1, DPL=0, S=1, E=1, DC=0, RW=1, A=0 = 0x98 | 0x20 = 0x9A
    // But with L-bit set in granularity: 0x20 -> 0xA0
    gdt_set_gate(1, 0, 0, 0x9A, 0x20);  // L-bit (bit 1 of gran) = 0x20, D-bit = 0

    // Kernel data (64-bit): DPL=0, present, writable
    gdt_set_gate(2, 0, 0, 0x92, 0x00);

    // User code (64-bit): DPL=3, present, executable, readable
    gdt_set_gate(3, 0, 0, 0xFA, 0x20);  // L-bit set, D-bit clear

    // User data (64-bit): DPL=3, present, writable
    gdt_set_gate(4, 0, 0, 0xF2, 0x00);

    // TSS (will be set up when task switching is needed)
    gdt_set_gate(5, 0, 0, 0x89, 0x00);

    gdt_flush((uint64_t)&gdtp);
}
