; x86_64 GDT flush
; Loads the GDT pointer and reloads segment registers
; void gdt_flush(uint64_t gdt_ptr)

[BITS 64]

section .text
global gdt_flush

gdt_flush:
    ; First argument in RDI (x86_64 calling convention)
    lgdt [rdi]

    ; Reload CS by doing a far return
    push 0x08                    ; Kernel code segment
    lea rax, [rel .flush]
    push rax
    retfq

.flush:
    ; Reload data segment registers
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ret
