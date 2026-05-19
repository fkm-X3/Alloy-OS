// ARM64 (aarch64) IDT/Exception vector implementation (minimal)
// ARM64 uses VBAR_EL1 (Vector Base Address Register) for exception handling

#include "boot/types.h"

// ARM64 Exception handling:
// - Uses exception vectors at VBAR_EL1 (must be 2KB aligned)
// - Four vector groups: EL0t, EL1t, EL1h, EL0_64 (synchronous, IRQ, FIQ, SError)
// - Each vector is 128 bytes (32 instructions)
// - GIC (Generic Interrupt Controller) replaces PIC/APIC

// Exception vector table (must be 2KB aligned)
__attribute__((aligned(2048)))
static const uint64_t exception_vectors[256] = {0};

// Minimal exception handler
extern "C" void exception_handler_el1() {
    // Placeholder: Handle EL1 exceptions
    // Read ESR_EL1 for exception syndrome
    // Read FAR_EL1 for fault address
    while (1) {
        asm volatile("wfi");
    }
}

// IRQ handler
extern "C" void irq_handler_el1() {
    // Placeholder: Handle EL1 IRQ
    // Read ICC_IAR1_EL1 for interrupt ID
    // Write ICC_EOIR1_EL1 for EOI
}

extern "C" void init_idt() {
    // Set VBAR_EL1 to exception vector table
    uint64_t vbar = (uint64_t)&exception_vectors;
    asm volatile("msr vbar_el1, %0" : : "r"(vbar));
    
    // Enable interrupts at CPU level
    asm volatile("msr daifclr, #0b0011");
}
