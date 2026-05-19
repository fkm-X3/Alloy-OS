#include "cpu.h"

#ifdef ARCH_I686
// i686 CPU detection using CPUID

static inline void cpuid(uint32_t code, uint32_t* eax, uint32_t* ebx, uint32_t* ecx, uint32_t* edx) {
    asm volatile("cpuid"
                 : "=a"(*eax), "=b"(*ebx), "=c"(*ecx), "=d"(*edx)
                 : "a"(code)
                 : "memory");
}

void cpu_get_vendor(char* vendor) {
    uint32_t eax, ebx, ecx, edx;
    cpuid(0, &eax, &ebx, &ecx, &edx);
    *((uint32_t*)(vendor + 0)) = ebx;
    *((uint32_t*)(vendor + 4)) = edx;
    *((uint32_t*)(vendor + 8)) = ecx;
    vendor[12] = '\0';
}

uint32_t cpu_get_features() {
    uint32_t eax, ebx, ecx, edx;
    cpuid(1, &eax, &ebx, &ecx, &edx);
    return edx;
}

void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping) {
    uint32_t eax, ebx, ecx, edx;
    cpuid(1, &eax, &ebx, &ecx, &edx);
    *stepping = eax & 0xF;
    *model = (eax >> 4) & 0xF;
    *family = (eax >> 8) & 0xF;
    uint32_t ext_model = (eax >> 16) & 0xF;
    uint32_t ext_family = (eax >> 20) & 0xFF;
    if (*family == 0xF) {
        *family += ext_family;
    }
    if (*family == 0x6 || *family == 0xF) {
        *model += (ext_model << 4);
    }
}

#elif defined(ARCH_X86_64)
// x86_64 CPU detection using CPUID (same as i686)

static inline void cpuid(uint32_t code, uint32_t* eax, uint32_t* ebx, uint32_t* ecx, uint32_t* edx) {
    asm volatile("cpuid"
                 : "=a"(*eax), "=b"(*ebx), "=c"(*ecx), "=d"(*edx)
                 : "a"(code)
                 : "memory");
}

void cpu_get_vendor(char* vendor) {
    uint32_t eax, ebx, ecx, edx;
    cpuid(0, &eax, &ebx, &ecx, &edx);
    *((uint32_t*)(vendor + 0)) = ebx;
    *((uint32_t*)(vendor + 4)) = edx;
    *((uint32_t*)(vendor + 8)) = ecx;
    vendor[12] = '\0';
}

uint32_t cpu_get_features() {
    uint32_t eax, ebx, ecx, edx;
    cpuid(1, &eax, &ebx, &ecx, &edx);
    return edx;
}

void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping) {
    uint32_t eax, ebx, ecx, edx;
    cpuid(1, &eax, &ebx, &ecx, &edx);
    *stepping = eax & 0xF;
    *model = (eax >> 4) & 0xF;
    *family = (eax >> 8) & 0xF;
    uint32_t ext_model = (eax >> 16) & 0xF;
    uint32_t ext_family = (eax >> 20) & 0xFF;
    if (*family == 0xF) {
        *family += ext_family;
    }
    if (*family == 0x6 || *family == 0xF) {
        *model += (ext_model << 4);
    }
}

#elif defined(ARCH_AARCH64)
// ARM64 CPU detection using system registers

void cpu_get_vendor(char* vendor) {
    uint64_t midr;
    asm volatile("mrs %0, midr_el1" : "=r"(midr));
    uint32_t implementer = (midr >> 16) & 0xFF;
    switch (implementer) {
        case 0x41: __builtin_memcpy(vendor, "ARM Limited", 12); break;
        case 0x42: __builtin_memcpy(vendor, "Broadcom", 9); break;
        case 0x43: __builtin_memcpy(vendor, "Cavium", 7); break;
        case 0x4E: __builtin_memcpy(vendor, "NVIDIA", 7); break;
        case 0x51: __builtin_memcpy(vendor, "Qualcomm", 9); break;
        case 0x53: __builtin_memcpy(vendor, "Samsung", 8); break;
        default: __builtin_memcpy(vendor, "Unknown", 8); break;
    }
    vendor[12] = '\0';
}

uint32_t cpu_get_features() {
    uint64_t isar0;
    asm volatile("mrs %0, id_aa64isar0_el1" : "=r"(isar0));
    return (uint32_t)isar0;
}

void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping) {
    uint64_t midr;
    asm volatile("mrs %0, midr_el1" : "=r"(midr));
    *family = ((midr >> 20) & 0xF);  // Variant
    *model = ((midr >> 4) & 0xFFF);  // PartNum
    *stepping = (midr & 0xF);        // Revision
}

#else
// Default (i686) for backward compatibility

static inline void cpuid(uint32_t code, uint32_t* eax, uint32_t* ebx, uint32_t* ecx, uint32_t* edx) {
    asm volatile("cpuid"
                 : "=a"(*eax), "=b"(*ebx), "=c"(*ecx), "=d"(*edx)
                 : "a"(code)
                 : "memory");
}

void cpu_get_vendor(char* vendor) {
    uint32_t eax, ebx, ecx, edx;
    cpuid(0, &eax, &ebx, &ecx, &edx);
    *((uint32_t*)(vendor + 0)) = ebx;
    *((uint32_t*)(vendor + 4)) = edx;
    *((uint32_t*)(vendor + 8)) = ecx;
    vendor[12] = '\0';
}

uint32_t cpu_get_features() {
    uint32_t eax, ebx, ecx, edx;
    cpuid(1, &eax, &ebx, &ecx, &edx);
    return edx;
}

void cpu_get_model_info(uint32_t* family, uint32_t* model, uint32_t* stepping) {
    uint32_t eax, ebx, ecx, edx;
    cpuid(1, &eax, &ebx, &ecx, &edx);
    *stepping = eax & 0xF;
    *model = (eax >> 4) & 0xF;
    *family = (eax >> 8) & 0xF;
}
#endif

// Common implementation
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
