/* Linker script for Alloy kernel */

ENTRY(start)

SECTIONS
{
    /* Kernel starts at 1MB physical address */
    . = 1M;
    
    /* Multiboot header must be in first 8KB */
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
    kernel_end = .;
}
