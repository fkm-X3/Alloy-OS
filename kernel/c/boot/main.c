#include "multiboot2.h"
#include "types.h"
#include "../mm/pmm.h"
#include "../mm/paging.h"
#include "../mm/vmm.h"
#include "../drivers/serial.h"
#include "../drivers/vga.h"
#include "../drivers/keyboard.h"
#include "../drivers/mouse.h"
#include "../drivers/timer.h"
#include "../drivers/vesa.h"
#include "../drivers/ata.h"
#include "../drivers/pci.h"
#include "../drivers/ahci.h"
#include "../drivers/initrd.h"

void init_gdt();
void init_idt();
void rust_main();

// PL110 framebuffer (aarch64)
void pl110_init(unsigned int fb_addr, unsigned short width, unsigned short height);
int pl110_is_available();

static void arch_halt() {
#ifdef ARCH_I686
    asm volatile("cli; hlt");
#elif defined(ARCH_X86_64)
    asm volatile("cli; hlt");
#elif defined(ARCH_AARCH64)
    asm volatile("msr daifset, #0b1111");
    asm volatile("wfi");
#else
    asm volatile("cli; hlt");
#endif
}

void kernel_main(uint32_t magic, uint32_t multiboot_addr) {
    init_serial();
    serial_print("DEBUG: serial_print test\n");
    serial_print("Alloy Kernel Booting...\n");

#ifdef ARCH_I686
    serial_print("Architecture: i686 (32-bit x86)\n");
#elif defined(ARCH_X86_64)
    serial_print("Architecture: x86_64 (64-bit x86)\n");
#elif defined(ARCH_AARCH64)
    serial_print("Architecture: aarch64 (64-bit ARM) [MINIMAL]\n");
#else
    serial_print("Architecture: unknown\n");
#endif

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_init();
#endif

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    if (magic != MULTIBOOT2_BOOTLOADER_MAGIC) {
        serial_print("ERROR: Invalid multiboot magic number\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
        vga_set_color(4, 0);
        vga_println("ERROR: Invalid multiboot magic");
#endif
        while(1) {
            arch_halt();
        }
    }

    serial_print("Multiboot2 detected successfully\n");

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(11, 0);
    vga_println("===============================");
    vga_println("    Alloy Kernel Bootloader    ");
    vga_println("===============================");
    vga_set_color(7, 0);
    vga_println("");
#endif
#else
    serial_print("ARM64 boot sequence\n");
#endif

    serial_print("Initializing GDT...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_print("[ ] Initializing GDT...");
#endif
    init_gdt();
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
#endif
    serial_print("GDT initialized\n");

    serial_print("Initializing IDT...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_print("[ ] Initializing IDT...");
#endif
    init_idt();
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
#endif
    serial_print("IDT initialized\n");

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    serial_print("Initializing keyboard...\n");
    vga_print("[ ] Initializing keyboard...");
    keyboard_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Keyboard initialized\n");

    serial_print("Initializing mouse...\n");
    vga_print("[ ] Initializing mouse...");
    if (mouse_init()) {
        vga_set_color(10, 0);
        vga_println(" OK");
        vga_set_color(7, 0);
        serial_print("Mouse initialized\n");
    } else {
        vga_set_color(14, 0);
        vga_println(" WARN");
        vga_set_color(7, 0);
        serial_print("Mouse initialization failed; continuing without mouse input\n");
    }
#else
    serial_print("Keyboard/mouse: HID/USB not yet implemented for ARM64\n");
#endif

    serial_print("Initializing timer...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_print("[ ] Initializing timer (100 Hz)...");
#endif
    timer_init_ffi(100);
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
#endif
    serial_print("Timer initialized\n");

    serial_print("Initializing physical memory manager...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_print("[ ] Initializing memory manager...");
#endif
    pmm_init(multiboot_addr);
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
#endif
    serial_print("PMM initialized\n");

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    serial_print("Initializing paging...\n");
    vga_print("[ ] Initializing paging...");
    paging_init();
    paging_enable();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Paging enabled\n");

    serial_print("Initializing virtual memory manager...\n");
    vga_print("[ ] Initializing VMM...");
    vmm_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("VMM initialized\n");

    serial_print("Initializing VESA graphics...\n");
    vga_print("[ ] Initializing VESA...");
    vesa_init_from_multiboot(multiboot_addr);
    if (vesa_is_available()) {
        vga_set_color(10, 0);
        vga_println(" OK");
        vga_set_color(7, 0);
        serial_print("[VESA] Graphics initialized successfully\n");
    } else {
        vga_set_color(14, 0);
        vga_println(" SKIP");
        vga_set_color(7, 0);
        serial_print("[VESA] Graphics not available (missing framebuffer metadata)\n");
    }
#elif defined(ARCH_AARCH64)
    serial_print("Initializing PL110 display...\n");
    pl110_init(0x48000000, 1024, 768);
    if (pl110_is_available()) {
        serial_print("[PL110] Display initialized\n");
    } else {
        serial_print("[PL110] Display not available\n");
    }
#endif

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_println("");
    vga_set_color(10, 0);
    vga_println("Kernel initialization complete!");
    vga_set_color(7, 0);
#endif
    serial_print("Kernel initialization complete\n");

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    serial_print("Initializing PCI bus...\n");
    vga_print("[ ] Initializing PCI bus...");
    pci_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("PCI initialized\n");

    serial_print("Initializing ATA PIO driver...\n");
    vga_print("[ ] Initializing ATA...");
    ata_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("ATA driver initialized\n");

    serial_print("Initializing AHCI driver...\n");
    vga_print("[ ] Initializing AHCI...");
    ahci_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("AHCI driver initialized\n");

    serial_print("Initializing initrd/ramdisk...\n");
    vga_print("[ ] Initializing initrd...");
    initrd_init(multiboot_addr);
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Initrd initialized\n");
#endif

    serial_print("Transferring control to Rust kernel...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_println("");
    vga_set_color(11, 0);
    vga_println("Transferring control to Rust kernel...");
    vga_set_color(7, 0);
#endif

    rust_main();

    serial_print("ERROR: Returned from Rust kernel!\n");
    while(1) {
        arch_halt();
    }
}
