#![no_std]

//! Hardware Abstraction Layer for Alloy OS
//!
//! This crate provides a unified interface to architecture-specific hardware operations,
//! enabling the kernel to support multiple CPU architectures (x86_64, aarch64).
//!
//! This crate no longer owns any implementation code. All trait
//! definitions, impls, inline-asm shims, and FFI declarations live in
//! `alloy-kernel-unsafe-core`; this crate re-exports the safe public boundary
//! (`unsafe_core::api`) so the kernel crate keeps a single dependency edge.

pub use alloy_kernel_unsafe_core::api::*;

/// Raw C FFI declarations, re-exported from `unsafe_core::raw::ffi`.
pub mod ffi;

/// Safe console output: `println!`/`print!`/`log!` macros and helpers.
pub mod console;

/// Global hardware platform initialization and access.
pub mod platform;

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
