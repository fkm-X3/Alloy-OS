//! The safe public boundary of `alloy-kernel-unsafe-core`.
//!
//! This is the only surface the safe kernel and the HAL are allowed to use.
//! Every item here must be callable from safe code with no UB possible; raw
//! pointers never cross this boundary.
//!
//! This module re-exports the moved HAL trait definitions, impls, and data
//! types verbatim so `alloy-kernel-hal` can keep exposing its existing
//! public API (`Arch`, `MemoryManager`, `InterruptController`, `SerialPort`,
//! `Timer`, `IoPort`, `Mmio`, ...) without writing any code that contains
//! `unsafe`. As the driver/arch/mem ports land, the raw impls retreat behind
//! safe wrappers and this module becomes the documented safe API.
//!
//! Submodules: io, mem, interrupt, serial, time, drivers, alloc, sync,
//! arch, callback.

pub use crate::arch::{self, Arch, CpuContext, CpuInfo};
pub use crate::callback::{self, FaultAction, SyscallHandler, SyscallTable};
pub use crate::callback::{set_page_fault_handler, set_timer_tick_handler};
pub use crate::interrupt::{self, InterruptController, InterruptGuard, IrqHandler, IrqLine};
pub use crate::io::{self, DefaultMmio, Mmio, MmioReg};
#[cfg(feature = "x86_64")]
pub use crate::io::X86IoPort;
#[cfg(feature = "x86_64")]
pub use crate::io::IoPort;
pub use crate::mem::{self, AddressSpace, PageFlags, PhysFrame, VmRegion};
#[cfg(feature = "x86_64")]
pub use crate::ported::arch::syscall::{current_user_syscall_frame, UserSyscallFrame};
pub use crate::memory::{self, MemoryManager};
pub use crate::serial::{self, SerialPort};
pub use crate::time::{self, Timer};

// --- Phase 3.1 safe driver facades ---
pub use crate::drivers::serial::Serial;
pub use crate::drivers::timer::SystemTimer;
#[cfg(feature = "x86_64")]
pub use crate::drivers::vga::VgaText;
#[cfg(feature = "aarch64")]
pub use crate::drivers::pl110::Pl110;
