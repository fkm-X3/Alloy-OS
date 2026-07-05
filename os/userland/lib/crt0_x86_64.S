.section .text
.global _start

_start:
    // Clear frame pointer for backtrace termination.
    xor  %rbp, %rbp

    // Call C++ static initializers
    call __alloy_init_cpp

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
