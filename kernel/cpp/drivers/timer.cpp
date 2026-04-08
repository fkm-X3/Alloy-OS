#include "timer.h"

// Forward declaration
extern "C" void serial_print(const char* str);

// I/O port access functions (inline assembly)
static inline void outb(uint16_t port, uint8_t value) {
    asm volatile ("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t value;
    asm volatile ("inb %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

// Global timer variables
volatile uint64_t g_timer_ticks = 0;
static uint32_t g_timer_frequency = 0;

void timer_init(uint32_t frequency) {
    serial_print("[Timer] Initializing PIT timer\n");
    
    // Store frequency for uptime calculations
    g_timer_frequency = frequency;
    
    // Calculate divisor for desired frequency
    // divisor = PIT_BASE_FREQ / desired_frequency
    uint32_t divisor = PIT_BASE_FREQ / frequency;
    
    // Make sure divisor fits in 16 bits
    if (divisor > 65535) {
        divisor = 65535;
    }
    
    // Send command byte to PIT
    outb(PIT_COMMAND, PIT_CMD_INIT);
    
    // Send divisor (LSB first, then MSB)
    outb(PIT_CHANNEL_0, (uint8_t)(divisor & 0xFF));
    outb(PIT_CHANNEL_0, (uint8_t)((divisor >> 8) & 0xFF));
    
    char msg[100];
    serial_print("[Timer] PIT initialized at ");
    // Simple number printing (divisor value)
    serial_print(" Hz\n");
    serial_print("[Timer] Divisor: ");
    serial_print("\n");
}

extern "C" uint64_t timer_get_ticks() {
    return g_timer_ticks;
}

extern "C" uint64_t timer_get_uptime_ms() {
    if (g_timer_frequency == 0) {
        return 0;
    }
    // Convert ticks to milliseconds
    // ms = (ticks * 1000) / frequency
    return (g_timer_ticks * 1000) / g_timer_frequency;
}

// Timer IRQ handler (will be called from IDT interrupt handler)
extern "C" void timer_handler() {
    // Increment tick counter
    g_timer_ticks++;
    
    // TODO: Call scheduler for task switching
    
    // Send EOI (End of Interrupt) to PIC
    // IRQ 0 is on master PIC, so send EOI to port 0x20
    outb(0x20, 0x20);
}

// Export functions for FFI
extern "C" {
    void timer_init_ffi(uint32_t frequency) {
        timer_init(frequency);
    }
    
    uint64_t timer_get_ticks_ffi() {
        return timer_get_ticks();
    }
    
    uint64_t timer_get_uptime_ms_ffi() {
        return timer_get_uptime_ms();
    }
}
