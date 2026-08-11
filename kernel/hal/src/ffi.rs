//! Consolidated C FFI declarations
//!
//! This module re-exports the extern "C" declarations owned by
//! `alloy-kernel-unsafe-core` (`unsafe_core::raw::ffi`). It exists so
//! `crate::ffi::*` keeps resolving unchanged from the kernel crate while the
//! declarations themselves live in the single crate allowed to hold them.
//!
//! `hal::ffi` disappears once the kernel no longer calls raw FFI.

pub use alloy_kernel_unsafe_core::raw::ffi::*;
