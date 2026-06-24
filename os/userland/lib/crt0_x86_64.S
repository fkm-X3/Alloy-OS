.section .text
.global _start

_start:
    // Kernel sets up stack and jumps to _start.
    // Clear frame pointer for backtrace termination.
    xor  %ebp, %ebp

    // Push argc, argv, envp (kernel passes them on stack)
    // Currently no real arguments — just call main(0, 0)
    push $0      // argv
    push $0      // argc
    call main

    // main() returned — exit with its return value
    push %rax
    call _exit
    // _exit never returns