#ifndef _ALLOY_SYSCALL_X86_64_H
#define _ALLOY_SYSCALL_X86_64_H

#ifdef __cplusplus
extern "C" {
#endif

static inline long syscall_x86_64(long n, long a1, long a2, long a3,
                                   long a4, long a5) {
    unsigned long ret;
    register long r10 asm("r10") = a4;
    register long r8  asm("r8")  = a5;
    asm volatile (
        "syscall"
        : "=a" (ret)
        : "a" (n), "D" (a1), "S" (a2), "d" (a3),
          "r" (r10), "r" (r8)
        : "rcx", "r11", "memory"
    );
    return ret;
}

#ifdef __cplusplus
}
#endif

#endif
