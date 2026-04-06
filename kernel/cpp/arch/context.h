#ifndef ARCH_CONTEXT_H
#define ARCH_CONTEXT_H

#include "../../boot/types.h"

// CPU context structure for task switching
// Holds all registers that need to be saved/restored during context switch
struct cpu_context {
    // General purpose registers
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
    uint32_t esi;
    uint32_t edi;
    uint32_t ebp;
    uint32_t esp;
    
    // Instruction pointer
    uint32_t eip;
    
    // Segment registers
    uint32_t cs;
    uint32_t ds;
    uint32_t es;
    uint32_t fs;
    uint32_t gs;
    uint32_t ss;
    
    // EFLAGS register
    uint32_t eflags;
};

// Context switch function (implemented in assembly)
// Saves current context to old_ctx and loads new context from new_ctx
extern "C" void context_switch(cpu_context* old_ctx, cpu_context* new_ctx);

#endif // ARCH_CONTEXT_H
