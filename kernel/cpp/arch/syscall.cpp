#include "arch/syscall.h"

// Forward declaration for serial_print
extern "C" void serial_print(const char* str);

// Forward declarations for Rust syscall handlers
extern "C" uint32_t rust_sys_exit(uint32_t code);
extern "C" uint32_t rust_sys_yield();
extern "C" uint32_t rust_sys_getpid();
extern "C" uint32_t rust_sys_sleep(uint32_t ms);

// Syscall dispatcher - routes syscalls to handlers
extern "C" uint32_t syscall_dispatcher(uint32_t syscall_no, 
                                       uint32_t arg0,
                                       uint32_t arg1,
                                       uint32_t arg2,
                                       uint32_t arg3,
                                       uint32_t arg4) {
    (void)arg1; // Unused for now
    (void)arg2;
    (void)arg3;
    (void)arg4;
    
    uint32_t result = 0;
    
    switch (syscall_no) {
        case SYS_EXIT:
            result = rust_sys_exit(arg0);
            break;
        case SYS_YIELD:
            result = rust_sys_yield();
            break;
        case SYS_GETPID:
            result = rust_sys_getpid();
            break;
        case SYS_SLEEP:
            result = rust_sys_sleep(arg0);
            break;
        default:
            serial_print("[Syscall] Unknown syscall number\n");
            result = (uint32_t)-1; // Error
            break;
    }
    
    return result;
}

// Initialize syscall system
extern "C" void syscall_entry();  // From syscall_entry.asm

extern "C" void syscall_init() {
    serial_print("[Syscall] Initializing system call interface\n");
    
    // INT 0x80 = syscall entry point
    // We need to add this to IDT
    // This will be called from init_idt() after we export it
    
    serial_print("[Syscall] System calls ready (INT 0x80)\n");
}
