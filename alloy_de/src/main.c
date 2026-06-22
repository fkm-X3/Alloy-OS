#include "draw.h"
#include "desktop.h"
#include "panel.h"
#include "launcher.h"
#include "shm.h"
#include "wl_client.h"
#include "stdio.h"
#include "stdlib.h"
#include "alloy_syscall.h"

#define NULL ((void*)0)

typedef __SIZE_TYPE__ size_t;

static int running = 1;
static int launcher_visible = 0;
static int g_seconds = 0;

struct wl_display *g_display = NULL;

static unsigned int compositor_obj = 0;
static unsigned int shm_obj = 0;
static unsigned int seat_obj = 0;
static unsigned int surface_id = 0;
static int got_compositor = 0;
static int got_shm = 0;
static int got_seat = 0;
static int shm_fd = -1;
static unsigned int *fb = NULL;

static int strequal(const char *a, const char *b) {
    while (*a && *b) {
        if (*a != *b) return 0;
        ++a; ++b;
    }
    return *a == *b;
}

static void handle_global(unsigned int name, const char *iface,
                          unsigned int version) {
    (void)version;
    if (strequal(iface, "wl_compositor") && !got_compositor) {
        compositor_obj = name;
        got_compositor = 1;
    } else if (strequal(iface, "wl_shm") && !got_shm) {
        shm_obj = name;
        got_shm = 1;
    } else if (strequal(iface, "wl_seat") && !got_seat) {
        seat_obj = name;
        got_seat = 1;
    }
}

static void full_redraw(void) {
    desktop_render(fb, SCREEN_W, SCREEN_H, g_seconds);
    panel_render(fb, SCREEN_W, SCREEN_H, g_seconds);
    if (launcher_visible)
        launcher_render(fb, SCREEN_W, SCREEN_H);
}

static void commit_full(void) {
    wl_surface_damage(g_display->fd, surface_id, 0, 0, SCREEN_W, SCREEN_H);
    wl_surface_commit(g_display->fd, surface_id);
}

static void on_key(int key, int pressed, struct input_state *state) {
    (void)state;
    if (!pressed) return;

    if (key == 1) {
        if (launcher_visible) {
            launcher_visible = 0;
            full_redraw();
            commit_full();
            puts("alloy_de: launcher closed (Escape)");
        }
    } else if (key == 125 || key == 127) {
        launcher_visible = !launcher_visible;
        full_redraw();
        commit_full();
        puts(launcher_visible ? "alloy_de: launcher opened" : "alloy_de: launcher closed");
    }
}

static void on_mouse_move(int x, int y, struct input_state *state) {
    (void)state;
    (void)x;
    (void)y;
}

static void on_click(int button, int pressed, int x, int y,
                     struct input_state *state) {
    (void)state;
    if (!pressed) return;

    if (launcher_visible) {
        const char *app = launcher_handle_click(x, y, SCREEN_W, SCREEN_H);
        if (app) {
            puts(app);
            launcher_visible = 0;
            full_redraw();
            commit_full();
        } else {
            launcher_visible = 0;
            full_redraw();
            commit_full();
            puts("alloy_de: launcher closed (click outside)");
        }
        return;
    }

    int action = panel_handle_click(x, y, SCREEN_W, SCREEN_H);
    if (action == PANEL_ACTION_LAUNCHER) {
        launcher_visible = !launcher_visible;
        full_redraw();
        commit_full();
        puts(launcher_visible ? "alloy_de: launcher opened" : "alloy_de: launcher closed");
    } else if (action == PANEL_ACTION_QUIT) {
        puts("alloy_de: quit requested");
        running = 0;
    }
}

static void dispatch_globals(void) {
    unsigned char buf[WL_MAX_MESSAGE_SIZE];
    int n = wl_message_receive(g_display->fd, buf);
    if (n <= 0) return;
    struct wl_wire_header *hdr = (struct wl_wire_header *)buf;
    if (hdr->object_id == WL_REGISTRY_ID && hdr->opcode == WL_REGISTRY_GLOBAL) {
        unsigned char *p = buf + WL_MESSAGE_HEADER_SIZE;
        unsigned int name, iface_len, version;
        char iface[68];
        iface_len = *(unsigned int *)(p + 4);
        if (iface_len > 64) return;
        name = *(unsigned int *)p;
        for (unsigned int i = 0; i < iface_len; ++i)
            iface[i] = p[8 + i];
        iface[iface_len] = 0;
        unsigned int off = 8 + ((iface_len + 3) & ~3u);
        version = *(unsigned int *)(p + off);
        handle_global(name, iface, version);
    }
}

