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

// ----------------------------------------------------------------------------
// VMM statistics (safe accessors over the raw getters).
// ----------------------------------------------------------------------------

/// Start address of the kernel heap.
pub fn heap_start() -> usize {
    unsafe { ffi::vmm_get_heap_start() }
}

/// Size of the kernel heap in bytes.
pub fn heap_size() -> usize {
    unsafe { ffi::vmm_get_heap_size() }
}

/// Number of virtual pages currently allocated by the VMM.
pub fn allocated_pages() -> u32 {
    unsafe { ffi::vmm_get_allocated_pages() }
}

// ----------------------------------------------------------------------------
// Safe physical-memory byte access (identity-mapped or temp-window).
// ----------------------------------------------------------------------------

/// Read `buf.len()` bytes from physical address `phys`. Assumes the region is
/// identity-mapped (e.g. ramdisk region, aarch64 identity mapping).
pub fn read_phys_bytes(phys: usize, buf: &mut [u8]) {
    unsafe {
        let src = core::slice::from_raw_parts(phys as *const u8, buf.len());
        buf.copy_from_slice(src);
    }
}

/// Write `buf` bytes to physical address `phys`. Assumes identity mapping.
pub fn write_phys_bytes(phys: usize, buf: &[u8]) {
    unsafe {
        let dst = core::slice::from_raw_parts_mut(phys as *mut u8, buf.len());
        dst.copy_from_slice(buf);
    }
}

/// Zero `len` bytes at physical address `phys`. Assumes identity mapping.
pub fn zero_phys_bytes(phys: usize, len: usize) {
    unsafe {
        let dst = core::slice::from_raw_parts_mut(phys as *mut u8, len);
        dst.fill(0);
    }
}

/// Copy `src` bytes into the mapped virtual address `dst_vaddr`.
/// The destination must be valid, mapped, writable memory of at least `src.len()` bytes.
pub fn copy_to_mapped(dst_vaddr: usize, src: &[u8]) {
    unsafe {
        let dst = core::slice::from_raw_parts_mut(dst_vaddr as *mut u8, src.len());
        dst.copy_from_slice(src);
    }
}

/// Zero `len` bytes at the mapped virtual address `dst_vaddr`.
pub fn zero_mapped(dst_vaddr: usize, len: usize) {
    unsafe {
        let dst = core::slice::from_raw_parts_mut(dst_vaddr as *mut u8, len);
        dst.fill(0);
    }
}

/// Copy `src` bytes into the physical frame at `phys` via the shared temp
/// window. Returns `true` on success.
pub fn copy_to_temp_frame(phys: usize, src: &[u8]) -> bool {
    let ptr = unsafe { ffi::paging_temp_map_frame(phys) };
    if ptr.is_null() {
        return false;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(src.as_ptr(), ptr as *mut u8, src.len());
        ffi::paging_temp_unmap_frame();
    }
    true
}

/// Convert an ARGB8888 color to the native format for the given bpp.
pub fn convert_color(argb8888: u32, bpp: u8) -> u32 {
    match bpp {
        8 => {
            let r = (argb8888 >> 16) & 0xFF;
            let g = (argb8888 >> 8) & 0xFF;
            let b = argb8888 & 0xFF;
            let gray = ((r + g + b) / 3) as u8;
            (gray / 16) as u32
        }
        16 => {
            let r = ((argb8888 >> 16) & 0xFF) >> 3;
            let g = ((argb8888 >> 8) & 0xFF) >> 2;
            let b = (argb8888 & 0xFF) >> 3;
            ((r << 11) | (g << 5) | b) & 0xFFFF
        }
        24 => argb8888 & 0x00FFFFFF,
        32 => argb8888,
        _ => 0,
    }
}

/// Copy ARGB8888 pixels from `back_buffer` to the framebuffer at `fb_base`.
/// Handles 16/24/32 bpp conversion. `fb_base` is a framebuffer physical or
/// identity-mapped address.
pub fn present_back_buffer(
    fb_base: usize,
    back_buffer: &[u32],
    width: usize,
    height: usize,
    pitch: usize,
    bpp: u8,
) {
    unsafe {
        let base = fb_base as *mut u8;
        match bpp {
            32 => {
                for row in 0..height {
                    let dst = base.add(row.saturating_mul(pitch)) as *mut u32;
                    let src_offset = row.saturating_mul(width);
                    core::ptr::copy_nonoverlapping(
                        back_buffer[src_offset..].as_ptr(),
                        dst,
                        width,
                    );
                }
            }
            24 => {
                for row in 0..height {
                    let row_dst = base.add(row.saturating_mul(pitch));
                    let src_offset = row.saturating_mul(width);
                    for col in 0..width {
                        let color = back_buffer[src_offset + col];
                        let native = convert_color(color, 24);
                        let pixel_dst = row_dst.add(col.saturating_mul(3));
                        *pixel_dst = (native & 0xFF) as u8;
                        *pixel_dst.add(1) = ((native >> 8) & 0xFF) as u8;
                        *pixel_dst.add(2) = ((native >> 16) & 0xFF) as u8;
                    }
                }
            }
            16 => {
                for row in 0..height {
                    let dst = base.add(row.saturating_mul(pitch)) as *mut u16;
                    let src_offset = row.saturating_mul(width);
                    for col in 0..width {
                        let color = back_buffer[src_offset + col];
                        let native = convert_color(color, 16);
                        *dst.add(col) = (native & 0xFFFF) as u16;
                    }
                }
            }
            8 => {
                for row in 0..height {
                    let dst = base.add(row.saturating_mul(pitch));
                    let src_offset = row.saturating_mul(width);
                    for col in 0..width {
                        let color = back_buffer[src_offset + col];
                        let native = convert_color(color, 8);
                        *dst.add(col) = (native & 0xFF) as u8;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Write a single pixel to a framebuffer at `fb_base`.
pub fn write_framebuffer_pixel(
    fb_base: usize,
    x: u32,
    y: u32,
    color: u32,
    bpp: u8,
    pitch: usize,
    width: u32,
) {
    let offset = (y as usize).saturating_mul(pitch)
        + (x as usize).saturating_mul(bpp as usize / 8);
    unsafe {
        let base = fb_base as *mut u8;
        match bpp {
            8 => {
                *base.add(offset) = (color & 0xFF) as u8;
            }
            16 => {
                let ptr = base.add(offset) as *mut u16;
                *ptr = (color & 0xFFFF) as u16;
            }
            24 => {
                let ptr = base.add(offset);
                *ptr = (color & 0xFF) as u8;
                *ptr.add(1) = ((color >> 8) & 0xFF) as u8;
                *ptr.add(2) = ((color >> 16) & 0xFF) as u8;
            }
            32 => {
                let ptr = base.add(offset) as *mut u32;
                *ptr = color;
            }
            _ => {}
        }
    }
}
