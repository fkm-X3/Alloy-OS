#include "wayland_client.h"
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

#define NULL ((void*)0)

typedef __SIZE_TYPE__ size_t;

static void mymemcpy(void *dst, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    for (size_t i = 0; i < n; ++i) d[i] = s[i];
}

static void mymemset(void *s, int c, size_t n) {
    unsigned char *p = (unsigned char *)s;
    for (size_t i = 0; i < n; ++i) p[i] = (unsigned char)c;
}

static size_t mystrlen(const char *s) {
    size_t n = 0;
    while (*s++) ++n;
    return n;
}

static int create_socket(void) {
    return SYSCALL_FN(SYS_SOCKET, 1, 1, 0, 0, 0);
}

static int do_connect(int fd, const char *path) {
    unsigned char addr[110];
    mymemset(addr, 0, sizeof(addr));
    addr[0] = 1;
    size_t len = mystrlen(path);
    if (len > 107) return -1;
    mymemcpy(addr + 2, path, len);
    return SYSCALL_FN(SYS_CONNECT, fd, (long)(size_t)addr, (long)(len + 2), 0, 0);
}

static int sock_read(int fd, void *buf, unsigned int len) {
    return SYSCALL_FN(SYS_SOCKET_READ, fd, (long)(size_t)buf, (long)len, 0, 0);
}

static int sock_write(int fd, const void *buf, unsigned int len) {
    return SYSCALL_FN(SYS_SOCKET_WRITE, fd, (long)(size_t)buf, (long)len, 0, 0);
}

struct wl_display *wl_display_connect(const char *socket_path) {
    int fd = create_socket();
    if (fd < 0) return 0;

    if (do_connect(fd, socket_path) < 0) {
        SYSCALL_FN(SYS_CLOSE_SOCKET, fd, 0, 0, 0, 0);
        return 0;
    }

    static struct wl_display d;
    d.fd = fd;
    d.next_id = 2;
    d.num_callbacks = 0;
    d.seat_id = 0;
    d.keyboard_id = 0;
    d.pointer_id = 0;
    d.on_key = NULL;
    d.on_mouse_move = NULL;
    d.on_click = NULL;
    mymemset(&d.input, 0, sizeof(d.input));
    return &d;
}

void wl_display_disconnect(struct wl_display *d) {
    if (!d) return;
    SYSCALL_FN(SYS_CLOSE_SOCKET, d->fd, 0, 0, 0, 0);
}

int wl_display_get_fd(struct wl_display *d) {
    return d ? d->fd : -1;
}

int wl_display_flush(struct wl_display *d) {
    (void)d;
    return 0;
}

int wl_message_send(int fd, unsigned int object_id, unsigned short opcode,
                    const void *payload, unsigned short payload_len) {
    unsigned char buf[WL_MAX_MESSAGE_SIZE];
    unsigned short total = WL_MESSAGE_HEADER_SIZE + payload_len;

    struct wl_wire_header hdr;
    hdr.object_id = object_id;
    hdr.opcode = opcode;
    hdr.length = total;

    mymemcpy(buf, &hdr, WL_MESSAGE_HEADER_SIZE);
    if (payload_len && payload)
        mymemcpy(buf + WL_MESSAGE_HEADER_SIZE, payload, payload_len);

    return sock_write(fd, buf, total);
}

int wl_message_receive(int fd, void *buf) {
    struct wl_wire_header hdr;
    int n = sock_read(fd, &hdr, WL_MESSAGE_HEADER_SIZE);
    if (n <= 0) return n;

    unsigned short total = hdr.length;
    if (total < WL_MESSAGE_HEADER_SIZE || total > WL_MAX_MESSAGE_SIZE)
        return -1;

    mymemcpy(buf, &hdr, WL_MESSAGE_HEADER_SIZE);

    if (total > WL_MESSAGE_HEADER_SIZE) {
        n = sock_read(fd, (unsigned char*)buf + WL_MESSAGE_HEADER_SIZE,
                      total - WL_MESSAGE_HEADER_SIZE);
        if (n <= 0) return n;
    }
    return (int)total;
}

int wl_display_roundtrip(struct wl_display *d) {
    unsigned int cb_id = d->next_id++;
    int ret = wl_message_send(d->fd, WL_DISPLAY_ID, WL_DISPLAY_SYNC,
                              &cb_id, sizeof(cb_id));
    if (ret < 0) return ret;

    wl_callback_add(d, cb_id);
    while (1) {
        ret = wl_display_dispatch(d);
        if (ret < 0) return ret;
        int found = 0;
        for (int i = 0; i < d->num_callbacks; ++i) {
            if (d->callbacks[i].active) { found = 1; break; }
        }
        if (!found) break;
    }
    return 0;
}

int wl_display_dispatch_pending(struct wl_display *d) {
    unsigned char buf[WL_MAX_MESSAGE_SIZE];
    int n = wl_message_receive(d->fd, buf);
    if (n <= 0) return n;
    wl_dispatch_raw(d, buf, n);
    return n;
}

