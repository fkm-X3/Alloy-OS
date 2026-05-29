#ifndef ALLOY_MOUSE_H
#define ALLOY_MOUSE_H

#include "boot/types.h"

#define MOUSE_BUTTON_LEFT   0x01
#define MOUSE_BUTTON_RIGHT  0x02
#define MOUSE_BUTTON_MIDDLE 0x04

#define MOUSE_EVENT_FLAG_X_OVERFLOW 0x01
#define MOUSE_EVENT_FLAG_Y_OVERFLOW 0x02

#define MOUSE_INIT_ERR_NONE                 0
#define MOUSE_INIT_ERR_INPUT_NOT_READY      1
#define MOUSE_INIT_ERR_OUTPUT_NOT_READY     2
#define MOUSE_INIT_ERR_SET_DEFAULTS         3
#define MOUSE_INIT_ERR_SET_DEFAULTS_ACK     4
#define MOUSE_INIT_ERR_ENABLE_STREAMING     5
#define MOUSE_INIT_ERR_ENABLE_STREAMING_ACK 6

typedef struct mouse_event {
    int8_t dx;
    int8_t dy;
    int8_t wheel;
    uint8_t buttons;
    uint8_t flags;
} mouse_event;

#ifdef __cplusplus
extern "C" {
#endif

bool mouse_init();
void mouse_handler();
bool mouse_has_data();
bool mouse_is_initialized();
uint8_t mouse_last_init_error();
bool mouse_read_event(
    int8_t* dx,
    int8_t* dy,
    int8_t* wheel,
    uint8_t* buttons,
    uint8_t* flags
);

#ifdef __cplusplus
}
#endif

#endif // ALLOY_MOUSE_H
