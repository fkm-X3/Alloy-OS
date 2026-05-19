#include "multiboot2.h"
#include "types.h"
#include "../mm/pmm.h"
#include "../mm/paging.h"
#include "../mm/vmm.h"

// Forward declarations
extern "C" void init_gdt();
extern "C" void init_idt();
extern "C" void init_serial();
extern "C" void serial_print(const char* str);
extern "C" void vga_init();
extern "C" void vga_print(const char* str);
extern "C" void vga_println(const char* str);
extern "C" void vga_set_color(uint8_t fg, uint8_t bg);
extern "C" void vga_putchar(char c);
extern "C" void keyboard_init();
extern "C" bool mouse_init();
extern "C" char keyboard_get_char();
extern "C" void timer_init_ffi(uint32_t frequency);
extern "C" void vesa_init();
extern "C" void vesa_init_from_multiboot(uint32_t multiboot_addr);
extern "C" uint8_t vesa_is_available();

// Rust kernel entry point
extern "C" void rust_main();

// Architecture-specific halt
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

// Kernel entry point called from boot.asm
extern "C" void kernel_main(uint32_t magic, uint32_t multiboot_addr) {
    // Initialize serial port for early debugging
    init_serial();
    serial_print("Alloy Kernel Booting...\n");

#ifdef ARCH_I686
    serial_print("Architecture: i686 (32-bit x86)\n");
#elif defined(ARCH_X86_64)
    serial_print("Architecture: x86_64 (64-bit x86) [PLACEHOLDER]\n");
#elif defined(ARCH_AARCH64)
    serial_print("Architecture: aarch64 (64-bit ARM) [MINIMAL]\n");
#else
    serial_print("Architecture: unknown\n");
#endif

    // Initialize VGA display (x86 only)
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_init();
#endif

    // Verify multiboot2 magic (x86 only)
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

    // Display boot banner
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(11, 0);
    vga_println("===============================");
    vga_println("    Alloy Kernel Bootloader    ");
    vga_println("===============================");
    vga_set_color(7, 0);
    vga_println("");
#endif
#else
    // ARM64: No multiboot, loaded directly by bootloader/UEFI
    serial_print("ARM64 boot sequence\n");
#endif

    // Initialize GDT (Global Descriptor Table)
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

    // Initialize IDT (Interrupt Descriptor Table)
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

    // Initialize keyboard (x86 only)
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    serial_print("Initializing keyboard...\n");
    vga_print("[ ] Initializing keyboard...");
    keyboard_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Keyboard initialized\n");

    // Initialize PS/2 mouse (x86 only)
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

    // Initialize timer
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

    // Initialize physical memory manager
    serial_print("Initializing physical memory manager...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_print("[ ] Initializing memory manager...");
#endif
    g_pmm.init(multiboot_addr);
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
#endif
    serial_print("Physical memory manager initialized\n");

    // Initialize paging (x86 only for now)
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    serial_print("Initializing paging...\n");
    vga_print("[ ] Initializing paging...");
    g_paging.init();
    g_paging.enable();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Paging enabled\n");

    // Initialize virtual memory manager
    serial_print("Initializing virtual memory manager...\n");
    vga_print("[ ] Initializing VMM...");
    g_vmm.init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Virtual memory manager initialized\n");

    // Initialize VESA graphics
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
#else
    serial_print("Paging/VESA: Not yet implemented for ARM64\n");
#endif

#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_println("");
    vga_set_color(10, 0);
    vga_println("Kernel initialization complete!");
    vga_set_color(7, 0);
#endif
    serial_print("C++ kernel initialization complete\n");

    // Hand off to Rust kernel
    serial_print("Transferring control to Rust kernel...\n");
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    vga_println("");
    vga_set_color(11, 0);
    vga_println("Transferring control to Rust kernel...");
    vga_set_color(7, 0);
#endif

    rust_main();

    // Should never reach here
    serial_print("ERROR: Returned from Rust kernel!\n");
    while(1) {
        arch_halt();
    }
}
