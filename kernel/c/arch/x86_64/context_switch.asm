; x86_64 context switching implementation
; void context_switch(cpu_context* old_ctx, cpu_context* new_ctx)
; x86_64 calling convention: RDI = old_ctx, RSI = new_ctx

[BITS 64]

global context_switch

section .text

context_switch:
    ; Save current context to old_ctx (in RDI)
    test rdi, rdi
    jz .load_only

    ; Save general purpose registers
    mov [rdi + 0],  rax       ; RAX
    mov [rdi + 8],  rbx       ; RBX
    mov [rdi + 16], rcx       ; RCX
    mov [rdi + 24], rdx       ; RDX
    mov [rdi + 32], rsi       ; RSI
    mov [rdi + 40], rdi       ; RDI
    mov [rdi + 48], rbp       ; RBP
    mov [rdi + 56], rsp       ; RSP

    mov [rdi + 64], r8        ; R8
    mov [rdi + 72], r9        ; R9
    mov [rdi + 80], r10       ; R10
    mov [rdi + 88], r11       ; R11
    mov [rdi + 96], r12       ; R12
    mov [rdi + 104], r13      ; R13
    mov [rdi + 112], r14      ; R14
    mov [rdi + 120], r15      ; R15

    ; Save return address as RIP
    mov rax, [rsp]
    mov [rdi + 128], rax      ; RIP

    ; Save segment registers
    mov rax, cs
    mov [rdi + 136], rax      ; CS
    mov rax, ds
    mov [rdi + 144], rax      ; DS
    mov rax, es
    mov [rdi + 152], rax      ; ES
    mov rax, fs
    mov [rdi + 160], rax      ; FS
    mov rax, gs
    mov [rdi + 168], rax      ; GS
    mov rax, ss
    mov [rdi + 176], rax      ; SS

    ; Save RFLAGS
    pushfq
    pop rax
    mov [rdi + 184], rax      ; RFLAGS

    ; Save CR3 (page directory)
    mov rax, cr3
    mov [rdi + 192], rax      ; CR3

.load_only:
    ; Load new context from new_ctx (in RSI)
    test rsi, rsi
    jz .done

    ; Disable interrupts during the critical section to prevent
    ; timer ISR from re-entering the scheduler mid-switch.
    cli

    ; Restore CR3 BEFORE restoring stack pointer
    mov rax, [rsi + 192]
    mov cr3, rax

    ; Restore segment registers (except CS/SS which are restored via retfq)
    mov ax, [rsi + 144]
    mov ds, ax
    mov ax, [rsi + 152]
    mov es, ax
    mov ax, [rsi + 160]
    mov fs, ax
    mov ax, [rsi + 168]
    mov gs, ax

    ; Restore stack pointer
    mov rsp, [rsi + 56]

    ; Push RIP onto stack and retire
    mov rax, [rsi + 128]
    push rax

    ; Restore general purpose registers (except RAX, RSP, RIP)
    mov rbx, [rsi + 8]
    mov rcx, [rsi + 16]
    mov rdx, [rsi + 24]
    mov rbp, [rsi + 48]
    mov r8,  [rsi + 64]
    mov r9,  [rsi + 72]
    mov r10, [rsi + 80]
    mov r11, [rsi + 88]
    mov r12, [rsi + 96]
    mov r13, [rsi + 104]
    mov r14, [rsi + 112]
    mov r15, [rsi + 120]

    ; Restore RFLAGS last — popfq + ret is atomically safe on x86
    mov rax, [rsi + 184]
    push rax
    popfq

    ; Restore RAX after all other registers
    mov rax, [rsi + 0]

    ; Jump to restored RIP
    ret

.done:
    ret
