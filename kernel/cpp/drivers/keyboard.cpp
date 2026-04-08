#include "keyboard.h"

// PS/2 Keyboard ports
#define KEYBOARD_DATA_PORT 0x60
#define KEYBOARD_STATUS_PORT 0x64
#define KEYBOARD_COMMAND_PORT 0x64

// Port I/O functions
static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

// Keyboard state
static bool shift_pressed = false;
static bool ctrl_pressed = false;
static bool alt_pressed = false;
static bool capslock_active = false;
static bool extended_scancode = false;  // Track if next scancode is extended (0xE0 prefix)

// Circular buffer for keyboard input
static char keyboard_buffer[KEYBOARD_BUFFER_SIZE];
static volatile uint32_t buffer_read_pos = 0;
static volatile uint32_t buffer_write_pos = 0;

// US QWERTY scancode to ASCII mapping
static const char scancode_to_ascii[128] = {
    0, 27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    0, // Left Ctrl
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    0, // Left Shift
    '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    0, // Right Shift
    '*',
    0, // Left Alt
    ' ', // Space
    0, // Caps Lock
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F1-F10
    0, // Num Lock
    0, // Scroll Lock
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Keypad
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

// Shifted characters
static const char scancode_to_ascii_shift[128] = {
    0, 27, '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\b',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n',
    0, // Left Ctrl
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~',
    0, // Left Shift
    '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
    0, // Right Shift
    '*',
    0, // Left Alt
    ' ', // Space
    0, // Caps Lock
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F1-F10
    0, // Num Lock
    0, // Scroll Lock
    0, 0, 0, 0, 0, 0, 0, 0, 0, // Keypad
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

// Add character to buffer
static void buffer_put(char c) {
    uint32_t next_pos = (buffer_write_pos + 1) % KEYBOARD_BUFFER_SIZE;
    if (next_pos != buffer_read_pos) {
        keyboard_buffer[buffer_write_pos] = c;
        buffer_write_pos = next_pos;
    }
}

// Initialize keyboard
extern "C" void keyboard_init() {
    // Clear buffer
    buffer_read_pos = 0;
    buffer_write_pos = 0;
    
    // Keyboard should already be initialized by BIOS
    // Just need to set up IRQ handler (done in IDT setup)
}

// Keyboard interrupt handler (called from IRQ1)
extern "C" void keyboard_handler() {
    uint8_t scancode = inb(KEYBOARD_DATA_PORT);
    
    // Check for extended scancode prefix (0xE0)
    if (scancode == 0xE0) {
        extended_scancode = true;
        return;
    }
    
    // Check if this is a key release (bit 7 set)
    bool key_released = (scancode & 0x80) != 0;
    scancode &= 0x7F; // Remove release bit
    
    // Handle extended keys (arrows, home, end, delete, etc.)
    if (extended_scancode) {
        extended_scancode = false;
        
        // Only process key presses, not releases
        if (key_released) {
            return;
        }
        
        // Map extended scancodes to special key codes
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
                return; // Unknown extended key
        }
        
        if (special_key != 0) {
            buffer_put(special_key);
        }
        return;
    }
    
    // Handle modifier keys
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
    
    // Only process key presses, not releases
    if (key_released) {
        return;
    }
    
    // Convert scancode to ASCII
    char ascii;
    if (shift_pressed) {
        ascii = scancode_to_ascii_shift[scancode];
    } else {
        ascii = scancode_to_ascii[scancode];
    }
    
    // Handle caps lock for letters
    if (capslock_active && ascii >= 'a' && ascii <= 'z') {
        ascii -= 32; // Convert to uppercase
    } else if (capslock_active && ascii >= 'A' && ascii <= 'Z' && shift_pressed) {
        ascii += 32; // Convert to lowercase when shift+capslock
    }
    
    // Add to buffer if valid ASCII
    if (ascii != 0) {
        buffer_put(ascii);
    }
}

// Check if keyboard has data
extern "C" bool keyboard_has_data() {
    return buffer_read_pos != buffer_write_pos;
}

// Get character from keyboard buffer (blocking)
extern "C" char keyboard_get_char() {
    // Wait for data
    while (!keyboard_has_data()) {
        asm volatile("hlt");
    }
    
    char c = keyboard_buffer[buffer_read_pos];
    buffer_read_pos = (buffer_read_pos + 1) % KEYBOARD_BUFFER_SIZE;
    return c;
}

// Try to get character (non-blocking, returns 0 if no data)
extern "C" char keyboard_try_get_char() {
    if (!keyboard_has_data()) {
        return 0;
    }
    
    char c = keyboard_buffer[buffer_read_pos];
    buffer_read_pos = (buffer_read_pos + 1) % KEYBOARD_BUFFER_SIZE;
    return c;
}
