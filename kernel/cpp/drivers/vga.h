#ifndef ALLOY_VGA_H
#define ALLOY_VGA_H

#include "boot/types.h"

// VGA text mode dimensions
#define VGA_WIDTH 80
#define VGA_HEIGHT 25

// VGA color codes
enum vga_color {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_BLUE = 1,
    VGA_COLOR_GREEN = 2,
    VGA_COLOR_CYAN = 3,
    VGA_COLOR_RED = 4,
    VGA_COLOR_MAGENTA = 5,
    VGA_COLOR_BROWN = 6,
    VGA_COLOR_LIGHT_GREY = 7,
    VGA_COLOR_DARK_GREY = 8,
    VGA_COLOR_LIGHT_BLUE = 9,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_LIGHT_CYAN = 11,
    VGA_COLOR_LIGHT_RED = 12,
    VGA_COLOR_LIGHT_MAGENTA = 13,
    VGA_COLOR_LIGHT_BROWN = 14,
    VGA_COLOR_WHITE = 15,
};

// VGA functions
extern "C" void vga_init();
extern "C" void vga_clear();
extern "C" void vga_putchar(char c);
extern "C" void vga_print(const char* str);
extern "C" void vga_set_color(uint8_t fg, uint8_t bg);
extern "C" void vga_set_cursor(uint8_t x, uint8_t y);

#endif // ALLOY_VGA_H
