; x86_64 context switching implementation
; void context_switch(cpu_context* old_ctx, cpu_context* new_ctx)
; x86_64 calling convention: RDI = old_ctx, RSI = new_ctx

[BITS 64]

global context_switch
extern g_current_user_cr3

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

    ; Save FS_BASE MSR (0xC0000100)
    mov ecx, 0xC0000100
    rdmsr
    mov [rdi + 200], rax      ; FS_BASE

.load_only:
    ; Load new context from new_ctx (in RSI)
    test rsi, rsi
    jz .done

    cli

    ; ================================================================
    ; Pre-read ALL new_ctx values BEFORE CR3 switch.
    ; new_ctx is in kernel heap (above 32 MB), which is NOT mapped in
    ; the user page table.  After CR3 switch we cannot dereference
    ; RSI, so we push everything onto the kernel stack now.
    ;
    ; Push order (last in = first out):
    ;   [19] RSP_new    — popped last, sets RSP
    ;   [18] RIP_new    — popped 2nd-last, stored in RDX
    ;   [17] GS
    ;   [16] FS
    ;   [15] ES
    ;   [14] DS
    ;   [13] R15
    ;   [12] R14
    ;   [11] R13
    ;   [10] R12
    ;   [9]  R11
    ;   [8]  R10
    ;   [7]  R9
    ;   [6]  R8
    ;   [5]  RBP
    ;   [4]  RDX
    ;   [3]  RCX
    ;   [2]  RBX
    ;   [1]  RFLAGS
    ;   [0]  RAX         — popped first
    ; ================================================================

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

    ; Switch CR3 (must happen after all pre-reads)
    mov rax, [rsi + 192]
    mov cr3, rax
    mov [g_current_user_cr3], rax

    ; ================================================================
    ; Pop all values in reverse order
    ; Stack layout at this point (RSP -> item[0]):
    ;   [rsp+0]   RAX      (item[0])
    ;   [rsp+8]   RFLAGS   (item[1])
    ;   [rsp+16]  RBX      (item[2])
    ;   [rsp+24]  RCX      (item[3])
    ;   [rsp+32]  RDX      (item[4])
    ;   [rsp+40]  RBP      (item[5])
    ;   [rsp+48]  R8       (item[6])
    ;   [rsp+56]  R9       (item[7])
    ;   [rsp+64]  R10      (item[8])
    ;   [rsp+72]  R11      (item[9])
    ;   [rsp+80]  R12      (item[10])
    ;   [rsp+88]  R13      (item[11])
    ;   [rsp+96]  R14      (item[12])
    ;   [rsp+104] R15      (item[13])
    ;   [rsp+112] DS       (item[14])
    ;   [rsp+120] ES       (item[15])
    ;   [rsp+128] FS       (item[16])
    ;   [rsp+136] GS       (item[17])
    ;   [rsp+144] RIP_new  (item[18])
    ;   [rsp+152] RSP_new  (item[19])
    ; ================================================================

    ; Use RDI and RSI as scratch — they are NOT restored from the new
    ; context.  Save RAX and RFLAGS here so they survive the pops.
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

    ; ================================================================
    ; Determine kernel vs user task based on saved CS selector.
    ; ================================================================

    cmp qword [rel cs_new], 0x23
    je .user_iretq

    ; ── KERNEL TASK: ring-0 -> ring-0 ──────────────────────────────
    ; No iretq needed.  Restore RFLAGS and jump to entry point.

    ; Ensure IF (bit 9) is set in RFLAGS so the task receives interrupts.
    or rsi, 0x200
    push rsi
    popfq

    ; Restore RAX (new task's RAX, held in RDI as scratch)
    mov rax, rdi

    ; Jump to the task's entry point (RIP, held in RDX as scratch)
    jmp rdx

.user_iretq:
    ; ── USER TASK: ring-0 -> ring-3 ────────────────────────────────
    ; Ensure IF (bit 9) is set in RFLAGS
    or rsi, 0x200

    ; Build 5-qword iretq frame on the user stack.
    ; iretq atomically restores RIP, CS, RFLAGS, RSP, SS —
    ; the CPL change to ring 3 is atomic with the RSP restore.
    ;
    ; Frame layout (RSP points to RIP after all pushes):
    ;   [RSP+32] SS
    ;   [RSP+24] RSP  (user stack top, BEFORE frame was pushed)
    ;   [RSP+16] RFLAGS
    ;   [RSP+8]  CS
    ;   [RSP+0]  RIP
    mov rax, rsp        ; RAX = user stack top (value for iretq RSP field)
    push qword 0x1B     ; SS  = user data segment
    push rax            ; RSP = original user stack top (before pushes)
    push rsi            ; RFLAGS
    push qword 0x23     ; CS  = user code segment
    push rdx            ; RIP = entry point

    ; Set GS_BASE MSR (0xC0000101) = 0 to prevent swapgs leakage.
    xor eax, eax
    xor edx, edx
    mov ecx, 0xC0000101
    wrmsr

    ; Set FS_BASE MSR (0xC0000100) — enables TLS via fs: segment.
    mov rax, [rel fs_base_new]
    xor edx, edx
    mov ecx, 0xC0000100       ; MSR_FS_BASE
    wrmsr

    ; Restore RAX (new task's RAX) — must happen after wrmsr clobbers eax
    mov rax, rdi

    ; Atomic ring-3 transition
    iretq

.done:
    ret

section .data
fs_base_new: dq 0
cs_new:      dq 0
