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
pub mod block;
pub mod process;
pub mod syscall;
pub mod elf;
pub mod graphics;
pub mod fusion;
pub mod net;
pub mod display_server;
pub mod shm_alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;

extern "C" fn display_server_entry() {
    if let Some(display) = graphics::PlatformDisplay::new() {
        #[cfg(any(feature = "i686", feature = "x86_64"))]
        unsafe {
            ffi::serial_print(c"[Spawn] VESA ready, booting display server task\n".as_ptr() as *const u8);
        }
        #[cfg(feature = "aarch64")]
        unsafe {
            ffi::serial_print(c"[Spawn] PL110 ready, booting display server task\n".as_ptr() as *const u8);
        }
        let _ = display_server::run(display);
    }
}

fn log_display_server_error(err: display_server::DisplayServerBootError) {
    unsafe {
        let msg = err.serial_message();
        ffi::serial_print(msg.as_ptr());
        let code = err.code();
        ffi::serial_print(c" (code: ".as_ptr() as *const u8);
        for &byte in code.as_bytes() {
            #[cfg(any(feature = "i686", feature = "x86_64"))]
            ffi::vga_putchar(byte);
        }
        ffi::serial_print(c")\n".as_ptr() as *const u8);
        #[cfg(any(feature = "i686", feature = "x86_64"))]
        ffi::vga_print(err.vga_message().as_ptr() as *const u8);
    }
}

#[no_mangle]
pub extern "C" fn rust_main() {
    // Initialize the HAL platform (marks FFI as ready)
    alloy_kernel_hal::platform::init();

    unsafe {
        ffi::serial_print(c"[Rust] Kernel entry - initializing subsystems\n".as_ptr() as *const u8);
        #[cfg(any(feature = "i686", feature = "x86_64"))]
        ffi::vga_clear();
    }

    crate::fs::vfs_init();
    unsafe { ffi::serial_print(c"[VFS] initialized\n".as_ptr() as *const u8); }

    // Auto-mount FAT32 on any block devices
    #[cfg(any(feature = "i686", feature = "x86_64"))]
    {
        let dev_count = fs::vfs_block_device_count();
        for dev_id in 0..dev_count {
            let ns = fs::vfs_block_device_sectors(dev_id);
            if ns < 512 { continue; }
            let _ = fs::vfs_mount_fat32(dev_id, "/mnt/disk");
            if let Ok(entries) = fs::vfs_list_fat32(dev_id) {
                unsafe {
                    let msg = c"[VFS] Mounted FAT32 dev #";
                    ffi::serial_print(msg.as_ptr() as *const u8);
                    ffi::serial_print_hex(dev_id as u32);
                    ffi::serial_print(c"\n".as_ptr() as *const u8);
                }
                for entry in entries {
                    let name_s = core::str::from_utf8(&entry.name[..entry.name_len]).unwrap_or("?");
                    unsafe {
                        ffi::serial_print(c"  ".as_ptr() as *const u8);
                        ffi::serial_print(name_s.as_ptr());
                        ffi::serial_print(c"\n".as_ptr() as *const u8);
                    }
                }
            }
        }
    }

    unsafe {
        ffi::serial_print(c"[Rust] Initializing scheduler\n".as_ptr() as *const u8);
    }
    process::Scheduler::init();

    // Create the display server task (LXQt shell + Wayland server)
    let display_task = Box::new(process::task::Task::new(display_server_entry, "display-server"));
    process::Scheduler::add_task(display_task);

    // Spawn the primary DE (alloy_de) as a userspace process
    #[cfg(any(feature = "i686", feature = "x86_64"))]
    {
        if let Ok(de_vnode) = fs::vfs_open("/bin/alloy_de", 0, 0) {
            if let Some(image) = fs::vfs_read_all(de_vnode) {
                if !image.is_empty() {
                    unsafe {
                        ffi::serial_print(c"[Spawn] Loading alloy_de DE\n".as_ptr() as *const u8);
                    }
                    if process::spawn_user_elf(&image) {
                        unsafe {
                            ffi::serial_print(c"[Spawn] alloy_de DE task created\n".as_ptr() as *const u8);
                        }
                    }
                }
            }
        } else {
            // Fall back to the compositor if alloy_de isn't available
            if let Ok(comp_vnode) = fs::vfs_open("/bin/compositor", 0, 0) {
                if let Some(image) = fs::vfs_read_all(comp_vnode) {
                    if !image.is_empty() {
                        unsafe {
                            ffi::serial_print(c"[Spawn] Loading userspace compositor (fallback)\n".as_ptr() as *const u8);
                        }
                        if process::spawn_user_elf(&image) {
                            unsafe {
                                ffi::serial_print(c"[Spawn] Compositor task created\n".as_ptr() as *const u8);
                            }
                        }
                    }
                }
            }
        }
    }

    unsafe {
        ffi::serial_print(c"[Rust] Starting scheduler — entering multitasking\n".as_ptr() as *const u8);
    }
    process::Scheduler::start();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
