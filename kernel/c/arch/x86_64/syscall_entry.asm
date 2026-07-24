; x86_64 system call entry using MSR-based syscall/sysret
;
; syscall instruction convention:
;   RAX = syscall number
;   RCX = user-space return RIP (set by CPU, preserved for sysret)
;   R11 = user-space RFLAGS (set by CPU, preserved for sysret)
;   RDI = arg0, RSI = arg1, RDX = arg2, R10 = arg3, R8 = arg4, R9 = arg5
;
; Return value in RAX. Returns via o64 sysret (pops RIP from RCX, RFLAGS from R11).
;
; C dispatcher signature:
;   uint32_t syscall_dispatcher(uint32_t syscall_no,
;                               uint32_t arg0, uint32_t arg1, uint32_t arg2,
;                               uint32_t arg3, uint32_t arg4,
;                               uint32_t* frame_ptr);

[BITS 64]

global syscall_entry
extern syscall_dispatcher
extern kernel_pml4_phys

section .text
syscall_entry:
    swapgs                          ; GS.base ↔ KernelGS.base

    ; Save original user RSP before any pushes
    mov gs:0, rsp

    ; Save syscall number on user stack, then user CR3
    push rax                        ; [user_rsp-8] = syscall number
    mov rax, cr3
    push rax                        ; [user_rsp-16] = user CR3

    ; Load kernel CR3
    mov rax, [kernel_pml4_phys]
    mov cr3, rax

    ; Switch to kernel stack
    mov rsp, [kernel_stack_top]

    ; Build saved context: original RSP, user CR3, RIP (RCX), RFLAGS (R11)
    push qword [gs:0]               ; original user RSP
    mov rax, [gs:0]                 ; RAX = original user RSP
    push qword [rax - 16]           ; user CR3 (at original RSP - 16, pushed second)
    push rcx                        ; return RIP
    push r11                        ; saved RFLAGS

    ; Save callee-saved registers
    push r15
    push r14
    push r13
    push r12
    push rbp
    push rbx

    ; Stack layout (top to bottom):
    ;   rsp+0:  rbx
    ;   rsp+8:  rbp
    ;   rsp+16: r12
    ;   rsp+24: r13
    ;   rsp+32: r14
    ;   rsp+40: r15
    ;   rsp+48: RFLAGS (r11)
    ;   rsp+56: RIP (rcx)
    ;   rsp+64: user CR3
    ;   rsp+72: original user RSP

    ; Arrange C calling convention for syscall_dispatcher:
    ;   RDI = syscall_no
    ;   RSI = arg0 (original RDI)
    ;   RDX = arg1 (original RSI)
    ;   RCX = arg2 (original RDX)
    ;   R8  = arg3 (original R10)
    ;   R9  = arg4 (original R8)
    ;   [stack] = frame pointer (RSP after callee saves)
    mov r9, r8                      ; arg4
    mov r8, r10                     ; arg3
    mov rcx, rdx                    ; arg2
    mov rdx, rsi                    ; arg1
    mov rsi, rdi                    ; arg0
    mov rax, [gs:0]                 ; RAX = original user RSP
    mov rdi, [rax - 8]             ; syscall_no from user stack (pushed first)

    ; 10 pushes = 80 bytes = 5*16, already 16-byte aligned.
    ; call pushes 8 bytes, so callee gets RSP ≡ 8 (mod 16) ✓
    call syscall_dispatcher

    ; Restore callee-saved registers
    pop rbx
    pop rbp
    pop r12
    pop r13
    pop r14
    pop r15

    ; Restore user context
    pop r11                         ; RFLAGS
    pop rcx                         ; RIP
    pop rax                         ; user CR3
    mov cr3, rax
    pop rsp                         ; original user RSP

    ; (syscall number and CR3 copy remain on user stack below RSP, harmless)

    swapgs
    o64 sysret

section .data
global kernel_stack_top
kernel_stack_top: dq 0

section .bss
align 16
global kernel_stack_bottom
kernel_stack_bottom:
    resb 16384
global kernel_stack_top_alias
kernel_stack_top_alias:
