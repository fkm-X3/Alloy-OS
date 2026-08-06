#![no_std]
#![allow(unsafe_code)]

//! Alloy OS unsafe core.
//!
//! The only crate in the kernel tree allowed to contain `unsafe` code. Every
//! `extern "C"` declaration, inline-asm shim, `#[no_mangle]` symbol, and raw
//! driver/arch implementation lives here, behind safe wrappers.
//!
//! The safe kernel crate (`alloy-kernel-rust`) and the HAL contract crate
//! (`alloy-kernel-hal`) must never write `unsafe`; they talk to the hardware
//! exclusively through the safe [`api`] surface re-exported here.
//!
//! Layout:
//! - [`raw`]: extern "C" blocks, inline-asm shims, `#[no_mangle]` symbols.
//! - [`api`]: the safe public boundary — the only surface the rest of the
//!   kernel is allowed to use.

/// Raw, unsafe-only primitives (extern "C" decls, asm shims, symbols).
///
/// Populated in Phase 1 (moved verbatim from `hal/src/ffi.rs` and the
/// inline-asm helpers currently scattered through the HAL).
pub mod raw;

/// The safe public boundary of this crate.
///
/// Every item here must be callable from safe code with no UB possible.
/// Raw pointers never cross this boundary; addresses are `usize`; buffers
/// are slices. Filled in as driver/arch/mem ports land (Phases 1-6).
pub mod api;

// ============================================================================
// Implementation modules (moved verbatim from the HAL in Phase 1).
//
// These hold the trait definitions, their impls, and the arch/driver data
// types the HAL used to own. The HAL no longer writes any implementation
// code here; it only re-exports this crate's public surface via [`api`].
// ============================================================================

/// Architecture implementations (Arch impls, CpuContext, CpuInfo, paging
/// structs, segment/exceptions/gic constants).
pub mod arch;

/// Port I/O + MMIO traits and impls (IoPort, X86IoPort, Mmio, DefaultMmio,
/// MmioReg).
pub mod io;

/// Interrupt controller abstractions (InterruptController trait, Pic8259,
/// Gic).
pub mod interrupt;

/// Memory management abstractions (MemoryManager trait, PageFlags, Pmm).
pub mod memory;

/// Serial port abstractions (SerialPort trait, Uart16550, Pl011Uart).
pub mod serial;

/// Timer abstractions (Timer trait, Pit, ArmGenericTimer).
pub mod time;
