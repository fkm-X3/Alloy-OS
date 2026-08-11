//! Safe physical-memory API (Phase 3.3).
//!
//! Replaces the raw `ffi::pmm_*`/`ffi::paging_*` call sites in the safe
//! kernel with types and functions that cannot cause UB from safe code.
//! Raw pointers never cross this module's public surface: physical frames
//! are `usize` addresses, buffers are slices.
//!
//! Session 3.3.1 delivers `PhysFrame` + the PMM stat accessors. Later
//! sub-sessions add `VmRegion` (3.3.2), `AddressSpace` (3.3.3), and the
//! validated user copies (3.3.4).

use core::ffi::c_void;

use crate::raw::ffi;

/// Page size in bytes (all archs here use 4 KiB pages).
pub const PAGE_SIZE: usize = 4096;

/// Page flags for memory mapping.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFlags {
    pub present: bool,
    pub writable: bool,
    pub user_accessible: bool,
    pub no_execute: bool,
    pub cache_disabled: bool,
}

impl PageFlags {
    pub const fn default() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: false,
            no_execute: false,
            cache_disabled: false,
        }
    }

    pub const fn kernel_read() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: false,
            no_execute: false,
            cache_disabled: false,
        }
    }

    pub const fn kernel_write() -> Self {
        Self {
            present: true,
            writable: true,
            user_accessible: false,
            no_execute: false,
            cache_disabled: false,
        }
    }

    pub const fn user_read() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: true,
            no_execute: false,
            cache_disabled: false,
        }
    }

    pub const fn user_write() -> Self {
        Self {
            present: true,
            writable: true,
            user_accessible: true,
            no_execute: false,
            cache_disabled: false,
        }
    }

    /// Raw page-table bits (x86 PTE / aarch64 descriptor flags).
    pub(crate) const fn raw(&self) -> u32 {
        let mut raw = 0u32;
        if self.present {
            raw |= 0x001;
        }
        if self.writable {
            raw |= 0x002;
        }
        if self.user_accessible {
            raw |= 0x004;
        }
        if self.cache_disabled {
            raw |= 0x010;
        }
        raw
    }
}

/// A single physical frame (4 KiB) allocated from the PMM, owned RAII-style.
///
/// The frame is reference-counted by the ported PMM: allocation starts the
/// count at 1, `clone()` increments it, and `drop` decrements it, freeing the
/// frame back to the bitmap when the count reaches zero.
///
/// A frame may be handed off to a long-lived owner (e.g. a page directory)
/// with [`PhysFrame::into_addr`], which forgets the RAII wrapper without
/// touching the refcount.
pub struct PhysFrame {
    addr: usize,
}

impl PhysFrame {
    /// Allocate a physical frame. Returns `None` on out-of-memory.
    pub fn alloc() -> Option<Self> {
        let ptr = unsafe { ffi::pmm_alloc_frame() };
        if ptr.is_null() {
            None
        } else {
            Some(PhysFrame {
                addr: ptr as usize,
            })
        }
    }

    /// Physical address of this frame (page-aligned).
    pub fn addr(&self) -> usize {
        self.addr
    }

    /// Reference this frame (increments the PMM refcount for COW sharing).
    pub fn clone_ref(&self) -> Self {
        unsafe { ffi::pmm_refcount_inc(self.addr as *mut c_void) };
        PhysFrame { addr: self.addr }
    }

    /// Take the physical address and hand ownership of the frame out of RAII.
    ///
    /// The refcount is NOT decremented and the frame is forgotten: callers
    /// use this when the frame is now owned by a page directory or a
    /// long-lived mapping that will reclaim it on its own teardown.
    pub fn into_addr(self) -> usize {
        let addr = self.addr;
        core::mem::forget(self);
        addr
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        unsafe { ffi::pmm_refcount_dec(self.addr as *mut c_void) };
    }
}

// ----------------------------------------------------------------------------
// PMM statistics (safe accessors over the raw getters).
// ----------------------------------------------------------------------------

/// Total physical memory in bytes.
pub fn total_memory() -> u64 {
    unsafe { ffi::pmm_get_total_memory() }
}

/// Currently available (free) physical memory in bytes.
pub fn available_memory() -> u64 {
    unsafe { ffi::pmm_get_available_memory() }
}

/// Total number of physical frames.
pub fn total_frames() -> u32 {
    unsafe { ffi::pmm_get_total_frames() }
}

/// Number of allocated physical frames.
pub fn used_frames() -> u32 {
    unsafe { ffi::pmm_get_used_frames() }
}

/// Number of free physical frames.
pub fn free_frames() -> u32 {
    total_frames().saturating_sub(used_frames())
}
