; x86_64 context switching implementation
; void context_switch(cpu_context* old_ctx, cpu_context* new_ctx)
; void save_context(cpu_context* ctx)
; void load_context(cpu_context* ctx)
; x86_64 calling convention: RDI = first arg
;
; save_context: save current CPU state into ctx, return normally.
; load_context: load CPU state from ctx, jump to saved RIP, never returns.
; context_switch: save into old_ctx, load from new_ctx, never returns.

[BITS 64]

global context_switch
global save_context
global load_context
extern g_current_user_cr3
extern g_saved_user_cr3
extern g_kernel_gs_base

; ── Diagnostic macros ──────────────────────────────────────────────
; SERIAL_PUTC: output AL to COM1, waiting for THRE. Preserves RAX.
%macro SERIAL_PUTC 0
    push rax
    push rdx
    mov dx, 0x3F8
    add dx, 5          ; LSR = base + 5
%%wait:
    in al, dx          ; read LSR (clobbers AL)
    test al, 0x20      ; THRE (bit 5)?
    jz %%wait          ; if not empty, wait
    sub dx, 5          ; back to data register
    mov al, [rsp + 8]  ; restore AL from saved RAX low byte
    out dx, al         ; write char
    pop rdx
    pop rax
%endmacro

; DIAG_CHAR: output a single byte to COM1 (0x3F8). Clobbers AX and DX.
%macro DIAG_CHAR 1
    push rax
    push rdx
    mov al, %1
    SERIAL_PUTC
    pop rdx
    pop rax
%endmacro

; SERIAL_PUTC_IMM: output an immediate byte to COM1. Clobbers AX and DX.
%macro SERIAL_PUTC_IMM 1
    push rax
    mov al, %1
    SERIAL_PUTC
    pop rax
%endmacro

; SERIAL_HEX64: print RAX as 8 hex digits to COM1. Saves/restores RAX,RCX,RDX.
%macro SERIAL_HEX64 0
    push rax
    push rcx
    push rdx
    mov rcx, 8
%%loop:
        rol rax, 4
        mov dl, al
        and dl, 0x0F
        cmp dl, 10
        jb %%digit
        add dl, 'A'-10
        jmp %%out
%%digit:
        add dl, '0'
%%out:
        mov al, dl
        SERIAL_PUTC
        dec rcx
        jnz %%loop
    pop rdx
    pop rcx
    pop rax
%endmacro

; SET_DIAG: write a 32-bit checkpoint tag to g_ctx_switch_diag (persistent).
%macro SET_DIAG 1
    mov dword [g_ctx_switch_diag], %1
%endmacro

section .text

; ================================================================
; save_context(cpu_context* ctx)
; Save current CPU registers into ctx and return normally.
; The saved RIP points to the instruction after the `call save_context`.
; When load_context later restores this context, execution resumes
; at that instruction.
; ================================================================
save_context:
    test rdi, rdi
    jz .save_done

    ; Save general purpose registers
    mov [rdi + 0],  rax
    mov [rdi + 8],  rbx
    mov [rdi + 16], rcx
    mov [rdi + 24], rdx
    mov [rdi + 32], rsi
    mov [rdi + 40], rdi
    mov [rdi + 48], rbp
    mov [rdi + 56], rsp

    mov [rdi + 64], r8
    mov [rdi + 72], r9
    mov [rdi + 80], r10
    mov [rdi + 88], r11
    mov [rdi + 96], r12
    mov [rdi + 104], r13
    mov [rdi + 112], r14
    mov [rdi + 120], r15

    ; Save return address as RIP (address after the `call save_context`)
    mov rax, [rsp]
    mov [rdi + 128], rax

    ; Save segment registers
    mov rax, cs
    mov [rdi + 136], rax
    mov rax, ds
    mov [rdi + 144], rax
    mov rax, es
    mov [rdi + 152], rax
    mov rax, fs
    mov [rdi + 160], rax
    mov rax, gs
    mov [rdi + 168], rax
    mov rax, ss
    mov [rdi + 176], rax

    ; Save RFLAGS
    pushfq
    pop rax
    mov [rdi + 184], rax

    ; Save CR3 (page directory)
    mov rax, cr3
    mov [rdi + 192], rax

    ; Save FS_BASE MSR (0xC0000100)
    mov ecx, 0xC0000100
    rdmsr
    mov [rdi + 200], rax

.save_done:
    ret

