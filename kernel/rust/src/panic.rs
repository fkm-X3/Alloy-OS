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
    
    // Print panic banner to serial
    let _ = write!(writer, "\n");
    let _ = write!(writer, "╔═══════════════════════════════════╗\n");
    let _ = write!(writer, "║    KERNEL PANIC - SYSTEM HALTED   ║\n");
    let _ = write!(writer, "╚═══════════════════════════════════╝\n\n");
    
    // Location information
    if let Some(location) = info.location() {
        let _ = write!(
            writer,
            "Location: {}:{}:{}\n",
            location.file(),
            location.line(),
            location.column()
        );
    }
    
    // Panic message
    let _ = write!(writer, "Message:  {}\n\n", info.message());
    
    // Dump some CPU registers (simplified to avoid register pressure)
    let _ = write!(writer, "Register dump:\n");
    unsafe {
        let esp: u32;
        let ebp: u32;
        let eflags: u32;
        
        core::arch::asm!(
            "mov {0}, esp",
            "mov {1}, ebp",
            out(reg) esp,
            out(reg) ebp,
        );
        
        // Get EFLAGS
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            out(reg) eflags,
        );
        
        let _ = write!(writer, "  EBP: 0x{:08X}  ESP: 0x{:08X}\n", ebp, esp);
        let _ = write!(writer, "  EFLAGS: 0x{:08X}\n", eflags);
    }
    
    let _ = write!(writer, "\nSystem halted. Please reboot.\n");
    
    // Also print to VGA
    unsafe {
        ffi::vga_set_color(4, 0); // Red text
        ffi::vga_println(b"\n!!! KERNEL PANIC !!!\0".as_ptr());
        if let Some(location) = info.location() {
            ffi::vga_print(b"Location: \0".as_ptr());
        }
        ffi::vga_println(b"Check serial output for details.\0".as_ptr());
    }
    
    // Halt the system
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
