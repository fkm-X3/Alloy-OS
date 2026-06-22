#ifndef _PANEL_H
#define _PANEL_H

#define PANEL_HEIGHT 48

#define PANEL_ACTION_NONE     0
#define PANEL_ACTION_LAUNCHER 1
#define PANEL_ACTION_QUIT     2

void panel_render(unsigned int *fb, int w, int h, int seconds);
void panel_render_clock(unsigned int *fb, int w, int h, int seconds);
int panel_handle_click(int x, int y, int screen_w, int screen_h);

#endif