int main(void) {
    puts("alloy_de: starting up");

    g_display = wl_display_connect("/tmp/wayland-0");
    if (!g_display) {
        puts("alloy_de: connection failed");
        return 1;
    }
    puts("alloy_de: connected");

    if (wl_display_roundtrip(g_display) < 0) {
        puts("alloy_de: sync failed");
        return 1;
    }

    struct wl_registry *registry = wl_display_get_registry(g_display);
    if (!registry) {
        puts("alloy_de: get_registry failed");
        return 1;
    }

    int timeout = 200;
    while ((!got_compositor || !got_shm || !got_seat) && timeout > 0) {
        dispatch_globals();
        --timeout;
    }

    if (!got_compositor || !got_shm) {
        puts("alloy_de: missing required globals");
        return 1;
    }
    puts("alloy_de: globals acquired");

    compositor_obj = wl_registry_bind_generic(registry, compositor_obj,
                                               "wl_compositor", 1);
    if (!compositor_obj) {
        puts("alloy_de: bind compositor failed");
        return 1;
    }
    puts("alloy_de: compositor bound");

    surface_id = wl_compositor_create_surface(registry, compositor_obj);
    if (!surface_id) {
        puts("alloy_de: create_surface failed");
        return 1;
    }
    puts("alloy_de: surface created");

    if (got_seat) {
        unsigned int seat_id = wl_seat_bind(registry, seat_obj, 1);
        if (seat_id) {
            puts("alloy_de: seat bound");
            wl_seat_get_keyboard(g_display, seat_id);
            wl_seat_get_pointer(g_display, seat_id);
            puts("alloy_de: input devices acquired");
        }
    }

    puts("alloy_de: allocating SHM buffer");
    shm_fd = alloy_shm_alloc(SCREEN_W, SCREEN_H, 32);
    if (shm_fd < 0) {
        puts("alloy_de: shm_alloc failed");
        return 1;
    }

    fb = (unsigned int *)alloy_shm_user_vaddr(shm_fd);
    if (!fb) {
        puts("alloy_de: shm_user_vaddr failed");
        return 1;
    }
    puts("alloy_de: SHM buffer ready");

    full_redraw();
    puts("alloy_de: desktop drawn");

    wl_surface_attach(g_display->fd, surface_id, (unsigned int)shm_fd, 0, 0);

    if (wl_surface_commit(g_display->fd, surface_id) < 0) {
        puts("alloy_de: commit failed");
        return 1;
    }
    puts("alloy_de: surface committed");

    wl_set_key_callback(g_display, on_key);
    wl_set_mouse_move_callback(g_display, on_mouse_move);
    wl_set_click_callback(g_display, on_click);

    puts("alloy_de: entering event loop");
    int tick_count = 0;
    int last_sec = 0;
    while (running) {
        int n = wl_display_dispatch_pending(g_display);
        if (n < 0) {
            puts("alloy_de: server disconnected");
            break;
        }

        tick_count++;
        g_seconds = tick_count / 20;

        if (g_seconds != last_sec) {
            last_sec = g_seconds;

            desktop_render_clock(fb, SCREEN_W, SCREEN_H, g_seconds);
            panel_render_clock(fb, SCREEN_W, SCREEN_H, g_seconds);

            wl_surface_damage(g_display->fd, surface_id,
                              SCREEN_W - 400, SCREEN_H - PANEL_HEIGHT - 100,
                              400, 100);
            wl_surface_damage(g_display->fd, surface_id,
                              SCREEN_W - 200, SCREEN_H - PANEL_HEIGHT,
                              200, PANEL_HEIGHT);
            wl_surface_commit(g_display->fd, surface_id);
        }

        syscall(SYS_SLEEP, 50, 0, 0, 0, 0);
    }

    wl_display_disconnect(g_display);
    puts("alloy_de: exiting");
    syscall(SYS_EXIT, 0, 0, 0, 0, 0);
    return 0;
}
