// ARM64 (aarch64) syscall implementation (minimal)
// ARM64 uses SVC (Supervisor Call) instruction for syscalls

#include "boot/types.h"

// ARM64 syscall convention:
// - SVC #0 triggers exception to EL1
// - syscall number in x8
// - arguments in x0-x5
// - return value in x0

extern "C" void syscall_handler() {
    // Placeholder: ARM64 syscall handler
    // Read ESR_EL1 to get syscall number
    // Dispatch to appropriate handler
}

// Syscall invocation helper
extern "C" uint64_t do_syscall(uint64_t num, uint64_t arg0, uint64_t arg1, 
                                uint64_t arg2, uint64_t arg3, uint64_t arg4) {
    uint64_t ret;
    asm volatile(
        "svc #0"
        : "=r"(ret)
        : "r"(num), "r"(arg0), "r"(arg1), "r"(arg2), "r"(arg3), "r"(arg4)
        : "memory"
    );
    return ret;
}
