#include "timer.h"
#include "serial.h"

static inline void outb(uint16_t port, uint8_t value) {
    asm volatile ("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t value;
    asm volatile ("inb %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

volatile uint64_t g_timer_ticks = 0;
static uint32_t g_timer_frequency = 0;

void timer_init_ffi(uint32_t frequency) {
    serial_print("[Timer] Initializing PIT timer\n");

    g_timer_frequency = frequency;

    uint32_t divisor = PIT_BASE_FREQ / frequency;

    if (divisor > 65535) {
        divisor = 65535;
    }

    outb(PIT_COMMAND, PIT_CMD_INIT);

    outb(PIT_CHANNEL_0, (uint8_t)(divisor & 0xFF));
    outb(PIT_CHANNEL_0, (uint8_t)((divisor >> 8) & 0xFF));

    serial_print("[Timer] PIT initialized\n");
}

uint64_t timer_get_ticks_ffi() {
    return g_timer_ticks;
}

uint64_t timer_get_uptime_ms_ffi() {
    if (g_timer_frequency == 0) {
        return 0;
    }
    return (g_timer_ticks * 1000) / g_timer_frequency;
}

uint32_t timer_get_frequency_ffi() {
    return g_timer_frequency;
}

void timer_handler() {
    g_timer_ticks++;
}