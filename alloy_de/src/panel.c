#include "panel.h"
#include "draw.h"

#define BTN_SIZE 40
#define BTN_MARGIN 4

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

void panel_render(unsigned int *fb, int w, int h, int seconds) {
    int py = h - PANEL_HEIGHT;

    fill_rect(fb, 0, py, w, PANEL_HEIGHT, 0xFF1A1A2E);

    draw_rounded_rect(fb, BTN_MARGIN, py + BTN_MARGIN,
                      BTN_SIZE, BTN_SIZE, 6, 0xFF2A2A4E);

    draw_str_scaled(fb, BTN_MARGIN + 12, py + BTN_MARGIN + 8,
                    "\xBF", 0xFFFFFFFF, 0xFF2A2A4E, 2);

    int quit_x = w - BTN_MARGIN - BTN_SIZE;
    draw_rounded_rect(fb, quit_x, py + BTN_MARGIN,
                      BTN_SIZE, BTN_SIZE, 6, 0xFF4E1A1A);

    draw_str_scaled(fb, quit_x + 12, py + BTN_MARGIN + 10,
                    "x", 0xFFFF6666, 0xFF4E1A1A, 2);

    panel_render_clock(fb, w, h, seconds);
}

void panel_render_clock(unsigned int *fb, int w, int h, int seconds) {
    char time_str[16];
    format_time(time_str, seconds);

    int py = h - PANEL_HEIGHT;
    int clock_y = py + (PANEL_HEIGHT - 16) / 2;

    draw_str_right_scaled(fb, w - BTN_MARGIN - BTN_SIZE - BTN_MARGIN - 8,
                          clock_y, time_str, 0xFFFFFFFF, 0xFF1A1A2E, 2);
}

int panel_handle_click(int x, int y, int screen_w, int screen_h) {
    int py = screen_h - PANEL_HEIGHT;
    if (y < py || y >= screen_h) return PANEL_ACTION_NONE;

    if (x >= BTN_MARGIN && x < BTN_MARGIN + BTN_SIZE) {
        if (y >= py + BTN_MARGIN && y < py + BTN_MARGIN + BTN_SIZE) {
            return PANEL_ACTION_LAUNCHER;
        }
    }

    int quit_x = screen_w - BTN_MARGIN - BTN_SIZE;
    if (x >= quit_x && x < quit_x + BTN_SIZE) {
        if (y >= py + BTN_MARGIN && y < py + BTN_MARGIN + BTN_SIZE) {
            return PANEL_ACTION_QUIT;
        }
    }

    return PANEL_ACTION_NONE;
}
