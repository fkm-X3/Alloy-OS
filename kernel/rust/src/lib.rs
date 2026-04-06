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
pub mod ffi;
pub mod panic;
pub mod terminal;
pub mod utils;

use core::panic::PanicInfo;

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] === RUST KERNEL ENTRY POINT ===\n\0".as_ptr());
        ffi::serial_print(b"[Rust] Step 1: Entered rust_main()\n\0".as_ptr());
    }
    
    // Initialize allocators before any heap usage
    unsafe {
        ffi::serial_print(b"[Rust] Step 2: Initializing allocators...\n\0".as_ptr());
        // TODO: Call allocator initialization functions
        ffi::serial_print(b"[Rust] Step 3: Allocators initialized (stub)\n\0".as_ptr());
    }
    
    // Test simple allocation first
    unsafe {
        ffi::serial_print(b"[Rust] Step 4: Testing simple allocation...\n\0".as_ptr());
    }
    
    // Quick allocator test
    unsafe {
        ffi::serial_print(b"[Rust] Step 5: Creating Vec...\n\0".as_ptr());
    }
    let test_vec: Vec<u32> = vec![1, 2, 3, 4, 5];
    unsafe {
        ffi::serial_print(b"[Rust] Step 6: Vec created successfully\n\0".as_ptr());
    }
    
    unsafe {
        ffi::serial_print(b"[Rust] Step 7: Creating String...\n\0".as_ptr());
    }
    let test_string = String::from("Alloy OS Terminal");
    unsafe {
        ffi::serial_print(b"[Rust] Step 8: String created successfully\n\0".as_ptr());
    }
    
    unsafe {
        ffi::serial_print(b"[Rust] Step 9: Memory allocators working!\n\0".as_ptr());
        
        // Display success message
        ffi::vga_println(b"[ ] Advanced memory management...\0".as_ptr());
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
    }
    
    // Drop test allocations
    unsafe {
        ffi::serial_print(b"[Rust] Step 10: Dropping test allocations...\n\0".as_ptr());
    }
    drop(test_vec);
    drop(test_string);
    unsafe {
        ffi::serial_print(b"[Rust] Step 11: Test allocations dropped\n\0".as_ptr());
    }
    
    // Create and run terminal
    unsafe {
        ffi::serial_print(b"[Rust] Step 12: Creating terminal...\n\0".as_ptr());
    }
    let mut term = terminal::Terminal::new();
    unsafe {
        ffi::serial_print(b"[Rust] Step 13: Terminal created, entering run loop...\n\0".as_ptr());
    }
    term.run(); // This function never returns
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
