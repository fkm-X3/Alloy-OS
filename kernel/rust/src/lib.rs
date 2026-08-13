#![no_std]
#![feature(alloc_error_handler)]

extern crate core;
extern crate alloc;

// Safe console + driver facades from the HAL (which re-exports
// `unsafe_core::api`). The kernel crate no longer touches raw `ffi::*`
// pointers for serial/VGA/timer output.
pub use alloy_kernel_hal::{log, print, println};
pub use alloy_kernel_hal::{Serial, SystemTimer};
#[cfg(feature = "x86_64")]
pub use alloy_kernel_hal::VgaText;

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
use crate::graphics::Display;

extern "C" fn display_server_entry() {
    // Disable interrupts during init so the timer can't preempt us.
    // The scheduler leaks any task that is preempted before it voluntarily
    // yields or exits (old_box_opt is never re-enqueued). The guard restores
    // the previous mask state on drop; `release` re-enables IRQs before the
    // run loop.
    let mut irq_guard = alloy_kernel_hal::InterruptGuard::new();

    unsafe {
        crate::println!("[DisplayServer] Entry reached");
    }
    match graphics::PlatformDisplay::new() {
        None => {
            crate::println!("[DisplayServer] FATAL: PlatformDisplay::new() returned None");
            irq_guard.release();
            loop {
                irq_guard.halt();
            }
        }
        Some(mut display) => {
            // Set background to #0f0f1a immediately (requirement 4.6.1)
            display.clear(0xFF0F0F1A);
            display.swap_buffer();

            #[cfg(feature = "x86_64")]
            crate::println!("[Spawn] VESA ready, booting display server task");
            #[cfg(feature = "aarch64")]
            crate::println!("[Spawn] PL110 ready, booting display server task");
            irq_guard.release();
            let _ = display_server::run(display);
            crate::println!("[DisplayServer] run() returned, halting");
            loop {
                irq_guard.halt();
            }
        }
    }
}

fn log_display_server_error(err: display_server::DisplayServerBootError) {
    let msg = err.serial_message();
    let len = msg.iter().position(|&b| b == 0).unwrap_or(msg.len());
    crate::Serial::write_bytes(&msg[..len]);
    let code = err.code();
    crate::print!(" (code: ");
    for &byte in code.as_bytes() {
        #[cfg(feature = "x86_64")]
        crate::VgaText::putchar(byte);
    }
    crate::println!(")");
    #[cfg(feature = "x86_64")]
    {
        let vmsg = err.vga_message();
        let vlen = vmsg.iter().position(|&b| b == 0).unwrap_or(vmsg.len());
        crate::VgaText::println_bytes(&vmsg[..vlen]);
    }
}

#[no_mangle]
pub extern "C" fn rust_main() {
    // Initialize the HAL platform (marks FFI as ready)
    alloy_kernel_hal::platform::init();

    // Register the kernel's syscall/timer/page-fault handlers with
    // unsafe-core before anything can fire them: no userland exists yet and
    // the timer is armed later by SystemTimer::init. This replaces the
    // `rust_sys_*`/`rust_timer_tick`/`rust_handle_page_fault` symbol calls.
    crate::syscall::register_all();
    alloy_kernel_hal::set_timer_tick_handler(process::Scheduler::rust_timer_tick);
    alloy_kernel_hal::set_page_fault_handler(process::Scheduler::rust_handle_page_fault);

    crate::println!("[Rust] Kernel entry - initializing subsystems");
    crate::println!("[Rust] About to vga_clear");
    #[cfg(feature = "x86_64")]
    crate::VgaText::clear();
    crate::println!("[Rust] vga_clear done");

    crate::fs::vfs_init();
    crate::println!("[VFS] initialized");

    // Auto-mount FAT32 on any block devices
    #[cfg(feature = "x86_64")]
    {
        let dev_count = fs::vfs_block_device_count();
        for dev_id in 0..dev_count {
            let ns = fs::vfs_block_device_sectors(dev_id);
            if ns < 512 { continue; }
            let _ = fs::vfs_mount_fat32(dev_id, "/mnt/disk");
            if let Ok(entries) = fs::vfs_list_fat32(dev_id) {
                crate::println!("[VFS] Mounted FAT32 dev #0x{:08X}", dev_id);
                for entry in entries {
                    let name_s = core::str::from_utf8(&entry.name[..entry.name_len]).unwrap_or("?");
                    crate::println!("  {name_s}");
                }
            }
        }
    }

    crate::println!("[Rust] Initializing scheduler");
    process::Scheduler::init();

    // Create the display server task (LXQt shell + Wayland server)
    let display_task = Box::new(process::task::Task::new(display_server_entry, "display-server"));
    process::Scheduler::add_task(display_task);

    // aarch64: no DE (x86_64-only), so exercise the EL0 svc syscall path
    // with the hello binary (identity-mapped, fixed physical base).
    #[cfg(feature = "aarch64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/hello", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        crate::println!("[Spawn] Loading hello (EL0 svc syscall test)");
                    }
                    if process::spawn_user_elf(&image) {
                        unsafe {
                            crate::println!("[Spawn] hello task created");
                        }
                    }
                }
            }
        }
    }

    // Spawn forktest (x86_64 COW fork smoke: parent forks, child writes a
    // shared page triggering a COW split; both print their views of the var)
    #[cfg(feature = "x86_64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/forktest", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        crate::println!("[Spawn] Loading forktest (COW fork smoke)");
                    }
                    if process::spawn_user_elf(&image) {
                        unsafe {
                            crate::println!("[Spawn] forktest task created");
                        }
                    }
                }
            }
        }
    }

    // Spawn test_wl_client (Wayland client protocol test)
    #[cfg(feature = "x86_64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/test_wl_client", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        crate::println!("[Spawn] Loading test_wl_client (Wayland client test)");
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
                        crate::println!("[Spawn] Loading hello_cpp (C++ test)");
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
                        crate::println!("[Spawn] Loading compositor");
                    }
                    if process::spawn_user_elf(&image) {
                        unsafe {
                            crate::println!("[Spawn] Compositor task created");
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
                        crate::println!("[Spawn] Loading test_window (Qt6 QPA test)");
                    }
                    process::spawn_user_elf(&image);
                }
            }
        }
    }

    // Spawn the QML desktop environment (Qt6 Quick + Wayland)
    #[cfg(feature = "x86_64")]
    {
        if let Ok(vnode) = fs::vfs_open("/bin/alloy_de_qml", 0, 0) {
            if let Some(image) = fs::vfs_read_all(vnode) {
                if !image.is_empty() {
                    unsafe {
                        crate::println!("[Spawn] Loading alloy_de_qml (QML desktop environment)");
                    }
                    process::spawn_user_elf(&image);
                }
            }
        }
    }

    crate::println!("[Rust] Starting scheduler — entering multitasking");
    crate::SystemTimer::init(1000);
    process::Scheduler::start();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
