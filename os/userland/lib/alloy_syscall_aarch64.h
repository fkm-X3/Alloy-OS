#ifndef _ALLOY_SYSCALL_AARCH64_H
#define _ALLOY_SYSCALL_AARCH64_H

// aarch64 SVC syscall ABI (matches boot/boot_aarch64.S sync_lower_64 handler)
//   x8 = syscall number
//   x1 = arg0, x2 = arg1, x3 = arg2, x4 = arg3, x5 = arg4
//   return in x0

static inline long syscall_aarch64(long n, long a1, long a2, long a3,
                                    long a4, long a5) {
    register long r8 asm("x8") = n;
    register long r1 asm("x1") = a1;
    register long r2 asm("x2") = a2;
    register long r3 asm("x3") = a3;
    register long r4 asm("x4") = a4;
    register long r5 asm("x5") = a5;
    register long ret asm("x0");
    asm volatile (
        "svc #0"
        : "=r" (ret)
        : "r" (r8), "r" (r1), "r" (r2), "r" (r3), "r" (r4), "r" (r5)
        : "memory"
    );
    return ret;
}

#endif
