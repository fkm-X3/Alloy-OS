#![no_std]
#![feature(alloc_error_handler)]

// Core library - available in no_std
extern crate core;

// Alloc library for heap allocations
extern crate alloc;

// Module declarations
pub mod allocator;
pub mod ffi;
pub mod panic;

use core::panic::PanicInfo;

use alloc::vec::Vec;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel initialized!\n\0".as_ptr());
        
        // Test the allocator with a vector
        ffi::serial_print(b"[Rust] Testing allocator...\n\0".as_ptr());
    }
    
    let mut test_vec: Vec<u32> = Vec::new();
    test_vec.push(42);
    test_vec.push(1337);
    test_vec.push(9001);
    
    unsafe {
        ffi::serial_print(b"[Rust] Allocator works! Created vector with 3 elements\n\0".as_ptr());
        
        // Display success message
        ffi::vga_println(b"[ ] Rust integration...\0".as_ptr());
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
        
        ffi::serial_print(b"[Rust] Ready for higher-level kernel services!\n\0".as_ptr());
    }
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
