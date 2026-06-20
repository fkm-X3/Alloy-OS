/* aarch64 (ARM64) boot entry point for Alloy kernel */
/* QEMU loads us at 0x40080000 in EL2 */

.section .boot
.global _start
_start:
    /* Check and drop to EL1 if we're in EL2 or EL3 */
    mrs x0, CurrentEL
    and x0, x0, #0xC            /* Extract EL bits (bits 3:2) */
    cmp x0, #0x8                /* EL2? */
    beq el2_entry
    cmp x0, #0xC                /* EL3? */
    beq el3_entry
    /* Already at EL1 */
    b el1_entry

el3_entry:
    /* Drop from EL3 to EL2 */
    mrs x0, scr_el3
    mov x1, #0xF                /* SCR: RW=1 (AArch64), HCE=1, SMD=1, NS=1 */
    orr x0, x0, x1
    msr scr_el3, x0

    /* Set SPSR_EL3 to EL2h with interrupts masked */
    mov x0, #0x3C9              /* EL2h, DAIF masked */
    msr spsr_el3, x0

    /* Set EL2 entry point */
    adr x0, el2_entry
    msr elr_el3, x0

    isb
    eret

el2_entry:
    /* Configure CNTVOFF_EL2 to 0 for EL1 virtual timer */
    msr cntvoff_el2, xzr

    /* Enable HCR_EL2.RW for AArch64 at EL1 */
    mov x0, #(1 << 31)          /* RW=1: AArch64 at EL1 */
    msr hcr_el2, x0

    /* Set SPSR_EL2 to EL1h with interrupts masked */
    mov x0, #0x3C5              /* EL1h, DAIF masked */
    msr spsr_el2, x0

    /* Set EL1 entry point */
    adr x0, el1_entry
    msr elr_el2, x0

    isb
    eret

