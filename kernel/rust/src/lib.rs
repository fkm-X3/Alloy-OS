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

use core::panic::PanicInfo;

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel initialized!\n\0".as_ptr());
        
        // Test allocators
        ffi::serial_print(b"[Rust] Testing memory allocators...\n\0".as_ptr());
    }
    
    // Quick allocator test
    let test_vec: Vec<u32> = vec![1, 2, 3, 4, 5];
    let test_string = String::from("Alloy OS Terminal");
    
    unsafe {
        ffi::serial_print(b"[Rust] Memory allocators working!\n\0".as_ptr());
        
        // Display success message
        ffi::vga_println(b"[ ] Advanced memory management...\0".as_ptr());
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
        
        ffi::serial_print(b"[Rust] Starting terminal...\n\0".as_ptr());
    }
    
    // Drop test allocations
    drop(test_vec);
    drop(test_string);
    
    // Create and run terminal
    let mut term = terminal::Terminal::new();
    term.run(); // This function never returns
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
