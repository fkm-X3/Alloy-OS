// ARM64 (aarch64) context switch implementation (minimal)
// Saves/restores callee-saved registers (x19-x30, SP, ELR, SPSR)

#include "boot/types.h"
#include "../context.h"

// ARM64 context switch
// Callee-saved registers: x19-x30, SP_EL1
// Special registers: ELR_EL1, SPSR_EL1, TTBR0_EL1
extern "C" void context_switch(cpu_context* old_ctx, cpu_context* new_ctx) {
    // Placeholder: ARM64 context switch not fully implemented
    // Need assembly to properly save/restore all registers
    // This is a minimal C stub for compilation
    
    if (old_ctx) {
        // Save current stack pointer
        asm volatile("mov %0, sp" : "=r"(old_ctx->esp));
    }
    
    if (new_ctx) {
        // Restore stack pointer
        asm volatile("mov sp, %0" : : "r"(new_ctx->esp));
    }
}
