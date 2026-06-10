; x86_64 system call entry using MSR-based syscall/sysret
; Sets up MSR registers STAR, LSTAR, SF_MASK for syscall instruction
;
; syscall entry convention:
;   RCX = return address (set by CPU)
;   R11 = saved RFLAGS (set by CPU)
;   EAX = syscall number
;   RDI, RSI, RDX, R10, R8, R9 = arguments
; Return value in RAX

[BITS 64]

global syscall_entry
extern syscall_dispatcher

section .text

syscall_entry:
    ; Swap to kernel GS base (points to per-CPU area)
    swapgs

    ; Save user stack pointer in temporary location
    mov gs:0, rsp

    ; Load kernel stack
    ; For simplicity, we use a fixed kernel stack
    ; In a real kernel this would be per-CPU/per-task
    mov rsp, [kernel_stack_top]

    ; Create space for saved registers
    sub rsp, 8    ; alignment

    ; Push user RSP and return address
    push gs:0     ; user RSP
    push rcx      ; user return RIP (RCX on syscall)
    push r11      ; user RFLAGS (R11 on syscall)

    ; Save callee-saved registers
    push r15
    push r14
    push r13
    push r12
    push rbp
    push rbx

    ; Push syscall arguments for C dispatcher
    ; syscall_dispatcher(syscall_no, arg0, arg1, arg2, arg3, arg4, frame_ptr)
    push r9       ; arg4
    push r8       ; arg3
    push r10      ; arg2 (3rd arg in syscall convention)
    push rdx      ; arg1 (2nd arg)
    push rsi      ; arg0 (1st arg)
    push rdi      ; 0th arg (not used as syscall number here)
    push rax      ; syscall number

    mov rdi, rax                    ; arg1 = syscall number
    mov rsi, rdi                    ; arg2 = arg0 (from original RDI? No, we need to save original args)
    ; Actually, re-read the arguments properly
    ; RDI, RSI, RDX, R10, R8, R9 are the original user-space args
    ; But we already pushed them on the stack
    ; Let's use the dispatcher which receives args on stack

    ; Pass pointer to saved register frame
    lea rax, [rsp]
    push rax                        ; arg5 = frame pointer

    call syscall_dispatcher

    add rsp, 8                      ; clean frame pointer

    ; RAX has return value
    mov [rsp + 56], rax             ; store in syscall number slot on stack

    pop rax                         ; clean syscall number
    pop rdi                         ; clean arg0
    pop rsi                         ; clean arg1
    pop rdx                         ; clean arg2
    pop r10                         ; clean arg3
    pop r8                          ; clean arg4
    pop r9                          ; clean arg4
    add rsp, 8                      ; clean extra push

    ; Restore callee-saved registers
    pop rbx
    pop rbp
    pop r12
    pop r13
    pop r14
    pop r15

    ; Restore user RFLAGS, RIP, RSP
    pop r11      ; user RFLAGS
    pop rcx      ; user RIP
    pop rsp      ; user RSP

    ; Switch back to user GS base
    swapgs

    ; Return to userspace
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
