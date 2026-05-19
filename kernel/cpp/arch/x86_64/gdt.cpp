// x86_64 GDT implementation (placeholder)
// TODO: Implement 64-bit GDT with TSS for proper long mode support

#include "boot/types.h"

// This file is a placeholder. The OS currently runs in i686 (32-bit) mode.
// For x86_64 support, we need:
// 1. 64-bit GDT entries (same structure, different access/granularity)
// 2. TSS (Task State Segment) for hardware task switching and IST
// 3. Proper segment selectors for long mode (most segments are flat/null)

extern "C" void init_gdt() {
    // Placeholder: x86_64 GDT not yet implemented
    // The current implementation uses i686 (32-bit) GDT
}
