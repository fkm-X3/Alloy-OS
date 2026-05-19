// ARM64 (aarch64) GDT implementation
// ARM64 does not use GDT - uses translation tables instead

#include "boot/types.h"

// ARM64 doesn't have a GDT concept like x86.
// Memory protection is handled through translation tables (page tables).
// This function is a no-op for ARM64.

extern "C" void init_gdt() {
    // ARM64: No GDT needed
    // Translation tables handle memory protection
}
