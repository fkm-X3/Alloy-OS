//! Safe console output for the kernel.
//!
//! `println!`/`print!`/`log!` write to the serial UART that the boot code
//! initialized (`Serial::init`). The macros are formatting macros over
//! `core::fmt` (no heap, no `std`). Raw pointers never appear here; the
//! unsafe register access lives behind the `Serial` facade in
//! `alloy-kernel-unsafe-core`.

use crate::Serial;

/// The kernel console. Implements `core::fmt::Write` so `write!`/`format!`
/// style formatting can target the serial UART.
pub struct Console;

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Serial::write_str(s);
        Ok(())
    }
}

/// Write formatted output to the console (no trailing newline).
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        let mut console = $crate::console::Console;
        let _ = core::fmt::Write::write_fmt(&mut console, format_args!($($arg)*));
    }};
}

/// Write formatted output to the console followed by a newline.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::print!($($arg)*);
        $crate::print!("\n");
    }};
}

/// Write a `[LOG]`-prefixed line to the console.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        $crate::print!("[LOG] ");
        $crate::print!($($arg)*);
        $crate::print!("\n");
    }};
}

/// Print a plain string (no newline).
pub fn print_str(s: &str) {
    Serial::write_str(s);
}

/// Print a plain string followed by a newline.
pub fn println_str(s: &str) {
    Serial::write_str(s);
    Serial::write_byte(b'\n');
}

/// Print `value` as `0x` + 8 uppercase hex digits.
pub fn print_hex(value: u32) {
    Serial::write_hex(value);
}

/// Print `value` as `0x` + 16 uppercase hex digits.
pub fn print_hex64(value: u64) {
    Serial::write_hex64(value);
}