int wl_display_dispatch(struct wl_display *d) {
    return wl_display_dispatch_pending(d);
}

int wl_callback_add(struct wl_display *d, unsigned int id) {
    if (d->num_callbacks >= WL_MAX_CALLBACKS) return -1;
    d->callbacks[d->num_callbacks].id = id;
    d->callbacks[d->num_callbacks].active = 1;
    d->num_callbacks++;
    return 0;
}

void wl_callback_remove(struct wl_display *d, unsigned int id) {
    for (int i = 0; i < d->num_callbacks; ++i) {
        if (d->callbacks[i].id == id) {
            d->callbacks[i].active = 0;
            for (int j = i; j < d->num_callbacks - 1; ++j)
                d->callbacks[j] = d->callbacks[j + 1];
            d->num_callbacks--;
            return;
        }
    }
}

struct wl_registry *wl_display_get_registry(struct wl_display *d) {
    static struct wl_registry reg;
    reg.display = d;
    reg.id = WL_REGISTRY_ID;

    int ret = wl_message_send(d->fd, WL_DISPLAY_ID, WL_DISPLAY_GET_REGISTRY,
                              &reg.id, sizeof(reg.id));
    if (ret < 0) return 0;
    return &reg;
}

unsigned int wl_registry_bind(struct wl_registry *reg, unsigned int name,
                               const char *interface, unsigned int version,
                               unsigned int new_id) {
    unsigned int iface_len = (unsigned int)(mystrlen(interface) + 1);
    unsigned int padded = (iface_len + 3) & ~3u;
    unsigned short plen = (unsigned short)(sizeof(name) + sizeof(iface_len)
                          + padded + sizeof(version) + sizeof(new_id));

    unsigned char payload[512];
    unsigned int off = 0;
    mymemcpy(payload + off, &name, sizeof(name)); off += sizeof(name);
    mymemcpy(payload + off, &iface_len, sizeof(iface_len)); off += sizeof(iface_len);
    mymemcpy(payload + off, interface, iface_len); off += iface_len;
    while (off < sizeof(name) + sizeof(iface_len) + padded)
        payload[off++] = 0;
    mymemcpy(payload + off, &version, sizeof(version)); off += sizeof(version);
    mymemcpy(payload + off, &new_id, sizeof(new_id)); off += sizeof(new_id);

    int ret = wl_message_send(reg->display->fd, reg->id,
                              WL_REGISTRY_BIND, payload, plen);
    if (ret < 0) return 0;
    return new_id;
}

unsigned int wl_registry_bind_generic(struct wl_registry *reg,
                                       unsigned int name,
                                       const char *interface,
                                       unsigned int version) {
    unsigned int new_id = reg->display->next_id++;
    return wl_registry_bind(reg, name, interface, version, new_id);
}

unsigned int wl_compositor_create_surface(struct wl_registry *reg,
                                           unsigned int compositor_id) {
    unsigned int surface_id = reg->display->next_id++;
    int ret = wl_message_send(reg->display->fd, compositor_id,
                               WL_COMPOSITOR_CREATE_SURFACE,
                               &surface_id, sizeof(surface_id));
    if (ret < 0) return 0;
    return surface_id;
}

int wl_surface_attach(int fd, unsigned int surface_id, unsigned int buffer,
                       int x, int y) {
    unsigned char payload[12];
    mymemcpy(payload, &buffer, 4);
    mymemcpy(payload + 4, &x, 4);
    mymemcpy(payload + 8, &y, 4);
    return wl_message_send(fd, surface_id, WL_SURFACE_ATTACH, payload, 12);
}

int wl_surface_damage(int fd, unsigned int surface_id,
                       int x, int y, int width, int height) {
    unsigned char payload[16];
    mymemcpy(payload, &x, 4);
    mymemcpy(payload + 4, &y, 4);
    mymemcpy(payload + 8, &width, 4);
    mymemcpy(payload + 12, &height, 4);
    return wl_message_send(fd, surface_id, WL_SURFACE_DAMAGE, payload, 16);
}

int wl_surface_commit(int fd, unsigned int surface_id) {
    return wl_message_send(fd, surface_id, WL_SURFACE_COMMIT, NULL, 0);
}

unsigned int wl_seat_bind(struct wl_registry *reg, unsigned int seat_name,
                           unsigned int version) {
    unsigned int seat_id = reg->display->next_id++;
    unsigned int ret = wl_registry_bind(reg, seat_name, "wl_seat",
                                         version, seat_id);
    if (!ret) return 0;
    reg->display->seat_id = seat_id;
    return seat_id;
}

void wl_seat_get_keyboard(struct wl_display *d, unsigned int seat_id) {
    unsigned int kb_id = d->next_id++;
    wl_message_send(d->fd, seat_id, WL_SEAT_GET_KEYBOARD,
                    &kb_id, sizeof(kb_id));
    d->keyboard_id = kb_id;
}

void wl_seat_get_pointer(struct wl_display *d, unsigned int seat_id) {
    unsigned int ptr_id = d->next_id++;
    wl_message_send(d->fd, seat_id, WL_SEAT_GET_POINTER,
                    &ptr_id, sizeof(ptr_id));
    d->pointer_id = ptr_id;
}

