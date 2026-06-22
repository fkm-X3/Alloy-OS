#include "desktop.h"
#include "draw.h"

#define PANEL_H 48
#define CLOCK_MARGIN 24

static void format_time(char *buf, int seconds) {
    int hrs = (seconds / 3600) % 24;
    int mins = (seconds / 60) % 60;
    buf[0] = '0' + hrs / 10;
    buf[1] = '0' + hrs % 10;
    buf[2] = ':';
    buf[3] = '0' + mins / 10;
    buf[4] = '0' + mins % 10;
    buf[5] = '\0';
}

void desktop_render(unsigned int *fb, int w, int h, int seconds) {
    fill_rect(fb, 0, 0, w, h, 0xFF0F0F1A);

    desktop_render_clock(fb, w, h, seconds);
}

void desktop_render_clock(unsigned int *fb, int w, int h, int seconds) {
    char time_str[16];
    format_time(time_str, seconds);

    int clock_y = h - PANEL_H - CLOCK_MARGIN - 64;

    draw_str_right_scaled(fb, w - CLOCK_MARGIN, clock_y,
                          time_str, 0xFFFFFFFF, 0xFF0F0F1A, 8);

    draw_str_right_scaled(fb, w - CLOCK_MARGIN, clock_y + 64 + 4,
                          "Tuesday, June 16", 0xFFAAAAAA, 0xFF0F0F1A, 2);
}
