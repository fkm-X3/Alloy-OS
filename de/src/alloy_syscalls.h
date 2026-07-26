#ifndef ALLOY_SYSCALLS_H
#define ALLOY_SYSCALLS_H

// Alloy OS syscall wrappers for C++ userland code.
// Uses the x86_64 syscall instruction directly — no libc dependency.

#include <cstdint>

#ifdef __x86_64__

static inline long alloy_syscall(long n, long a1, long a2, long a3,
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

#else
#error "alloy_syscalls.h: unsupported architecture"
#endif

// Syscall numbers (must match kernel)
#define ALLOY_SYS_EXIT           0
#define ALLOY_SYS_FORK          20
#define ALLOY_SYS_YIELD          1
#define ALLOY_SYS_GETPID         2
#define ALLOY_SYS_SLEEP          3
#define ALLOY_SYS_OPEN           4
#define ALLOY_SYS_READ           5
#define ALLOY_SYS_WRITE          6
#define ALLOY_SYS_CLOSE          7
#define ALLOY_SYS_DUP            8
#define ALLOY_SYS_LSEEK          9
#define ALLOY_SYS_PIPE          10
#define ALLOY_SYS_EXECVE        11
#define ALLOY_SYS_WAITPID       22
#define ALLOY_SYS_DUP2          29
#define ALLOY_SYS_KILL          30

// Convenience wrappers

static inline int alloy_fork() {
    return (int)alloy_syscall(ALLOY_SYS_FORK, 0, 0, 0, 0, 0);
}

static inline int alloy_execve(const char *path) {
    return (int)alloy_syscall(ALLOY_SYS_EXECVE, (long)path, 0, 0, 0, 0);
}

static inline int alloy_pipe(int fds[2]) {
    return (int)alloy_syscall(ALLOY_SYS_PIPE, (long)fds, 0, 0, 0, 0);
}

static inline int alloy_dup2(int oldfd, int newfd) {
    return (int)alloy_syscall(ALLOY_SYS_DUP2, oldfd, newfd, 0, 0, 0);
}

static inline int alloy_close(int fd) {
    return (int)alloy_syscall(ALLOY_SYS_CLOSE, fd, 0, 0, 0, 0);
}

static inline int alloy_read(int fd, void *buf, unsigned int len) {
    return (int)alloy_syscall(ALLOY_SYS_READ, fd, (long)buf, len, 0, 0);
}

static inline int alloy_write(int fd, const void *buf, unsigned int len) {
    return (int)alloy_syscall(ALLOY_SYS_WRITE, fd, (long)buf, len, 0, 0);
}

static inline int alloy_kill(int pid, int sig) {
    return (int)alloy_syscall(ALLOY_SYS_KILL, pid, sig, 0, 0, 0);
}

static inline int alloy_waitpid(int pid, int options) {
    return (int)alloy_syscall(ALLOY_SYS_WAITPID, pid, options, 0, 0, 0);
}

static inline int alloy_getpid() {
    return (int)alloy_syscall(ALLOY_SYS_GETPID, 0, 0, 0, 0, 0);
}

#endif // ALLOY_SYSCALLS_H
