ENTRY(_start)

SECTIONS
{
    /* aarch64: the kernel runs with the MMU disabled (identity mapping), so
     * there is no separate virtual address space.  Userland binaries are
     * linked at this fixed physical RAM base (0x47A00000), which is reserved
     * from the PMM in pmm_init and loaded directly by spawn_user_elf. */
    . = 0x47A00000;

    .text : {
        *(.text)
        *(.text.*)
    }

    .rodata : {
        *(.rodata)
        *(.rodata.*)
    }

    .preinit_array : {
        PROVIDE_HIDDEN(__preinit_array_start = .);
        KEEP(*(.preinit_array))
        PROVIDE_HIDDEN(__preinit_array_end = .);
    }

    .init_array : {
        PROVIDE_HIDDEN(__init_array_start = .);
        KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)))
        KEEP(*(.init_array EXCLUDE_FILE(*crtbegin.o *crtbegin?.o *crtend.o *crtend?.o) .ctors))
        PROVIDE_HIDDEN(__init_array_end = .);
    }

    .fini_array : {
        PROVIDE_HIDDEN(__fini_array_start = .);
        KEEP(*(SORT_BY_INIT_PRIORITY(.fini_array.*) SORT_BY_INIT_PRIORITY(.dtors.*)))
        KEEP(*(.fini_array EXCLUDE_FILE(*crtbegin.o *crtbegin?.o *crtend.o *crtend?.o) .dtors))
        PROVIDE_HIDDEN(__fini_array_end = .);
    }

    .ctors : {
        /* gcc uses crtbegin.o to mark the start of the ctors section */
        KEEP(*(.ctors))
    }

    .dtors : {
        KEEP(*(.dtors))
    }

    .data : {
        *(.data)
        *(.data.*)
    }

    .bss : {
        *(COMMON)
        *(.bss)
        *(.bss.*)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame.*)
        *(.comment)
        *(.note.*)
        *(.note.gnu.build-id)
        *(.gnu.build-id)
    }
}
