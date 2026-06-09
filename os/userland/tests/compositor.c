/*
 * Alloy OS Userspace Compositor
 *
 * Connects to the in-kernel Wayland server, receives surface updates,
 * and composites the desktop. Currently a skeleton — full compositing
 * requires framebuffer mmap for userspace pixel access.
 *
 * Built as a standard userland ELF via os/userland/Makefile.
 */

#include "wayland_client.h"
#include "stdio.h"
#include "alloy_syscall.h"
#include "stdlib.h"

typedef __SIZE_TYPE__ size_t;

static void mymemcpy(void *dst, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    for (size_t i = 0; i < n; ++i) d[i] = s[i];
}

/* ───── Event dispatch ───── */

static void print_u32(unsigned int n) {
    char tmp[16];
    int i = 0;
    if (n == 0) { tmp[i++] = '0'; }
    else { while (n > 0) { tmp[i++] = '0' + (n % 10); n /= 10; } }
    while (i > 0) write(1, &tmp[--i], 1);
}

static void print_str(const char *s) {
    while (*s) { write(1, s, 1); ++s; }
}

static void handle_wl_registry_global(const unsigned char *payload) {
    unsigned int name;
    unsigned int iface_len;
    mymemcpy(&name, payload, 4);
    mymemcpy(&iface_len, payload + 4, 4);

    char iface[64];
    unsigned int copy = iface_len < sizeof(iface) ? iface_len : sizeof(iface) - 1;
    mymemcpy(iface, payload + 8, copy);
    iface[copy] = 0;

    unsigned int version;
    unsigned int off = 8 + ((iface_len + 3) & ~3u);
    mymemcpy(&version, payload + off, 4);

    print_str("global: name=");
    print_u32(name);
    print_str(" interface=");
    print_str(iface);
    print_str("\n");
}

/* ───── Main event loop ───── */

static void compositor_run(struct wl_display *display) {
    unsigned char buf[WL_MAX_MESSAGE_SIZE];

    for (;;) {
        int n = wl_message_receive(display->fd, buf);
        if (n <= 0) break;

        struct wl_wire_header *hdr = (struct wl_wire_header *)buf;
        unsigned char *payload = buf + WL_MESSAGE_HEADER_SIZE;

        if (hdr->object_id == WL_REGISTRY_ID && hdr->opcode == WL_REGISTRY_GLOBAL) {
            handle_wl_registry_global(payload);
        } else if (hdr->object_id == WL_DISPLAY_ID && hdr->opcode == WL_DISPLAY_ERROR) {
            print_str("server error received, exiting\n");
            break;
        }
    }
}

int main(void) {
    print_str("compositor: starting up\n");

    struct wl_display *display = wl_display_connect("/tmp/wayland-0");
    if (!display) {
        print_str("compositor: connection failed\n");
        return 1;
    }
    print_str("compositor: connected to server\n");

    if (wl_display_roundtrip(display) < 0) {
        print_str("compositor: roundtrip failed\n");
        wl_display_disconnect(display);
        return 1;
    }
    print_str("compositor: initial roundtrip complete\n");

    struct wl_registry *registry = wl_display_get_registry(display);
    if (!registry) {
        print_str("compositor: get_registry failed\n");
        wl_display_disconnect(display);
        return 1;
    }

    compositor_run(display);

    wl_display_disconnect(display);
    print_str("compositor: exiting\n");
    return 0;
}
