#include "alloy_syscall.h"
#ifdef __x86_64__
#include "alloy_syscall_x86_64.h"
#define SYSCALL_FN syscall_x86_64
#elif defined(__aarch64__)
#include "alloy_syscall_aarch64.h"
#define SYSCALL_FN syscall_aarch64
#else
#define SYSCALL_FN syscall
#endif

void _exit(int status) {
    SYSCALL_FN(SYS_EXIT, status, 0, 0, 0, 0);
    __builtin_unreachable();
}

void *brk(void *addr) {
    return (void *)SYSCALL_FN(SYS_BRK, (long)addr, 0, 0, 0, 0);
}

void *sbrk(int incr) {
    void *current = brk(0);
    if (incr == 0)
        return current;
    void *new = (void *)((char *)current + incr);
    void *result = brk(new);
    if (result == (void *)-1)
        return (void *)-1;
    return current;
}
