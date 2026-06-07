#include "alloy_syscall.h"

void _exit(int status) {
    syscall(SYS_EXIT, status, 0, 0, 0, 0);
    __builtin_unreachable();
}

void *brk(void *addr) {
    return (void *)syscall(SYS_BRK, (int)addr, 0, 0, 0, 0);
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
