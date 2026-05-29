; IDT flush and ISR stubs

section .text

; IDT flush function
global idt_flush
idt_flush:
    mov eax, [esp+4]
    lidt [eax]
    ret

; Common ISR stub - saves state and calls handler
extern exception_handler
isr_common_stub:
    pusha
    
    push ds
    push es
    push fs
    push gs
    
    mov ax, 0x10    ; Load kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    push esp        ; Push stack pointer for registers struct
    call exception_handler
    add esp, 4
    
    pop gs
    pop fs
    pop es
    pop ds
    
    popa
    add esp, 8      ; Clean up error code and ISR number
    iret

; Common IRQ stub - saves state and calls handler
extern irq_handler
irq_common_stub:
    pusha
    
    push ds
    push es
    push fs
    push gs
    
    mov ax, 0x10    ; Load kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    push esp        ; Push stack pointer
    call irq_handler
    add esp, 4
    
    pop gs
    pop fs
    pop es
    pop ds
    
    popa
    add esp, 8      ; Clean up IRQ number
    iret

; Macro for ISRs without error codes
%macro ISR_NOERRCODE 1
global isr%1
isr%1:
    cli
    push 0          ; Dummy error code
    push %1         ; Interrupt number
    jmp isr_common_stub
%endmacro

; Macro for ISRs with error codes
%macro ISR_ERRCODE 1
global isr%1
isr%1:
    cli
    push %1         ; Interrupt number (error code already pushed)
    jmp isr_common_stub
%endmacro

; Macro for IRQs
%macro IRQ 2
global irq%1
irq%1:
    cli
    push 0          ; Dummy error code
    push %2         ; IRQ number
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

