/* Linker script for Alloy kernel (x86_64) */
/* TODO: This is a placeholder - x86_64 boot not yet implemented */

ENTRY(start)

SECTIONS
{
    /* Kernel starts at 2MB physical address (standard for x86_64) */
    . = 2M;
    
    _kernel_start = .;
    
    /* Multiboot2 header must be in first 8KB */
    .boot :
    {
        *(.multiboot_header)
        *(.boot)
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
        *(COMMON)
        *(.bss)
        *(.bss.*)
    }
    
    /* End of kernel */
    _kernel_end = .;
}
