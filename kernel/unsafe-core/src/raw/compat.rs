//! Drop-in C-ABI compat shims.
//!
//! Scaffolding: `#[cfg]`-off / no-op. When the Makefile swap
//! empties `C_SOURCES`, this module re-exports every symbol
//! the outside world references (`kernel_main`, `g_pmm`, `init_serial`,
//! ...) so the boot asm and the Rust kernel keep resolving unchanged.
//!
//! Kept empty and un-wired until the swap so it can never conflict with the
//! still-compiled C.
