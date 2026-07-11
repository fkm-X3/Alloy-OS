#include "keyboard.h"

#define KEYBOARD_DATA_PORT 0x60
#define KEYBOARD_STATUS_PORT 0x64
#define KEYBOARD_COMMAND_PORT 0x64
#define KEYBOARD_STATUS_OUTPUT_FULL 0x01
#define KEYBOARD_STATUS_INPUT_FULL 0x02

static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static bool keyboard_wait_input_ready() {
    for (int i = 0; i < 20000000; i++) {
        if ((inb(KEYBOARD_STATUS_PORT) & KEYBOARD_STATUS_INPUT_FULL) == 0) {
            return true;
        }
    }
    serial_print("[KBD] Timed out waiting for input ready\n");
    return false;
}

static void keyboard_flush_output_buffer() {
    for (uint32_t i = 0; i < KEYBOARD_BUFFER_SIZE; i++) {
        if ((inb(KEYBOARD_STATUS_PORT) & KEYBOARD_STATUS_OUTPUT_FULL) == 0) {
            break;
        }
        (void)inb(KEYBOARD_DATA_PORT);
    }
}

static bool shift_pressed = false;
static bool ctrl_pressed = false;
static bool alt_pressed = false;
static bool capslock_active = false;
static bool extended_scancode = false;

static char keyboard_buffer[KEYBOARD_BUFFER_SIZE];
static volatile uint32_t buffer_read_pos = 0;
static volatile uint32_t buffer_write_pos = 0;

static const char scancode_to_ascii[128] = {
    0, 27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    0,
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    0,
    '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    0,
    '*',
    0,
    ' ',
    0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
    0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

static const char scancode_to_ascii_shift[128] = {
    0, 27, '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\b',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n',
    0,
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~',
    0,
    '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
    0,
    '*',
    0,
    ' ',
    0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
    0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

static void buffer_put(char c) {
    uint32_t next_pos = (buffer_write_pos + 1) % KEYBOARD_BUFFER_SIZE;
    if (next_pos != buffer_read_pos) {
        keyboard_buffer[buffer_write_pos] = c;
        buffer_write_pos = next_pos;
    }
}

void keyboard_init() {
    buffer_read_pos = 0;
    buffer_write_pos = 0;

    shift_pressed = false;
    ctrl_pressed = false;
    alt_pressed = false;
    capslock_active = false;
    extended_scancode = false;

    serial_print("[KBD] Skip init for testing\n");
}

void keyboard_handler() {
    uint8_t scancode = inb(KEYBOARD_DATA_PORT);

    if (scancode == 0xE0) {
        extended_scancode = true;
        return;
    }

    bool key_released = (scancode & 0x80) != 0;
    scancode &= 0x7F;

    if (extended_scancode) {
        extended_scancode = false;

        if (key_released) {
            return;
        }

        char special_key = 0;
        switch (scancode) {
            case KEY_UP_ARROW:
                special_key = SPECIAL_KEY_UP;
                break;
            case KEY_DOWN_ARROW:
                special_key = SPECIAL_KEY_DOWN;
                break;
            case KEY_LEFT_ARROW:
                special_key = SPECIAL_KEY_LEFT;
                break;
            case KEY_RIGHT_ARROW:
                special_key = SPECIAL_KEY_RIGHT;
                break;
            case KEY_HOME:
                special_key = SPECIAL_KEY_HOME;
                break;
            case KEY_END:
                special_key = SPECIAL_KEY_END;
                break;
            case KEY_DELETE:
                special_key = SPECIAL_KEY_DELETE;
                break;
            case KEY_PGUP:
                special_key = SPECIAL_KEY_PGUP;
                break;
            case KEY_PGDN:
                special_key = SPECIAL_KEY_PGDN;
                break;
            default:
                return;
        }

        if (special_key != 0) {
            buffer_put(special_key);
        }
        return;
    }

    if (scancode == KEY_LSHIFT || scancode == KEY_RSHIFT) {
        shift_pressed = !key_released;
        return;
    }
    if (scancode == KEY_LCTRL) {
        ctrl_pressed = !key_released;
        return;
    }
    if (scancode == KEY_LALT) {
        alt_pressed = !key_released;
        return;
    }
    if (scancode == KEY_CAPSLOCK && !key_released) {
        capslock_active = !capslock_active;
        return;
    }

    if (key_released) {
        return;
    }

    char ascii;
    if (shift_pressed) {
        ascii = scancode_to_ascii_shift[scancode];
    } else {
        ascii = scancode_to_ascii[scancode];
    }

    if (capslock_active && ascii >= 'a' && ascii <= 'z') {
        ascii -= 32;
    } else if (capslock_active && ascii >= 'A' && ascii <= 'Z' && shift_pressed) {
        ascii += 32;
    }

    if (ascii != 0) {
        buffer_put(ascii);
    }
}

bool keyboard_has_data() {
    return buffer_read_pos != buffer_write_pos;
}

char keyboard_get_char() {
    while (!keyboard_has_data()) {
        asm volatile("hlt");
    }

    char c = keyboard_buffer[buffer_read_pos];
    buffer_read_pos = (buffer_read_pos + 1) % KEYBOARD_BUFFER_SIZE;
    return c;
}