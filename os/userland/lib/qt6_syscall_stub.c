// Minimal syscall() stub for Qt6 build.
// The standard C library's syscall() has signature: long syscall(long number, ...)
// This file is compiled WITHOUT alloy_syscall.h to avoid the static inline conflict.
// The Qt6 objects reference this symbol directly.

long syscall(long number, ...) {
    (void)number;
    return -1;
}
