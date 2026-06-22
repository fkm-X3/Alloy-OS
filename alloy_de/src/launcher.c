#include "launcher.h"
#include "draw.h"

#define CELL_W 100
#define CELL_H 100
#define GRID_COLS 4
#define GRID_SPACING 16
#define ICON_SIZE 64
#define ICON_RADIUS 12
#define LABEL_Y_OFFSET (ICON_SIZE + 8)

const struct app_entry launcher_apps[4] = {
    {"Terminal", ICON_TERMINAL},
    {"Files",    ICON_FILES},
    {"Settings", ICON_SETTINGS},
    {"Browser",  ICON_BROWSER},
};

const int launcher_apps_count = 4;

static void draw_launcher_bg(unsigned int *fb, int w, int h) {
    for (int y = 0; y < h; ++y) {
        for (int x = 0; x < w; ++x) {
            unsigned int c = 0xFF000000;
            unsigned char a = 0xD9;
            unsigned char inv_a = (unsigned char)(255 - a);
            unsigned int bg = fb[y * w + x];
            unsigned char r = (unsigned char)((((c >> 16) & 0xFF) * a
                         + ((bg >> 16) & 0xFF) * inv_a) / 255);
            unsigned char g = (unsigned char)((((c >> 8) & 0xFF) * a
                         + ((bg >> 8) & 0xFF) * inv_a) / 255);
            unsigned char b = (unsigned char)(((c & 0xFF) * a
                         + (bg & 0xFF) * inv_a) / 255);
            fb[y * w + x] = (0xFF << 24) | (r << 16) | (g << 8) | b;
        }
    }
}

static void draw_app_cell(unsigned int *fb, int cx, int cy,
                           const struct app_entry *app) {
    draw_rounded_rect(fb, cx + (CELL_W - ICON_SIZE) / 2,
                      cy + (CELL_H - ICON_SIZE - 16) / 2,
                      ICON_SIZE, ICON_SIZE, ICON_RADIUS, 0xFF2A2A4E);

    unsigned int icon_colors[4] = {
        0xFF00CC66, 0xFF6699FF, 0xFFFF9944, 0xFF44CCCC,
    };

    unsigned int ic = icon_colors[app->icon_id];
    draw_icon(fb, cx + (CELL_W - 32) / 2,
              cy + (CELL_H - ICON_SIZE - 16) / 2 + (ICON_SIZE - 32) / 2,
              app->icon_id, ic);

    int label_w = 0;
    while (app->name[label_w]) ++label_w;
    label_w *= 8;

    draw_str(fb, cx + (CELL_W - label_w) / 2,
             cy + (CELL_H - ICON_SIZE - 16) / 2 + ICON_SIZE + 8,
             app->name, 0xFFCCCCCC, 0x00000000);
}

void launcher_render(unsigned int *fb, int w, int h) {
    draw_launcher_bg(fb, w, h);

    int grid_w = GRID_COLS * CELL_W + (GRID_COLS - 1) * GRID_SPACING;
    int start_x = (w - grid_w) / 2;
    int start_y = (h - CELL_H) / 2;

    for (int i = 0; i < launcher_apps_count; ++i) {
        int col = i % GRID_COLS;
        int cx = start_x + col * (CELL_W + GRID_SPACING);
        int cy = start_y;
        draw_app_cell(fb, cx, cy, &launcher_apps[i]);
    }
}

const char *launcher_handle_click(int x, int y, int screen_w, int screen_h) {
    int grid_w = GRID_COLS * CELL_W + (GRID_COLS - 1) * GRID_SPACING;
    int start_x = (screen_w - grid_w) / 2;
    int start_y = (screen_h - CELL_H) / 2;

    for (int i = 0; i < launcher_apps_count; ++i) {
        int col = i % GRID_COLS;
        int cx = start_x + col * (CELL_W + GRID_SPACING);
        int cy = start_y;

        if (x >= cx && x < cx + CELL_W && y >= cy && y < cy + CELL_H)
            return launcher_apps[i].name;
    }

    return (const char *)0;
}
