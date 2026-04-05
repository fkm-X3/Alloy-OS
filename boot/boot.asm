; Boot entry point for Alloy kernel
; This code is executed first by GRUB after loading the kernel

section .boot
bits 32                 ; Start in 32-bit protected mode

global start
extern kernel_main      ; C++ kernel entry point

; Constants
KERNEL_STACK_SIZE equ 16384

start:
    ; GRUB has loaded us in 32-bit protected mode
    ; EAX = Multiboot2 magic (0x36d76289)
    ; EBX = Physical address of Multiboot2 information structure
    
    ; Set up stack
    mov esp, stack_top
    mov ebp, esp
    
    ; Save multiboot info
    push ebx        ; Multiboot info pointer
    push eax        ; Multiboot magic
    
    ; Verify we booted via Multiboot2
    cmp eax, 0x36d76289
    jne .no_multiboot
    
    ; Clear direction flag
    cld
    
    ; Call C++ kernel main
    call kernel_main
    
    ; If kernel_main returns, halt
    cli
.hang:
    hlt
    jmp .hang

.no_multiboot:
    ; Print 'E' to screen if not booted via multiboot
    mov dword [0xb8000], 0x4f524f45
    jmp .hang

section .bss
align 16
stack_bottom:
    resb KERNEL_STACK_SIZE
stack_top:
