//! Safe physical-memory API.
//!
//! Replaces the raw `ffi::pmm_*`/`ffi::paging_*` call sites in the safe
//! kernel with types and functions that cannot cause UB from safe code.
//! Raw pointers never cross this module's public surface: physical frames
//! are `usize` addresses, buffers are slices.
//!
//! The `#[no_mangle]` C-ABI entry points in the `pmm`/`vmm`/`paging`
//! submodules keep the boot mains, the surviving ported modules (idt, vesa,
//! ahci, ...) and `raw::ffi` resolving against the same symbols as before.

// Hand-written replacements for the ported PMM/VMM/paging.
pub mod pmm;
pub mod vmm;
#[cfg(feature = "x86_64")]
pub mod paging;
#[cfg(feature = "aarch64")]
pub mod paging_aarch64;
pub mod user;

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
// Virtual regions and page mapping.
// ----------------------------------------------------------------------------

fn align_up(value: usize, align: usize) -> usize {
    value.checked_add(align - 1).map(|v| v & !(align - 1)).unwrap_or(value)
}

/// A contiguous range of mapped virtual pages, owned RAII-style.
///
/// Allocated with [`VmRegion::alloc`] (which allocates a physical frame per
/// page and maps it with the requested flags in the current address space).
/// Dropping the region unmaps it and returns its frames to the PMM, so the
/// only way to keep a region alive past its scope is [`VmRegion::leak`]
/// (used by the heap, slab, brk and mmap call sites, whose regions live as
/// long as their owner does and are reclaimed manually).
pub struct VmRegion {
    addr: usize,
    size: usize,
}

impl VmRegion {
    /// Allocate a contiguous mapped region of `size` bytes.
    pub fn alloc(size: usize, flags: PageFlags) -> Option<Self> {
        let ptr = unsafe { ffi::vmm_alloc_region(size, flags.raw()) };
        if ptr.is_null() {
            None
        } else {
            Some(VmRegion {
                addr: ptr as usize,
                size: align_up(size, PAGE_SIZE),
            })
        }
    }

    /// Virtual (or identity-physical, on aarch64) base address of the region.
    pub fn addr(&self) -> usize {
        self.addr
    }

    /// Page-aligned size of the region in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Take the address and keep the region mapped permanently.
    ///
    /// The region is forgotten (not unmapped) so it outlives this wrapper;
    /// callers own the range and reclaim it with [`free_region`].
    pub fn leak(self) -> usize {
        let addr = self.addr;
        core::mem::forget(self);
        addr
    }
}

impl Drop for VmRegion {
    fn drop(&mut self) {
        unsafe { ffi::vmm_free_region(self.addr as *mut c_void, self.size) };
    }
}

/// Map a single physical frame at `virt` without allocating (e.g. device
/// framebuffers). Returns false if the mapping could not be installed.
pub fn map_page(virt: usize, phys: usize, flags: PageFlags) -> bool {
    unsafe { ffi::vmm_map(virt as *mut c_void, phys as *mut c_void, flags.raw()) }
}

/// Map `frame` at `virt` in the current address space, taking ownership of
/// the frame (the page table owns it from then on).
///
/// On failure the frame is returned to the PMM instead of leaking. Used by
/// the execve stack mapping, which populates the freshly-switched user page
/// tables through the current-space `vmm_map` path.
pub fn map_frame(virt: usize, frame: PhysFrame, flags: PageFlags) -> bool {
    let phys = frame.addr();
    let ok = unsafe { ffi::vmm_map(virt as *mut c_void, phys as *mut c_void, flags.raw()) };
    if ok {
        core::mem::forget(frame);
    }
    ok
}

/// Unmap a single page.
pub fn unmap_page(virt: usize) {
    unsafe { ffi::vmm_unmap(virt as *mut c_void) };
}

/// Free a previously allocated region by address and size.
///
/// Used where a region's lifetime is tracked manually (e.g. brk shrink frees
/// the tail of the heap region without dropping the owning wrapper).
pub fn free_region(addr: usize, size: usize) {
    unsafe { ffi::vmm_free_region(addr as *mut c_void, size) };
}

/// Map a physical frame into the shared temp window, run `f` with its kernel
/// address, then unmap it.
///
/// The window is a single slot shared by the whole system and is not safe
/// against interrupts or task switches, so callers must keep IRQs disabled
/// across the call (the ported spawn path already does this). Consumed by
/// the spawn migration.
#[allow(dead_code)]
pub fn with_temp_frame<R>(phys: usize, f: impl FnOnce(*mut u8) -> R) -> Option<R> {
    let ptr = unsafe { ffi::paging_temp_map_frame(phys) };
    if ptr.is_null() {
        return None;
    }
    let result = f(ptr as *mut u8);
    unsafe { ffi::paging_temp_unmap_frame() };
    Some(result)
}

// ----------------------------------------------------------------------------
// Address spaces .
// ----------------------------------------------------------------------------

/// A per-process address space (an x86_64 page directory), owned RAII-style.
///
/// [`AddressSpace::kernel`] is a non-owning view of the shared kernel
/// directory and is never destroyed. `create`/`clone_of`/`fork_of` return
/// owned directories that are destroyed on drop, returning their frames to
/// the PMM (honouring COW refcounts).
///
/// On aarch64 the translated paging is identity/no-op, so these operations
/// degenerate to the kernel directory — the wrappers keep the API uniform
/// while the MMU is disabled.
pub struct AddressSpace {
    pd_phys: usize,
    owned: bool,
}

impl AddressSpace {
    /// True when `pd_phys` is a fresh directory that drop must destroy.
    ///
    /// The aarch64 paging stubs return the kernel directory for
    /// create/clone/fork, so a directory that degenerates to the kernel
    /// directory is never owned.
    fn owned(pd_phys: usize) -> bool {
        unsafe { ffi::paging_get_kernel_directory_phys() != pd_phys }
    }

    /// The kernel's own shared address space (not owned; never destroyed).
    pub fn kernel() -> Self {
        AddressSpace {
            pd_phys: unsafe { ffi::paging_get_kernel_directory_phys() },
            owned: false,
        }
    }

    /// Allocate a fresh address space whose kernel half mirrors the kernel
    /// directory (used by spawn/execve).
    pub fn create() -> Option<Self> {
        let pd = unsafe { ffi::paging_create_directory_phys() };
        if pd == 0 {
            None
        } else {
            Some(AddressSpace { pd_phys: pd, owned: Self::owned(pd) })
        }
    }

    /// Deep-clone the address space at `pd_phys` into a new owned one
    /// (used by `clone`).
    pub fn clone_of(pd_phys: usize) -> Option<Self> {
        let pd = unsafe { ffi::paging_clone_directory(pd_phys) };
        if pd == 0 {
            None
        } else {
            Some(AddressSpace { pd_phys: pd, owned: Self::owned(pd) })
        }
    }

    /// COW-fork the address space at `pd_phys` into a new owned one
    /// (used by `fork`).
    pub fn fork_of(pd_phys: usize) -> Option<Self> {
        let pd = unsafe { ffi::paging_fork_directory(pd_phys) };
        if pd == 0 {
            None
        } else {
            Some(AddressSpace { pd_phys: pd, owned: Self::owned(pd) })
        }
    }

    /// Physical address of the page directory backing this address space
    /// (stored in `CpuContext.cr3` / `CpuContext.ttbr0`).
    pub fn addr(&self) -> usize {
        self.pd_phys
    }

    /// True when this is the kernel's own (non-owned) address space.
    pub fn is_kernel(&self) -> bool {
        !self.owned
    }

    /// Make this the CPU's active address space.
    pub fn switch(&self) -> bool {
        unsafe { ffi::paging_switch_to_directory(self.pd_phys) }
    }

    /// Record this as the current user address space — the base the user-copy
    /// helpers switch CR3 to when touching user memory.
    pub fn set_current_user(&self) {
        unsafe { ffi::g_current_user_cr3 = self.pd_phys as u64 };
    }

    /// Map one frame into this address space at `virt`, taking ownership of
    /// the frame (the page table owns it from then on).
    ///
    /// On failure the frame is returned to the PMM. Callers must keep IRQs
    /// disabled across the call (the ported map uses the shared temp window).
    pub fn map_page(&mut self, virt: usize, frame: PhysFrame, flags: PageFlags) -> bool {
        let phys = frame.addr();
        let ok = unsafe { ffi::paging_map_page_in_pd(self.pd_phys, virt, phys, flags.raw()) };
        if ok {
            core::mem::forget(frame);
        }
        ok
    }

    /// Handle a copy-on-write page fault at `fault_addr` in the address space
    /// rooted at `pd_phys`. Returns true when the fault was resolved by the
    /// ported COW machinery.
    pub fn handle_cow_fault(pd_phys: usize, fault_addr: usize) -> bool {
        unsafe { ffi::paging_handle_cow_fault(pd_phys, fault_addr) != 0 }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ffi::paging_destroy_directory(self.pd_phys) };
        }
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
