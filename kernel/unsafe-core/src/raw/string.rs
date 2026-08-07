//! Freestanding C memory primitives.
//!
//! C2Rust emits `::libc::memcpy`/`::libc::memset` for the kernel C's
//! string.h-style helpers. The kernel is freestanding (`core` + `alloc`
//! only, no `libc` crate), so these resolve to the compiler-builtins
//! `memcpy`/`memset` symbols instead (provided by the
//! `-Zbuild-std-features=compiler-builtins-mem` flag used for the alloy
//! targets).

use core::ffi::c_void;

extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
}

/// c2rust maps C's `size_t` to `::libc::size_t`; freestanding it is `usize`.
pub type size_t = usize;
