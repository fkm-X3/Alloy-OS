#include "boot/types.h"

__attribute__((aligned(2048)))
static const uint64_t exception_vectors[256] = {0};

void exception_handler_el1() {
    while (1) {
        asm volatile("wfi");
    }
}

void irq_handler_el1() {
}

void init_idt() {
    uint64_t vbar = (uint64_t)&exception_vectors;
    asm volatile("msr vbar_el1, %0" : : "r"(vbar));

    asm volatile("msr daifclr, #0b0011");
}