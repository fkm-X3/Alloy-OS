//! Safe VGA text-mode driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/vga.rs`. Provides the `VgaText` facade
//! over the 0xB8000 text buffer and the 0x3D4/0x3D5 cursor registers. No
//! C-ABI entry points are needed: after the kernel crate migrates to
//! `VgaText` no surviving code references the `vga_*` symbols.

use crate::raw::asm::x86_64::{inb, outb};

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_MEMORY: usize = 0xB8000;
const VGA_CTRL_REGISTER: u16 = 0x3D4;
const VGA_DATA_REGISTER: u16 = 0x3D5;
const VGA_CURSOR_HIGH: u8 = 14;
const VGA_CURSOR_LOW: u8 = 15;

static mut VGA_BUFFER: *mut u16 = VGA_MEMORY as *mut u16;
static mut CURSOR_X: u8 = 0;
static mut CURSOR_Y: u8 = 0;
static mut CURRENT_COLOR: u8 = 0x0f;

#[inline]
fn vga_entry(c: u8, color: u8) -> u16 {
    (c as u16) | ((color as u16) << 8)
}

#[inline]
fn update_cursor() {
    let x = unsafe { CURSOR_X };
    let y = unsafe { CURSOR_Y };
    let pos = (y as u16) * (VGA_WIDTH as u16) + (x as u16);
    unsafe {
        outb(VGA_CTRL_REGISTER, VGA_CURSOR_HIGH);
        outb(VGA_DATA_REGISTER, (pos >> 8) as u8);
        outb(VGA_CTRL_REGISTER, VGA_CURSOR_LOW);
        outb(VGA_DATA_REGISTER, (pos & 0xff) as u8);
    }
}

fn scroll() {
    let buffer = unsafe { VGA_BUFFER };
    unsafe {
        for y in 0..(VGA_HEIGHT - 1) {
            for x in 0..VGA_WIDTH {
                *buffer.add(y * VGA_WIDTH + x) =
                    *buffer.add((y + 1) * VGA_WIDTH + x);
            }
        }
        let color = CURRENT_COLOR;
        for x in 0..VGA_WIDTH {
            *buffer.add((VGA_HEIGHT - 1) * VGA_WIDTH + x) = vga_entry(b' ', color);
        }
    }
}

/// Safe VGA text-mode console (x86_64 only).
///
/// All methods update the on-screen 80x25 buffer and the hardware cursor.
/// Byte-oriented output accepts Code Page 437 values (used by the terminal
/// banner), so [`putchar`](Self::putchar) takes `u8` rather than `char`.
pub struct VgaText;

impl VgaText {
    /// Initialize the console: default color (light grey on black) and a
    /// cleared screen.
    pub fn init() {
        unsafe {
            CURRENT_COLOR = 0x07;
            CURSOR_X = 0;
            CURSOR_Y = 0;
        }
        Self::clear();
    }

    /// Clear the screen and reset the cursor to (0, 0).
    pub fn clear() {
        let buffer = unsafe { VGA_BUFFER };
        let color = unsafe { CURRENT_COLOR };
        unsafe {
            for y in 0..VGA_HEIGHT {
                for x in 0..VGA_WIDTH {
                    *buffer.add(y * VGA_WIDTH + x) = vga_entry(b' ', color);
                }
            }
            CURSOR_X = 0;
            CURSOR_Y = 0;
        }
        update_cursor();
    }

    /// Set the foreground/background color (4-bit VGA palette values).
    pub fn set_color(fg: u8, bg: u8) {
        unsafe {
            CURRENT_COLOR = ((bg & 0x0f) << 4) | (fg & 0x0f);
        }
    }

    /// Move the hardware cursor to `(x, y)`. Out-of-range positions are
    /// ignored (matches the C driver).
    pub fn set_cursor(x: u8, y: u8) {
        if (x as usize) < VGA_WIDTH && (y as usize) < VGA_HEIGHT {
            unsafe {
                CURSOR_X = x;
                CURSOR_Y = y;
            }
            update_cursor();
        }
    }

    /// Current cursor column.
    pub fn cursor_x() -> u8 {
        unsafe { CURSOR_X }
    }

    /// Current cursor row.
    pub fn cursor_y() -> u8 {
        unsafe { CURSOR_Y }
    }

    /// Write a single byte (Code Page 437 / control char) to the console.
    pub fn putchar(c: u8) {
        match c {
            b'\n' => {
                unsafe {
                    CURSOR_X = 0;
                    CURSOR_Y = CURSOR_Y.wrapping_add(1);
                }
            }
            b'\r' => unsafe { CURSOR_X = 0; },
            b'\t' => unsafe { CURSOR_X = ((CURSOR_X as usize + 8) & !7) as u8; },
            b'\x08' => unsafe {
                if CURSOR_X > 0 {
                    CURSOR_X = CURSOR_X.wrapping_sub(1);
                }
            },
            _ => {
                let buffer = unsafe { VGA_BUFFER };
                let color = unsafe { CURRENT_COLOR };
                let x = unsafe { CURSOR_X };
                let y = unsafe { CURSOR_Y };
                unsafe { *buffer.add(y as usize * VGA_WIDTH + x as usize) = vga_entry(c, color); }
                unsafe { CURSOR_X = CURSOR_X.wrapping_add(1); }
            }
        }
        if unsafe { CURSOR_X } as usize >= VGA_WIDTH {
            unsafe { CURSOR_X = 0; }
            unsafe { CURSOR_Y = CURSOR_Y.wrapping_add(1); }
        }
        if unsafe { CURSOR_Y } as usize >= VGA_HEIGHT {
            scroll();
            unsafe { CURSOR_Y = (VGA_HEIGHT - 1) as u8; }
        }
        update_cursor();
    }

    /// Write raw bytes to the console (no newline appended).
    pub fn print_bytes(bytes: &[u8]) {
        for &b in bytes {
            Self::putchar(b);
        }
    }

    /// Write raw bytes followed by a newline.
    pub fn println_bytes(bytes: &[u8]) {
        Self::print_bytes(bytes);
        Self::putchar(b'\n');
    }

    /// Write a UTF-8 string to the console.
    pub fn print(s: &str) {
        Self::print_bytes(s.as_bytes());
    }

    /// Write a UTF-8 string followed by a newline.
    pub fn println(s: &str) {
        Self::println_bytes(s.as_bytes());
    }

    /// Write `value` as `0x` + 8 uppercase hex digits.
    pub fn print_hex(value: u32) {
        Self::print_bytes(b"0x");
        let digits = b"0123456789ABCDEF";
        let mut buf = [b'0'; 8];
        for i in 0..8 {
            buf[i] = digits[((value >> (28 - i * 4)) & 0xF) as usize];
        }
        Self::print_bytes(&buf);
    }

    /// Write `value` as a decimal number.
    pub fn print_dec(value: u32) {
        if value == 0 {
            Self::putchar(b'0');
            return;
        }
        let mut buf = [0u8; 12];
        let mut i = 0;
        let mut v = value;
        while v > 0 {
            buf[i] = b'0' + (v % 10) as u8;
            v /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            Self::putchar(buf[i]);
        }
    }
}
