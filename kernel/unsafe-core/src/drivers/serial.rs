//! Safe serial console driver: UART 16550 on x86_64, PL011 on aarch64.
//!
//! Replaces `ported/common/drivers_serial.rs`. The C-ABI entry points
//! (`init_serial`, `serial_print`, `serial_print_hex`, `serial_print_hex64`,
//! `serial_print_hex_with_prefix`) are kept here because surviving ported
//! modules (pmm, paging, vesa, idt, boot mains) still call them by symbol.

use core::ffi::c_char;

#[cfg(feature = "x86_64")]
use crate::raw::asm::x86_64::{inb, outb};

#[cfg(feature = "aarch64")]
use crate::io::{DefaultMmio, Mmio};
#[cfg(feature = "aarch64")]
use crate::raw::asm::aarch64::dsb_sy;

/// COM1 base port (x86_64).
#[cfg(feature = "x86_64")]
const COM1: u16 = 0x3f8;

/// QEMU virt PL011 base address (aarch64).
#[cfg(feature = "aarch64")]
const PL011_BASE: usize = 0x0900_0000;

/// PL011 registers (offsets from PL011_BASE).
#[cfg(feature = "aarch64")]
const UARTDR: usize = 0x00;
#[cfg(feature = "aarch64")]
const UARTFR: usize = 0x18;
#[cfg(feature = "aarch64")]
const UARTIBRD: usize = 0x24;
#[cfg(feature = "aarch64")]
const UARTFBRD: usize = 0x28;
#[cfg(feature = "aarch64")]
const UARTLCR_H: usize = 0x2c;
#[cfg(feature = "aarch64")]
const UARTCR: usize = 0x30;
#[cfg(feature = "aarch64")]
const UARTIFLS: usize = 0x34;
#[cfg(feature = "aarch64")]
const UARTIMSC: usize = 0x38;
#[cfg(feature = "aarch64")]
const UARTICR: usize = 0x44;

/// UARTFR flags.
#[cfg(feature = "aarch64")]
const FR_TXFF: u32 = 1 << 5; // Transmit FIFO full
#[cfg(feature = "aarch64")]
const FR_BUSY: u32 = 1 << 3; // Transmit busy

#[cfg(feature = "x86_64")]
#[inline]
fn putc(byte: u8) {
    // Wait until the transmit-holding register is empty.
    while inb(COM1 + 5) & 0x20 == 0 {}
    outb(COM1, byte);
}

#[cfg(feature = "aarch64")]
#[inline]
fn mmio_read(offset: usize) -> u32 {
    unsafe { <DefaultMmio as Mmio>::read32(PL011_BASE + offset) }
}

#[cfg(feature = "aarch64")]
#[inline]
fn mmio_write(offset: usize, value: u32) {
    unsafe { <DefaultMmio as Mmio>::write32(PL011_BASE + offset, value) }
}

#[cfg(feature = "aarch64")]
#[inline]
fn putc(byte: u8) {
    // Wait until the transmit FIFO is not full.
    while mmio_read(UARTFR) & FR_TXFF != 0 {}
    mmio_write(UARTDR, byte as u32);
}

/// Safe serial console facade.
///
/// Writes go to the primary console UART (COM1 on x86_64, PL011 on aarch64).
/// A line feed `\n` is expanded to `\r\n` exactly like the original C
/// driver, so output stays legible on any terminal emulator.
pub struct Serial;

