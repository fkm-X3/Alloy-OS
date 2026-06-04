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
        ffi::serial_print(c" (code: ".as_ptr() as *const u8);
        for &byte in code.as_bytes() {
            ffi::vga_putchar(byte);
        }
        ffi::serial_print(c")\n".as_ptr() as *const u8);
    }
}

#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(c"[Rust] Kernel entry - starting Display Server\n".as_ptr() as *const u8);
        ffi::vga_clear();
    }

    crate::fs::vfs_init();
    unsafe { ffi::serial_print(c"[VFS] initialized\n".as_ptr() as *const u8); }

    if let Some(display) = graphics::vesa::VesaDisplay::new() {
        unsafe {
            ffi::serial_print(c"[Rust] VESA display initialized, booting display server\n".as_ptr() as *const u8);
        }

        match display_server::run(display) {
            Ok(()) => {
                unsafe {
                    ffi::serial_print(c"[Rust] Display server exited normally\n".as_ptr() as *const u8);
                }
            }
            Err(err) => {
                log_display_server_error(err);
                unsafe {
                    ffi::serial_print(c"[Rust] Display server boot failed\n".as_ptr() as *const u8);
                }
            }
        }
    } else {
        unsafe {
            ffi::serial_print(c"[Rust] Failed to initialize VESA display (headless mode)\n".as_ptr() as *const u8);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
