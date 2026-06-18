#include "draw.h"
#include "shm.h"
#include "wayland_client.h"
#include "stdio.h"
#include "stdlib.h"
#include "alloy_syscall.h"

#define NULL ((void*)0)

typedef __SIZE_TYPE__ size_t;

struct wl_display *g_display = NULL;

static unsigned int compositor_obj = 0;
static unsigned int shm_obj = 0;
static int got_compositor = 0;
static int got_shm = 0;

static int strequal(const char *a, const char *b) {
    while (*a && *b) {
        if (*a != *b) return 0;
        ++a; ++b;
    }
    return *a == *b;
}

static void mymemcpy(void *dst, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    for (size_t i = 0; i < n; ++i) d[i] = s[i];
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
    }
}

static void dispatch_events(void) {
    unsigned char buf[4096];
    while (1) {
        int n = wl_message_receive(g_display->fd, buf);
        if (n <= 0) break;
        struct wl_wire_header *hdr = (struct wl_wire_header *)buf;
        if (hdr->object_id == WL_REGISTRY_ID && hdr->opcode == WL_REGISTRY_GLOBAL) {
            unsigned char *p = buf + WL_MESSAGE_HEADER_SIZE;
            unsigned int name, iface_len, version;
            mymemcpy(&name, p, 4);
            mymemcpy(&iface_len, p + 4, 4);
            if (iface_len > 64) continue;
            char iface[68];
            mymemcpy(iface, p + 8, iface_len);
            iface[iface_len] = 0;
            unsigned int off = 8 + ((iface_len + 3) & ~3u);
            mymemcpy(&version, p + off, 4);
            handle_global(name, iface, version);
        }
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
    while ((!got_compositor || !got_shm) && timeout > 0) {
        dispatch_events();
        --timeout;
    }

    if (!got_compositor || !got_shm) {
        puts("alloy_de: missing required globals");
        return 1;
    }
    puts("alloy_de: globals acquired");

    unsigned int comp_id = 3;
    comp_id = wl_registry_bind(registry, compositor_obj,
                                "wl_compositor", 1, comp_id);
    if (!comp_id) {
        puts("alloy_de: bind compositor failed");
        return 1;
    }
    compositor_obj = comp_id;
    puts("alloy_de: compositor bound");

    unsigned int surface_id = 4;
    {
        unsigned char payload[4];
        mymemcpy(payload, &surface_id, 4);
        if (wl_message_send(g_display->fd, compositor_obj, 0,
                            payload, 4) < 0) {
            puts("alloy_de: create_surface failed");
            return 1;
        }
    }
    puts("alloy_de: surface created");

    puts("alloy_de: allocating SHM buffer");
    int shm_fd = alloy_shm_alloc(SCREEN_W, SCREEN_H, 32);
    if (shm_fd < 0) {
        puts("alloy_de: shm_alloc failed");
        return 1;
    }

    unsigned int *fb = (unsigned int *)alloy_shm_user_vaddr(shm_fd);
    if (!fb) {
        puts("alloy_de: shm_user_vaddr failed");
        return 1;
    }
    puts("alloy_de: SHM buffer ready");

    draw_desktop(fb);
    puts("alloy_de: desktop drawn");

    {
        unsigned char payload[12];
        unsigned int zero = 0;
        mymemcpy(payload, &shm_fd, 4);
        mymemcpy(payload + 4, &zero, 4);
        mymemcpy(payload + 8, &zero, 4);
        wl_message_send(g_display->fd, surface_id, 1, payload, 12);
    }

    {
        unsigned char payload[16];
        unsigned int zero = 0;
        unsigned int w = SCREEN_W;
        unsigned int h = SCREEN_H;
        mymemcpy(payload, &zero, 4);
        mymemcpy(payload + 4, &zero, 4);
        mymemcpy(payload + 8, &w, 4);
        mymemcpy(payload + 12, &h, 4);
        wl_message_send(g_display->fd, surface_id, 0, payload, 16);
    }

    {
        if (wl_message_send(g_display->fd, surface_id, 2, NULL, 0) < 0) {
            puts("alloy_de: commit failed");
            return 1;
        }
    }
    puts("alloy_de: surface committed");

    puts("alloy_de: entering idle loop");
    for (;;) {
        dispatch_events();
        syscall(SYS_YIELD, 0, 0, 0, 0, 0);
    }

    wl_display_disconnect(g_display);
    puts("alloy_de: exiting");
    return 0;
}
