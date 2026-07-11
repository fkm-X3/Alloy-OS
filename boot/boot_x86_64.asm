; x86_64 boot entry point for Alloy kernel
; GRUB loads us in 32-bit protected mode (Multiboot2)
; We transition to long mode and call kernel_main

section .boot
bits 32

global start
extern kernel_main

KERNEL_STACK_SIZE equ 16384

; Page table structures for early long-mode transition
; PML4 at a known location (0x1000, within first MB)
PML4_BASE   equ 0x1000
PDPT_BASE   equ 0x2000
PD_BASE     equ 0x3000

start:
    ; Save multiboot info
    mov [mb_magic], eax
    mov [mb_info], ebx

    ; Initialize serial port early for debugging
    call init_serial_asm

    mov esi, msg_boot
    call print_serial_asm

    ; Verify multiboot magic
    cmp eax, 0x36d76289
    jne .no_multiboot

    mov esi, msg_multiboot_ok
    call print_serial_asm

    ; Check for CPUID support
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd
    cmp eax, ecx
    je .no_cpuid

    ; Check for long mode support via CPUID
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode

    ; Set up page tables for long mode
    call setup_page_tables

    ; Enable PAE (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    ; Enable SSE/SSE2: OSFXSR (bit 9) + OSXMMEXCPT (bit 10)
    or eax, (1 << 9) | (1 << 10)
    mov cr4, eax

    ; Initialize x87 FPU
    finit

    ; Load PML4 address into CR3
    mov eax, PML4_BASE
    mov cr3, eax

    ; Enable long mode in EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8    ; LME = Long Mode Enable
    wrmsr

    ; Enable paging (and thus enter compatibility mode)
    mov eax, cr0
    or eax, 1 << 31   ; PG = Paging enable
    or eax, 1 << 16   ; WP = Write protect
    mov cr0, eax

    ; Load 64-bit GDT (far jump to switch to long mode)
    lgdt [gdtp]
    jmp 0x08:long_mode_entry

.no_multiboot:
    mov esi, msg_no_multiboot
    call print_serial_asm
    mov dword [0xb8000], 0x4f524f45
    jmp .hang

.no_cpuid:
    mov esi, msg_no_cpuid
    call print_serial_asm
    jmp .hang

.no_long_mode:
    mov esi, msg_no_long_mode
    call print_serial_asm
    jmp .hang

.hang:
    cli
.hang_loop:
    hlt
    jmp .hang_loop

; Set up identity-mapped page tables for the first 4MB
; PML4[0] -> PDPT[0] -> PD[0..1] -> 2MB pages identity mapping
setup_page_tables:
    ; Zero out page table region
    mov edi, PML4_BASE
    mov cr3, edi        ; temporary CR3 for zeroing
    xor eax, eax
    mov ecx, 0x3000 / 4 ; Zero 12KB (PML4 + PDPT + PD)
    rep stosd

    ; PML4 entry 0 -> PDPT at 0x2000 (present, writable)
    mov dword [PML4_BASE + 0], PDPT_BASE | 0x03
    mov dword [PML4_BASE + 4], 0

    ; PDPT entry 0 -> PD at 0x3000 (present, writable)
    mov dword [PDPT_BASE + 0], PD_BASE | 0x03
    mov dword [PDPT_BASE + 4], 0

    ; PD entries 0-6: 2MB pages identity mapping (covers ~14MB)
    ; PD entry 0: 0x000000 -> 0x1FFFFF (2MB)
    mov dword [PD_BASE + 0], 0x000000 | 0x83  ; present, writable, huge page
    mov dword [PD_BASE + 4], 0x000000

    ; PD entry 1: 0x200000 -> 0x3FFFFF (2MB)
    mov dword [PD_BASE + 8], 0x200000 | 0x83
    mov dword [PD_BASE + 12], 0x000000

    ; PD entry 2: 0x400000 -> 0x5FFFFF (2MB)
    mov dword [PD_BASE + 16], 0x400000 | 0x83
    mov dword [PD_BASE + 20], 0x000000

    ; PD entry 3: 0x600000 -> 0x7FFFFF (2MB)
    mov dword [PD_BASE + 24], 0x600000 | 0x83
    mov dword [PD_BASE + 28], 0x000000

    ; PD entry 4: 0x800000 -> 0x9FFFFF (2MB)
    mov dword [PD_BASE + 32], 0x800000 | 0x83
    mov dword [PD_BASE + 36], 0x000000

    ; PD entry 5: 0xA00000 -> 0xBFFFFF (2MB)
    mov dword [PD_BASE + 40], 0xA00000 | 0x83
    mov dword [PD_BASE + 44], 0x000000

    ; PD entry 6: 0xC00000 -> 0xDFFFFF (2MB)
    mov dword [PD_BASE + 48], 0xC00000 | 0x83
    mov dword [PD_BASE + 52], 0x000000

    ; PD entry 7: 0xE00000 -> 0xFFFFFF (2MB)
    mov dword [PD_BASE + 56], 0xE00000 | 0x83
    mov dword [PD_BASE + 60], 0x000000

    ret

bits 64

long_mode_entry:
    ; Now in 64-bit long mode
    mov ax, 0x10       ; Kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Set up stack
    mov rsp, stack_top
    and rsp, -16        ; Ensure 16-byte stack alignment for x86_64 ABI
    mov rbp, rsp

    mov esi, msg_long_mode
    call print_serial_asm_64

    ; Clear direction flag
    cld

    ; Call kernel_main(magic, mb_info)
    mov rdi, [mb_magic]
    mov rsi, [mb_info]
    call kernel_main

    ; If kernel_main returns, halt
    cli
.hang_64:
    hlt
    jmp .hang_64

; 64-bit serial print helper
print_serial_asm_64:
    push rax
    push rdx
    push rsi
    push rcx
.loop:
    lodsb
    test al, al
    jz .done
.wait:
    mov dx, 0x3F8 + 5
    in al, dx
    test al, 0x20
    jz .wait
    mov dx, 0x3F8
    mov al, [rsi - 1]
    out dx, al
    jmp .loop
.done:
    pop rcx
    pop rsi
    pop rdx
    pop rax
    ret

; Rest of boot code in 32-bit for initial setup
bits 32

init_serial_asm:
    push eax
    push edx

    mov dx, 0x3F8 + 1
    xor al, al
    out dx, al

    mov dx, 0x3F8 + 3
    mov al, 0x80
    out dx, al

    mov dx, 0x3F8
    mov al, 0x03
    out dx, al

    mov dx, 0x3F8 + 1
    xor al, al
    out dx, al

    mov dx, 0x3F8 + 3
    mov al, 0x03
    out dx, al

    mov dx, 0x3F8 + 2
    mov al, 0xC7
    out dx, al

    mov dx, 0x3F8 + 4
    mov al, 0x0B
    out dx, al

    pop edx
    pop eax
    ret

print_serial_asm:
    push eax
    push edx
    push esi
.loop32:
    lodsb
    test al, al
    jz .done32
.wait32:
    mov dx, 0x3F8 + 5
    in al, dx
    test al, 0x20
    jz .wait32
    mov dx, 0x3F8
    mov al, [esi - 1]
    out dx, al
    jmp .loop32
.done32:
    pop esi
    pop edx
    pop eax
    ret

section .data
align 16
; GDT for long mode: null, kernel code 64-bit, kernel data, user code 64-bit, user data
gdt:
    dq 0x0000000000000000 ; Null descriptor
    dq 0x00209A0000000000 ; Kernel code (64-bit, DPL=0, L=1)
    dq 0x0000920000000000 ; Kernel data (DPL=0)
    dq 0x0020FA0000000000 ; User code (64-bit, DPL=3, L=1)
    dq 0x0000F20000000000 ; User data (DPL=3)
gdt_end:

gdtp:
    dw gdt_end - gdt - 1
    dq gdt

section .bss
align 16
mb_magic: resd 1
mb_info:  resd 1
stack_bottom:
    resb KERNEL_STACK_SIZE
stack_top:

section .rodata
msg_boot:       db "[ASM] x86_64 boot entry reached", 13, 10, 0
msg_multiboot_ok: db "[ASM] Multiboot2 magic verified", 13, 10, 0
msg_no_multiboot: db "[ASM] ERROR: Invalid Multiboot2 magic!", 13, 10, 0
msg_no_cpuid:   db "[ASM] ERROR: No CPUID support!", 13, 10, 0
msg_no_long_mode: db "[ASM] ERROR: No long mode support!", 13, 10, 0
msg_long_mode:  db "[ASM] Entered 64-bit long mode", 13, 10, 0
