#![no_std]
#![feature(alloc_error_handler)]

// Core library - available in no_std
extern crate core;

// Alloc library for heap allocations
extern crate alloc;

// Module declarations
pub mod allocator;
pub mod heap;
pub mod slab;
pub mod sync;
pub mod ffi;
pub mod panic;
pub mod terminal;
pub mod utils;
pub mod process;
pub mod syscall;
pub mod graphics;
pub mod fusion;
pub mod display_server;

use core::panic::PanicInfo;

const PRIMARY_BOOT_UI_MODE: display_server::BootUiMode = display_server::BootUiMode::IcedPrimary;

fn log_display_server_error(err: display_server::DisplayServerBootError) {
    unsafe {
        let msg = err.serial_message();
        ffi::serial_print(msg.as_ptr());
        let code = err.code();
        ffi::serial_print(b" (code: \0".as_ptr());
        for &byte in code.as_bytes() {
            ffi::vga_putchar(byte);
        }
        ffi::serial_print(b")\n\0".as_ptr());
    }
}

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel entry - starting Display Server\n\0".as_ptr());
        
        // Clear screen
        ffi::vga_clear();
    }
    
    // Initialize and run the desktop display server
    if let Some(display) = graphics::vesa::VesaDisplay::new() {
        unsafe {
            ffi::serial_print(b"[Rust] VESA display initialized, booting display server\n\0".as_ptr());
        }
        
        match display_server::run(display, PRIMARY_BOOT_UI_MODE) {
            Ok(()) => {
                unsafe {
                    ffi::serial_print(b"[Rust] Display server exited normally\n\0".as_ptr());
                }
            }
            Err(err) => {
                log_display_server_error(err);
                unsafe {
                    ffi::serial_print(
                        b"[Rust] Iced-primary boot failed; desktop-shell fallback is disabled\n\0"
                            .as_ptr(),
                    );
                }
            }
        }
    } else {
        unsafe {
            ffi::serial_print(b"[Rust] Failed to initialize VESA display\n\0".as_ptr());
        }
    }
}


/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
