#ifndef DRIVERS_TIMER_H
#define DRIVERS_TIMER_H

#include "../boot/types.h"

// PIT (Programmable Interval Timer) constants
#define PIT_CHANNEL_0     0x40  // Channel 0 data port
#define PIT_CHANNEL_1     0x41  // Channel 1 data port  
#define PIT_CHANNEL_2     0x42  // Channel 2 data port
#define PIT_COMMAND       0x43  // Command register

// PIT base frequency (Hz)
#define PIT_BASE_FREQ     1193180

// PIT command byte format:
// Bits 6-7: Select channel (00=0, 01=1, 10=2, 11=read-back)
// Bits 4-5: Access mode (01=LSB only, 10=MSB only, 11=LSB then MSB)
// Bits 1-3: Operating mode (0-5, we use mode 3 = square wave)
// Bit 0: BCD/Binary mode (0=16-bit binary, 1=4-digit BCD)

#define PIT_CMD_BINARY    0x00  // Use binary mode (not BCD)
#define PIT_CMD_MODE3     0x06  // Mode 3: Square wave generator
#define PIT_CMD_RW_BOTH   0x30  // Read/write LSB then MSB
#define PIT_CMD_CHANNEL0  0x00  // Select channel 0

// Combined command byte for channel 0, mode 3, binary, LSB+MSB
#define PIT_CMD_INIT      (PIT_CMD_CHANNEL0 | PIT_CMD_RW_BOTH | PIT_CMD_MODE3 | PIT_CMD_BINARY)

// Global timer state
extern volatile uint64_t g_timer_ticks;  // Number of timer ticks since boot

// Initialize the PIT timer
// frequency: Desired interrupt frequency in Hz (e.g., 100 for 10ms tick)
void timer_init(uint32_t frequency);

// Get the current tick count
extern "C" uint64_t timer_get_ticks();

// Get system uptime in milliseconds
extern "C" uint64_t timer_get_uptime_ms();

// Get configured PIT frequency in Hz
extern "C" uint32_t timer_get_frequency();

// Timer IRQ handler (called from IDT)
extern "C" void timer_handler();

#endif // DRIVERS_TIMER_H
