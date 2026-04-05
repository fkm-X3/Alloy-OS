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

    ; Framebuffer tag (optional, for VGA text mode)
    align 8
framebuffer_tag_start:
    dw 5    ; type = framebuffer
    dw 0    ; flags
    dd framebuffer_tag_end - framebuffer_tag_start  ; size
    dd 80   ; width (80 columns)
    dd 25   ; height (25 rows)
    dd 0    ; depth (0 for text mode)
framebuffer_tag_end:

    ; End tag
    align 8
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
