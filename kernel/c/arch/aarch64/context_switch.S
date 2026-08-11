/* aarch64 context switching implementation */
/* void context_switch(cpu_context* old_ctx, cpu_context* new_ctx) */
/* x0 = old_ctx, x1 = new_ctx */

.section .text

/* void save_context(cpu_context* ctx) */
/* Save current context into ctx and return normally (new_ctx = NULL). */
/* The saved x30 (LR) is the instruction after `bl save_context`, so a
 * later load_context resumes by `ret`-ing back into the caller (i.e.
 * schedule()'s resume path), which unwinds through the IRQ frame. */
.global save_context
.type save_context, @function
save_context:
    mov x1, xzr              /* new_ctx = NULL -> save only */
    b context_switch

/* void load_context(cpu_context* ctx) */
/* Restore ctx and never return (old_ctx = NULL). */
.global load_context
.type load_context, @function
load_context:
    mov x1, x0              /* new_ctx = ctx (C arg arrives in x0) */
    mov x0, xzr             /* old_ctx = NULL -> load only */
    b context_switch

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
    stp x29, x30, [x0, #80]    /* FP (x29), LR (x30 = resume point) */

    /* Save stack pointer */
    mov x2, sp
    str x2, [x0, #96]           /* SP */

    /* NOTE: elr_el1/spsr_el1 slots (#104/#112) are intentionally NOT
     * saved.  A preempted task resumes by `ret`-ing to its saved LR and
     * unwinding back up the IRQ stack; irq_handler_el1_asm then restores
     * the task's registers and erets to the interrupted PC (still in
     * ELR_EL1).  Saving elr_el1 here and eret-ing directly would skip the
     * IRQ epilogue, leaving the task with the scheduler's registers. */

.Lload_only:
    /* Load new context if new_ctx is not NULL */
    cbz x1, .Ldone

    /* Restore translation table base */
    ldr x2, [x1, #120]
    msr ttbr0_el1, x2
    tlbi vmalle1
    dsb sy
    isb

    /* Restore stack pointer */
    ldr x2, [x1, #96]
    mov sp, x2

    /* Restore SP_EL0 (user stack) for userland tasks.  The EL0t SP is
     * preserved by hardware across exceptions, so it only needs to be set
     * here on the fresh-task path (first entry to EL0).  For EL1 tasks it
     * is unused and harmless to write. */
    ldr x2, [x1, #128]
    msr sp_el0, x2

    /* Restore callee-saved registers */
    ldp x19, x20, [x1, #0]
    ldp x21, x22, [x1, #16]
    ldp x23, x24, [x1, #32]
    ldp x25, x26, [x1, #48]
    ldp x27, x28, [x1, #64]
    ldp x29, x30, [x1, #80]    /* FP, LR */

    /* Fresh task (LR == 0 sentinel set by Task::new / spawn): start it
     * via ERET so the initial SPSR (IRQs unmasked at EL1, or EL0 for
     * userland) is applied.  Otherwise resume by `ret`-ing to the saved
     * LR (after `bl save_context`); the IRQ epilogue finishes the job
     * for preempted tasks. */
    cbnz x30, .Lret_resume

    ldr x2, [x1, #104]          /* ELR = task entry */
    msr elr_el1, x2
    ldr x2, [x1, #112]          /* SPSR = initial PSTATE */
    msr spsr_el1, x2
    eret

.Lret_resume:
    ret

.Ldone:
    ret
