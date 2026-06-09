#ifndef _WAYLAND_CLIENT_H
#define _WAYLAND_CLIENT_H

/* ───── Wayland wire protocol ───── */

#define WL_MESSAGE_HEADER_SIZE 8
#define WL_MAX_MESSAGE_SIZE   4096

struct wl_wire_header {
    unsigned int object_id;
    unsigned short opcode;
    unsigned short length;
} __attribute__((packed));

/* Standard object IDs */
#define WL_DISPLAY_ID   1
#define WL_REGISTRY_ID  2

/* wl_display requests */
#define WL_DISPLAY_SYNC         0
#define WL_DISPLAY_GET_REGISTRY 1

/* wl_display events */
#define WL_DISPLAY_ERROR     0
#define WL_DISPLAY_DELETE_ID 1

/* wl_registry request */
#define WL_REGISTRY_BIND   0

/* wl_registry event */
#define WL_REGISTRY_GLOBAL 0

/* wl_callback event */
#define WL_CALLBACK_DONE 0

/* ───── Client API ───── */

struct wl_display {
    int fd;
    unsigned int next_id;
};

struct wl_registry {
    struct wl_display *display;
    unsigned int id;
};

/* Allocate a socket and connect to the server */
struct wl_display *wl_display_connect(const char *socket_path);
void wl_display_disconnect(struct wl_display *d);

/* Send sync, wait for callback.done event */
int wl_display_roundtrip(struct wl_display *d);

/* Get the registry object */
struct wl_registry *wl_display_get_registry(struct wl_display *d);

/* Bind to a global — returns the new object id */
unsigned int wl_registry_bind(struct wl_registry *reg, unsigned int name,
                              const char *interface, unsigned int version,
                              unsigned int new_id);

/* Low-level message send/receive */
int wl_message_send(int fd, unsigned int object_id, unsigned short opcode,
                    const void *payload, unsigned short payload_len);
int wl_message_receive(int fd, void *buf);

#endif
