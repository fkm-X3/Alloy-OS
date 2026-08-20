//! Panic handler for no_std Rust kernel
//!
//! This module handles panics in the Rust kernel by printing
//! panic information to serial output and halting the system.

use core::fmt::Write;
use core::panic::PanicInfo;

/// Custom writer for serial output
struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::print!("{s}");
        Ok(())
    }
}

/// Panic handler - called when Rust code panics
pub fn panic_handler(info: &PanicInfo) -> ! {
    let mut writer = SerialWriter;

    // Print panic banner to serial
    let _ = writeln!(writer);
    let _ = writeln!(writer, "╔═══════════════════════════════════╗");
    let _ = writeln!(writer, "║    KERNEL PANIC - SYSTEM HALTED   ║");
    let _ = write!(writer, "╚═══════════════════════════════════╝\n\n");

    // Location information
    if let Some(location) = info.location() {
        let _ = writeln!(
            writer,
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    // Panic message
    let _ = write!(writer, "Message:  {}\n\n", info.message());

    // Dump some CPU registers (simplified to avoid register pressure)
    #[cfg(feature = "x86_64")]
    {
        let regs = alloy_kernel_hal::capture_panic_regs();
        let _ = writeln!(writer, "Register dump:");
        let _ = writeln!(writer, "  RBP: 0x{:016X}  RSP: 0x{:016X}", regs.rbp, regs.rsp);
        let _ = writeln!(writer, "  RFLAGS: 0x{:016X}", regs.rflags);
    }
    let _ = write!(writer, "\nSystem halted. Please reboot.\n");

    // Also print to VGA (x86 only)
    #[cfg(feature = "x86_64")]
    {
        crate::VgaText::set_color(4, 0); // Red text
        crate::VgaText::println_bytes(b"\n!!! KERNEL PANIC !!!\n");
        if let Some(_location) = info.location() {
            crate::VgaText::print_bytes(b"Location: ");
        }
        crate::VgaText::println_bytes(b"Check serial output for details.");
    }

    // Halt the system
    loop {
        alloy_kernel_hal::cpu_halt();
    }
}