; ================================================================
; load_context(cpu_context* ctx)
; Load all CPU registers from ctx and jump to the saved RIP.
; Never returns (or "returns" to wherever ctx was saved).
; ================================================================
load_context:
    cli

    ; Pre-read ALL values from ctx before switching CR3.
    ; After CR3 switch, ctx (in kernel heap) may not be accessible.
    ; Push everything onto the current stack.
    push qword [rdi + 56]     ; RSP_new
    push qword [rdi + 128]    ; RIP_new

    ; Segment registers
    push qword [rdi + 168]    ; GS
    push qword [rdi + 160]    ; FS
    push qword [rdi + 152]    ; ES
    push qword [rdi + 144]    ; DS

    ; General-purpose registers
    push qword [rdi + 120]    ; R15
    push qword [rdi + 112]    ; R14
    push qword [rdi + 104]    ; R13
    push qword [rdi + 96]     ; R12
    push qword [rdi + 88]     ; R11
    push qword [rdi + 80]     ; R10
    push qword [rdi + 72]     ; R9
    push qword [rdi + 64]     ; R8
    push qword [rdi + 48]     ; RBP
    push qword [rdi + 24]     ; RDX
    push qword [rdi + 16]     ; RCX
    push qword [rdi + 8]      ; RBX

    push qword [rdi + 184]    ; RFLAGS
    push qword [rdi + 0]      ; RAX

    ; Pre-read new FS_BASE and CS before CR3 switch
    mov rax, [rdi + 200]      ; FS_BASE
    mov [rel fs_base_new], rax
    movzx eax, word [rdi + 136]  ; CS (16-bit selector)
    mov [rel cs_new], rax

    ; Switch CR3
    mov rax, [rdi + 192]
    mov cr3, rax
    mov [g_current_user_cr3], rax
    mov [g_saved_user_cr3], rax

    ; Pop all values in reverse order
    ; RDI and RSI used as scratch — saved RAX and RFLAGS first
    pop rdi             ; RDI = RAX (restored at the very end)
    pop rsi             ; RSI = RFLAGS (restored via popfq)

    ; Pop general-purpose registers
    pop rbx
    pop rcx
    pop rdx
    pop rbp
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15

    ; Set segment registers
    pop rax
    mov ds, ax
    pop rax
    mov es, ax
    pop rax
    mov fs, ax
    pop rax
    mov gs, ax

    ; Pop RIP_new into rdx, then RSP_new into rsp
    pop rdx             ; RDX = RIP_new
    pop rsp             ; RSP = RSP_new

    ; Check if kernel or user task
    cmp qword [rel cs_new], 0x23
    je .load_user_iretq

    ; ── KERNEL TASK ──────────────────────────────────────────────
    or rsi, 0x200           ; Ensure IF set
    push rsi
    popfq
    mov rax, rdi            ; Restore RAX
    jmp rdx                 ; Jump to entry point

.load_user_iretq:
    ; ── USER TASK via iretq ──────────────────────────────────────
    or rsi, 0x200
    mov rax, rsp
    push qword 0x1B         ; SS
    push rax                ; RSP (user stack)
    push rsi                ; RFLAGS
    push qword 0x23         ; CS
    push rdx                ; RIP

    ; GS_BASE = 0, KERNEL_GS_BASE = g_kernel_gs_base, FS_BASE = fs_base_new
    xor eax, eax
    xor edx, edx
    mov ecx, 0xC0000101
    wrmsr

    mov rax, [g_kernel_gs_base]
    xor edx, edx
    mov ecx, 0xC0000102
    wrmsr

    mov rax, [rel fs_base_new]
    xor edx, edx
    mov ecx, 0xC0000100
    wrmsr

    mov rax, rdi            ; Restore RAX
    iretq

; ================================================================
; context_switch(cpu_context* old_ctx, cpu_context* new_ctx)
; Save current CPU context into old_ctx, load new context from new_ctx.
; The saved RIP points to the instruction after the `call context_switch`.
; When context_switch later restores this context, execution resumes
; at that instruction with the stack intact.
;
; Never returns to the caller — it "returns" to the old task when
; that task is later restored via another context_switch call.
; ================================================================
context_switch:
    DIAG_CHAR '!'
    ; Save current context to old_ctx (in RDI)
    test rdi, rdi
    jz .load_only

    DIAG_CHAR 'A'

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

    ; Save FS_BASE MSR (0xC0000100)
    mov ecx, 0xC0000100
    rdmsr
    mov [rdi + 200], rax      ; FS_BASE

