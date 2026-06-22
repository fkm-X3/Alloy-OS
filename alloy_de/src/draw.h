#ifndef _DRAW_H
#define _DRAW_H

#define SCREEN_W 1024
#define SCREEN_H 768

#define ICON_TERMINAL 0
#define ICON_FILES    1
#define ICON_SETTINGS 2
#define ICON_BROWSER  3

void put_pixel(unsigned int *fb, int x, int y, unsigned int color);
void fill_rect(unsigned int *fb, int x, int y, int w, int h, unsigned int c);
void draw_char(unsigned int *fb, int x, int y, unsigned char c,
               unsigned int fg, unsigned int bg);
void draw_str(unsigned int *fb, int x, int y, const char *s,
              unsigned int fg, unsigned int bg);
void draw_str_centered(unsigned int *fb, int y, const char *s,
                       unsigned int fg, unsigned int bg);
void draw_str_right(unsigned int *fb, int x_right, int y, const char *s,
                    unsigned int fg, unsigned int bg);
void draw_str_scaled(unsigned int *fb, int x, int y, const char *s,
                     unsigned int fg, unsigned int bg, int scale);
void draw_str_centered_scaled(unsigned int *fb, int y, const char *s,
                              unsigned int fg, unsigned int bg, int scale);
void draw_str_right_scaled(unsigned int *fb, int x_right, int y,
                           const char *s, unsigned int fg,
                           unsigned int bg, int scale);
void draw_rounded_rect(unsigned int *fb, int x, int y, int w, int h,
                       int r, unsigned int c);
void draw_icon(unsigned int *fb, int x, int y, int icon_id,
               unsigned int color);

#endif
