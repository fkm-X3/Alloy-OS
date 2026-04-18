#include "mouse.h"

#define PS2_DATA_PORT 0x60
#define PS2_STATUS_PORT 0x64
#define PS2_COMMAND_PORT 0x64

#define PS2_STATUS_OUTPUT_FULL 0x01
#define PS2_STATUS_INPUT_FULL 0x02

#define PS2_CMD_ENABLE_AUX_DEVICE 0xA8
#define PS2_CMD_READ_CONFIG 0x20
#define PS2_CMD_WRITE_CONFIG 0x60
#define PS2_CMD_WRITE_TO_AUX 0xD4

#define PS2_MOUSE_CMD_SET_DEFAULTS 0xF6
#define PS2_MOUSE_CMD_ENABLE_STREAMING 0xF4
#define PS2_MOUSE_RESP_ACK 0xFA
#define PS2_MOUSE_RESP_RESEND 0xFE

#define MOUSE_EVENT_BUFFER_SIZE 128

struct mouse_event {
    int8_t dx;
    int8_t dy;
    int8_t wheel;
    uint8_t buttons;
    uint8_t flags;
};

static volatile uint32_t g_mouse_read_pos = 0;
static volatile uint32_t g_mouse_write_pos = 0;
static mouse_event g_mouse_events[MOUSE_EVENT_BUFFER_SIZE];
static uint8_t g_packet[3];
static uint8_t g_packet_index = 0;
static bool g_mouse_initialized = false;

static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t value;
    asm volatile("inb %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

static bool ps2_wait_input_ready() {
    for (uint32_t i = 0; i < 100000; i++) {
        if ((inb(PS2_STATUS_PORT) & PS2_STATUS_INPUT_FULL) == 0) {
            return true;
        }
    }
    return false;
}

static bool ps2_wait_output_ready() {
    for (uint32_t i = 0; i < 100000; i++) {
        if ((inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL) != 0) {
            return true;
        }
    }
    return false;
}

static void ps2_flush_output() {
    for (uint32_t i = 0; i < MOUSE_EVENT_BUFFER_SIZE; i++) {
        if ((inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL) == 0) {
            break;
        }
        (void)inb(PS2_DATA_PORT);
    }
}

static bool mouse_send_device_command(uint8_t command) {
    if (!ps2_wait_input_ready()) {
        return false;
    }
    outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_TO_AUX);
    if (!ps2_wait_input_ready()) {
        return false;
    }
    outb(PS2_DATA_PORT, command);
    return true;
}

static bool mouse_wait_ack() {
    for (uint32_t i = 0; i < 32; i++) {
        if (!ps2_wait_output_ready()) {
            return false;
        }
        uint8_t response = inb(PS2_DATA_PORT);
        if (response == PS2_MOUSE_RESP_ACK) {
            return true;
        }
        if (response == PS2_MOUSE_RESP_RESEND) {
            return false;
        }
    }
    return false;
}

static void buffer_put(mouse_event event) {
    uint32_t next = (g_mouse_write_pos + 1) % MOUSE_EVENT_BUFFER_SIZE;
    if (next == g_mouse_read_pos) {
        return;
    }
    g_mouse_events[g_mouse_write_pos] = event;
    g_mouse_write_pos = next;
}

extern "C" void mouse_init() {
    g_mouse_read_pos = 0;
    g_mouse_write_pos = 0;
    g_packet_index = 0;
    g_mouse_initialized = false;

    ps2_flush_output();

    if (!ps2_wait_input_ready()) {
        return;
    }
    outb(PS2_COMMAND_PORT, PS2_CMD_ENABLE_AUX_DEVICE);

    if (!ps2_wait_input_ready()) {
        return;
    }
    outb(PS2_COMMAND_PORT, PS2_CMD_READ_CONFIG);
    if (!ps2_wait_output_ready()) {
        return;
    }
    uint8_t config = inb(PS2_DATA_PORT);
    config |= (1u << 1);   // Enable IRQ12.
    config &= (uint8_t)~(1u << 5);  // Ensure mouse clock is enabled.

    if (!ps2_wait_input_ready()) {
        return;
    }
    outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_CONFIG);
    if (!ps2_wait_input_ready()) {
        return;
    }
    outb(PS2_DATA_PORT, config);

    if (!mouse_send_device_command(PS2_MOUSE_CMD_SET_DEFAULTS)) {
        return;
    }
    if (!mouse_wait_ack()) {
        return;
    }

    if (!mouse_send_device_command(PS2_MOUSE_CMD_ENABLE_STREAMING)) {
        return;
    }
    if (!mouse_wait_ack()) {
        return;
    }

    g_mouse_initialized = true;
}

extern "C" void mouse_handler() {
    uint8_t byte = inb(PS2_DATA_PORT);
    if (!g_mouse_initialized) {
        return;
    }

    if (g_packet_index == 0 && (byte & 0x08) == 0) {
        return;
    }

    g_packet[g_packet_index++] = byte;
    if (g_packet_index < 3) {
        return;
    }
    g_packet_index = 0;

    uint8_t status = g_packet[0];
    mouse_event event = {};
    event.dx = (int8_t)g_packet[1];
    event.dy = (int8_t)g_packet[2];
    event.wheel = 0;
    event.buttons = status & (MOUSE_BUTTON_LEFT | MOUSE_BUTTON_RIGHT | MOUSE_BUTTON_MIDDLE);
    event.flags = 0;
    if ((status & 0x40) != 0) {
        event.flags |= MOUSE_EVENT_FLAG_X_OVERFLOW;
    }
    if ((status & 0x80) != 0) {
        event.flags |= MOUSE_EVENT_FLAG_Y_OVERFLOW;
    }

    buffer_put(event);
}

extern "C" bool mouse_has_data() {
    return g_mouse_read_pos != g_mouse_write_pos;
}

extern "C" bool mouse_read_event(
    int8_t* dx,
    int8_t* dy,
    int8_t* wheel,
    uint8_t* buttons,
    uint8_t* flags
) {
    if (!mouse_has_data()) {
        return false;
    }

    mouse_event event = g_mouse_events[g_mouse_read_pos];
    g_mouse_read_pos = (g_mouse_read_pos + 1) % MOUSE_EVENT_BUFFER_SIZE;

    if (dx != NULL) {
        *dx = event.dx;
    }
    if (dy != NULL) {
        *dy = event.dy;
    }
    if (wheel != NULL) {
        *wheel = event.wheel;
    }
    if (buttons != NULL) {
        *buttons = event.buttons;
    }
    if (flags != NULL) {
        *flags = event.flags;
    }
    return true;
}
