/// Panic handler for no_std Rust kernel
/// 
/// This module handles panics in the Rust kernel by printing
/// panic information to serial output and halting the system.

use core::panic::PanicInfo;
use core::fmt::Write;
use crate::ffi;

/// Custom writer for serial output
struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        ffi::print_str(s);
        Ok(())
    }
}

/// Panic handler - called when Rust code panics
pub fn panic_handler(info: &PanicInfo) -> ! {
    let mut writer = SerialWriter;
    
    // Print panic message to serial
    let _ = write!(writer, "\n!!! KERNEL PANIC !!!\n");
    
    if let Some(location) = info.location() {
        let _ = write!(
            writer,
            "Panic at {}:{}:{}\n",
            location.file(),
            location.line(),
            location.column()
        );
    }
    
    let _ = write!(writer, "Message: {}\n", info.message());
    
    // Also print to VGA
    unsafe {
        ffi::vga_set_color(4, 0); // Red text
        ffi::vga_println(b"\n!!! KERNEL PANIC !!!\0".as_ptr());
    }
    
    // Halt the system
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
