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

/* PL011 UART on QEMU virt machine */
#define PL011_BASE      0x09000000
#define UARTDR          (PL011_BASE + 0x000)
#define UARTFR          (PL011_BASE + 0x018)
#define UARTIBRD        (PL011_BASE + 0x024)
#define UARTFBRD        (PL011_BASE + 0x028)
#define UARTLCR_H       (PL011_BASE + 0x02C)
#define UARTCR          (PL011_BASE + 0x030)
#define UARTIFLS        (PL011_BASE + 0x034)
#define UARTIMSC        (PL011_BASE + 0x038)
#define UARTICR         (PL011_BASE + 0x044)

/* UARTFR bit definitions */
#define TXFF            (1 << 5)
#define BUSY            (1 << 3)

static inline void mmio_write32(uintptr_t addr, uint32_t value) {
    volatile uint32_t *ptr = (volatile uint32_t *)addr;
    *ptr = value;
}

static inline uint32_t mmio_read32(uintptr_t addr) {
    volatile uint32_t *ptr = (volatile uint32_t *)addr;
    return *ptr;
}

static inline void dsb(void) {
    asm volatile("dsb sy" : : : "memory");
}

void init_serial() {
    /* Disable UART */
    mmio_write32(UARTCR, 0);
    dsb();

    /* Wait for BUSY to clear */
    while (mmio_read32(UARTFR) & BUSY) {}
    dsb();

    /* Set baud rate (24MHz UART clock, 115200 baud).
       Divider = 24MHz / (16 * 115200) = 13.02 -> IBRD=13, FBRD=1 */
    mmio_write32(UARTIBRD, 13);
    mmio_write32(UARTFBRD, 1);
    dsb();

    /* 8 bits, no parity, 1 stop bit, FIFO enabled */
    mmio_write32(UARTLCR_H, 0x70);
    dsb();

    /* Set FIFO threshold: half-full */
    mmio_write32(UARTIFLS, 0x012);
    dsb();

    /* Mask all interrupts */
    mmio_write32(UARTIMSC, 0);
    dsb();

    /* Clear any pending interrupts */
    mmio_write32(UARTICR, 0x3FF);
    dsb();

    /* Enable UART, TX, RX */
    mmio_write32(UARTCR, 0x301);
    dsb();
}

static void serial_putchar(char c) {
    /* Wait for TX FIFO not full */
    while (mmio_read32(UARTFR) & TXFF) {}
    mmio_write32(UARTDR, (uint32_t)c);
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