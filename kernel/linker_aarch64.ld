/* Linker script for Alloy kernel (aarch64) */
/* For QEMU virt machine - loaded at 0x40080000 */

ENTRY(_start)

SECTIONS
{
    /* Kernel loaded at 0x40080000 (QEMU virt machine default) */
    . = 0x40080000;
    
    _kernel_start = .;
    
    /* Boot section - exception vectors must be first and 2KB aligned */
    .boot : ALIGN(2K)
    {
        *(.boot)
        *(.text.exception_vectors)
    }
    
    /* Read-only sections */
    .text ALIGN(4K) : AT(ADDR(.text))
    {
        *(.text)
        *(.text.*)
    }
    
    .rodata ALIGN(4K) : AT(ADDR(.rodata))
    {
        *(.rodata)
        *(.rodata.*)
    }
    
    /* Read-write sections */
    .data ALIGN(4K) : AT(ADDR(.data))
    {
        *(.data)
        *(.data.*)
    }
    
    .bss ALIGN(4K) : AT(ADDR(.bss))
    {
        _bss_start = .;
        *(COMMON)
        *(.bss)
        *(.bss.*)
        _bss_end = .;
    }
    
    /* End of kernel */
    _kernel_end = .;
    
    /* Stack grows down from here (just below PL110 framebuffer at 0x48000000) */
    _stack_top = 0x47FFF000;
}