.load_only:
    ; Load new context from new_ctx (in RSI)
    test rsi, rsi
    jz .done

    cli

    DIAG_CHAR 'B'

    ; Pre-read ALL new_ctx values BEFORE CR3 switch
    push qword [rsi + 56]     ; RSP_new
    push qword [rsi + 128]    ; RIP_new

    ; Segment registers
    push qword [rsi + 168]    ; GS
    push qword [rsi + 160]    ; FS
    push qword [rsi + 152]    ; ES
    push qword [rsi + 144]    ; DS

    ; General-purpose registers
    push qword [rsi + 120]    ; R15
    push qword [rsi + 112]    ; R14
    push qword [rsi + 104]    ; R13
    push qword [rsi + 96]     ; R12
    push qword [rsi + 88]     ; R11
    push qword [rsi + 80]     ; R10
    push qword [rsi + 72]     ; R9
    push qword [rsi + 64]     ; R8
    push qword [rsi + 48]     ; RBP
    push qword [rsi + 24]     ; RDX
    push qword [rsi + 16]     ; RCX
    push qword [rsi + 8]      ; RBX

    push qword [rsi + 184]    ; RFLAGS
    push qword [rsi + 0]      ; RAX

    ; Pre-read new FS_BASE and CS before CR3 switch
    mov rax, [rsi + 200]      ; FS_BASE from new_ctx
    mov [rel fs_base_new], rax
    movzx eax, word [rsi + 136]  ; CS from new_ctx (16-bit selector)
    mov [rel cs_new], rax

    DIAG_CHAR 'C'

    ; Dump new CR3
    push rsi
    mov rsi, [rsi + 192]
    mov rax, rsi
    SERIAL_HEX64
    mov al, ' '
    SERIAL_PUTC
    pop rsi

    ; Dump new CS
    push rax
    movzx eax, word [rsi + 136]
    SERIAL_HEX64
    mov al, ' '
    SERIAL_PUTC
    pop rax

    ; Dump new RIP
    push rax
    push rsi
    mov rax, [rsi + 128]
    SERIAL_HEX64
    mov al, ' '
    SERIAL_PUTC
    pop rsi
    pop rax

    ; Dump new RSP
    push rax
    push rsi
    mov rax, [rsi + 56]
    SERIAL_HEX64
    mov al, 0x0A
    SERIAL_PUTC
    pop rsi
    pop rax

    ; Switch CR3
    mov rax, [rsi + 192]
    mov cr3, rax
    mov [g_current_user_cr3], rax
    mov [g_saved_user_cr3], rax

    DIAG_CHAR 'D'

    ; Dump actual CR3
    push rax
    mov rax, cr3
    SERIAL_HEX64
    mov al, ' '
    SERIAL_PUTC
    pop rax

    DIAG_CHAR 'E'

    ; Pop all values in reverse order
    pop rdi             ; RDI = RAX (restored at the very end)
    pop rsi             ; RSI = RFLAGS (restored via popfq)

    pop rbx
    pop rcx
    pop rdx
    pop rbp
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15

    DIAG_CHAR 'F'

    ; Set segment registers
    pop rax
    mov ds, ax
    pop rax
    mov es, ax
    pop rax
    mov fs, ax
    pop rax
    mov gs, ax

    ; Pop RIP_new into rdx, then RSP_new into rsp
    pop rdx             ; RDX = RIP_new
    pop rsp             ; RSP = RSP_new

    DIAG_CHAR 'G'

    cmp qword [rel cs_new], 0x23
    je .user_iretq

    ; ── KERNEL TASK: ring-0 -> ring-0 ──────────────────────────────
    or rsi, 0x200
    push rsi
    popfq

    mov rax, rdi

    DIAG_CHAR 'K'

    jmp rdx

.user_iretq:
    DIAG_CHAR 'H'

    or rsi, 0x200

    mov rax, rsp
    push qword 0x1B     ; SS
    push rax            ; RSP
    push rsi            ; RFLAGS
    push qword 0x23     ; CS
    push rdx            ; RIP

    DIAG_CHAR 'I'

    xor eax, eax
    xor edx, edx
    mov ecx, 0xC0000101
    wrmsr

    mov rax, [g_kernel_gs_base]
    xor edx, edx
    mov ecx, 0xC0000102
    wrmsr

    mov rax, [rel fs_base_new]
    xor edx, edx
    mov ecx, 0xC0000100
    wrmsr

    mov rax, rdi

    DIAG_CHAR 'J'

    push rax
    mov rax, rsp
    SERIAL_HEX64
    mov al, ' '
    SERIAL_PUTC
    pop rax

    iretq

.done:
    ret

section .data
fs_base_new: dq 0
cs_new:      dq 0
global g_ctx_switch_diag
g_ctx_switch_diag: dd 0
