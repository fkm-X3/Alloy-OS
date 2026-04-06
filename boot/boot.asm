; Boot entry point for Alloy kernel
; This code is executed first by GRUB after loading the kernel

section .boot
bits 32                 ; Start in 32-bit protected mode

global start
extern kernel_main      ; C++ kernel entry point

; Constants
KERNEL_STACK_SIZE equ 16384
SERIAL_COM1 equ 0x3F8

start:
    ; GRUB has loaded us in 32-bit protected mode
    ; EAX = Multiboot2 magic (0x36d76289)
    ; EBX = Physical address of Multiboot2 information structure
    
    ; Initialize serial port early for debugging
    call init_serial_asm
    
    ; Print boot message
    mov esi, msg_boot
    call print_serial_asm
    
    ; Set up stack
    mov esp, stack_top
    mov ebp, esp
    
    ; Save multiboot info
    push ebx        ; Multiboot info pointer
    push eax        ; Multiboot magic
    
    ; Verify we booted via Multiboot2
    cmp eax, 0x36d76289
    jne .no_multiboot
    
    ; Print multiboot OK
    mov esi, msg_multiboot_ok
    call print_serial_asm
    
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
    ; Print error to serial
    mov esi, msg_no_multiboot
    call print_serial_asm
    
    ; Print 'E' to screen if not booted via multiboot
    mov dword [0xb8000], 0x4f524f45
    jmp .hang

; Initialize serial port COM1
init_serial_asm:
    push eax
    push edx
    
    mov dx, SERIAL_COM1 + 1
    xor al, al
    out dx, al              ; Disable interrupts
    
    mov dx, SERIAL_COM1 + 3
    mov al, 0x80
    out dx, al              ; Enable DLAB
    
    mov dx, SERIAL_COM1
    mov al, 0x03
    out dx, al              ; Set divisor low byte (38400 baud)
    
    mov dx, SERIAL_COM1 + 1
    xor al, al
    out dx, al              ; Set divisor high byte
    
    mov dx, SERIAL_COM1 + 3
    mov al, 0x03
    out dx, al              ; 8 bits, no parity, one stop bit
    
    mov dx, SERIAL_COM1 + 2
    mov al, 0xC7
    out dx, al              ; Enable FIFO
    
    mov dx, SERIAL_COM1 + 4
    mov al, 0x0B
    out dx, al              ; Enable IRQs
    
    pop edx
    pop eax
    ret

; Print null-terminated string to serial port
; ESI = pointer to string
print_serial_asm:
    push eax
    push edx
    push esi
    
.loop:
    lodsb                   ; Load byte from [ESI] into AL, increment ESI
    test al, al
    jz .done
    
    ; Wait for transmit buffer empty
.wait:
    mov dx, SERIAL_COM1 + 5
    in al, dx
    test al, 0x20
    jz .wait
    
    ; Send character
    mov dx, SERIAL_COM1
    mov al, [esi - 1]       ; Get character we just loaded
    out dx, al
    
    jmp .loop
    
.done:
    pop esi
    pop edx
    pop eax
    ret

section .rodata
msg_boot: db "[ASM] Boot entry point reached", 13, 10, 0
msg_multiboot_ok: db "[ASM] Multiboot2 magic verified", 13, 10, 0
msg_no_multiboot: db "[ASM] ERROR: Invalid Multiboot2 magic!", 13, 10, 0

section .bss
align 16
stack_bottom:
    resb KERNEL_STACK_SIZE
stack_top:
