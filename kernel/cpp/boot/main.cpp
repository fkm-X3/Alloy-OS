#include "multiboot2.h"
#include "types.h"

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
extern "C" char keyboard_get_char();

// Kernel entry point called from boot.asm
extern "C" void kernel_main(uint32_t magic, uint32_t multiboot_addr) {
    // Initialize serial port for early debugging
    init_serial();
    serial_print("Alloy Kernel Booting...\n");
    
    // Initialize VGA display
    vga_init();
    
    // Verify multiboot2 magic
    if (magic != MULTIBOOT2_BOOTLOADER_MAGIC) {
        serial_print("ERROR: Invalid multiboot magic number\n");
        vga_set_color(4, 0); // Red on black
        vga_println("ERROR: Invalid multiboot magic");
        while(1) {
            asm volatile("hlt");
        }
    }
    
    serial_print("Multiboot2 detected successfully\n");
    
    // Display boot banner
    vga_set_color(11, 0); // Light cyan
    vga_println("==============================");
    vga_println("    Alloy Operating System    ");
    vga_println("==============================");
    vga_set_color(7, 0); // Light grey
    vga_println("");
    
    // Initialize GDT (Global Descriptor Table)
    serial_print("Initializing GDT...\n");
    vga_print("[ ] Initializing GDT...");
    init_gdt();
    vga_set_color(10, 0); // Green
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("GDT initialized\n");
    
    // Initialize IDT (Interrupt Descriptor Table)
    serial_print("Initializing IDT...\n");
    vga_print("[ ] Initializing IDT...");
    init_idt();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("IDT initialized\n");
    
    // Initialize keyboard
    serial_print("Initializing keyboard...\n");
    vga_print("[ ] Initializing keyboard...");
    keyboard_init();
    vga_set_color(10, 0);
    vga_println(" OK");
    vga_set_color(7, 0);
    serial_print("Keyboard initialized\n");
    
    vga_println("");
    vga_set_color(10, 0);
    vga_println("Kernel initialization complete!");
    vga_set_color(7, 0);
    vga_println("");
    vga_println("Alloy OS is running.");
    vga_println("");
    vga_set_color(14, 0); // Yellow
    vga_println("Type something on the keyboard:");
    vga_set_color(7, 0);
    
    serial_print("Kernel initialization complete\n");
    serial_print("Alloy OS is running!\n");
    
    // Simple keyboard echo loop
    while(1) {
        char c = keyboard_get_char();
        vga_putchar(c);
        serial_print("Key pressed: ");
        char buf[2] = {c, '\0'};
        serial_print(buf);
        serial_print("\n");
    }
}
