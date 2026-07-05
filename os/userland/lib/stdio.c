#include "alloy_syscall.h"
#ifdef __x86_64__
#include "alloy_syscall_x86_64.h"
#define SYSCALL_FN syscall_x86_64
#else
#define SYSCALL_FN syscall
#endif

int write(int fd, const void *buf, int len) {
    return (int)SYSCALL_FN(SYS_WRITE, fd, (long)buf, len, 0, 0);
}

int puts(const char *s) {
    int len = 0;
    while (s[len]) len++;
    if (write(1, s, len) < 0)
        return -1;
    if (write(1, "\n", 1) < 0)
        return -1;
    return len + 1;
}
