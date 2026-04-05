; GDT flush function
; Loads the GDT pointer and reloads segment registers

section .text
global gdt_flush

gdt_flush:
    ; Load GDT pointer (passed in RDI on x86_64, but we're in 32-bit mode)
    ; In 32-bit, first argument is on stack at [esp+4]
    mov eax, [esp+4]
    lgdt [eax]
    
    ; Reload CS by doing a far jump
    jmp 0x08:.flush
.flush:
    ; Reload all data segment registers
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ret
