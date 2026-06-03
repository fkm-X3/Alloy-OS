#ifndef DRIVERS_TIMER_H
#define DRIVERS_TIMER_H

#include "../boot/types.h"

// PIT (Programmable Interval Timer) constants
#define PIT_CHANNEL_0     0x40
#define PIT_CHANNEL_1     0x41
#define PIT_CHANNEL_2     0x42
#define PIT_COMMAND       0x43

#define PIT_BASE_FREQ     1193180

#define PIT_CMD_BINARY    0x00
#define PIT_CMD_MODE3     0x06
#define PIT_CMD_RW_BOTH   0x30
#define PIT_CMD_CHANNEL0  0x00

#define PIT_CMD_INIT      (PIT_CMD_CHANNEL0 | PIT_CMD_RW_BOTH | PIT_CMD_MODE3 | PIT_CMD_BINARY)

extern volatile uint64_t g_timer_ticks;

#ifdef __cplusplus
extern "C" {
#endif

void timer_init_ffi(uint32_t frequency);
uint64_t timer_get_ticks_ffi();
uint64_t timer_get_uptime_ms_ffi();
uint32_t timer_get_frequency_ffi();
void timer_handler();

#ifdef __cplusplus
}
#endif

#endif // DRIVERS_TIMER_H