impl Serial {
    /// Initialize the serial port (the C `init_serial` sequence).
    /// Called once during early boot, before any print.
    pub fn init() {
        #[cfg(feature = "x86_64")]
        unsafe {
            outb(COM1 + 1, 0x00); // Disable interrupts
            outb(COM1 + 3, 0x80); // Enable DLAB
            outb(COM1, 0x03); // Divisor low byte (115200 / 3 = 38400 baud)
            outb(COM1 + 1, 0x00); // Divisor high byte
            outb(COM1 + 3, 0x03); // 8N1, DLAB off
            outb(COM1 + 2, 0xc7); // FIFO on, clear, 14-byte threshold
            outb(COM1 + 4, 0x0b); // RTS/DSR set
        }
        #[cfg(feature = "aarch64")]
        {
            // Disable the UART, wait for BUSY to clear.
            mmio_write(UARTCR, 0);
            dsb_sy();
            while mmio_read(UARTFR) & FR_BUSY != 0 {}
            dsb_sy();
            // 3 MHz UART clock: IBRD 13, FBRD 1 for 38400 baud.
            mmio_write(UARTIBRD, 13);
            mmio_write(UARTFBRD, 1);
            dsb_sy();
            mmio_write(UARTLCR_H, 0x70); // 8 bits, no parity, 1 stop, FIFO
            dsb_sy();
            mmio_write(UARTIFLS, 0x12); // RX/TX interrupt FIFO levels
            dsb_sy();
            mmio_write(UARTIMSC, 0); // No interrupts
            dsb_sy();
            mmio_write(UARTICR, 0x3ff); // Clear all interrupts
            dsb_sy();
            mmio_write(UARTCR, 0x301); // Enable UART, TXE, RXE
            dsb_sy();
        }
    }

    /// Transmit a single byte.
    pub fn write_byte(byte: u8) {
        putc(byte);
    }

    /// Write a string, expanding `\n` to `\r\n` (matches the C driver).
    pub fn write_str(s: &str) {
        for &b in s.as_bytes() {
            if b == b'\n' {
                putc(b'\r');
            }
            putc(b);
        }
    }

    /// Write raw bytes with `\n` to `\r\n` expansion.
    pub fn write_bytes(bytes: &[u8]) {
        for &b in bytes {
            if b == b'\n' {
                putc(b'\r');
            }
            putc(b);
        }
    }

    /// Write `value` as exactly 8 uppercase hex digits (matches
    /// `serial_print_hex`: no prefix, no trailing newline).
    pub fn write_hex(value: u32) {
        let digits = b"0123456789ABCDEF";
        for i in 0..8 {
            let nibble = ((value >> (28 - i * 4)) & 0xF) as usize;
            putc(digits[nibble]);
        }
    }

    /// Write `value` as exactly 16 uppercase hex digits (matches
    /// `serial_print_hex64`: no prefix, no trailing newline).
    pub fn write_hex64(value: u64) {
        let digits = b"0123456789ABCDEF";
        for i in 0..16 {
            let nibble = ((value >> (60 - i * 4)) & 0xF) as usize;
            putc(digits[nibble]);
        }
    }
}

// ============================================================================
// C-ABI entry points kept for surviving ported callers.
// ============================================================================

/// `init_serial()`: early-boot serial initialization.
#[no_mangle]
pub extern "C" fn init_serial() {
    Serial::init();
}

/// `serial_print(s)`: print a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn serial_print(s: *const c_char) {
    if s.is_null() {
        return;
    }
    let mut p = s;
    while *p != 0 {
        let b = *p as u8;
        if b == b'\n' {
            putc(b'\r');
        }
        putc(b);
        p = p.add(1);
    }
}

/// `serial_print_hex(value)`: print 8 uppercase hex digits.
#[no_mangle]
pub unsafe extern "C" fn serial_print_hex(value: u32) {
    Serial::write_hex(value);
}

/// `serial_print_hex64(value)`: print 16 uppercase hex digits.
#[no_mangle]
pub unsafe extern "C" fn serial_print_hex64(value: u64) {
    Serial::write_hex64(value);
}

/// `serial_print_hex_with_prefix(prefix, value)`: `prefix` + `0x` + 8 hex
/// digits + `\n`. Used by the (kept) ported VESA driver.
#[no_mangle]
pub unsafe extern "C" fn serial_print_hex_with_prefix(
    prefix: *const c_char,
    value: u32,
) {
    if !prefix.is_null() {
        unsafe { serial_print(prefix) };
    }
    unsafe { serial_print(b"0x\0".as_ptr() as *const c_char) };
    Serial::write_hex(value);
    unsafe { serial_print(b"\n\0".as_ptr() as *const c_char) };
}

