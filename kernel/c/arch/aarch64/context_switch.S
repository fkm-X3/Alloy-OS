/* aarch64 context switching implementation */
/* void context_switch(cpu_context* old_ctx, cpu_context* new_ctx) */
/* x0 = old_ctx, x1 = new_ctx */

.section .text
.global context_switch
.type context_switch, @function

context_switch:
    /* Save current context if old_ctx is not NULL */
    cbz x0, .Lload_only

    /* Save callee-saved registers (x19-x30) */
    stp x19, x20, [x0, #0]
    stp x21, x22, [x0, #16]
    stp x23, x24, [x0, #32]
    stp x25, x26, [x0, #48]
    stp x27, x28, [x0, #64]
    stp x29, x30, [x0, #80]    /* FP (x29), LR (x30) */

    /* Save stack pointer */
    mov x2, sp
    str x2, [x0, #96]           /* SP */

    /* Save exception link register (ELR_EL1) */
    mrs x2, elr_el1
    str x2, [x0, #104]          /* ELR */

    /* Save saved processor state register (SPSR_EL1) */
    mrs x2, spsr_el1
    str x2, [x0, #112]          /* SPSR */

    /* Save translation table base (TTBR0_EL1) */
    mrs x2, ttbr0_el1
    str x2, [x0, #120]          /* TTBR0 */

.Lload_only:
    /* Load new context if new_ctx is not NULL */
    cbz x1, .Ldone

    /* Restore translation table base */
    ldr x2, [x1, #120]
    msr ttbr0_el1, x2
    tlbi vmalle1
    dsb sy
    isb

    /* Restore SPSR */
    ldr x2, [x1, #112]
    msr spsr_el1, x2

    /* Restore ELR */
    ldr x2, [x1, #104]
    msr elr_el1, x2

    /* Restore stack pointer */
    ldr x2, [x1, #96]
    mov sp, x2

    /* Restore callee-saved registers */
    ldp x19, x20, [x1, #0]
    ldp x21, x22, [x1, #16]
    ldp x23, x24, [x1, #32]
    ldp x25, x26, [x1, #48]
    ldp x27, x28, [x1, #64]
    ldp x29, x30, [x1, #80]    /* FP, LR */

    /* Return to new context via ERET */
    eret

.Ldone:
    ret
