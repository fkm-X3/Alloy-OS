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

    /* aarch64 SVC ABI (alloy_syscall_aarch64.h): x8 = syscall number,
     * x1..x5 = args, svc #0, return in x0.  All user registers were saved
     * above; pull the number and args from their saved frame slots:
     *   x8  at +176, x1 at +248, x2 at +224, x3 at +232,
     *   x4  at +208, x5 at +216, user x0 (return slot) at +240. */
    ldr x0, [sp, #176]          /* x8  -> syscall number */
    ldr x1, [sp, #248]          /* x1  -> arg0 */
    ldr x2, [sp, #224]          /* x2  -> arg1 */
    ldr x3, [sp, #232]          /* x3  -> arg2 */
    ldr x4, [sp, #208]          /* x4  -> arg3 */
    ldr x5, [sp, #216]          /* x5  -> arg4 */
    bl svc_handler
    str x0, [sp, #240]          /* return value -> saved user x0 slot */

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
 * per AAPCS and the C handler chain clobbers them).  NOTE: x16-x18 are used
 * as scratch for ELR/SPSR, so they must be saved FIRST — capturing them
 * before the register block would lose the interrupted values. */
irq_handler_el1_asm:
    /* Save ALL general-purpose registers (true interrupted values). */
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

    /* Capture the interrupted PC/PSTATE into the frame top.  Hardware sets
     * PSTATE.I on exception entry, so IRQs are already masked here.  DAIF is
     * NOT saved: eret applies SPSR_EL1 (which carries the interrupt-mask
     * bits) back to PSTATE. */
    msr daifset, #2             /* mask IRQs: handler must not be re-entered */
    mrs x17, elr_el1            /* save interrupted PC */
    mrs x18, spsr_el1           /* save interrupted PSTATE */
    stp x17, x18, [sp, #-16]!   /* ELR, SPSR at lowest addr (frame top) */
    bl irq_handler_el1
    /* The scheduler may have switched tasks.  If so, `load_context` resumed
     * us through this very epilogue, so the frame below is the (possibly
     * different) task being resumed.  Pop ELR/SPSR and apply them NOW, while
     * x17/x18 still hold them and before the GPR restore overwrites the
     * scratch registers.  eret to THE TASK'S interrupted PC — not whatever
     * ERET last wrote ELR_EL1 to. */
    ldp x17, x18, [sp], #16     /* ELR, SPSR */
    msr daifset, #2             /* keep IRQs masked through the restore */
    msr elr_el1, x17
    msr spsr_el1, x18
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

sync_lower_el_handler:
    b sync_handler_el1

/* IRQ from lower EL (EL0): identical handling.  Tasks run at EL1 in this
 * kernel, so this vector is unused in practice, but keep it correct. */
irq_lower_el_handler:
    b irq_handler_el1_asm

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
