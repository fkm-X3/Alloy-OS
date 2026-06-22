#ifndef _LAUNCHER_H
#define _LAUNCHER_H

struct app_entry {
    const char *name;
    int icon_id;
};

extern const struct app_entry launcher_apps[4];
extern const int launcher_apps_count;

void launcher_render(unsigned int *fb, int w, int h);
const char *launcher_handle_click(int x, int y, int screen_w, int screen_h);

#endif
