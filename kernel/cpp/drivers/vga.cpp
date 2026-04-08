#include "vga.h"

// VGA text buffer address
static uint16_t* const VGA_BUFFER = (uint16_t*)0xB8000;

// VGA I/O ports for cursor control
#define VGA_CTRL_REGISTER 0x3D4
#define VGA_DATA_REGISTER 0x3D5
#define VGA_CURSOR_HIGH 14
#define VGA_CURSOR_LOW 15

// Current cursor position and color
static uint8_t cursor_x = 0;
static uint8_t cursor_y = 0;
static uint8_t current_color = 0x0F; // White on black

// Port I/O functions
static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

// Create a VGA entry (character + color)
static inline uint16_t vga_entry(char c, uint8_t color) {
    // Cast to uint8_t first to prevent sign extension of high-bit characters (Code Page 437)
    return (uint16_t)(uint8_t)c | ((uint16_t)color << 8);
}

// Update hardware cursor position
static void update_cursor() {
    uint16_t pos = cursor_y * VGA_WIDTH + cursor_x;
    
    outb(VGA_CTRL_REGISTER, VGA_CURSOR_HIGH);
    outb(VGA_DATA_REGISTER, (pos >> 8) & 0xFF);
    outb(VGA_CTRL_REGISTER, VGA_CURSOR_LOW);
    outb(VGA_DATA_REGISTER, pos & 0xFF);
}

// Scroll the screen up by one line
static void scroll() {
    // Move all lines up
    for (uint8_t y = 0; y < VGA_HEIGHT - 1; y++) {
        for (uint8_t x = 0; x < VGA_WIDTH; x++) {
            VGA_BUFFER[y * VGA_WIDTH + x] = VGA_BUFFER[(y + 1) * VGA_WIDTH + x];
        }
    }
    
    // Clear the last line
    for (uint8_t x = 0; x < VGA_WIDTH; x++) {
        VGA_BUFFER[(VGA_HEIGHT - 1) * VGA_WIDTH + x] = vga_entry(' ', current_color);
    }
}

// Initialize VGA driver
extern "C" void vga_init() {
    current_color = (VGA_COLOR_BLACK << 4) | VGA_COLOR_LIGHT_GREY;
    cursor_x = 0;
    cursor_y = 0;
    vga_clear();
}

// Clear the screen
extern "C" void vga_clear() {
    for (uint8_t y = 0; y < VGA_HEIGHT; y++) {
        for (uint8_t x = 0; x < VGA_WIDTH; x++) {
            VGA_BUFFER[y * VGA_WIDTH + x] = vga_entry(' ', current_color);
        }
    }
    cursor_x = 0;
    cursor_y = 0;
    update_cursor();
}

// Set foreground and background color
extern "C" void vga_set_color(uint8_t fg, uint8_t bg) {
    current_color = (bg << 4) | (fg & 0x0F);
}

// Set cursor position
extern "C" void vga_set_cursor(uint8_t x, uint8_t y) {
    if (x < VGA_WIDTH && y < VGA_HEIGHT) {
        cursor_x = x;
        cursor_y = y;
        update_cursor();
    }
}

// Get cursor X position
extern "C" uint8_t vga_get_cursor_x() {
    return cursor_x;
}

// Get cursor Y position
extern "C" uint8_t vga_get_cursor_y() {
    return cursor_y;
}

// Put a single character on the screen
extern "C" void vga_putchar(char c) {
    // Handle special characters
    if (c == '\n') {
        cursor_x = 0;
        cursor_y++;
    } else if (c == '\r') {
        cursor_x = 0;
    } else if (c == '\t') {
        cursor_x = (cursor_x + 8) & ~7; // Align to 8-column tab stops
    } else if (c == '\b') {
        if (cursor_x > 0) {
            cursor_x--;
        }
    } else {
        // Regular character
        VGA_BUFFER[cursor_y * VGA_WIDTH + cursor_x] = vga_entry(c, current_color);
        cursor_x++;
    }
    
    // Handle line wrap
    if (cursor_x >= VGA_WIDTH) {
        cursor_x = 0;
        cursor_y++;
    }
    
    // Handle scrolling
    if (cursor_y >= VGA_HEIGHT) {
        scroll();
        cursor_y = VGA_HEIGHT - 1;
    }
    
    update_cursor();
}

// Print a string to the screen
extern "C" void vga_print(const char* str) {
    if (!str) return;
    
    while (*str) {
        vga_putchar(*str);
        str++;
    }
}

// Print a string with a newline
extern "C" void vga_println(const char* str) {
    vga_print(str);
    vga_putchar('\n');
}

// Print a hex value
extern "C" void vga_print_hex(uint32_t value) {
    vga_print("0x");
    
    char hex_chars[] = "0123456789ABCDEF";
    char buffer[9];
    buffer[8] = '\0';
    
    for (int i = 7; i >= 0; i--) {
        buffer[i] = hex_chars[value & 0xF];
        value >>= 4;
    }
    
    vga_print(buffer);
}

// Print a decimal value
extern "C" void vga_print_dec(uint32_t value) {
    if (value == 0) {
        vga_putchar('0');
        return;
    }
    
    char buffer[12];
    int i = 0;
    
    while (value > 0) {
        buffer[i++] = '0' + (value % 10);
        value /= 10;
    }
    
    // Print in reverse
    while (i > 0) {
        vga_putchar(buffer[--i]);
    }
}
