#ifndef ALLOY_CPU_H
#define ALLOY_CPU_H

#include "../boot/types.h"

// CPU feature flags (from CPUID)
#define CPU_FEATURE_FPU     (1 << 0)   // Floating Point Unit
#define CPU_FEATURE_VME     (1 << 1)   // Virtual Mode Extensions
#define CPU_FEATURE_DE      (1 << 2)   // Debugging Extensions
#define CPU_FEATURE_PSE     (1 << 3)   // Page Size Extension
#define CPU_FEATURE_TSC     (1 << 4)   // Time Stamp Counter
#define CPU_FEATURE_MSR     (1 << 5)   // Model Specific Registers
#define CPU_FEATURE_PAE     (1 << 6)   // Physical Address Extension
#define CPU_FEATURE_MCE     (1 << 7)   // Machine Check Exception
#define CPU_FEATURE_CX8     (1 << 8)   // CMPXCHG8 instruction
#define CPU_FEATURE_APIC    (1 << 9)   // On-chip APIC
#define CPU_FEATURE_SEP     (1 << 11)  // SYSENTER/SYSEXIT
#define CPU_FEATURE_MTRR    (1 << 12)  // Memory Type Range Registers
#define CPU_FEATURE_PGE     (1 << 13)  // Page Global Enable
#define CPU_FEATURE_MCA     (1 << 14)  // Machine Check Architecture
#define CPU_FEATURE_CMOV    (1 << 15)  // Conditional Move
#define CPU_FEATURE_PAT     (1 << 16)  // Page Attribute Table
#define CPU_FEATURE_PSE36   (1 << 17)  // 36-bit Page Size Extension
#define CPU_FEATURE_PSN     (1 << 18)  // Processor Serial Number
#define CPU_FEATURE_CLFLUSH (1 << 19)  // CLFLUSH instruction
#define CPU_FEATURE_MMX     (1 << 23)  // MMX instructions
#define CPU_FEATURE_FXSR    (1 << 24)  // FXSAVE/FXRSTOR
#define CPU_FEATURE_SSE     (1 << 25)  // SSE instructions
#define CPU_FEATURE_SSE2    (1 << 26)  // SSE2 instructions

// CPU information structure
struct cpu_info {
    char vendor[13];        // CPU vendor string (e.g., "GenuineIntel")
    uint32_t features;      // Feature flags from CPUID
    uint32_t family;        // Processor family
    uint32_t model;         // Processor model
    uint32_t stepping;      // Stepping ID
};

// CPU detection and information functions
void cpu_detect(struct cpu_info* info);
void cpu_get_vendor(char* vendor);
uint32_t cpu_get_features();
void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping);

#endif // ALLOY_CPU_H
