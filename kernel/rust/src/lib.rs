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

use core::panic::PanicInfo;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel entry - Phase 8\n\0".as_ptr());
        ffi::vga_println(b"[ ] Rust kernel initialization...\0".as_ptr());
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
        
        ffi::vga_println(b"\n======================================\0".as_ptr());
        ffi::vga_set_color(11, 0); // Cyan
        ffi::vga_println(b"  Alloy Operating System - Phase 8\0".as_ptr());
        ffi::vga_set_color(7, 0);
        ffi::vga_println(b"======================================\n\0".as_ptr());
        ffi::vga_println(b"System Integration Complete!\n\0".as_ptr());
        
        ffi::serial_print(b"[Rust] Testing timer functionality...\n\0".as_ptr());
        ffi::vga_println(b"[Timer] Testing PIT timer...\0".as_ptr());
        
        // Test timer - wait a bit and show uptime
        let start = ffi::timer_get_ticks_ffi();
        ffi::serial_print(b"[Timer] Initial ticks: ...\n\0".as_ptr());
        
        // Busy wait for a moment
        for _ in 0..10000000 {
            core::arch::asm!("nop");
        }
        
        let end = ffi::timer_get_ticks_ffi();
        ffi::serial_print(b"[Timer] Ticks after delay: ...\n\0".as_ptr());
        
        ffi::vga_set_color(10, 0);
        ffi::vga_println(b" Timer working!\0".as_ptr());
        ffi::vga_set_color(7, 0);
        
        // Test syscall interface
        ffi::serial_print(b"[Rust] Testing syscall interface...\n\0".as_ptr());
        ffi::vga_println(b"\n[Syscall] Testing system calls...\0".as_ptr());
        
        // Test getpid syscall
        let pid = syscall::syscall(syscall::SyscallNumber::GetPid, 0, 0, 0);
        ffi::serial_print(b"[Syscall] getpid returned: ...\n\0".as_ptr());
        
        ffi::vga_set_color(10, 0);
        ffi::vga_println(b" Syscalls working!\0".as_ptr());
        ffi::vga_set_color(7, 0);
        
        // Show success message
        ffi::vga_println(b"\n======================================\0".as_ptr());
        ffi::vga_set_color(10, 0);
        ffi::vga_println(b"  Phase 8 Complete!\0".as_ptr());
        ffi::vga_set_color(7, 0);
        ffi::vga_println(b"======================================\n\0".as_ptr());
        
        ffi::vga_println(b"Features implemented:\0".as_ptr());
        ffi::vga_println(b"  [*] PIT Timer (100 Hz)\0".as_ptr());
        ffi::vga_println(b"  [*] Timer IRQ Handler\0".as_ptr());
        ffi::vga_println(b"  [*] Uptime Tracking\0".as_ptr());
        ffi::vga_println(b"  [*] System Call Interface (INT 0x80)\0".as_ptr());
        ffi::vga_println(b"  [*] Basic Syscalls (exit, yield, getpid, sleep)\0".as_ptr());
        ffi::vga_println(b"\nKernel running. Press any key...\0".as_ptr());
        
        ffi::serial_print(b"[Rust] Phase 8 demonstration complete\n\0".as_ptr());
        ffi::serial_print(b"[Rust] Kernel entering idle loop\n\0".as_ptr());
    }
    
    // Enter idle loop
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
