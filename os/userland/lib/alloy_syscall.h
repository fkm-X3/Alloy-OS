#ifndef _ALLOY_SYSCALL_H
#define _ALLOY_SYSCALL_H

// Syscall numbers (match kernel/rust/src/syscall/table.rs)
#define SYS_EXIT    0
#define SYS_YIELD   1
#define SYS_GETPID  2
#define SYS_SLEEP   3
#define SYS_OPEN    4
#define SYS_READ    5
#define SYS_WRITE   6
#define SYS_CLOSE   7
#define SYS_DUP     8
#define SYS_LSEEK   9
#define SYS_PIPE    10
#define SYS_EXECVE  11
#define SYS_SOCKET  12
#define SYS_BIND    13
#define SYS_LISTEN  14
#define SYS_ACCEPT  15
#define SYS_CONNECT 16
#define SYS_CLOSE_SOCKET 17
#define SYS_HAS_PENDING_CONNECTIONS 18
#define SYS_BRK     19
#define SYS_FORK    20
#define SYS_CLONE   21
#define SYS_WAITPID 22
#define SYS_SOCKET_READ  23
#define SYS_SOCKET_WRITE 24
#define SYS_ALLOC_SHM   25
#define SYS_SHM_USER_VADDR 26
#define SYS_MMAP        27
#define SYS_GETTIMEOFDAY 28

// INT 0x80 calling convention (x86 only):
//   eax = syscall number
//   ebx = arg0, ecx = arg1, edx = arg2, esi = arg3, edi = arg4
//   return in eax

#if !defined(__aarch64__)
static inline int syscall(int num, int arg0, int arg1, int arg2, int arg3, int arg4) {
    int ret;
    asm volatile (
        "int $0x80"
        : "=a" (ret)
        : "a" (num), "b" (arg0), "c" (arg1), "d" (arg2), "S" (arg3), "D" (arg4)
        : "memory"
    );
    return ret;
}
#endif

#endif
