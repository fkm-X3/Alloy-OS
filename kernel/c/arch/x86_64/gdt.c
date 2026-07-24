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

/* 64-bit TSS: 104 bytes, must be 16-byte aligned */
struct tss {
    uint32_t reserved0;
    uint64_t rsp0;
    uint64_t rsp1;
    uint64_t rsp2;
    uint64_t reserved1;
    uint64_t ist1;
    uint64_t ist2;
    uint64_t ist3;
    uint64_t ist4;
    uint64_t ist5;
    uint64_t ist6;
    uint64_t ist7;
    uint64_t reserved2;
    uint16_t reserved3;
    uint16_t iopb_offset;
} __attribute__((packed, aligned(16)));

/* 7 GDT entries: null, kernel code, kernel data, user code, user data, TSS low, TSS high */
struct gdt_entry gdt[7];
struct gdt_ptr gdtp;
struct tss kernel_tss;

extern void gdt_flush(uint64_t gdt_ptr);
extern uint64_t kernel_stack_top;

static void gdt_set_gate(int num, uint64_t base, uint64_t limit, uint8_t access, uint8_t gran) {
    gdt[num].base_low = (uint16_t)(base & 0xFFFF);
    gdt[num].base_middle = (uint8_t)((base >> 16) & 0xFF);
    gdt[num].base_high = (uint8_t)((base >> 24) & 0xFF);

    gdt[num].limit_low = (uint16_t)(limit & 0xFFFF);
    gdt[num].granularity = (uint8_t)(((limit >> 16) & 0x0F) | (gran & 0xF0));
    gdt[num].access = access;
}

static void tss_set_gate(int num, uint64_t base, uint32_t limit) {
    /* TSS descriptor spans two GDT entries (16 bytes total) */
    gdt[num].limit_low = (uint16_t)(limit & 0xFFFF);
    gdt[num].base_low = (uint16_t)(base & 0xFFFF);
    gdt[num].base_middle = (uint8_t)((base >> 16) & 0xFF);
    gdt[num].access = 0x89;  /* Present, DPL=0, System, Type=9 (available64-bit TSS) */
    gdt[num].granularity = (uint8_t)((limit >> 16) & 0x0F);
    gdt[num].base_high = (uint8_t)((base >> 24) & 0xFF);

    /* High8 bytes: base[63:32], reserved */
    struct gdt_entry *high = &gdt[num + 1];
    high->limit_low = (uint16_t)((base >> 32) & 0xFFFF);
    high->base_low = (uint16_t)((base >> 48) & 0xFFFF);
    high->base_middle = 0;
    high->access = 0;
    high->granularity = 0;
    high->base_high = 0;
}

void init_gdt() {
    gdtp.limit = (uint16_t)(sizeof(struct gdt_entry) * 7 - 1);
    gdtp.base = (uint64_t)&gdt;

    /* Null descriptor */
    gdt_set_gate(0, 0, 0, 0, 0);

    /* Kernel code (64-bit): DPL=0, L-bit set, present, executable, readable */
    gdt_set_gate(1, 0, 0, 0x9A, 0x20);

    /* Kernel data (64-bit): DPL=0, present, writable */
    gdt_set_gate(2, 0, 0, 0x92, 0x00);

    /* User data (64-bit): DPL=3, present, writable — GDT[3] = selector 0x1B
     * SYSRET: SS = (STAR[63:48] + 8) | 3 = (0x10 + 8) | 3 = 0x1B        */
    gdt_set_gate(3, 0, 0, 0xF2, 0x00);

    /* User code (64-bit): DPL=3, present, executable, readable — GDT[4] = selector 0x23
     * SYSRET: CS = (STAR[63:48] + 16) | 3 = (0x10 + 16) | 3 = 0x23      */
    gdt_set_gate(4, 0, 0, 0xFA, 0x20);

    /* Initialize TSS */
    __builtin_memset(&kernel_tss, 0, sizeof(struct tss));
    kernel_tss.rsp0 = kernel_stack_top;
    kernel_tss.iopb_offset = sizeof(struct tss);

    /* TSS descriptor (entries 5-6, 16 bytes) */
    tss_set_gate(5, (uint64_t)&kernel_tss, sizeof(struct tss) - 1);

    gdt_flush((uint64_t)&gdtp);

    /* Load TSS selector (selector = index 5 * 8 = 0x28) */
    asm volatile("ltr %%ax" : : "a"((uint16_t)0x28));
}

void tss_update_rsp0(uint64_t rsp0) {
    kernel_tss.rsp0 = rsp0;
}
