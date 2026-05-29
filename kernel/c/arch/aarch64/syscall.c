#include "boot/types.h"

void syscall_handler() {
}

uint64_t do_syscall(uint64_t num, uint64_t arg0, uint64_t arg1,
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