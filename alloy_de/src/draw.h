#ifndef _DRAW_H
#define _DRAW_H

#define SCREEN_W 1024
#define SCREEN_H 768

void put_pixel(unsigned int *fb, int x, int y, unsigned int color);
void fill_rect(unsigned int *fb, int x, int y, int w, int h, unsigned int c);
void draw_char(unsigned int *fb, int x, int y, unsigned char c,
               unsigned int fg, unsigned int bg);
void draw_str(unsigned int *fb, int x, int y, const char *s,
              unsigned int fg, unsigned int bg);

#endif
