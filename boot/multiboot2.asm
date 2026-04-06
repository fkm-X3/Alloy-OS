; Multiboot2 header for GRUB bootloader
; This file provides the boot header that GRUB recognizes

section .multiboot_header
header_start:
    ; Multiboot2 magic number
    dd 0xe85250d6
    
    ; Architecture: i386 (0 = 32-bit protected mode)
    dd 0
    
    ; Header length
    dd header_end - header_start
    
    ; Checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; End tag (required)
    align 8
    dw 0    ; type = end tag
    dw 0    ; flags
    dd 8    ; size
header_end:
