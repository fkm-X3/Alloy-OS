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

/* ───── Socket helpers via syscalls ───── */

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

/* ───── Public API ───── */

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
    return &d;
}

void wl_display_disconnect(struct wl_display *d) {
    if (!d) return;
    SYSCALL_FN(SYS_CLOSE_SOCKET, d->fd, 0, 0, 0, 0);
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

    unsigned char buf[WL_MAX_MESSAGE_SIZE];
    ret = wl_message_receive(d->fd, buf);
    if (ret <= 0) return ret;
    return 0;
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
