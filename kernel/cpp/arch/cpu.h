#ifndef ALLOY_CPU_H
#define ALLOY_CPU_H

#include "../boot/types.h"

// Architecture-specific CPU feature flags

#ifdef ARCH_I686
// i686 CPUID feature flags (from EDX of CPUID leaf 1)
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

#elif defined(ARCH_X86_64)
// x86_64 CPUID feature flags (same as i686, plus additional)
#define CPU_FEATURE_FPU     (1 << 0)
#define CPU_FEATURE_SSE     (1 << 25)
#define CPU_FEATURE_SSE2    (1 << 26)
#define CPU_FEATURE_SSE3    (1 << 0)   // ECX bit 0
#define CPU_FEATURE_SSSE3   (1 << 9)   // ECX bit 9
#define CPU_FEATURE_SSE41   (1 << 19)  // ECX bit 19
#define CPU_FEATURE_SSE42   (1 << 20)  // ECX bit 20
#define CPU_FEATURE_AVX     (1 << 28)  // ECX bit 28
#define CPU_FEATURE_AES     (1 << 25)  // ECX bit 25

#elif defined(ARCH_AARCH64)
// ARM64 feature flags (from ID_AA64ISAR0_EL1, etc.)
#define CPU_FEATURE_AES     (1 << 0)
#define CPU_FEATURE_SHA1    (1 << 1)
#define CPU_FEATURE_SHA2    (1 << 2)
#define CPU_FEATURE_CRC32   (1 << 3)
#define CPU_FEATURE_ATOMICS (1 << 4)
#define CPU_FEATURE_RDM     (1 << 5)
#define CPU_FEATURE_SHA512  (1 << 6)
#define CPU_FEATURE_DP      (1 << 7)

#else
// Default to i686 for backward compatibility
#define CPU_FEATURE_FPU     (1 << 0)
#define CPU_FEATURE_TSC     (1 << 4)
#define CPU_FEATURE_MSR     (1 << 5)
#define CPU_FEATURE_APIC    (1 << 9)
#define CPU_FEATURE_SSE     (1 << 25)
#define CPU_FEATURE_SSE2    (1 << 26)
#endif

// CPU information structure
struct cpu_info {
    char vendor[16];        // CPU vendor string
    uint32_t features;      // Feature flags
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
