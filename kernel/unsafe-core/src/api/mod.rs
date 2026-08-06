//! The safe public boundary of `alloy-kernel-unsafe-core`.
//!
//! This is the only surface the safe kernel and the HAL are allowed to use.
//! Every item here must be callable from safe code with no UB possible; raw
//! pointers never cross this boundary.
//!
//! In Phase 1 this module re-exports the moved HAL trait definitions,
//! impls, and data types verbatim so `alloy-kernel-hal` can keep exposing
//! its existing public API (`Arch`, `MemoryManager`, `InterruptController`,
//! `SerialPort`, `Timer`, `IoPort`, `Mmio`, ...) without writing any code
//! that contains `unsafe`. As the driver/arch/mem ports land (Phases 2-6)
//! the raw impls retreat behind safe wrappers and this module becomes the
//! documented safe API.
//!
//! Submodules: io, mem, interrupt, serial, time, drivers, alloc, sync,
//! arch, callback.

pub use crate::arch::{self, Arch, CpuContext, CpuInfo};
pub use crate::interrupt::{self, InterruptController, IrqHandler};
pub use crate::io::{self, DefaultMmio, Mmio, MmioReg};
#[cfg(feature = "x86_64")]
pub use crate::io::X86IoPort;
#[cfg(feature = "x86_64")]
pub use crate::io::IoPort;
pub use crate::memory::{self, MemoryManager, PageFlags};
pub use crate::serial::{self, SerialPort};
pub use crate::time::{self, Timer};
