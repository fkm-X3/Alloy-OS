//! Safe device-driver facades.
//!
//! Each module wraps one hardware console/timer device behind a safe type
//! that the kernel crate can call without `unsafe`. The C2Rust modules in
//! [`crate::ported`] that used to implement these devices are deleted as the
//! facades land; the `#[no_mangle]` C-ABI entry points the surviving ported
//! code (paging, pmm, idt, boot mains, vesa, ...) still references are kept
//! here so the swap stays atomic.

pub mod serial;
pub mod timer;

#[cfg(feature = "x86_64")]
pub mod vga;

#[cfg(feature = "aarch64")]
pub mod pl110;
