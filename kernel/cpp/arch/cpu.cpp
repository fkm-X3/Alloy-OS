#include "cpu.h"

// Helper function to execute CPUID instruction
static inline void cpuid(uint32_t code, uint32_t* eax, uint32_t* ebx, uint32_t* ecx, uint32_t* edx) {
    asm volatile("cpuid"
                 : "=a"(*eax), "=b"(*ebx), "=c"(*ecx), "=d"(*edx)
                 : "a"(code)
                 : "memory");
}

// Get CPU vendor string
void cpu_get_vendor(char* vendor) {
    uint32_t eax, ebx, ecx, edx;
    
    // CPUID function 0 returns vendor string
    cpuid(0, &eax, &ebx, &ecx, &edx);
    
    // Vendor string is returned in EBX, EDX, ECX (in that order)
    *((uint32_t*)(vendor + 0)) = ebx;
    *((uint32_t*)(vendor + 4)) = edx;
    *((uint32_t*)(vendor + 8)) = ecx;
    vendor[12] = '\0';
}

// Get CPU features
uint32_t cpu_get_features() {
    uint32_t eax, ebx, ecx, edx;
    
    // CPUID function 1 returns feature flags in EDX
    cpuid(1, &eax, &ebx, &ecx, &edx);
    
    return edx;
}

// Get CPU model information
void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping) {
    uint32_t eax, ebx, ecx, edx;
    
    // CPUID function 1 returns processor info in EAX
    cpuid(1, &eax, &ebx, &ecx, &edx);
    
    *stepping = eax & 0xF;                           // Bits 0-3
    *model = (eax >> 4) & 0xF;                       // Bits 4-7
    *family = (eax >> 8) & 0xF;                      // Bits 8-11
    
    // Extended model and family (for newer CPUs)
    uint32_t ext_model = (eax >> 16) & 0xF;          // Bits 16-19
    uint32_t ext_family = (eax >> 20) & 0xFF;        // Bits 20-27
    
    // Calculate display model and family
    if (*family == 0xF) {
        *family += ext_family;
    }
    if (*family == 0x6 || *family == 0xF) {
        *model += (ext_model << 4);
    }
}

// Detect CPU and populate info structure
void cpu_detect(struct cpu_info* info) {
    if (!info) {
        return;
    }
    
    cpu_get_vendor(info->vendor);
    info->features = cpu_get_features();
    cpu_get_model_info(&info->family, &info->model, &info->stepping);
}

// C FFI wrappers for Rust
extern "C" void cpu_get_vendor_ffi(char* vendor) {
    cpu_get_vendor(vendor);
}

extern "C" uint32_t cpu_get_features_ffi() {
    return cpu_get_features();
}

extern "C" void cpu_get_model_info_ffi(uint32_t* family, uint32_t* model, uint32_t* stepping) {
    cpu_get_model_info(family, model, stepping);
}
