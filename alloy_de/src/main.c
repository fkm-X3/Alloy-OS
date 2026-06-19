#include "draw.h"
#include "shm.h"
#include "wl_client.h"
#include "stdio.h"
#include "stdlib.h"
#include "alloy_syscall.h"

#define NULL ((void*)0)

typedef __SIZE_TYPE__ size_t;

#define MAX_POLL_FDS 8

typedef void (*poll_callback_t)(void *userdata);

struct poll_fd {
    int fd;
    poll_callback_t callback;
    void *userdata;
    int active;
};

static struct poll_fd poll_fds[MAX_POLL_FDS];
static int num_poll_fds = 0;
static int running = 1;

static void poll_init(void) {
    num_poll_fds = 0;
}

static int poll_add_fd(int fd, poll_callback_t cb, void *userdata) {
    if (num_poll_fds >= MAX_POLL_FDS) return -1;
    poll_fds[num_poll_fds].fd = fd;
    poll_fds[num_poll_fds].callback = cb;
    poll_fds[num_poll_fds].userdata = userdata;
    poll_fds[num_poll_fds].active = 1;
    num_poll_fds++;
    return 0;
}

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

static void on_key(int key, int pressed, struct input_state *state) {
    (void)state;
    if (pressed) {
        if (key == 1) { /* Escape */
            puts("alloy_de: escape pressed");
        }
    }
}

static void on_mouse_move(int x, int y, struct input_state *state) {
    (void)state;
    (void)x;
    (void)y;
}

static void on_click(int button, int pressed, int x, int y,
                     struct input_state *state) {
    (void)button;
    (void)pressed;
    (void)x;
    (void)y;
    (void)state;
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

static void draw_desktop(unsigned int *fb) {
    fill_rect(fb, 0, 0, SCREEN_W, SCREEN_H, 0xFF0F0F1A);
    fill_rect(fb, 0, 0, SCREEN_W, 48, 0xFF1A1A2E);
    draw_str(fb, 12, 16, "Alloy DE", 0xFF888899, 0xFF1A1A2E);
    draw_str(fb, SCREEN_W / 2 - 40, SCREEN_H / 2 - 8,
             "Alloy OS", 0xFFFFFFFF, 0xFF0F0F1A);
    draw_str(fb, SCREEN_W / 2 - 72, SCREEN_H / 2 + 10,
             "Press Meta to launch apps", 0xFFAAAAAA, 0xFF0F0F1A);
}

static void on_wl_event(void *userdata) {
    (void)userdata;
    int n = wl_display_dispatch_pending(g_display);
    if (n <= 0) {
        puts("alloy_de: server disconnected");
        running = 0;
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

    draw_desktop(fb);
    puts("alloy_de: desktop drawn");

    wl_surface_attach(g_display->fd, surface_id, (unsigned int)shm_fd, 0, 0);
    wl_surface_damage(g_display->fd, surface_id, 0, 0, SCREEN_W, SCREEN_H);

    if (wl_surface_commit(g_display->fd, surface_id) < 0) {
        puts("alloy_de: commit failed");
        return 1;
    }
    puts("alloy_de: surface committed");

    wl_set_key_callback(g_display, on_key);
    wl_set_mouse_move_callback(g_display, on_mouse_move);
    wl_set_click_callback(g_display, on_click);

    poll_init();
    poll_add_fd(wl_display_get_fd(g_display), on_wl_event, NULL);

    puts("alloy_de: entering event loop");
    while (running) {
        on_wl_event(NULL);
        syscall(SYS_YIELD, 0, 0, 0, 0, 0);
    }

    wl_display_disconnect(g_display);
    puts("alloy_de: exiting");
    syscall(SYS_EXIT, 0, 0, 0, 0, 0);
    return 0;
}
