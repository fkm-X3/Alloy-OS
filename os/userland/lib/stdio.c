#include "alloy_syscall.h"

int write(int fd, const void *buf, int len) {
    return syscall(SYS_WRITE, fd, (int)buf, len, 0, 0);
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
