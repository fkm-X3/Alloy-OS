/* test_shm_client, deterministic rendering probe.
 *
 * Paints a known gradient into an alloy SHM buffer and drives it through
 * the attach -> commit -> composite pipeline so kernel-side [RenderTrace]
 * lines can be correlated against known pixel values.
 *
 * Gradient definition (deterministic, frame-parameterised):
 *   pixel(x, y, f) = 0xFF000000
 *                  | ((x * 255) / (W-1)) << 16    red ramp left->right
 *                  | ((y * 255) / (H-1)) << 8     green ramp top->bottom
 *                  | ((f * 37) & 0xFF)            blue channel = frame id
 *
 * The client exercises BOTH buffer identification paths:
 *
 *   Phase A ("QPA behaviour"): wl_surface.attach(fd-as-buffer-id).
 *     This is what de/qpa/alloybackingstore.cpp does. Expected trace:
 *     composite SKIPs with "buffer NOT FOUND".
 *
 *   Phase B ("protocol flow"): wl_shm.create_pool ->
 *     wl_shm_pool.create_buffer -> attach(server-assigned buffer id).
 *     Expected trace: pool/buffer creation IS accepted but the client
 *     never learns the assigned ids (no constructor events), so it must
 *     GUESS id=1; even then the compositor reads zeros because the kernel
 *     never maps the SHM region (kernel_vaddr == None).
 *
 * Startup object-id allocation intentionally mirrors test_wl_client.c:
 * the server's per-client object registry assigns ids sequentially and
 * ignores client-chosen new_ids, so deviating from the known-good order
 * would change which code path each message hits (itself a Session 0.1
 * finding).
 */

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

#define W      160
#define H      120
#define STRIDE (W * 4)
#define SIZE   (STRIDE * H)

#define POS_X 200
#define POS_Y 150
#define Z_ORD 10

static unsigned int gradient_pixel(unsigned int x, unsigned int y,
                                   unsigned int frame) {
    unsigned int r = (x * 255u) / (W - 1u);
    unsigned int g = (y * 255u) / (H - 1u);
    unsigned int b = (frame * 37u) & 0xFFu;
    return 0xFF000000u | (r << 16) | (g << 8) | b;
}

static void paint_gradient(unsigned int *fb, unsigned int frame) {
    for (unsigned int y = 0; y < H; ++y)
        for (unsigned int x = 0; x < W; ++x)
            fb[y * W + x] = gradient_pixel(x, y, frame);
}

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

static void print_i32(int n) {
    if (n < 0) { write(1, "-", 1); print_u32((unsigned int)(-n)); }
    else print_u32((unsigned int)n);
}

static void marker(const char *tag) {
    print_str("[test_shm_client] ");
    print_str(tag);
    print_str("\n");
}

/* ---- registry global discovery (mirrors test_wl_client.c) ---- */

static unsigned int compositor_obj;
static unsigned int shm_name;
static int got_compositor;
static int got_shm;

