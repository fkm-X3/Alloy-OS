#ifndef ARCH_CONTEXT_H
#define ARCH_CONTEXT_H

#include "boot/types.h"

// CPU context structure for task switching
// Architecture-specific register layout

#ifdef ARCH_I686
// i686 (32-bit x86) context
struct cpu_context {
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
    uint32_t esi;
    uint32_t edi;
    uint32_t ebp;
    uint32_t esp;
    uint32_t eip;
    uint32_t cs;
    uint32_t ds;
    uint32_t es;
    uint32_t fs;
    uint32_t gs;
    uint32_t ss;
    uint32_t eflags;
    uint32_t cr3;
};
#elif defined(ARCH_X86_64)
// x86_64 (64-bit) context
struct cpu_context {
    uint64_t rax;
    uint64_t rbx;
    uint64_t rcx;
    uint64_t rdx;
    uint64_t rsi;
    uint64_t rdi;
    uint64_t rbp;
    uint64_t rsp;
    uint64_t r8;
    uint64_t r9;
    uint64_t r10;
    uint64_t r11;
    uint64_t r12;
    uint64_t r13;
    uint64_t r14;
    uint64_t r15;
    uint64_t rip;
    uint64_t cs;
    uint64_t ds;
    uint64_t es;
    uint64_t fs;
    uint64_t gs;
    uint64_t ss;
    uint64_t rflags;
    uint64_t cr3;
};
#elif defined(ARCH_AARCH64)
// ARM64 (aarch64) context
struct cpu_context {
    uint64_t x19;
    uint64_t x20;
    uint64_t x21;
    uint64_t x22;
    uint64_t x23;
    uint64_t x24;
    uint64_t x25;
    uint64_t x26;
    uint64_t x27;
    uint64_t x28;
    uint64_t fp;   // x29
    uint64_t lr;   // x30
    uint64_t sp;   // SP_EL1
    uint64_t elr;  // ELR_EL1
    uint64_t spsr; // SPSR_EL1
    uint64_t ttbr0; // TTBR0_EL1
};
#else
// Default to i686 for backward compatibility
struct cpu_context {
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
    uint32_t esi;
    uint32_t edi;
    uint32_t ebp;
    uint32_t esp;
    uint32_t eip;
    uint32_t cs;
    uint32_t ds;
    uint32_t es;
    uint32_t fs;
    uint32_t gs;
    uint32_t ss;
    uint32_t eflags;
    uint32_t cr3;
};
#endif

typedef struct cpu_context cpu_context;

// Context switch function (implemented in assembly)
#ifdef __cplusplus
extern "C" {
#endif

void context_switch(cpu_context* old_ctx, cpu_context* new_ctx);

#ifdef __cplusplus
}
#endif

#endif // ARCH_CONTEXT_H
