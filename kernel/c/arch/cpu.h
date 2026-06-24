#ifndef ALLOY_CPU_H
#define ALLOY_CPU_H

#include "../boot/types.h"

// Architecture-specific CPU feature flags

#ifdef ARCH_I686
// x86_64 CPUID feature flags
#define CPUID_FEATURE_FPU         (1 << 0)
#define CPUID_FEATURE_VME         (1 << 1)
#define CPUID_FEATURE_DE          (1 << 2)
#define CPUID_FEATURE_PSE         (1 << 3)
#define CPUID_FEATURE_TSC         (1 << 4)
#define CPUID_FEATURE_MSR         (1 << 5)
#define CPUID_FEATURE_PAE         (1 << 6)
#define CPUID_FEATURE_MCE         (1 << 7)
#define CPUID_FEATURE_CX8         (1 << 8)
#define CPUID_FEATURE_APIC        (1 << 9)
#define CPUID_FEATURE_SEP         (1 << 11)
#define CPUID_FEATURE_MTRR        (1 << 12)
#define CPUID_FEATURE_PGE         (1 << 13)
#define CPUID_FEATURE_MCA         (1 << 14)
#define CPUID_FEATURE_CMOV        (1 << 15)
#define CPUID_FEATURE_PAT         (1 << 16)
#define CPUID_FEATURE_PSE36       (1 << 17)
#define CPUID_FEATURE_PSN         (1 << 18)
#define CPUID_FEATURE_CLFSH       (1 << 19)
#define CPUID_FEATURE_DS          (1 << 21)
#define CPUID_FEATURE_ACPI        (1 << 22)
#define CPUID_FEATURE_MMX         (1 << 23)
#define CPUID_FEATURE_FXSR        (1 << 24)
#define CPUID_FEATURE_SSE         (1 << 25)
#define CPUID_FEATURE_SSE2        (1 << 26)
#define CPUID_FEATURE_SS          (1 << 27)
#define CPUID_FEATURE_HTT         (1 << 28)
#define CPUID_FEATURE_TM          (1 << 29)
#define CPUID_FEATURE_IA64        (1 << 30)
#define CPUID_FEATURE_PBE         (1 << 31)

// x86_64 additional CPUID feature flags
#define CPUID_X86_64_FEATURE_SYSCALL (1 << 11)
#define CPUID_X86_64_FEATURE_NX      (1 << 20)
#define CPUID_X86_64_FEATURE_LONG    (1 << 29)
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
#ifdef __cplusplus
extern "C" {
#endif

void cpu_detect(struct cpu_info* info);
void cpu_get_vendor(char* vendor);
uint32_t cpu_get_features();
void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping);
void cpu_get_vendor_ffi(char* vendor);
uint32_t cpu_get_features_ffi();
void cpu_get_model_info_ffi(uint32_t* family, uint32_t* model, uint32_t* stepping);

#ifdef __cplusplus
}
#endif

#endif // ALLOY_CPU_H
