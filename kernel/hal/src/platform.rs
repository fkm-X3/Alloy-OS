//! Global hardware platform initialization and access
//!
//! This module provides a singleton `Platform` that is initialized once
//! during kernel boot and provides safe access to all hardware components.
//!
//! Usage:
//!   hal::platform::init();
//!   hal::println!("Hello");

use core::sync::atomic::{AtomicBool, Ordering};

static PLATFORM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the hardware platform. Must be called once during boot,
/// after the C boot code has completed early initialization.
///
/// This sets up the HAL's FFI-accessible state and marks the platform
/// as ready. After this call, all [`crate::ffi`] functions are usable.
///
/// # Panics
/// Panics if called more than once.
pub fn init() {
    if PLATFORM_INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        panic!("hal::platform::init() called twice");
    }
}

/// Check whether the platform has been initialized.
pub fn is_initialized() -> bool {
    PLATFORM_INITIALIZED.load(Ordering::Relaxed)
}

/// Print a string to the serial port.
#[inline]
pub fn serial_print(s: &str) {
    crate::console::print_str(s);
}

/// Print a hex value to the serial port.
#[inline]
pub fn serial_print_hex(value: u32) {
    crate::console::print_hex(value);
}
