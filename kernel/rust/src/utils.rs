// Utility helpers for kernel

const USER_SPACE_LIMIT: usize = 0xC0000000usize; // conservative user-space upper bound (3GB)
const USER_MAX_COPY: usize = 1024 * 1024; // 1MB max per copy to avoid large accidental copies

use crate::ffi;

/// Helper to check that a user range is plausibly valid by comparing against VMM's next virt addr.
fn user_range_check(start: usize, len: usize) -> bool {
    if start >= USER_SPACE_LIMIT { return false; }
    if len == 0 { return true; }
    if len > USER_MAX_COPY { return false; }
    if start.checked_add(len).map_or(true, |end| end > USER_SPACE_LIMIT) { return false; }
    // Ensure end is below vmm_get_next_virt_addr() which represents allocated virtual space
    let next = unsafe { ffi::vmm_get_next_virt_addr() } as usize;
    if start.checked_add(len).map_or(true, |end| end > next) { return false; }

    // Additionally, verify that each page in the range is mapped (non-zero physical address)
    const PAGE_SIZE: usize = 0x1000;
    let end = start.checked_add(len).unwrap_or(usize::MAX);
    let mut addr = start & !(PAGE_SIZE - 1);
    let mut pages_checked = 0;
    while addr < end {
        let phys = unsafe { ffi::paging_get_physical_address(addr as u32) } as usize;
        if phys == 0 { return false; }
        pages_checked += 1;
        if pages_checked > 1024 { // safety cap for extremely large ranges
            return false;
        }
        addr = addr.saturating_add(PAGE_SIZE);
    }

    true
}

// Copy helpers which are page-aware and return partial progress if possible

/// Copy from a user pointer (u32 virtual address) into a kernel buffer.
/// Copies up to the provided buffer length; returns Ok(bytes_copied) or Err(-1) if nothing could be copied.
pub unsafe fn copy_from_user(user_ptr: u32, buf: &mut [u8]) -> Result<usize, i32> {
    if user_ptr == 0 { return Err(-1); }
    let mut remaining = buf.len();
    if remaining == 0 { return Ok(0); }

    const PAGE_SIZE: usize = 0x1000;
    let mut cur = user_ptr as usize;
    let mut out_off = 0usize;

    while remaining > 0 {
        // Check page mapping
        let phys = ffi::paging_get_physical_address((cur & !(PAGE_SIZE - 1)) as u32) as usize;
        if phys == 0 {
            if out_off > 0 { return Ok(out_off); } else { return Err(-1); }
        }

        let page_off = cur & (PAGE_SIZE - 1);
        let chunk = core::cmp::min(remaining, PAGE_SIZE - page_off);

        let src = cur as *const u8;
        let src_slice = core::slice::from_raw_parts(src, chunk);
        buf[out_off..out_off+chunk].copy_from_slice(src_slice);

        out_off += chunk;
        remaining -= chunk;
        cur = cur.saturating_add(chunk);
    }

    Ok(out_off)
}

/// Copy into a user pointer from a kernel buffer.
/// Writes up to buf.len(); returns Ok(bytes_written) or Err(-1) if nothing could be written.
pub unsafe fn copy_to_user(user_ptr: u32, buf: &[u8]) -> Result<usize, i32> {
    if user_ptr == 0 { return Err(-1); }
    let mut remaining = buf.len();
    if remaining == 0 { return Ok(0); }

    const PAGE_SIZE: usize = 0x1000;
    let mut cur = user_ptr as usize;
    let mut in_off = 0usize;

    while remaining > 0 {
        let phys = ffi::paging_get_physical_address((cur & !(PAGE_SIZE - 1)) as u32) as usize;
        if phys == 0 {
            if in_off > 0 { return Ok(in_off); } else { return Err(-1); }
        }

        let page_off = cur & (PAGE_SIZE - 1);
        let chunk = core::cmp::min(remaining, PAGE_SIZE - page_off);

        let dst = cur as *mut u8;
        let dst_slice = core::slice::from_raw_parts_mut(dst, chunk);
        dst_slice.copy_from_slice(&buf[in_off..in_off+chunk]);

        in_off += chunk;
        remaining -= chunk;
        cur = cur.saturating_add(chunk);
    }

    Ok(in_off)
}
