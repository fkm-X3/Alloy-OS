#include "../boot/types.h"

// Serial port addresses
#define SERIAL_COM1 0x3F8

// Serial port registers
#define SERIAL_DATA(base) (base)
#define SERIAL_INT_EN(base) (base + 1)
#define SERIAL_FIFO_CTRL(base) (base + 2)
#define SERIAL_LINE_CTRL(base) (base + 3)
#define SERIAL_MODEM_CTRL(base) (base + 4)
#define SERIAL_LINE_STATUS(base) (base + 5)

// Port I/O functions
static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

// Initialize serial port for debugging
extern "C" void init_serial() {
    outb(SERIAL_INT_EN(SERIAL_COM1), 0x00);    // Disable interrupts
    outb(SERIAL_LINE_CTRL(SERIAL_COM1), 0x80); // Enable DLAB
    outb(SERIAL_DATA(SERIAL_COM1), 0x03);      // Set divisor to 3 (38400 baud)
    outb(SERIAL_INT_EN(SERIAL_COM1), 0x00);
    outb(SERIAL_LINE_CTRL(SERIAL_COM1), 0x03); // 8 bits, no parity, one stop bit
    outb(SERIAL_FIFO_CTRL(SERIAL_COM1), 0xC7); // Enable FIFO, clear, 14-byte threshold
    outb(SERIAL_MODEM_CTRL(SERIAL_COM1), 0x0B); // IRQs enabled, RTS/DSR set
}

// Check if transmit buffer is empty
static int serial_transmit_empty() {
    return inb(SERIAL_LINE_STATUS(SERIAL_COM1)) & 0x20;
}

// Write a character to serial port
static void serial_putchar(char c) {
    while (serial_transmit_empty() == 0);
    outb(SERIAL_DATA(SERIAL_COM1), c);
}

// Write a string to serial port
extern "C" void serial_print(const char* str) {
    if (!str) return;
    
    while (*str) {
        if (*str == '\n') {
            serial_putchar('\r');
        }
        serial_putchar(*str);
        str++;
    }
}

// Write a string with hex value
extern "C" void serial_print_hex(const char* prefix, uint32_t value) {
    serial_print(prefix);
    serial_print("0x");
    
    char hex_chars[] = "0123456789ABCDEF";
    char buffer[9];
    buffer[8] = '\0';
    
    for (int i = 7; i >= 0; i--) {
        buffer[i] = hex_chars[value & 0xF];
        value >>= 4;
    }
    
    serial_print(buffer);
    serial_print("\n");
}