el1_entry:
    /* Set up exception vectors */
    adr x0, exception_vectors
    msr vbar_el1, x0

    /* Set up stack pointer */
    ldr x0, =_stack_top
    mov sp, x0

    /* Save boot arguments (none from QEMU, but we'll pass 0 for magic) */
    mov x0, #0                   /* magic = 0 (no multiboot on ARM) */
    mov x1, #0                   /* info = 0 */

    /* Print boot message via UART (PL011 at 0x09000000) */
    adr x2, msg_boot
    bl uart_print

    /* Initialize serial port for kernel output */
    /* PL011 UART at 0x09000000 on QEMU virt */
    ldr x2, =0x09000000

    /* Disable UART */
    mov w3, #0
    str w3, [x2, #0x30]         /* UARTCR = 0 */

    /* Set baud rate (assuming 3MHz UART clock, 115200 baud) */
    mov w3, #1
    str w3, [x2, #0x24]         /* UARTIBRD = 1 */
    mov w3, #1
    str w3, [x2, #0x28]         /* UARTFBRD = 1 */

    /* Set line control: 8 bits, no parity, 1 stop bit, FIFO enabled */
    mov w3, #0x70
    str w3, [x2, #0x2C]         /* UARTLCR_H */

    /* Enable UART, TX, RX */
    mov w3, #0x301
    str w3, [x2, #0x30]         /* UARTCR */

    /* Clear BSS */
    ldr x0, =_kernel_start
    ldr x1, =_kernel_end
    sub x1, x1, x0
    mov x2, #0
    bl memset

    /* Print boot complete message */
    adr x2, msg_ready
    bl uart_print

    /* Call C kernel_main */
    bl kernel_main

    /* Halt if kernel_main returns */
hang:
    wfi
    b hang

/* Simple UART print through PL011 */
uart_print:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    ldr x3, =0x09000000         /* PL011 base */
1:
    ldrb w4, [x2], #1
    cbz w4, 2f

    /* Wait for TX FIFO not full */
3:
    ldr w5, [x3, #0x18]        /* UARTFR */
    tst w5, #(1 << 5)          /* TXFF bit */
    b.ne 3b

    /* Send character */
    str w4, [x3]                /* UARTDR */
    b 1b
2:
    ldp x29, x30, [sp], #16
    ret

memset:
    cbz x1, 2f
1:
    strb w2, [x0], #1
    subs x1, x1, #1
    b.ne 1b
2:
    ret

/* Exception vectors - 2KB aligned, 16 entries of 128 bytes each */
.align 11
exception_vectors:
    /* Current EL with SP0 */
    b sync_cur_el0              /* at +0x000 */
    .balign 0x80
    b irq_cur_el0               /* at +0x080 */
    .balign 0x80
    b fiq_cur_el0               /* at +0x100 */
    .balign 0x80
    b serr_cur_el0              /* at +0x180 */

    /* Current EL with SPx */
    .balign 0x80
    b sync_cur_elx              /* at +0x200 */
    .balign 0x80
    b irq_cur_elx               /* at +0x280 */
    .balign 0x80
    b fiq_cur_elx               /* at +0x300 */
    .balign 0x80
    b serr_cur_elx              /* at +0x380 */

    /* Lower EL, AArch64 */
    .balign 0x80
    b sync_lower_64             /* at +0x400 (SVC from userspace) */
    .balign 0x80
    b irq_lower_64              /* at +0x480 */
    .balign 0x80
    b fiq_lower_64              /* at +0x500 */
    .balign 0x80
    b serr_lower_64             /* at +0x580 */

    /* Lower EL, AArch32 */
    .balign 0x80
    b sync_lower_32             /* at +0x600 */
    .balign 0x80
    b irq_lower_32              /* at +0x680 */
    .balign 0x80
    b fiq_lower_32              /* at +0x700 */
    .balign 0x80
    b serr_lower_32             /* at +0x780 */

sync_cur_el0:
    /* Sync exception at current EL using SP0 - shouldn't happen */
    b hang
irq_cur_el0:
    b hang
fiq_cur_el0:
    b hang
serr_cur_el0:
    b hang

sync_cur_elx:
    /* Sync exception at current EL using SPx */
    /* Save context */
    stp x29, x30, [sp, #-16]!
    mrs x0, esr_el1
    mrs x1, far_el1
    /* Check exception class */
    lsr x0, x0, #26
    cmp x0, #0x25               /* Data abort from current EL */
    beq data_abort_cur
    cmp x0, #0x21               /* Instruction abort from current EL */
    beq inst_abort_cur
    /* Unknown sync exception */
    ldp x29, x30, [sp], #16
    b hang

data_abort_cur:
    /* Data abort - print fault address and halt */
    ldp x29, x30, [sp], #16
    b hang

inst_abort_cur:
    ldp x29, x30, [sp], #16
    b hang

irq_cur_elx:
    /* IRQ at current EL - dispatch to C handler */
    stp x29, x30, [sp, #-16]!
    bl irq_handler_el1
    ldp x29, x30, [sp], #16
    eret

fiq_cur_elx:
    b hang
serr_cur_elx:
    b hang

sync_lower_64:
    /* SVC from lower EL (AArch64 userspace) */
    stp x29, x30, [sp, #-16]!
    stp x0, x1, [sp, #-16]!
    /* Read syscall number and args from registers */
    /* x0-x5 have syscall args, x8 has syscall number */
    mov x0, x8                  /* syscall number */
    /* x1-x5 are already in place as args */
    bl svc_handler
    ldp x0, x1, [sp], #16
    ldp x29, x30, [sp], #16
    eret

irq_lower_64:
    stp x29, x30, [sp, #-16]!
    bl irq_handler_el1
    ldp x29, x30, [sp], #16
    eret

fiq_lower_64:
    b hang
serr_lower_64:
    b hang

sync_lower_32:
    b hang
irq_lower_32:
    b hang
fiq_lower_32:
    b hang
serr_lower_32:
    b hang

.section .rodata
msg_boot:  .asciz "[ASM] ARM64 boot entry reached\r\n"
msg_ready: .asciz "[ASM] Boot initialization complete\r\n"
