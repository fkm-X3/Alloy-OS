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
pub mod net;
pub mod display_server;

use core::panic::PanicInfo;

fn mode_from_cosmos_runtime(runtime: alloy_os_cosmos_de::CosmosRuntime) -> display_server::BootUiMode {
    match runtime {
        alloy_os_cosmos_de::CosmosRuntime::Cosmos => display_server::BootUiMode::Cosmos,
        alloy_os_cosmos_de::CosmosRuntime::IcedPrimary => display_server::BootUiMode::IcedPrimary,
    }
}

fn log_selected_boot_mode(mode: display_server::BootUiMode) {
    unsafe {
        match mode {
            display_server::BootUiMode::Cosmos => {
                ffi::serial_print(b"[Rust] Boot mode selected: Cosmos\n\0".as_ptr())
            }
            display_server::BootUiMode::IcedPrimary => {
                ffi::serial_print(b"[Rust] Boot mode selected: Iced-primary\n\0".as_ptr())
            }
        }
    }
}

fn log_fallback_boot_mode(mode: display_server::BootUiMode) {
    unsafe {
        match mode {
            display_server::BootUiMode::Cosmos => {
                ffi::serial_print(b"[Rust] Retrying display server with Cosmos mode\n\0".as_ptr())
            }
            display_server::BootUiMode::IcedPrimary => {
                ffi::serial_print(b"[Rust] Retrying display server with Iced-primary mode\n\0".as_ptr())
            }
        }
    }
}

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

let cosmos_bootstrap = alloy_os_cosmos_de::bootstrap();
     unsafe {
         ffi::serial_print(cosmos_bootstrap.summary_serial_line().as_ptr());
     }
     let primary_boot_ui_mode = mode_from_cosmos_runtime(cosmos_bootstrap.profile.primary_runtime);
     let fallback_boot_ui_mode = mode_from_cosmos_runtime(cosmos_bootstrap.profile.fallback_runtime);
     log_selected_boot_mode(primary_boot_ui_mode);

     // Initialize and run the desktop display server
     // Pass the Cosmos bootstrap report to the display server so it can
     // configure session boundaries and the Wayland compositor bridge
     if let Some(display) = graphics::vesa::VesaDisplay::new() {
         unsafe {
             ffi::serial_print(b"[Rust] VESA display initialized, booting display server\n\0".as_ptr());
         }

         match display_server::run_with_bootstrap(display, primary_boot_ui_mode, cosmos_bootstrap.clone()) {
             Ok(()) => {
                 unsafe {
                     ffi::serial_print(b"[Rust] Display server exited normally\n\0".as_ptr());
                 }
             }
             Err(err) => {
                 log_display_server_error(err);
                 if fallback_boot_ui_mode != primary_boot_ui_mode {
                     log_fallback_boot_mode(fallback_boot_ui_mode);
                     // Re-run bootstrap for fallback mode
                     let fallback_bootstrap = if cfg!(feature = "cosmos") {
                         alloy_os_cosmos_de::bootstrap()
                     } else {
                         cosmos_bootstrap.clone()
                     };
                     if let Some(fallback_display) = graphics::vesa::VesaDisplay::new() {
                        match display_server::run_with_bootstrap(fallback_display, fallback_boot_ui_mode, fallback_bootstrap) {
                            Ok(()) => unsafe {
                                ffi::serial_print(b"[Rust] Display server fallback exited normally\n\0".as_ptr())
                            },
                            Err(fallback_err) => {
                                log_display_server_error(fallback_err);
                                unsafe {
                                    ffi::serial_print(
                                        b"[Rust] Cosmos DE boot and fallback mode both failed\n\0".as_ptr(),
                                    );
                                }
                            }
                        }
                    } else {
                        unsafe {
                            ffi::serial_print(
                                b"[Rust] Failed to initialize VESA display for fallback mode\n\0".as_ptr(),
                            );
                        }
                    }
                } else {
                    unsafe {
                        ffi::serial_print(
                            b"[Rust] Cosmos DE boot failed and integration surface did not provide a fallback mode\n\0"
                                .as_ptr(),
                        );
                    }
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
