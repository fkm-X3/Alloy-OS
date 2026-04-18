#ifndef ALLOY_MOUSE_H
#define ALLOY_MOUSE_H

#include "boot/types.h"

// Mouse button bitmask values.
#define MOUSE_BUTTON_LEFT   0x01
#define MOUSE_BUTTON_RIGHT  0x02
#define MOUSE_BUTTON_MIDDLE 0x04

// Mouse event flags.
#define MOUSE_EVENT_FLAG_X_OVERFLOW 0x01
#define MOUSE_EVENT_FLAG_Y_OVERFLOW 0x02

extern "C" void mouse_init();
extern "C" void mouse_handler();
extern "C" bool mouse_has_data();
extern "C" bool mouse_read_event(
    int8_t* dx,
    int8_t* dy,
    int8_t* wheel,
    uint8_t* buttons,
    uint8_t* flags
);

#endif // ALLOY_MOUSE_H
