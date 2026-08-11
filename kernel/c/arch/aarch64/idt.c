#include "boot/types.h"
#include "../../drivers/timer.h"

// Exception vectors are defined in exception_vectors.S
// This file provides C handlers for the assembly vector stubs

// Forward declarations for C handlers called from assembly
extern void rust_handle_page_fault(uintptr_t addr, uint32_t err_code);

void exception_handler_el1() {
    // Called from sync_handler_el1 for synchronous exceptions
    while (1) {
        asm volatile("wfi");
    }
}

void irq_handler_el1() {
    // Called from irq_handler_el1_asm for IRQs
    // Dispatch to the timer (GIC PPI 30); timer_handler acks and reloads.
    timer_handler();
}

void page_fault_handler(uint64_t far, uint64_t esr) {
    // Handle page faults from userspace
    uint32_t err_code = (uint32_t)(esr & 0xFF);

    // Forward to Rust handler for task termination
    rust_handle_page_fault(far, err_code);
}

void svc_handler(uint64_t num, uint64_t arg0, uint64_t arg1,
                 uint64_t arg2, uint64_t arg3, uint64_t arg4) {
    // System call handler - dispatches to Rust
    // The return value is stored in x0 in the SVC handler
    extern uint32_t syscall_dispatcher(uint32_t syscall_no,
                                       uint32_t arg0, uint32_t arg1,
                                       uint32_t arg2, uint32_t arg3,
                                       uint32_t arg4, uint32_t* frame);

    syscall_dispatcher((uint32_t)num, (uint32_t)arg0, (uint32_t)arg1,
                       (uint32_t)arg2, (uint32_t)arg3, (uint32_t)arg4, 0);
}

// Full kernel exception vector table (exception_vectors.S).
// The boot-time table (boot_aarch64.S) only handles the bootstrap window.
extern uint8_t _exception_vectors;

void init_idt() {
    // Point VBAR_EL1 at the full kernel vector table. Its IRQ handlers
    // save ALL registers (boot_aarch64.S's bootstrap table did not).
    asm volatile("msr vbar_el1, %0" : : "r"(&_exception_vectors));
    asm volatile("isb");

    // Enable IRQs (use DAIF)
    asm volatile("msr daifclr, #0b0010");  // Clear IRQ mask only
}

uint64_t get_system_uptime_ms() {
    return timer_get_uptime_ms_ffi();
}
