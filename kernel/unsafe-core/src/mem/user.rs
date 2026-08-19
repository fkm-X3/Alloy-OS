//! Validated user-memory copies.
//!
//! `copy_from_user`/`copy_to_user` replace the duplicated helpers in the
//! kernel crate's `utils` copies. They validate the whole `[start, end)`
//! range before touching user memory:
//!
//! - the range must lie inside user space (conservative 3 GiB bound) and be
//!   at most [`USER_MAX_COPY`] bytes;
//! - every page in the range must be mapped in the current user address
//!   space (`g_current_user_cr3` on x86_64), checked through
//!   `paging_get_physical_address` while running under the user CR3.
//!
//! The actual copy is then page-aware and returns partial progress if a page
//! turns out to be unmapped mid-range (`Ok(n)` for `n > 0`, `Err(-1)` when
//! nothing could be copied) — the contract the fs/syscall callers already
//! rely on.
//!
//! On aarch64 (MMU disabled, identity) the CR3 switch is a no-op and the
//! mapping check always passes, so these reduce to plain bounded copies.
//!
//! All UB risk is contained inside `unsafe {}` blocks in this module; callers
//! can invoke these functions from entirely safe code.

#[cfg(feature = "x86_64")]
use crate::raw::asm::x86_64::{read_cr3, write_cr3};
use crate::raw::ffi;

/// Conservative upper bound of the user address space (3 GiB).
const USER_SPACE_LIMIT: usize = 0xC000_0000;
/// Maximum bytes per copy, to avoid accidental huge copies.
const USER_MAX_COPY: usize = 1024 * 1024;
const PAGE_SIZE: usize = 4096;

/// Run the copy against the current user address space, returning the saved
/// CR3 for [`restore_user_cr3`].
#[cfg(feature = "x86_64")]
fn switch_to_user_cr3() -> u64 {
    let saved = read_cr3();
    write_cr3(unsafe { ffi::g_current_user_cr3 });
    saved
}

/// Restore the kernel CR3 saved by [`switch_to_user_cr3`].
#[cfg(feature = "x86_64")]
fn restore_user_cr3(saved: u64) {
    write_cr3(saved);
}

#[cfg(feature = "aarch64")]
fn switch_to_user_cr3() -> u64 {
    0
}

#[cfg(feature = "aarch64")]
fn restore_user_cr3(_saved: u64) {}

/// Validate `[start, start+len)` as a copyable user range (bounds only).
fn range_sane(start: usize, len: usize) -> bool {
    if start == 0 || start >= USER_SPACE_LIMIT {
        return false;
    }
    if len == 0 {
        return true;
    }
    if len > USER_MAX_COPY {
        return false;
    }
    match start.checked_add(len) {
        Some(end) => end <= USER_SPACE_LIMIT,
        None => false,
    }
}

/// Copy from a user pointer (`u32` virtual address) into a kernel buffer.
///
/// Returns `Ok(bytes_copied)` (which may be less than `buf.len()` if a page
/// in the middle of the range is unmapped) or `Err(-1)` if nothing could be
/// copied.
///
/// The range is validated page-by-page before each copy chunk; invalid or
/// unmapped addresses return `Err(-1)` without causing UB.
pub fn copy_from_user(user_ptr: u32, buf: &mut [u8]) -> Result<usize, i32> {
    let start = user_ptr as usize;
    let len = buf.len();
    if !range_sane(start, len) {
        return Err(-1);
    }
    if len == 0 {
        return Ok(0);
    }

    let saved_cr3 = switch_to_user_cr3();
    let mut cur = start;
    let mut out_off = 0usize;
    let mut pages_checked = 0usize;

    while out_off < len {
        let page_addr = cur & !(PAGE_SIZE - 1);
        let phys = unsafe { ffi::paging_get_physical_address(page_addr) };
        if phys == 0 {
            restore_user_cr3(saved_cr3);
            return if out_off > 0 { Ok(out_off) } else { Err(-1) };
        }
        pages_checked += 1;
        if pages_checked > USER_MAX_COPY / PAGE_SIZE {
            restore_user_cr3(saved_cr3);
            return Err(-1);
        }

        let page_off = cur & (PAGE_SIZE - 1);
        let chunk = core::cmp::min(len - out_off, PAGE_SIZE - page_off);
        unsafe {
            core::ptr::copy_nonoverlapping(
                cur as *const u8,
                buf[out_off..].as_mut_ptr(),
                chunk,
            );
        }
        out_off += chunk;
        cur += chunk;
    }

    restore_user_cr3(saved_cr3);
    Ok(out_off)
}

/// Copy from a kernel buffer into a user pointer (`u32` virtual address).
///
/// Returns `Ok(bytes_written)` (which may be less than `buf.len()` if a page
/// in the middle of the range is unmapped) or `Err(-1)` if nothing could be
/// written.
///
/// The range is validated page-by-page before each copy chunk; invalid or
/// unmapped addresses return `Err(-1)` without causing UB.
pub fn copy_to_user(user_ptr: u32, buf: &[u8]) -> Result<usize, i32> {
    let start = user_ptr as usize;
    let len = buf.len();
    if !range_sane(start, len) {
        return Err(-1);
    }
    if len == 0 {
        return Ok(0);
    }

    let saved_cr3 = switch_to_user_cr3();
    let mut cur = start;
    let mut in_off = 0usize;
    let mut pages_checked = 0usize;

    while in_off < len {
        let page_addr = cur & !(PAGE_SIZE - 1);
        let phys = unsafe { ffi::paging_get_physical_address(page_addr) };
        if phys == 0 {
            restore_user_cr3(saved_cr3);
            return if in_off > 0 { Ok(in_off) } else { Err(-1) };
        }
        pages_checked += 1;
        if pages_checked > USER_MAX_COPY / PAGE_SIZE {
            restore_user_cr3(saved_cr3);
            return Err(-1);
        }

        let page_off = cur & (PAGE_SIZE - 1);
        let chunk = core::cmp::min(len - in_off, PAGE_SIZE - page_off);
        unsafe {
            core::ptr::copy_nonoverlapping(
                buf[in_off..].as_ptr(),
                cur as *mut u8,
                chunk,
            );
        }
        in_off += chunk;
        cur += chunk;
    }

    restore_user_cr3(saved_cr3);
    Ok(in_off)
}
