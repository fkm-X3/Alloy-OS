#include "wayland_client.h"
#include "stdio.h"
#include "alloy_syscall.h"
#ifdef __x86_64__
#include "alloy_syscall_x86_64.h"
#define SYSCALL_FN syscall_x86_64
#elif defined(__aarch64__)
#include "alloy_syscall_aarch64.h"
#define SYSCALL_FN syscall_aarch64
#else
#define SYSCALL_FN syscall
#endif

typedef __SIZE_TYPE__ size_t;

static void print_str(const char *s) {
    while (*s) { write(1, s, 1); ++s; }
}

static void print_u32(unsigned int n) {
    char tmp[16];
    int i = 0;
    if (n == 0) { tmp[i++] = '0'; }
    else { while (n > 0) { tmp[i++] = '0' + (n % 10); n /= 10; } }
    while (i > 0) write(1, &tmp[--i], 1);
}

static unsigned int compositor_obj;
static unsigned int shm_name;
static unsigned int seat_name;
static int got_compositor;
static int got_shm;
static int got_seat;

static void handle_global(unsigned int name, const char *iface,
                           unsigned int version) {
    (void)version;
    if (iface[0] == 'w' && iface[1] == 'l') {
        if (iface[12] == 'o' && !got_compositor) {
            compositor_obj = name;
            got_compositor = 1;
            print_str("  found wl_compositor\n");
        } else if (iface[6] == 'h' && !got_shm) {
            shm_name = name;
            got_shm = 1;
            print_str("  found wl_shm\n");
        } else if (iface[6] == 'e' && !got_seat) {
            seat_name = name;
            got_seat = 1;
            print_str("  found wl_seat\n");
        }
    }
}

static void dispatch_globals(struct wl_display *d) {
    unsigned char buf[WL_MAX_MESSAGE_SIZE];
    int n = wl_message_receive(d->fd, buf);
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
    print_str("test_wl_client: starting up\n");

    struct wl_display *d = wl_display_connect("/tmp/wayland-0");
    if (!d) {
        print_str("test_wl_client: connection failed\n");
        return 1;
    }
    print_str("test_wl_client: connected\n");

    if (wl_display_roundtrip(d) < 0) {
        print_str("test_wl_client: roundtrip failed\n");
        return 1;
    }

    struct wl_registry *registry = wl_display_get_registry(d);
    if (!registry) {
        print_str("test_wl_client: get_registry failed\n");
        return 1;
    }

    int timeout = 200;
    while ((!got_compositor || !got_shm || !got_seat) && timeout > 0) {
        dispatch_globals(d);
        --timeout;
    }

    if (!got_compositor || !got_shm) {
        print_str("test_wl_client: missing compositor or shm\n");
        return 1;
    }
    print_str("test_wl_client: globals acquired\n");

    unsigned int comp = wl_registry_bind_generic(registry, compositor_obj,
                                                  "wl_compositor", 1);
    if (!comp) {
        print_str("test_wl_client: bind compositor failed\n");
        return 1;
    }
    print_str("test_wl_client: compositor bound\n");

    unsigned int surf = wl_compositor_create_surface(registry, comp);
    if (!surf) {
        print_str("test_wl_client: create_surface failed\n");
        return 1;
    }
    print_str("test_wl_client: surface created, id=");
    print_u32(surf);
    print_str("\n");

    if (got_seat) {
        unsigned int seat = wl_seat_bind(registry, seat_name, 1);
        if (seat) {
            wl_seat_get_keyboard(d, seat);
            wl_seat_get_pointer(d, seat);
            print_str("test_wl_client: seat acquired\n");
        }
    }

    print_str("test_wl_client: allocating SHM buffer\n");
    int shm_fd = alloy_shm_alloc(128, 128, 32);
    if (shm_fd < 0) {
        print_str("test_wl_client: shm_alloc failed\n");
        return 1;
    }

    unsigned int *fb = (unsigned int *)alloy_shm_user_vaddr(shm_fd);
    if (!fb) {
        print_str("test_wl_client: shm_user_vaddr failed\n");
        return 1;
    }
    print_str("test_wl_client: SHM buffer ready\n");

    for (int frame = 0; frame < 10; ++frame) {
        unsigned int color = 0;
        if (frame == 0)      color = 0xFF0000;
        else if (frame == 1) color = 0x00FF00;
        else if (frame == 2) color = 0x0000FF;
        else if (frame == 3) color = 0xFFFF00;
        else if (frame == 4) color = 0xFF00FF;
        else if (frame == 5) color = 0x00FFFF;
        else                 color = 0x888888;

        for (int y = 0; y < 128; ++y)
            for (int x = 0; x < 128; ++x)
                fb[y * 128 + x] = color;

        if (wl_surface_attach(d->fd, surf, (unsigned int)shm_fd, 0, 0) < 0) {
            print_str("test_wl_client: attach failed\n");
            return 1;
        }
        if (wl_surface_damage(d->fd, surf, 0, 0, 128, 128) < 0) {
            print_str("test_wl_client: damage failed\n");
            return 1;
        }
        if (wl_surface_commit(d->fd, surf) < 0) {
            print_str("test_wl_client: commit failed\n");
            return 1;
        }

        print_str("test_wl_client: frame ");
        print_u32(frame + 1);
        print_str("/10 committed\n");

        wl_display_dispatch_pending(d);

        SYSCALL_FN(SYS_SLEEP, 100, 0, 0, 0, 0);
    }

    wl_display_disconnect(d);
    print_str("test_wl_client: done\n");
    return 0;
}