static void handle_global(unsigned int name, const char *iface) {
    if (iface[0] == 'w' && iface[1] == 'l') {
        if (iface[12] == 'o' && !got_compositor) {
            compositor_obj = name;
            got_compositor = 1;
            marker("  found wl_compositor");
        } else if (iface[6] == 'h' && !got_shm) {
            shm_name = name;
            got_shm = 1;
            marker("  found wl_shm");
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
        handle_global(name, iface);
    }
}

static void commit_frame(int fd, unsigned int surf, unsigned int buffer_arg,
                         unsigned int frame) {
    if (wl_surface_attach(fd, surf, buffer_arg, 0, 0) < 0) {
        marker("attach send failed");
        return;
    }
    wl_surface_damage(fd, surf, 0, 0, W, H);
    wl_surface_commit(fd, surf);
    print_str("[test_shm_client] GRADIENT FRAME ");
    print_u32(frame);
    print_str(" committed (attach buffer_arg=");
    print_u32(buffer_arg);
    print_str(")\n");
}

int main(void) {
    marker("starting up");

    /* --- connection & globals (same allocation order as test_wl_client) --- */
    struct wl_display *d = wl_display_connect("/tmp/wayland-0");
    if (!d) { marker("connection failed"); return 1; }
    marker("connected");                      /* d->next_id == 2 */

    if (wl_display_roundtrip(d) < 0) {       /* cb id 2, next_id -> 3 */
        marker("roundtrip failed");
        return 1;
    }

    struct wl_registry *registry = wl_display_get_registry(d);
    if (!registry) { marker("get_registry failed"); return 1; }

    int spins = 0;
    while ((!got_compositor || !got_shm) && spins < 100) {
        dispatch_globals(d);
        ++spins;
    }
    if (!got_compositor || !got_shm) {
        marker("missing compositor or shm global");
        return 1;
    }

    /* bind compositor as object 3 (server-side registry aligns by accident) */
    unsigned int comp = wl_registry_bind_generic(registry, compositor_obj,
                                                 "wl_compositor", 1);
    if (!comp) { marker("bind compositor failed"); return 1; }

    /* surface becomes client object 4 */
    unsigned int surf = wl_compositor_create_surface(registry, comp);
    if (!surf) { marker("create_surface failed"); return 1; }
    print_str("[test_shm_client] surface object id=");
    print_u32(surf);
    print_str("\n");

    wl_surface_set_position(d->fd, surf, POS_X, POS_Y);
    wl_surface_set_zorder(d->fd, surf, Z_ORD);

    /* bind wl_shm as client object 5 */
    unsigned int shm_obj = wl_registry_bind_generic(registry, shm_name,
                                                    "wl_shm", 1);
    if (!shm_obj) { marker("bind wl_shm failed"); return 1; }

    /* --- SHM allocation + local mapping verification --- */
    marker("allocating SHM buffer");
    int shm_fd = alloy_shm_alloc(W, H, 32);
    if (shm_fd < 0) { marker("shm_alloc FAILED"); return 1; }
    print_str("[test_shm_client] shm fd=");
    print_i32(shm_fd);
    print_str("\n");

    unsigned int *fb = (unsigned int *)alloy_shm_user_vaddr(shm_fd);
    if (!fb) { marker("shm_user_vaddr FAILED"); return 1; }
    print_str("[test_shm_client] user vaddr mapped\n");

    /* Write frame 0 and read it back: proves THIS task's address space has
     * a working mapping (the mapping follows whoever calls the syscall). */
    paint_gradient(fb, 0);
    unsigned int mismatches = 0;
    for (unsigned int y = 0; y < H && mismatches < 3; ++y) {
        for (unsigned int x = 0; x < W && mismatches < 3; ++x) {
            unsigned int want = gradient_pixel(x, y, 0);
            if (fb[y * W + x] != want) {
                ++mismatches;
                print_str("[test_shm_client] VERIFY MISMATCH at (");
                print_u32(x); print_str(","); print_u32(y);
                print_str(") wrote "); print_u32(want);
                print_str(" read "); print_u32(fb[y * W + x]);
                print_str("\n");
            }
        }
    }
    if (mismatches == 0) marker("VERIFY PASS: gradient readback exact");
    else marker("VERIFY FAIL");

    /* --- Phase A: QPA-style attach(fd-as-buffer-id) --- */
    marker("PHASE A: attach(fd-as-buffer-id), mirrors alloybackingstore.cpp");
    commit_frame(d->fd, surf, (unsigned int)shm_fd, 1);
    SYSCALL_FN(SYS_SLEEP, 300, 0, 0, 0, 0);
    paint_gradient(fb, 2);
    commit_frame(d->fd, surf, (unsigned int)shm_fd, 2);
    SYSCALL_FN(SYS_SLEEP, 300, 0, 0, 0, 0);

    /* --- Phase B: protocol flow --- */
    marker("PHASE B: wl_shm.create_pool + create_buffer flow");
    unsigned int pool_obj = d->next_id++;   /* client object 6 */
    if (wl_shm_create_pool(d->fd, shm_obj, pool_obj, shm_fd, SIZE) < 0) {
        marker("create_pool send failed");
        return 1;
    }
    print_str("[test_shm_client] create_pool sent (client obj id=");
    print_u32(pool_obj);
    print_str(", size=");
    print_u32(SIZE);
    print_str(")\n");

    unsigned int buf_obj = d->next_id++;    /* client object 7 */
    if (wl_shm_pool_create_buffer(d->fd, pool_obj, buf_obj,
                                  0, W, H, STRIDE,
                                  WL_SHM_FORMAT_ARGB8888) < 0) {
        marker("create_buffer send failed");
        return 1;
    }
    print_str("[test_shm_client] create_buffer sent (client obj id=");
    print_u32(buf_obj);
    print_str(")\n");
    /* Server ignores our new_ids and numbers pools/buffers from 1 with no
     * constructor event back — the ONLY usable guess for the first pool's
     * first buffer is server id 1. That guess IS the finding. */
    marker("attaching GUESSED server buffer_id=1");
    paint_gradient(fb, 3);
    commit_frame(d->fd, surf, 1, 3);
    SYSCALL_FN(SYS_SLEEP, 300, 0, 0, 0, 0);
    paint_gradient(fb, 4);
    commit_frame(d->fd, surf, 1, 4);
    SYSCALL_FN(SYS_SLEEP, 300, 0, 0, 0, 0);

    /* --- steady state: keep repainting so post-fix sessions only need a
     *     screenshot to validate --- */
    marker("entering steady-state repaint loop (250ms/frame)");
    unsigned int frame = 5;
    while (1) {
        paint_gradient(fb, frame);
        commit_frame(d->fd, surf, 1, frame);
        if (frame % 16 == 0) marker("steady-state heartbeat");
        SYSCALL_FN(SYS_SLEEP, 250, 0, 0, 0, 0);
        ++frame;
    }
    return 0;
}
