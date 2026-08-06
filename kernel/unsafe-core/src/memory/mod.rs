//! Memory management abstraction
//!
//! Moved from `hal/src/memory/mod.rs` in Phase 1; the HAL re-exports this
//! module.

use core::ffi::c_void;

use crate::raw::ffi;

/// Page flags for memory mapping
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
}

/// Memory manager trait - architecture-agnostic interface
pub trait MemoryManager {
    /// Initialize the memory manager
    fn init(&mut self, multiboot_info_addr: u32);

    /// Allocate a physical frame (typically 4KB)
    fn alloc_frame(&mut self) -> Option<*mut c_void>;

    /// Free a physical frame
    fn free_frame(&mut self, addr: *mut c_void);

    /// Map a virtual address to a physical address with flags
    unsafe fn map_page(
        &mut self,
        virt_addr: *mut c_void,
        phys_addr: *mut c_void,
        flags: PageFlags,
    ) -> bool;

    /// Unmap a virtual address
    unsafe fn unmap_page(&mut self, virt_addr: *mut c_void);

    /// Allocate a virtual memory region
    fn alloc_region(&mut self, size: usize, flags: PageFlags) -> *mut c_void;

    /// Free a virtual memory region
    fn free_region(&mut self, addr: *mut c_void, size: usize);

    /// Get total physical memory in bytes
    fn total_memory(&self) -> u64;

    /// Get available physical memory in bytes
    fn available_memory(&self) -> u64;

    /// Get total number of frames
    fn total_frames(&self) -> u32;

    /// Get used number of frames
    fn used_frames(&self) -> u32;

    /// Create a new page directory / translation table root
    fn create_page_directory(&mut self) -> usize;

    /// Switch to a page directory (by physical address)
    fn switch_page_directory(&mut self, pd_phys: usize);

    /// Get the physical address of the kernel's page directory
    fn kernel_page_directory(&self) -> usize;

    /// Get the physical address for a virtual address
    fn get_physical_address(&self, virt: usize) -> usize;

    /// Destroy a page directory
    fn destroy_page_directory(&mut self, pd_phys: usize);
}

/// Physical Memory Manager (bitmap allocator)
pub struct Pmm {
    pub bitmap: *mut u8,
    pub total_frames: u32,
    pub used_frames: u32,
    pub memory_size: u64,
}

impl Pmm {
    pub const fn new() -> Self {
        Self {
            bitmap: core::ptr::null_mut(),
            total_frames: 0,
            used_frames: 0,
            memory_size: 0,
        }
    }
}

fn flags_to_raw(flags: PageFlags) -> u32 {
    let mut raw = 0u32;
    if flags.present { raw |= 0x001; }
    if flags.writable { raw |= 0x002; }
    if flags.user_accessible { raw |= 0x004; }
    if flags.cache_disabled { raw |= 0x010; }
    raw
}

impl MemoryManager for Pmm {
    fn init(&mut self, multiboot_info_addr: u32) {
        unsafe {
            ffi::pmm_init(multiboot_info_addr);
            ffi::vmm_init();
        }
        self.bitmap = core::ptr::null_mut();
        self.total_frames = unsafe { ffi::pmm_get_total_frames() };
        self.used_frames = unsafe { ffi::pmm_get_used_frames() };
        self.memory_size = unsafe { ffi::pmm_get_total_memory() };
    }

    fn alloc_frame(&mut self) -> Option<*mut c_void> {
        let ptr = unsafe { ffi::pmm_alloc_frame() };
        if ptr.is_null() {
            None
        } else {
            self.used_frames += 1;
            Some(ptr)
        }
    }

    fn free_frame(&mut self, addr: *mut c_void) {
        unsafe { ffi::pmm_free_frame(addr); }
        self.used_frames = self.used_frames.saturating_sub(1);
    }

    unsafe fn map_page(
        &mut self,
        virt_addr: *mut c_void,
        phys_addr: *mut c_void,
        flags: PageFlags,
    ) -> bool {
        ffi::vmm_map(virt_addr, phys_addr, flags_to_raw(flags))
    }

    unsafe fn unmap_page(&mut self, virt_addr: *mut c_void) {
        ffi::vmm_unmap(virt_addr);
    }

    fn alloc_region(&mut self, size: usize, flags: PageFlags) -> *mut c_void {
        unsafe { ffi::vmm_alloc_region(size, flags_to_raw(flags)) }
    }

