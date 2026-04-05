#ifndef ALLOY_KEYBOARD_H
#define ALLOY_KEYBOARD_H

#include "../boot/types.h"

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

// Keyboard buffer size
#define KEYBOARD_BUFFER_SIZE 256

// Keyboard functions
extern "C" void keyboard_init();
extern "C" void keyboard_handler();
extern "C" bool keyboard_has_data();
extern "C" char keyboard_get_char();

#endif // ALLOY_KEYBOARD_H
