//! Kernel utility helpers.
//!
//! The user-copy primitives were deduplicated into `unsafe-core`
//! (`mem/user.rs`); this module re-exports them through the HAL
//! so existing `crate::utils::copy_from_user`/`copy_to_user` call sites keep
//! working unchanged.

pub mod format;
pub mod pointer;

pub use alloy_kernel_hal::copy_from_user;
pub use alloy_kernel_hal::copy_to_user;
pub use alloy_kernel_hal::{cpu_halt, heap_start, heap_size, allocated_pages};

// Re-export byte-slice helpers from unsafe-core via the HAL.
pub use alloy_kernel_hal::as_byte_slice;
pub use alloy_kernel_hal::as_byte_slice_of;
