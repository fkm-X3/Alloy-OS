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

section .text
syscall_entry:
    swapgs                          ; GS.base ↔ KernelGS.base

    ; Save user RSP to per-CPU scratch
    mov gs:0, rsp

    ; Switch to kernel stack
    mov rsp, [kernel_stack_top]

    ; Build saved context: user RSP, RIP (RCX), RFLAGS (R11)
    push qword [gs:0]               ; user RSP
    push rcx                        ; return RIP
    push r11                        ; saved RFLAGS

    ; Save callee-saved registers
    push r15
    push r14
    push r13
    push r12
    push rbp
    push rbx

    ; Arrange C calling convention for syscall_dispatcher:
    ;   RDI = syscall_no (RAX)
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
    mov rdi, rax                    ; syscall_no

    lea rax, [rsp]                  ; frame pointer = current RSP
    push rax                        ; 7th argument on stack

    call syscall_dispatcher

    add rsp, 8                      ; pop frame pointer

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
    pop rsp                         ; user RSP

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
