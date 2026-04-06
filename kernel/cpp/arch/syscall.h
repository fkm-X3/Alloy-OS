#ifndef ARCH_SYSCALL_H
#define ARCH_SYSCALL_H

#include "boot/types.h"

// Syscall numbers
#define SYS_EXIT    0
#define SYS_YIELD   1
#define SYS_GETPID  2
#define SYS_SLEEP   3

// Syscall dispatcher (called from assembly stub)
extern "C" uint32_t syscall_dispatcher(uint32_t syscall_no, 
                                       uint32_t arg0,
                                       uint32_t arg1,
                                       uint32_t arg2,
                                       uint32_t arg3,
                                       uint32_t arg4);

// Initialize syscalls (adds INT 0x80 to IDT)
extern "C" void syscall_init();

#endif // ARCH_SYSCALL_H
