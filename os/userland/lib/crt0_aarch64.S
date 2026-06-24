.section .text
.global _start

_start:
    // Kernel sets up stack and jumps to _start.
    // Clear frame pointer for backtrace termination (x29 is FP on ARM64).
    mov x29, xzr

    // ARM64 calling convention (AAPCS64):
    // x0 = argc, x1 = argv, x2 = envp
    // Currently no real arguments — set them to 0
    mov x0, #0       // argc
    mov x1, #0       // argv
    bl main

    // main() returned — exit with its return value (x0 already contains return value)
    bl _exit
    // _exit never returns