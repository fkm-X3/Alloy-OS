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
#[cfg(feature = "x86_64")]
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
        #[cfg(feature = "x86_64")]
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
            #[cfg(feature = "x86_64")]
            ffi::vga_putchar(byte);
        }
        ffi::serial_print(c")\n".as_ptr() as *const u8);
        #[cfg(feature = "x86_64")]
        ffi::vga_print(err.vga_message().as_ptr() as *const u8);
    }
}

#[no_mangle]
pub extern "C" fn rust_main() {
    // Initialize the HAL platform (marks FFI as ready)
    alloy_kernel_hal::platform::init();

    unsafe {
        ffi::serial_print(c"[Rust] Kernel entry - initializing subsystems\n".as_ptr() as *const u8);
        ffi::serial_print(c"[Rust] About to vga_clear\n".as_ptr() as *const u8);
        #[cfg(feature = "x86_64")]
        ffi::vga_clear();
        ffi::serial_print(c"[Rust] vga_clear done\n".as_ptr() as *const u8);
    }

    crate::fs::vfs_init();
    unsafe { ffi::serial_print(c"[VFS] initialized\n".as_ptr() as *const u8); }

    // Auto-mount FAT32 on any block devices
    #[cfg(feature = "x86_64")]
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

    // Spawn test_wl_client (Wayland client protocol test)
    #[cfg(feature = "x86_64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/test_wl_client", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        ffi::serial_print(c"[Spawn] Loading test_wl_client (Wayland client test)\n".as_ptr() as *const u8);
                    }
                    process::spawn_user_elf(&image);
                }
            }
        }
    }

    // Spawn hello_cpp test (C++ static init + main test)
    #[cfg(feature = "x86_64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/hello_cpp", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        ffi::serial_print(c"[Spawn] Loading hello_cpp (C++ test)\n".as_ptr() as *const u8);
                    }
                    process::spawn_user_elf(&image);
                }
            }
        }
    }

    // Spawn the Wayland compositor as a userspace process
    #[cfg(feature = "x86_64")]
    {
        if let Ok(comp_vnode) = fs::vfs_open("/bin/compositor", 0, 0) {
            if let Some(image) = fs::vfs_read_all(comp_vnode) {
                if !image.is_empty() {
                    unsafe {
                        ffi::serial_print(c"[Spawn] Loading compositor\n".as_ptr() as *const u8);
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

    // Spawn the Qt6 test window (requires compositor to be ready)
    #[cfg(feature = "x86_64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/test_window", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        ffi::serial_print(c"[Spawn] Loading test_window (Qt6 QPA test)\n".as_ptr() as *const u8);
                    }
                    process::spawn_user_elf(&image);
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
