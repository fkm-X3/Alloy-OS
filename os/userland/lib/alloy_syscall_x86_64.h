#ifndef _ALLOY_SYSCALL_X86_64_H
#define _ALLOY_SYSCALL_X86_64_H

// x86_64 syscall instruction ABI
//   RAX = syscall number
//   RDI, RSI, RDX, R10, R8, R9 = arguments (up to 6)
//   RCX = return RIP (clobbered by CPU)
//   R11 = RFLAGS (clobbered by CPU)
//   Return value in RAX

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
        : "rcx", "r11", "r9", "memory"
    );
    return ret;
}

#endif
