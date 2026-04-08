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
        ffi::serial_print(b"[Rust] Kernel entry - starting terminal\n\0".as_ptr());
        
        // Clear screen to hide C++ boot messages
        ffi::vga_clear();
    }
    
    // Start terminal - this never returns
    let mut terminal = terminal::Terminal::new();
    terminal.run();
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
