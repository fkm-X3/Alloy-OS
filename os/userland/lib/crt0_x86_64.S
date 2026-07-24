.section .rodata
.debug_s: .byte 'S'
.debug_i: .byte 'I'

.section .text
.global _start

_start:
    // Debug: sys_write(1, "S", 1) to confirm _start reached
    mov  $6, %rax          // SYS_WRITE
    mov  $1, %rdi          // fd = stdout
    lea  .debug_s(%rip), %rsi  // buf
    mov  $1, %rdx          // len
    syscall

    // Clear frame pointer for backtrace termination.
    xor  %rbp, %rbp

    // Call C++ static initializers
    call __alloy_init_cpp

    // Debug: sys_write(1, "I", 1) to confirm __alloy_init_cpp returned
    mov  $6, %rax          // SYS_WRITE
    mov  $1, %rdi          // fd = stdout
    lea  .debug_i(%rip), %rsi  // buf
    mov  $1, %rdx          // len
    syscall

    // Push argc, argv (kernel passes them on stack)
    // Currently no real arguments — just call main(0, 0)
    xor  %rdi, %rdi   // argc = 0
    xor  %rsi, %rsi   // argv = NULL
    call main

    // main() returned — call finalizers and exit
    mov  %rax, %rbx   // save return value

    call __alloy_fini_cpp

    mov  %rbx, %rdi   // status = main's return value
    call _exit
    // _exit never returns
