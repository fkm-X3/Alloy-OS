; x86_64 IDT flush and ISR/IRQ stubs
; 64-bit IDT entries are 16 bytes, stacks use IST

[BITS 64]

section .text

; IDT flush function
; void idt_flush(uint64_t idt_ptr)
global idt_flush
idt_flush:
    lidt [rdi]       ; Load IDT register (x86_64 calling convention: first arg in RDI)
    ret

; Common ISR stub - saves state and calls C handler
extern exception_handler
isr_common_stub:
    ; Save all general purpose registers
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    ; Save segment registers
    mov rax, ds
    push rax
    mov rax, es
    push rax
    mov rax, fs
    push rax
    mov rax, gs
    push rax

    ; Load kernel data segment
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Pass stack pointer (interrupt frame) as argument
    mov rdi, rsp
    call exception_handler

    ; Restore segment registers
    pop rax
    mov gs, ax
    pop rax
    mov fs, ax
    pop rax
    mov es, ax
    pop rax
    mov ds, ax

    ; Restore general purpose registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    ; Clean up error code and interrupt number (8 bytes)
    add rsp, 16
    iretq

; Common IRQ stub
extern irq_handler
irq_common_stub:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    mov rax, ds
    push rax
    mov rax, es
    push rax
    mov rax, fs
    push rax
    mov rax, gs
    push rax

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    mov rdi, rsp
    call irq_handler

    pop rax
    mov gs, ax
    pop rax
    mov fs, ax
    pop rax
    mov es, ax
    pop rax
    mov ds, ax

    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    add rsp, 16
    iretq

; ISR without error code: push dummy error code, then ISR number
%macro ISR_NOERRCODE 1
global isr%1
isr%1:
    cli
    push 0          ; Dummy error code
    push %1         ; Interrupt number
    jmp isr_common_stub
%endmacro

; ISR with error code: push ISR number only (error code already on stack)
%macro ISR_ERRCODE 1
global isr%1
isr%1:
    cli
    push %1
    jmp isr_common_stub
%endmacro

; IRQ: push dummy error code, then IRQ number
%macro IRQ 2
global irq%1
irq%1:
    cli
    push 0
    push %2
    jmp irq_common_stub
%endmacro

; CPU Exception handlers (0-31)
ISR_NOERRCODE 0     ; Divide by zero
ISR_NOERRCODE 1     ; Debug
ISR_NOERRCODE 2     ; Non-maskable interrupt
ISR_NOERRCODE 3     ; Breakpoint
ISR_NOERRCODE 4     ; Overflow
ISR_NOERRCODE 5     ; Bound range exceeded
ISR_NOERRCODE 6     ; Invalid opcode
ISR_NOERRCODE 7     ; Device not available
ISR_ERRCODE   8     ; Double fault
ISR_NOERRCODE 9     ; Coprocessor segment overrun
ISR_ERRCODE   10    ; Invalid TSS
ISR_ERRCODE   11    ; Segment not present
ISR_ERRCODE   12    ; Stack-segment fault
ISR_ERRCODE   13    ; General protection fault
ISR_ERRCODE   14    ; Page fault
ISR_NOERRCODE 15    ; Reserved
ISR_NOERRCODE 16    ; x87 FPU error
ISR_ERRCODE   17    ; Alignment check
ISR_NOERRCODE 18    ; Machine check
ISR_NOERRCODE 19    ; SIMD floating-point exception
ISR_NOERRCODE 20    ; Virtualization exception
ISR_NOERRCODE 21    ; Reserved
ISR_NOERRCODE 22    ; Reserved
ISR_NOERRCODE 23    ; Reserved
ISR_NOERRCODE 24    ; Reserved
ISR_NOERRCODE 25    ; Reserved
ISR_NOERRCODE 26    ; Reserved
ISR_NOERRCODE 27    ; Reserved
ISR_NOERRCODE 28    ; Reserved
ISR_NOERRCODE 29    ; Reserved
ISR_ERRCODE   30    ; Security exception
ISR_NOERRCODE 31    ; Reserved

; IRQ handlers (32-47, mapped from IRQ 0-15)
IRQ 0, 0    ; Timer
IRQ 1, 1    ; Keyboard
IRQ 2, 2    ; Cascade
IRQ 3, 3    ; COM2
IRQ 4, 4    ; COM1
IRQ 5, 5    ; LPT2
IRQ 6, 6    ; Floppy
IRQ 7, 7    ; LPT1
IRQ 8, 8    ; RTC
IRQ 9, 9    ; Free
IRQ 10, 10  ; Free
IRQ 11, 11  ; Free
IRQ 12, 12  ; PS/2 Mouse
IRQ 13, 13  ; FPU
IRQ 14, 14  ; Primary ATA
IRQ 15, 15  ; Secondary ATA
