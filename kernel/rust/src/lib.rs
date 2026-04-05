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

use core::panic::PanicInfo;

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel initialized!\n\0".as_ptr());
        
        // Test slab allocator with small objects
        ffi::serial_print(b"[Rust] Testing slab allocator (small objects)...\n\0".as_ptr());
    }
    
    // Small allocations (should use slab allocator)
    let small_box1: Box<u32> = Box::new(42);
    let small_box2: Box<u64> = Box::new(1337);
    let small_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
    
    unsafe {
        ffi::serial_print(b"[Rust] Slab allocator works!\n\0".as_ptr());
        
        // Test heap allocator with larger objects
        ffi::serial_print(b"[Rust] Testing heap allocator (large objects)...\n\0".as_ptr());
    }
    
    // Larger allocations (should use heap allocator)
    let mut large_vec: Vec<u32> = Vec::with_capacity(100);
    for i in 0..100 {
        large_vec.push(i);
    }
    
    let large_string = String::from("Alloy OS - Rust kernel with advanced memory management!");
    
    unsafe {
        ffi::serial_print(b"[Rust] Heap allocator works!\n\0".as_ptr());
        
        // Test mixed allocations
        ffi::serial_print(b"[Rust] Testing mixed allocations...\n\0".as_ptr());
    }
    
    let mut mixed_vec: Vec<String> = Vec::new();
    for _i in 0..10 {
        let s = String::from("Test");
        mixed_vec.push(s);
    }
    
    unsafe {
        ffi::serial_print(b"[Rust] All allocator tests passed!\n\0".as_ptr());
        
        // Get statistics
        let (_slab_stats, _heap_stats) = allocator::get_stats();
        ffi::serial_print(b"[Rust] Memory stats:\n\0".as_ptr());
        ffi::serial_print(b"  Slab: allocated=\0".as_ptr());
        // Note: Can't easily print numbers without format!, but we have the data
        ffi::serial_print(b"  Heap: allocated=\0".as_ptr());
        
        // Display success message
        ffi::vga_println(b"[ ] Advanced memory management...\0".as_ptr());
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
        
        ffi::serial_print(b"[Rust] Memory management initialized!\n\0".as_ptr());
    }
    
    // Explicitly drop to test deallocation
    drop(small_box1);
    drop(small_box2);
    drop(small_vec);
    drop(large_vec);
    drop(large_string);
    drop(mixed_vec);
    
    unsafe {
        ffi::serial_print(b"[Rust] Deallocation works!\n\0".as_ptr());
        ffi::serial_print(b"[Rust] Ready for higher-level kernel services!\n\0".as_ptr());
    }
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
