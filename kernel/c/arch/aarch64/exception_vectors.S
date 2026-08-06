/* aarch64 exception vector table and handlers */
/* Must be 2KB aligned. Vector offsets within each 128-byte entry:
 *   0x000: Sync (SP0)
 *   0x080: IRQ   (SP0)
 *   0x100: FIQ   (SP0)
 *   0x180: SError (SP0)
 *   0x200: Sync (SPx)
 *   0x280: IRQ   (SPx)
 *   0x300: FIQ   (SPx)
 *   0x380: SError (SPx)
 *   0x400: Sync (Lower EL, AArch64)
 *   0x480: IRQ   (Lower EL, AArch64)
 *   0x500: FIQ   (Lower EL, AArch64)
 *   0x580: SError (Lower EL, AArch64)
 *   0x600: Sync (Lower EL, AArch32)
 *   0x680: IRQ   (Lower EL, AArch32)
 *   0x700: FIQ   (Lower EL, AArch32)
 *   0x780: SError (Lower EL, AArch32)
 */

.section .text.exception_vectors, "ax"
.global _exception_vectors
.align 11

_exception_vectors:
    /* ===== Current EL with SP0 ===== */
.org 0x000
vector_sync_sp0:
    b vector_sync_sp0_hang
.org 0x080
vector_irq_sp0:
    b vector_irq_sp0_hang
.org 0x100
vector_fiq_sp0:
    b vector_fiq_sp0_hang
.org 0x180
vector_serr_sp0:
    b vector_serr_sp0_hang

    /* ===== Current EL with SPx ===== */
.org 0x200
vector_sync_spx:
    b sync_handler_el1
.org 0x280
vector_irq_spx:
    b irq_handler_el1_asm
.org 0x300
vector_fiq_spx:
    b fiq_handler_el1
.org 0x380
vector_serr_spx:
    b serr_handler_el1

    /* ===== Lower EL, AArch64 ===== */
.org 0x400
vector_sync_lower_64:
    b sync_lower_el_handler
.org 0x480
vector_irq_lower_64:
    b irq_lower_el_handler
.org 0x500
vector_fiq_lower_64:
    b fiq_lower_el_handler
.org 0x580
vector_serr_lower_64:
    b serr_lower_el_handler

    /* ===== Lower EL, AArch32 ===== */
.org 0x600
vector_sync_lower_32:
    b vector_sync_lower_32_hang
.org 0x680
vector_irq_lower_32:
    b vector_irq_lower_32_hang
.org 0x700
vector_fiq_lower_32:
    b vector_fiq_lower_32_hang
.org 0x780
vector_serr_lower_32:
    b vector_serr_lower_32_hang

/* Halt handlers for unexpected vectors */
vector_sync_sp0_hang:
    b .
vector_irq_sp0_hang:
    b .
vector_fiq_sp0_hang:
    b .
vector_serr_sp0_hang:
    b .
vector_sync_lower_32_hang:
    b .
vector_irq_lower_32_hang:
    b .
vector_fiq_lower_32_hang:
    b .
vector_serr_lower_32_hang:
    b .

/* ===== Sync handler (current EL, SPx) ===== */
sync_handler_el1:
    /* Save callee-saved regs */
    stp x29, x30, [sp, #-16]!
    stp x0, x1, [sp, #-16]!

    mrs x0, esr_el1
    mrs x1, far_el1
    /* ESR_EL1: exception class is bits [31:26] */
    lsr x0, x0, #26

    /* Check exception class */
    cmp x0, #0x25               /* Data Abort (current EL) */
    beq data_abort_el1
    cmp x0, #0x21               /* Instruction Abort (current EL) */
    beq inst_abort_el1
    cmp x0, #0x24               /* Data Abort (lower EL) */
    beq data_abort_lower
    cmp x0, #0x20               /* Instruction Abort (lower EL) */
    beq inst_abort_lower
    cmp x0, #0x15               /* SVC (AArch64) */
    beq svc_from_lower

    /* Unknown - halt */
    ldp x0, x1, [sp], #16
    ldp x29, x30, [sp], #16
    b .

data_abort_el1:
    ldp x0, x1, [sp], #16
    ldp x29, x30, [sp], #16
    b .

inst_abort_el1:
    ldp x0, x1, [sp], #16
    ldp x29, x30, [sp], #16
    b .

data_abort_lower:
    /* Page fault in userspace - call C handler */
    ldp x0, x1, [sp], #16
    ldp x29, x30, [sp], #16
    /* x0 = fault address (FAR_EL1), x1 = ESR_EL1 */
    mov x0, x1
    /* Call page fault handler */
    bl page_fault_handler
    eret

inst_abort_lower:
    mov x0, x1
    bl page_fault_handler
    eret

svc_from_lower:
    /* System call from userspace */
    ldp x0, x1, [sp], #16
    ldp x29, x30, [sp], #16
    /* Save user context */
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    str x30, [sp, #-16]!        /* LR */

    /* Read ESR_EL1 to get SVC immediate (syscall number) */
    mrs x0, esr_el1
    and w0, w0, #0xFFFF         /* SVC immediate = syscall number */
    mov x1, x0                  /* arg0 */
    mov x2, x1                  /* arg1 (from user x1) */
    mov x3, x2                  /* arg2 */
    mov x4, x3                  /* arg3 */
    mov x5, x4                  /* arg4 */
    /* x0 = syscall number */
    bl svc_handler

    /* Restore user context */
    ldr x30, [sp], #16
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    eret

/* ===== IRQ handlers ===== */
/* Must preserve ALL general-purpose registers across the C call, since the
 * interrupted context can be arbitrary kernel code (x19-x28 are callee-saved
 * per AAPCS and the C handler chain clobbers them). */
irq_handler_el1_asm:
    mrs x16, daif               /* save DAIF (IRQ state) */
    msr daifset, #2             /* mask IRQs: handler must not be re-entered */
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    str x30, [sp, #-16]!        /* LR */
    bl irq_handler_el1
    ldr x30, [sp], #16
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    msr daif, x16               /* restore interrupted DAIF (re-enable IRQ if it was on) */
    eret

sync_lower_el_handler:
    b sync_handler_el1

irq_lower_el_handler:
    mrs x16, daif               /* save DAIF (IRQ state) */
    msr daifset, #2             /* mask IRQs: handler must not be re-entered */
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    str x30, [sp, #-16]!        /* LR */
    bl irq_handler_el1
    ldr x30, [sp], #16
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    msr daif, x16               /* restore interrupted DAIF (re-enable IRQ if it was on) */
    eret

fiq_handler_el1:
    b .

serr_handler_el1:
    b .

fiq_lower_el_handler:
    b .

serr_lower_el_handler:
    b .

/* C function declarations */
.global irq_handler_el1
.global svc_handler
.global page_fault_handler
