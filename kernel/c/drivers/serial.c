#include "serial.h"

#if defined(ARCH_X86_64) || defined(ARCH_I686)

#define SERIAL_COM1 0x3F8

#define SERIAL_DATA(base) (base)
#define SERIAL_INT_EN(base) (base + 1)
#define SERIAL_FIFO_CTRL(base) (base + 2)
#define SERIAL_LINE_CTRL(base) (base + 3)
#define SERIAL_MODEM_CTRL(base) (base + 4)
#define SERIAL_LINE_STATUS(base) (base + 5)

static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

void init_serial() {
    outb(SERIAL_INT_EN(SERIAL_COM1), 0x00);
    outb(SERIAL_LINE_CTRL(SERIAL_COM1), 0x80);
    outb(SERIAL_DATA(SERIAL_COM1), 0x03);
    outb(SERIAL_INT_EN(SERIAL_COM1), 0x00);
    outb(SERIAL_LINE_CTRL(SERIAL_COM1), 0x03);
    outb(SERIAL_FIFO_CTRL(SERIAL_COM1), 0xC7);
    outb(SERIAL_MODEM_CTRL(SERIAL_COM1), 0x0B);
}

static int serial_transmit_empty() {
    return inb(SERIAL_LINE_STATUS(SERIAL_COM1)) & 0x20;
}

static void serial_putchar(char c) {
    while (serial_transmit_empty() == 0);
    outb(SERIAL_DATA(SERIAL_COM1), (uint8_t)c);
}

#elif defined(ARCH_AARCH64)

/*
 * Temporary aarch64-safe stubs.
 * Replace with PL011 MMIO UART implementation for real output.
 */
void init_serial() {}

static void serial_putchar(char c) {
    (void)c;
}

#else
#error "Unsupported architecture for serial driver"
#endif

void serial_print(const char* str) {
    if (!str) return;
    while (*str) {
        if (*str == '\n') serial_putchar('\r');
        serial_putchar(*str++);
    }
}

void serial_print_hex(uint32_t value) {
    char hex_chars[] = "0123456789ABCDEF";
    char buffer[9];
    buffer[8] = '\0';
    for (int i = 7; i >= 0; i--) {
        buffer[i] = hex_chars[value & 0xF];
        value >>= 4;
    }
    serial_print(buffer);
}

void serial_print_hex64(uint64_t value) {
    char hex_chars[] = "0123456789ABCDEF";
    char buffer[17];
    buffer[16] = '\0';
    for (int i = 15; i >= 0; i--) {
        buffer[i] = hex_chars[value & 0xF];
        value >>= 4;
    }
    serial_print(buffer);
}

void serial_print_hex_with_prefix(const char* prefix, uint32_t value) {
    serial_print(prefix);
    serial_print("0x");
    serial_print_hex(value);
    serial_print("\n");
}