    fn free_region(&mut self, addr: *mut c_void, size: usize) {
        unsafe { ffi::vmm_free_region(addr, size); }
    }

    fn total_memory(&self) -> u64 {
        if self.memory_size == 0 {
            unsafe { ffi::pmm_get_total_memory() }
        } else {
            self.memory_size
        }
    }

    fn available_memory(&self) -> u64 {
        unsafe { ffi::pmm_get_available_memory() }
    }

    fn total_frames(&self) -> u32 {
        if self.total_frames == 0 {
            unsafe { ffi::pmm_get_total_frames() }
        } else {
            self.total_frames
        }
    }

    fn used_frames(&self) -> u32 {
        if self.used_frames == 0 {
            unsafe { ffi::pmm_get_used_frames() }
        } else {
            self.used_frames
        }
    }

    fn create_page_directory(&mut self) -> usize {
        unsafe { ffi::paging_create_directory_phys() }
    }

    fn switch_page_directory(&mut self, pd_phys: usize) {
        unsafe { ffi::paging_switch_to_directory(pd_phys); }
    }

    fn kernel_page_directory(&self) -> usize {
        unsafe { ffi::paging_get_kernel_directory_phys() }
    }

    fn get_physical_address(&self, virt: usize) -> usize {
        unsafe { ffi::paging_get_physical_address(virt) }
    }

    fn destroy_page_directory(&mut self, pd_phys: usize) {
        unsafe { ffi::paging_destroy_directory(pd_phys); }
    }
}

/// x86_64 page table structures
#[cfg(feature = "x86_64")]
pub mod x86_64 {
    /// Page map level 4 entry
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Pml4Entry {
        pub entries: [u64; 512],
    }

    /// Page directory pointer table entry
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PdptEntry {
        pub entries: [u64; 512],
    }

    /// Page directory entry (64-bit)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageDirectoryEntry {
        pub entries: [u64; 512],
    }

    /// Page table entry (64-bit)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableEntry {
        pub entries: [u64; 512],
    }

    /// PTE flags for x86_64
    pub const PTE_PRESENT: u64 = 1 << 0;
    pub const PTE_WRITABLE: u64 = 1 << 1;
    pub const PTE_USER: u64 = 1 << 2;
    pub const PTE_WRITE_THROUGH: u64 = 1 << 3;
    pub const PTE_CACHE_DISABLE: u64 = 1 << 4;
    pub const PTE_ACCESSED: u64 = 1 << 5;
    pub const PTE_DIRTY: u64 = 1 << 6;
    pub const PTE_PS: u64 = 1 << 7;
    pub const PTE_GLOBAL: u64 = 1 << 8;
    pub const PTE_NX: u64 = 1 << 63;
}

/// ARM64 translation table structures
#[cfg(feature = "aarch64")]
pub mod aarch64 {
    /// Translation table descriptor for ARM64 (Level 0-2 block, Level 3 page)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct TranslationTableEntry {
        pub entries: [u64; 512],
    }

    /// Descriptor type bits
    pub const DESC_VALID: u64 = 1 << 0;
    pub const DESC_BLOCK: u64 = 0;
    pub const DESC_TABLE: u64 = 3;
    pub const DESC_PAGE: u64 = 3;

    /// Attribute index (MAIR)
    pub const ATTR_INDEX_NORMAL: u64 = 0 << 2;
    pub const ATTR_INDEX_DEVICE: u64 = 1 << 2;

    /// Access permissions (AP)
    pub const AP_RW_ALL: u64 = 0 << 6;
    pub const AP_RW_ALL_PL0_RO: u64 = 1 << 6;
    pub const AP_RO_ALL: u64 = 2 << 6;
    pub const AP_RO_ALL_PL0_RO: u64 = 3 << 6;

    /// Shareability
    pub const SH_OUTER: u64 = 2 << 8;
    pub const SH_INNER: u64 = 3 << 8;

    /// Execute never
    pub const UXN: u64 = 1 << 54;
    pub const PXN: u64 = 1 << 53;

    /// Memory attributes for MAIR_EL1
    pub const ATTR_NORMAL: u8 = 0xFF; // Normal cacheable
    pub const ATTR_DEVICE: u8 = 0x04; // Device-nGnRE
}
