#ifndef PL110_H
#define PL110_H

#include "boot/types.h"

void pl110_init(uint32_t fb_addr, uint16_t width, uint16_t height);
int pl110_is_available();
uint32_t pl110_get_framebuffer();
void pl110_get_resolution(uint32_t* width, uint32_t* height);
uint8_t pl110_get_bits_per_pixel();
void pl110_set_pixel(uint16_t x, uint16_t y, uint16_t color);
void pl110_fill_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint16_t color);

#endif
