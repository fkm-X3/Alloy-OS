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

// --- Safe byte-slice reinterpret helpers ---
// Centralises the `unsafe` for `T → &[u8]` reinterpretation so individual
// call sites in the safe kernel don't need raw pointer casts.

/// Reinterpret a single `&T` as `&[u8]`.
///
/// # Panics
/// Panics if `T` is a zero-sized type.
pub fn as_byte_slice<T: Copy>(val: &T) -> &[u8] {
    assert!(core::mem::size_of::<T>() > 0, "cannot convert ZST to byte slice");
    unsafe { core::slice::from_raw_parts(val as *const T as *const u8, core::mem::size_of::<T>()) }
}

/// Reinterpret a `&[T]` as `&[u8]`.
pub fn as_byte_slice_of<T: Copy>(slice: &[T]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const u8, core::mem::size_of_val(slice)) }
}
