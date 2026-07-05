// hello_x86_64.c -- Test MSR-based syscall instruction from userland
// Build: x86_64-elf-gcc -ffreestanding -nostdlib -m64 -I lib -c tests/hello_x86_64.c
// Link:  x86_64-elf-ld -m elf_x86_64 -T userland.ld -o hello_x86_64 crt0_x86_64.o hello_x86_64.o
// Expected serial: "Hello from x86_64 syscall!"

#include <stdint.h>

#define SYS_EXIT   0
#define SYS_WRITE  6

#include "alloy_syscall_x86_64.h"

// Override _exit from stdlib -- use MSR syscall instead of int 0x80
void _exit(int status) {
    syscall_x86_64(SYS_EXIT, status, 0, 0, 0, 0);
    __builtin_unreachable();
}

int main(void) {
    char msg[] = "Hello from x86_64 syscall!\n";
    syscall_x86_64(SYS_WRITE, 1, (uintptr_t)msg, 27, 0, 0);
    return 0;
}
