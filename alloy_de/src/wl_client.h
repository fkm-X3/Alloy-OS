#ifndef _WL_CLIENT_H
#define _WL_CLIENT_H

#define WL_MESSAGE_HEADER_SIZE 8
#define WL_MAX_MESSAGE_SIZE   4096

struct wl_wire_header {
    unsigned int object_id;
    unsigned short opcode;
    unsigned short length;
} __attribute__((packed));

#define WL_DISPLAY_ID   1
#define WL_REGISTRY_ID  2

#define WL_DISPLAY_SYNC         0
#define WL_DISPLAY_GET_REGISTRY 1

#define WL_DISPLAY_ERROR     0
#define WL_DISPLAY_DELETE_ID 1

#define WL_REGISTRY_BIND   0
#define WL_REGISTRY_GLOBAL 0

#define WL_CALLBACK_DONE 0

#define WL_SURFACE_DAMAGE 0
#define WL_SURFACE_ATTACH 1
#define WL_SURFACE_COMMIT 2

#define WL_COMPOSITOR_CREATE_SURFACE 0

#define WL_SEAT_GET_POINTER  0
#define WL_SEAT_GET_KEYBOARD 1

#define WL_SEAT_CAPABILITIES 0
#define WL_SEAT_NAME         1

#define WL_KEYBOARD_KEYMAP      0
#define WL_KEYBOARD_ENTER       1
#define WL_KEYBOARD_LEAVE       2
#define WL_KEYBOARD_KEY         3
#define WL_KEYBOARD_MODIFIERS   4
#define WL_KEYBOARD_REPEAT_INFO 5

#define WL_POINTER_ENTER   0
#define WL_POINTER_LEAVE   1
#define WL_POINTER_MOTION  2
#define WL_POINTER_BUTTON  3
#define WL_POINTER_AXIS    4
#define WL_POINTER_FRAME   5

#define WL_KEY_STATE_RELEASED 0
#define WL_KEY_STATE_PRESSED  1

#define WL_POINTER_BTN_LEFT   0x110
#define WL_POINTER_BTN_RIGHT  0x111
#define WL_POINTER_BTN_MIDDLE 0x112

#define WL_MAX_CALLBACKS 16

struct wl_callback {
    unsigned int id;
    int active;
};

struct input_state {
    int cursor_x;
    int cursor_y;
    unsigned int keys_pressed[256];
    int num_keys_pressed;
    int mouse_buttons[3];
    int mod_shift;
    int mod_ctrl;
    int mod_alt;
};

struct wl_display {
    int fd;
    unsigned int next_id;
    struct wl_callback callbacks[WL_MAX_CALLBACKS];
    int num_callbacks;
    unsigned int seat_id;
    unsigned int keyboard_id;
    unsigned int pointer_id;
    struct input_state input;
    void (*on_key)(int key, int pressed, struct input_state *state);
    void (*on_mouse_move)(int x, int y, struct input_state *state);
    void (*on_click)(int button, int pressed, int x, int y,
                     struct input_state *state);
};

struct wl_registry {
    struct wl_display *display;
    unsigned int id;
};

struct wl_display *wl_display_connect(const char *socket_path);
void wl_display_disconnect(struct wl_display *d);

int wl_display_get_fd(struct wl_display *d);
int wl_display_flush(struct wl_display *d);

int wl_display_roundtrip(struct wl_display *d);

int wl_display_dispatch_pending(struct wl_display *d);
int wl_display_dispatch(struct wl_display *d);

int wl_callback_add(struct wl_display *d, unsigned int id);
void wl_callback_remove(struct wl_display *d, unsigned int id);

struct wl_registry *wl_display_get_registry(struct wl_display *d);

unsigned int wl_registry_bind(struct wl_registry *reg, unsigned int name,
                               const char *interface, unsigned int version,
                               unsigned int new_id);

unsigned int wl_registry_bind_generic(struct wl_registry *reg,
                                       unsigned int name,
                                       const char *interface,
                                       unsigned int version);

unsigned int wl_compositor_create_surface(struct wl_registry *reg,
                                           unsigned int compositor_id);

int wl_surface_attach(int fd, unsigned int surface_id, unsigned int buffer,
                       int x, int y);
int wl_surface_damage(int fd, unsigned int surface_id,
                       int x, int y, int width, int height);
int wl_surface_commit(int fd, unsigned int surface_id);

unsigned int wl_seat_bind(struct wl_registry *reg, unsigned int seat_name,
                           unsigned int version);

void wl_seat_get_keyboard(struct wl_display *d, unsigned int seat_id);
void wl_seat_get_pointer(struct wl_display *d, unsigned int seat_id);

void wl_set_key_callback(struct wl_display *d,
                          void (*cb)(int key, int pressed,
                                     struct input_state *state));
void wl_set_mouse_move_callback(struct wl_display *d,
                                 void (*cb)(int x, int y,
                                            struct input_state *state));
void wl_set_click_callback(struct wl_display *d,
                            void (*cb)(int button, int pressed,
                                       int x, int y,
                                       struct input_state *state));

int wl_message_send(int fd, unsigned int object_id, unsigned short opcode,
                    const void *payload, unsigned short payload_len);
int wl_message_receive(int fd, void *buf);

void wl_dispatch_raw(struct wl_display *d, const unsigned char *buf, int len);

#endif
