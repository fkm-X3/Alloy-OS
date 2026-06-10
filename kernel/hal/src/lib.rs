#![no_std]

//! Hardware Abstraction Layer for Alloy OS
//!
//! This crate provides a unified interface to architecture-specific hardware operations,
//! enabling the kernel to support multiple CPU architectures (i686, x86_64, aarch64).

pub mod arch;
pub mod ffi;
pub mod interrupt;
pub mod io;
pub mod memory;
pub mod platform;
pub mod serial;
pub mod time;

pub use arch::{Arch, CpuContext, CpuInfo};
pub use interrupt::{InterruptController, IrqHandler};
#[cfg(any(feature = "i686", feature = "x86_64"))]
pub use io::IoPort;
pub use io::Mmio;
pub use memory::{MemoryManager, PageFlags};
pub use serial::SerialPort;
pub use time::Timer;

/// Architecture-specific initialization
pub trait HalPlatform {
    type Arch: Arch;
    type InterruptCtrl: InterruptController;
    type MemManager: MemoryManager;
    type Serial: SerialPort;
    type Timer: Timer;

    fn init_early() -> Self::Serial;
    fn init_interrupts() -> Self::InterruptCtrl;
    fn init_memory() -> Self::MemManager;
    fn init_timer(frequency: u32) -> Self::Timer;
}
