#![no_std]
#![feature(alloc_error_handler)]

extern crate core;
extern crate alloc;

pub mod allocator;
pub mod heap;
pub mod slab;
pub mod sync;
pub mod ffi;
pub mod panic;
pub mod terminal;
pub mod utils_rs;
pub use utils_rs as utils;
pub mod fs;
pub mod process;
pub mod syscall;
pub mod elf;
pub mod graphics;
pub mod fusion;
pub mod net;
pub mod display_server;

use core::panic::PanicInfo;

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

#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel entry - starting Display Server\n\0".as_ptr());
        ffi::vga_clear();
    }

    crate::fs::vfs_init();
    unsafe { ffi::serial_print(b"[VFS] initialized\n\0".as_ptr()); }

    if let Some(display) = graphics::vesa::VesaDisplay::new() {
        unsafe {
            ffi::serial_print(b"[Rust] VESA display initialized, booting display server\n\0".as_ptr());
        }

        match display_server::run(display) {
            Ok(()) => {
                unsafe {
                    ffi::serial_print(b"[Rust] Display server exited normally\n\0".as_ptr());
                }
            }
            Err(err) => {
                log_display_server_error(err);
                unsafe {
                    ffi::serial_print(b"[Rust] Display server boot failed\n\0".as_ptr());
                }
            }
        }
    } else {
        unsafe {
            ffi::serial_print(b"[Rust] Failed to initialize VESA display (headless mode)\n\0".as_ptr());
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
