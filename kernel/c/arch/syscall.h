#ifndef ARCH_SYSCALL_H
#define ARCH_SYSCALL_H

#include "boot/types.h"

// Syscall numbers
#define SYS_EXIT    0
#define SYS_FORK    20
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
#define SYS_SOCKET    12
#define SYS_BIND      13
#define SYS_LISTEN    14
#define SYS_ACCEPT    15
#define SYS_CONNECT   16
#define SYS_CLOSE_SOCKET 17
#define SYS_HAS_PENDING_CONNECTIONS 18
#define SYS_BRK        19
#define SYS_FORK       20
#define SYS_CLONE      21
#define SYS_WAITPID    22

// Syscall dispatcher (called from assembly stub)
#ifdef __cplusplus
extern "C" {
#endif

uint32_t syscall_dispatcher(uint32_t syscall_no, 
                            uint32_t arg0,
                            uint32_t arg1,
                            uint32_t arg2,
                            uint32_t arg3,
                            uint32_t arg4,
                            uint32_t* int80_frame);

void syscall_init();

#ifdef __cplusplus
}
#endif

#endif // ARCH_SYSCALL_H
