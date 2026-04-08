#ifndef ALLOY_KEYBOARD_H
#define ALLOY_KEYBOARD_H

#include "boot/types.h"

// Keyboard special keys
#define KEY_ESCAPE 0x01
#define KEY_BACKSPACE 0x0E
#define KEY_TAB 0x0F
#define KEY_ENTER 0x1C
#define KEY_LCTRL 0x1D
#define KEY_LSHIFT 0x2A
#define KEY_RSHIFT 0x36
#define KEY_LALT 0x38
#define KEY_CAPSLOCK 0x3A
#define KEY_F1 0x3B
#define KEY_F2 0x3C
#define KEY_F3 0x3D
#define KEY_F4 0x3E
#define KEY_F5 0x3F
#define KEY_F6 0x40
#define KEY_F7 0x41
#define KEY_F8 0x42
#define KEY_F9 0x43
#define KEY_F10 0x44

// Extended key scancodes (require 0xE0 prefix)
#define KEY_HOME 0x47
#define KEY_UP_ARROW 0x48
#define KEY_PGUP 0x49
#define KEY_LEFT_ARROW 0x4B
#define KEY_RIGHT_ARROW 0x4D
#define KEY_END 0x4F
#define KEY_DOWN_ARROW 0x50
#define KEY_PGDN 0x51
#define KEY_DELETE 0x53

// Special key codes for Rust FFI (above ASCII range 128-255)
#define SPECIAL_KEY_UP 128
#define SPECIAL_KEY_DOWN 129
#define SPECIAL_KEY_LEFT 130
#define SPECIAL_KEY_RIGHT 131
#define SPECIAL_KEY_HOME 132
#define SPECIAL_KEY_END 133
#define SPECIAL_KEY_DELETE 134
#define SPECIAL_KEY_PGUP 135
#define SPECIAL_KEY_PGDN 136

// Keyboard buffer size
#define KEYBOARD_BUFFER_SIZE 256

// Keyboard functions
extern "C" void keyboard_init();
extern "C" void keyboard_handler();
extern "C" bool keyboard_has_data();
extern "C" char keyboard_get_char();

#endif // ALLOY_KEYBOARD_H
