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
pub mod utils_rs;
pub use utils_rs as utils;
pub mod fs;
pub mod process;
pub mod syscall;
pub mod elf;
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

    // Initialize VFS early so userland can use files
    crate::fs::vfs_init();
    unsafe { ffi::serial_print(b"[VFS] initialized\n\0".as_ptr()); }
    
    // Quick test: verify /hello exists in VFS by trying to open and read it
    if let Ok(hello_id) = crate::fs::vfs_open("/hello", 0, 0) {
        if let Some(hello_data) = crate::fs::vfs_read_all(hello_id) {
            unsafe {
                ffi::serial_print(b"[Test] SUCCESS: /hello found in VFS, size=\0".as_ptr());
                ffi::serial_print(b"\0".as_ptr());
                // Print size via serial (easier than VGA in headless mode)
                let sz = hello_data.len();
                let sz_str = alloc::format!("{}", sz);
                let sz_bytes = sz_str.as_bytes();
                for &b in sz_bytes {
                    ffi::vga_putchar(b);
                }
                ffi::serial_print(b" bytes\n\0".as_ptr());
            }
        }
    }
    
    // TEST: Execute /hello to verify userland execution works
    unsafe {
        ffi::serial_print(b"[Rust] TESTING: Executing /hello via execve...\n\0".as_ptr());
    }
    let hello_path = "/hello";
    let path_bytes = hello_path.as_bytes();
    let mut path_buf = [0u8; 256];
    path_buf[..path_bytes.len()].copy_from_slice(path_bytes);
    let path_ptr = &path_buf as *const _ as u32;
    let exec_result = crate::syscall::rust_sys_execve(path_ptr);
    unsafe {
        if exec_result != core::u32::MAX {
            ffi::serial_print(b"[Rust] EXECVE TEST COMPLETE\n\0".as_ptr());
        } else {
            ffi::serial_print(b"[Rust] EXECVE TEST FAILED\n\0".as_ptr());
        }
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
            ffi::serial_print(b"[Rust] Failed to initialize VESA display (headless mode)\n\0".as_ptr());
        }
    }
}


/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
