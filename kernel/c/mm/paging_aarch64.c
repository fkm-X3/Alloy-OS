#include "paging.h"
#include "pmm.h"
#include "../drivers/serial.h"
#include <stdbool.h>

// Minimal aarch64 paging stubs.
// ARM64 uses translation tables configured via TTBR0_EL1/TTBR1_EL1,
// MAIR_EL1, TCR_EL1. This provides FFI-compatible stubs for the Rust kernel.

static uintptr_t kernel_page_dir_phys = 0;

// 4KB aligned page for first-level translation table (L0)
static uint64_t kernel_tt_l0[512] __attribute__((aligned(4096)));

void paging_init() {
    serial_print("Paging: Initializing ARM64 translation tables\n");

    // Zero out L0 table
    for (int i = 0; i < 512; i++) {
        kernel_tt_l0[i] = 0;
    }

    // Identity-map first 2GB using L0 block entries (512 x 1GB blocks)
    // This is a minimal identity map so the kernel can run with MMU on
    for (int i = 0; i < 2; i++) {
        // Block descriptor: valid, block, Device memory, RW, EL1
        uint64_t block_addr = (uint64_t)i << 30;  // 1GB aligned
        kernel_tt_l0[i] = block_addr | 0xC01;     // Valid | Block | AF | Device | RW
    }

    kernel_page_dir_phys = (uintptr_t)&kernel_tt_l0[0];
}

void paging_enable() {
    serial_print("Paging: Enabling MMU\n");
    // MMU already enabled in boot_aarch64.S
    // Set TTBR0_EL1 to point to L0 table
    uint64_t ttbr0 = (uint64_t)(uintptr_t)kernel_tt_l0;
    asm volatile("msr ttbr0_el1, %0" : : "r"(ttbr0));
    asm volatile("isb");
}

uintptr_t paging_create_directory_phys() {
    return kernel_page_dir_phys;
}

bool paging_switch_to_directory(uintptr_t pd_phys) {
    if (pd_phys == 0) return false;
    asm volatile("msr ttbr0_el1, %0" : : "r"((uint64_t)pd_phys));
    asm volatile("isb; tlbi vmalle1; dsb sy; isb");
    return true;
}

uintptr_t paging_get_kernel_directory_phys() {
    return kernel_page_dir_phys;
}

uintptr_t paging_get_physical_address(uintptr_t virt_addr) {
    return virt_addr;
}

void paging_destroy_directory(uintptr_t pd_phys) {
    (void)pd_phys;
}

uintptr_t paging_clone_directory(uintptr_t pd_phys) {
    return pd_phys;
}

uintptr_t paging_fork_directory(uintptr_t pd_phys) {
    return pd_phys;
}

uint8_t paging_handle_cow_fault(uintptr_t fault_addr) {
    (void)fault_addr;
    return 0;
}

bool paging_map_page_in_pd(uintptr_t pd_phys, uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags) {
    (void)pd_phys;
    (void)virt_addr;
    (void)phys_addr;
    (void)flags;
    return true;
}

void* paging_temp_map_frame(uintptr_t phys_addr) {
    return (void*)(uintptr_t)phys_addr;
}

void paging_temp_unmap_frame() {
}