void wl_set_key_callback(struct wl_display *d,
                          void (*cb)(int key, int pressed,
                                     struct input_state *state)) {
    d->on_key = cb;
}

void wl_set_mouse_move_callback(struct wl_display *d,
                                 void (*cb)(int x, int y,
                                            struct input_state *state)) {
    d->on_mouse_move = cb;
}

void wl_set_click_callback(struct wl_display *d,
                            void (*cb)(int button, int pressed,
                                       int x, int y,
                                       struct input_state *state)) {
    d->on_click = cb;
}

static void handle_keyboard_key(struct wl_display *d,
                                 const unsigned char *payload) {
    unsigned int key, state;
    mymemcpy(&key, payload + 8, 4);
    mymemcpy(&state, payload + 12, 4);

    if (state == WL_KEY_STATE_PRESSED) {
        if (d->input.num_keys_pressed < 256)
            d->input.keys_pressed[d->input.num_keys_pressed++] = key;
    } else {
        for (int i = 0; i < d->input.num_keys_pressed; ++i) {
            if (d->input.keys_pressed[i] == key) {
                for (int j = i; j < d->input.num_keys_pressed - 1; ++j)
                    d->input.keys_pressed[j] = d->input.keys_pressed[j + 1];
                d->input.num_keys_pressed--;
                break;
            }
        }
    }

    if (d->on_key)
        d->on_key((int)key, (int)state, &d->input);
}

static void handle_keyboard_modifiers(struct wl_display *d,
                                       const unsigned char *payload) {
    unsigned int depressed;
    mymemcpy(&depressed, payload + 8, 4);
    d->input.mod_shift = (depressed & (1 << 0)) != 0;
    d->input.mod_ctrl = (depressed & (1 << 2)) != 0;
    d->input.mod_alt = (depressed & (1 << 3)) != 0;
}

static void handle_pointer_motion(struct wl_display *d,
                                   const unsigned char *payload) {
    unsigned int x_fixed, y_fixed;
    mymemcpy(&x_fixed, payload + 4, 4);
    mymemcpy(&y_fixed, payload + 8, 4);
    d->input.cursor_x = (int)(x_fixed >> 8);
    d->input.cursor_y = (int)(y_fixed >> 8);
    if (d->on_mouse_move)
        d->on_mouse_move(d->input.cursor_x, d->input.cursor_y, &d->input);
}

static void handle_pointer_button(struct wl_display *d,
                                   const unsigned char *payload) {
    unsigned int button, state;
    mymemcpy(&button, payload + 8, 4);
    mymemcpy(&state, payload + 12, 4);

    int btn_idx = -1;
    if (button == WL_POINTER_BTN_LEFT) btn_idx = 0;
    else if (button == WL_POINTER_BTN_RIGHT) btn_idx = 1;
    else if (button == WL_POINTER_BTN_MIDDLE) btn_idx = 2;
    if (btn_idx >= 0)
        d->input.mouse_buttons[btn_idx] = (state == WL_KEY_STATE_PRESSED) ? 1 : 0;

    if (d->on_click)
        d->on_click((int)button, (int)state,
                     d->input.cursor_x, d->input.cursor_y, &d->input);
}

static void handle_pointer_axis(struct wl_display *d,
                                 const unsigned char *payload) {
    (void)d;
    (void)payload;
}

void wl_dispatch_raw(struct wl_display *d, const unsigned char *buf, int len) {
    (void)len;
    struct wl_wire_header *hdr = (struct wl_wire_header *)buf;
    unsigned int oid = hdr->object_id;
    unsigned short op = hdr->opcode;
    const unsigned char *p = buf + WL_MESSAGE_HEADER_SIZE;

    if (oid == WL_DISPLAY_ID && op == WL_DISPLAY_DELETE_ID) {
        unsigned int del_id;
        mymemcpy(&del_id, p, 4);
        wl_callback_remove(d, del_id);
        return;
    }

    if (oid == d->keyboard_id && d->keyboard_id) {
        if (op == WL_KEYBOARD_KEY)
            handle_keyboard_key(d, p);
        else if (op == WL_KEYBOARD_MODIFIERS)
            handle_keyboard_modifiers(d, p);
        return;
    }

    if (oid == d->pointer_id && d->pointer_id) {
        if (op == WL_POINTER_MOTION)
            handle_pointer_motion(d, p);
        else if (op == WL_POINTER_BUTTON)
            handle_pointer_button(d, p);
        else if (op == WL_POINTER_AXIS)
            handle_pointer_axis(d, p);
        return;
    }
}

int alloy_shm_alloc(unsigned int width, unsigned int height,
                    unsigned int bpp) {
    return SYSCALL_FN(SYS_ALLOC_SHM, (long)width, (long)height, (long)bpp, 0, 0);
}

void *alloy_shm_user_vaddr(int fd) {
    return (void*)SYSCALL_FN(SYS_SHM_USER_VADDR, fd, 0, 0, 0, 0);
}
