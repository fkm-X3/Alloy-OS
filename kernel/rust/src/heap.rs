/// Heap allocator module
/// 
/// Provides a proper heap allocator with better granularity than
/// the page-based allocator. Uses a linked list of free blocks.

use core::ptr::null_mut;
use core::alloc::Layout;
use crate::ffi;

/// Minimum allocation size (to store free list node)
const MIN_BLOCK_SIZE: usize = 16;

/// Alignment for all allocations
const HEAP_ALIGN: usize = 8;

/// Magic number to detect corruption
const MAGIC: u32 = 0xDEADBEEF;

/// Header for each allocated block
#[repr(C)]
struct BlockHeader {
    magic: u32,
    size: usize,
    next: *mut BlockHeader,
}

impl BlockHeader {
    fn new(size: usize) -> Self {
        BlockHeader {
            magic: MAGIC,
            size,
            next: null_mut(),
        }
    }
    
    /// Check if header is valid (not corrupted)
    fn is_valid(&self) -> bool {
        // Check magic number
        if self.magic != MAGIC {
            return false;
        }
        
        // Check size is reasonable (not zero, not huge)
        if self.size == 0 || self.size > 1024 * 1024 * 1024 {
            return false;
        }
        
        // Check size is properly aligned
        if self.size % MIN_BLOCK_SIZE != 0 {
            return false;
        }
        
        true
    }
    
    fn data_ptr(&self) -> *mut u8 {
        unsafe {
            (self as *const BlockHeader as *mut u8).add(core::mem::size_of::<BlockHeader>())
        }
    }
    
    fn from_data_ptr(ptr: *mut u8) -> *mut BlockHeader {
        unsafe {
            ptr.sub(core::mem::size_of::<BlockHeader>()) as *mut BlockHeader
        }
    }
}

/// Heap allocator with free list
pub struct HeapAllocator {
    free_list: *mut BlockHeader,
    total_allocated: usize,
    total_freed: usize,
}

impl HeapAllocator {
    /// Create a new heap allocator with empty free list and zero counters
    /// This is safe because all pointers start as null and counters as zero
    pub const fn new() -> Self {
        HeapAllocator {
            free_list: null_mut(),
            total_allocated: 0,
            total_freed: 0,
        }
    }
    
    /// Allocate a block from the heap
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(MIN_BLOCK_SIZE);
        let _align = layout.align().max(HEAP_ALIGN);
        
        // Total size needed including header
        let total_size = size + core::mem::size_of::<BlockHeader>();
        
        // Try to find a suitable free block
        if let Some(block) = self.find_free_block(total_size) {
            self.total_allocated += total_size;
            return block;
        }
        
        // No suitable block found, allocate new pages from VMM
        let pages_needed = (total_size + 4095) / 4096;
        let alloc_size = pages_needed * 4096;
        
        let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE;
        let ptr = ffi::vmm_alloc_region(alloc_size as u32, flags) as *mut u8;
        
        if ptr.is_null() {
            ffi::serial_print(b"[Heap] ERROR: VMM allocation failed!\n\0".as_ptr());
            return null_mut();
        }
        
        // Create header
        let header = ptr as *mut BlockHeader;
        (*header) = BlockHeader::new(size);
        
        // If we allocated more than needed, add remainder to free list
        let remaining = alloc_size - total_size;
        if remaining > core::mem::size_of::<BlockHeader>() + MIN_BLOCK_SIZE {
            let remainder_ptr = ptr.add(total_size);
            let remainder_header = remainder_ptr as *mut BlockHeader;
            (*remainder_header) = BlockHeader::new(remaining - core::mem::size_of::<BlockHeader>());
            (*remainder_header).next = self.free_list;
            self.free_list = remainder_header;
        }
        
        self.total_allocated += total_size;
        (*header).data_ptr()
    }
    
    /// Deallocate a block
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        
        let header = BlockHeader::from_data_ptr(ptr);
        
        // Validate header before proceeding
        if !(*header).is_valid() {
            // Detailed corruption reporting
            use crate::ffi;
            ffi::serial_print(b"[Heap] CRITICAL: Heap corruption detected!\n\0".as_ptr());
            ffi::serial_print(b"  Pointer: \0".as_ptr());
            ffi::serial_print(b"  Expected magic: 0xDEADBEEF\n\0".as_ptr());
            ffi::serial_print(b"  Actual magic: \0".as_ptr());
            
            panic!("Heap corruption at {:p}", ptr);
        }
        
        let size = (*header).size + core::mem::size_of::<BlockHeader>();
        self.total_freed += size;
        
        // Add to free list
        (*header).next = self.free_list;
        self.free_list = header;
    }
    
    /// Find a free block that can satisfy the allocation
    unsafe fn find_free_block(&mut self, size: usize) -> Option<*mut u8> {
        let mut prev: *mut *mut BlockHeader = &mut self.free_list;
        let mut current = self.free_list;
        
        while !current.is_null() {
            let block_size = (*current).size + core::mem::size_of::<BlockHeader>();
            
            if block_size >= size {
                // Remove from free list
                *prev = (*current).next;
                
                // If block is much larger, split it
                let remaining = block_size - size;
                if remaining > core::mem::size_of::<BlockHeader>() + MIN_BLOCK_SIZE {
                    let split_ptr = (current as *mut u8).add(size);
                    let split_header = split_ptr as *mut BlockHeader;
                    (*split_header) = BlockHeader::new(remaining - core::mem::size_of::<BlockHeader>());
                    (*split_header).next = self.free_list;
                    self.free_list = split_header;
                    
                    // Update current block size
                    (*current).size = size - core::mem::size_of::<BlockHeader>();
                }
                
                return Some((*current).data_ptr());
            }
            
            prev = &mut (*current).next;
            current = (*current).next;
        }
        
        None
    }
    
    /// Get allocation statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.total_allocated, self.total_freed)
    }
}

// Thread-safety: In a single-threaded kernel, we don't need synchronization
unsafe impl Send for HeapAllocator {}
unsafe impl Sync for HeapAllocator {}
