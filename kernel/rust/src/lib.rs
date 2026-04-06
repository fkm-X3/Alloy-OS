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

use core::panic::PanicInfo;

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel entry - Phase 7\n\0".as_ptr());
        ffi::vga_println(b"[ ] Rust kernel initialization...\0".as_ptr());
    }
    
    // Skip memory test for now, go straight to process management
    unsafe {
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
        
        ffi::serial_print(b"[Rust] Initializing process management (Phase 7)\n\0".as_ptr());
        ffi::vga_println(b"[ ] Process management...\0".as_ptr());
    }
    
    process::Scheduler::init();
    
    unsafe {
        ffi::serial_print(b"[Rust] Creating demo tasks\n\0".as_ptr());
    }
    
    // Create demo tasks
    let task_a = Box::new(process::Task::new(task_a_entry, "Task A"));
    let task_b = Box::new(process::Task::new(task_b_entry, "Task B"));
    let task_c = Box::new(process::Task::new(task_c_entry, "Task C"));
    
    process::Scheduler::add_task(task_a);
    process::Scheduler::add_task(task_b);
    process::Scheduler::add_task(task_c);
    
    unsafe {
        ffi::vga_set_color(10, 0); // Green
        ffi::vga_println(b" OK\0".as_ptr());
        ffi::vga_set_color(7, 0); // Reset
        
        ffi::vga_println(b"\n======================================\0".as_ptr());
        ffi::vga_set_color(11, 0); // Cyan
        ffi::vga_println(b"  Alloy Operating System - Phase 7\0".as_ptr());
        ffi::vga_set_color(7, 0);
        ffi::vga_println(b"======================================\n\0".as_ptr());
        ffi::vga_println(b"Demonstrating cooperative multitasking...\0".as_ptr());
        ffi::vga_println(b"Watch the tasks switch in round-robin!\n\0".as_ptr());
        
        ffi::serial_print(b"[Rust] Starting scheduler...\n\0".as_ptr());
    }
    
    // Start scheduler (never returns)
    process::Scheduler::start();
}

// Demo task entry points
extern "C" fn task_a_entry() {
    for i in 0..5 {
        unsafe {
            ffi::vga_set_color(14, 0); // Yellow
            ffi::vga_print(b"[Task A] Running iteration \0".as_ptr());
            ffi::vga_set_color(7, 0);
            
            // Print number (simplified)
            ffi::vga_println(b"#\n\0".as_ptr());
            
            ffi::serial_print(b"[Task A] Iteration complete, yielding...\n\0".as_ptr());
        }
        
        // Simulate some work
        for _ in 0..1000000 {
            unsafe { core::arch::asm!("nop"); }
        }
        
        // Yield to other tasks
        process::Scheduler::yield_cpu();
    }
    
    unsafe {
        ffi::vga_set_color(14, 0);
        ffi::vga_println(b"[Task A] Completed!\n\0".as_ptr());
        ffi::vga_set_color(7, 0);
        ffi::serial_print(b"[Task A] Finished\n\0".as_ptr());
    }
    
    // Task complete - just halt for now
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

extern "C" fn task_b_entry() {
    for i in 0..5 {
        unsafe {
            ffi::vga_set_color(10, 0); // Green
            ffi::vga_print(b"[Task B] Running iteration \0".as_ptr());
            ffi::vga_set_color(7, 0);
            
            ffi::vga_println(b"#\n\0".as_ptr());
            
            ffi::serial_print(b"[Task B] Iteration complete, yielding...\n\0".as_ptr());
        }
        
        // Simulate some work
        for _ in 0..1000000 {
            unsafe { core::arch::asm!("nop"); }
        }
        
        process::Scheduler::yield_cpu();
    }
    
    unsafe {
        ffi::vga_set_color(10, 0);
        ffi::vga_println(b"[Task B] Completed!\n\0".as_ptr());
        ffi::vga_set_color(7, 0);
        ffi::serial_print(b"[Task B] Finished\n\0".as_ptr());
    }
    
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

extern "C" fn task_c_entry() {
    for i in 0..5 {
        unsafe {
            ffi::vga_set_color(13, 0); // Magenta
            ffi::vga_print(b"[Task C] Counting: \0".as_ptr());
            ffi::vga_set_color(7, 0);
            
            // Simple counter display
            ffi::vga_println(b"...\n\0".as_ptr());
            
            ffi::serial_print(b"[Task C] Count iteration, yielding...\n\0".as_ptr());
        }
        
        // Simulate some work
        for _ in 0..1000000 {
            unsafe { core::arch::asm!("nop"); }
        }
        
        process::Scheduler::yield_cpu();
    }
    
    unsafe {
        ffi::vga_set_color(13, 0);
        ffi::vga_println(b"[Task C] Completed!\n\0".as_ptr());
        ffi::vga_set_color(7, 0);
        ffi::serial_print(b"[Task C] Finished\n\0".as_ptr());
    }
    
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
