/// Slab allocator for efficient small object allocation
/// 
/// Manages fixed-size blocks to reduce fragmentation and improve
/// performance for common allocation sizes.

use core::ptr::null_mut;
use crate::ffi;

/// Size classes for slab allocator (powers of 2)
const SLAB_SIZES: [usize; 8] = [8, 16, 32, 64, 128, 256, 512, 1024];

/// Number of objects per slab page
const OBJECTS_PER_SLAB: usize = 32;

/// Slab header
#[repr(C)]
struct SlabHeader {
    size_class: usize,
    free_count: usize,
    free_list: *mut FreeNode,
    next: *mut SlabHeader,
}

/// Free list node within a slab
#[repr(C)]
struct FreeNode {
    next: *mut FreeNode,
}

/// Slab cache for a specific size class
struct SlabCache {
    size: usize,
    partial_slabs: *mut SlabHeader,
    full_slabs: *mut SlabHeader,
    empty_slabs: *mut SlabHeader,
    objects_allocated: usize,
    objects_freed: usize,
}

impl SlabCache {
    const fn new(size: usize) -> Self {
        SlabCache {
            size,
            partial_slabs: null_mut(),
            full_slabs: null_mut(),
            empty_slabs: null_mut(),
            objects_allocated: 0,
            objects_freed: 0,
        }
    }
    
    /// Allocate an object from this cache
    unsafe fn alloc(&mut self) -> *mut u8 {
        // Try partial slabs first
        if !self.partial_slabs.is_null() {
            return self.alloc_from_slab(self.partial_slabs);
        }
        
        // Try to reuse an empty slab
        if !self.empty_slabs.is_null() {
            let slab = self.empty_slabs;
            self.empty_slabs = (*slab).next;
            (*slab).next = self.partial_slabs;
            self.partial_slabs = slab;
            return self.alloc_from_slab(slab);
        }
        
        // Need to allocate a new slab
        let slab = self.create_slab();
        if slab.is_null() {
            return null_mut();
        }
        
        (*slab).next = self.partial_slabs;
        self.partial_slabs = slab;
        
        self.alloc_from_slab(slab)
    }
    
    /// Allocate from a specific slab
    unsafe fn alloc_from_slab(&mut self, slab: *mut SlabHeader) -> *mut u8 {
        if (*slab).free_list.is_null() {
            return null_mut();
        }
        
        let node = (*slab).free_list;
        (*slab).free_list = (*node).next;
        (*slab).free_count -= 1;
        
        self.objects_allocated += 1;
        
        // If slab is now full, move it to full list
        if (*slab).free_count == 0 {
            self.move_to_full(slab);
        }
        
        node as *mut u8
    }
    
    /// Free an object back to the cache
    unsafe fn free(&mut self, ptr: *mut u8) {
        let slab = self.find_slab(ptr);
        if slab.is_null() {
            panic!("Slab allocator: invalid pointer {:p}", ptr);
        }
        
        let node = ptr as *mut FreeNode;
        (*node).next = (*slab).free_list;
        (*slab).free_list = node;
        (*slab).free_count += 1;
        
        self.objects_freed += 1;
        
        // Move slab to appropriate list
        if (*slab).free_count == OBJECTS_PER_SLAB {
            self.move_to_empty(slab);
        } else if (*slab).free_count == 1 {
            // Was full, now partial
            self.move_to_partial(slab);
        }
    }
    
    /// Create a new slab
    unsafe fn create_slab(&mut self) -> *mut SlabHeader {
        use crate::ffi;
        ffi::serial_print(b"[Slab] Creating new slab, calling vmm_alloc_region...\n\0".as_ptr());
        
        // Allocate memory for slab (1 page = 4KB)
        let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE;
        let ptr = ffi::vmm_alloc_region(4096, flags) as *mut u8;
        
        if ptr.is_null() {
            ffi::serial_print(b"[Slab] ERROR: vmm_alloc_region returned NULL!\n\0".as_ptr());
            return null_mut();
        }
        
        ffi::serial_print(b"[Slab] vmm_alloc_region succeeded, initializing slab...\n\0".as_ptr());
        
        let header = ptr as *mut SlabHeader;
        (*header).size_class = self.size;
        (*header).free_count = OBJECTS_PER_SLAB;
        (*header).free_list = null_mut();
        (*header).next = null_mut();
        
        // Initialize free list
        let data_start = ptr.add(core::mem::size_of::<SlabHeader>());
        let mut current = data_start;
        
        for i in 0..OBJECTS_PER_SLAB {
            let node = current as *mut FreeNode;
            
            if i < OBJECTS_PER_SLAB - 1 {
                (*node).next = current.add(self.size) as *mut FreeNode;
            } else {
                (*node).next = null_mut();
            }
            
            current = current.add(self.size);
        }
        
        (*header).free_list = data_start as *mut FreeNode;
        ffi::serial_print(b"[Slab] Slab initialization complete\n\0".as_ptr());
        header
    }
    
    /// Find which slab contains this pointer
    unsafe fn find_slab(&self, ptr: *mut u8) -> *mut SlabHeader {
        // Align pointer to page boundary to get slab header
        let page_addr = (ptr as usize) & !0xFFF;
        page_addr as *mut SlabHeader
    }
    
    /// Move slab to full list
    unsafe fn move_to_full(&mut self, slab: *mut SlabHeader) {
        // Remove from partial
        let mut prev = &mut self.partial_slabs as *mut *mut SlabHeader;
        let mut current = self.partial_slabs;
        
        while !current.is_null() {
            if current == slab {
                *prev = (*current).next;
                (*current).next = null_mut();
                break;
            }
            prev = &mut (*current).next;
            current = (*current).next;
        }
        
        // Add to full
        (*slab).next = self.full_slabs;
        self.full_slabs = slab;
    }
    
    /// Move slab to partial list
    unsafe fn move_to_partial(&mut self, slab: *mut SlabHeader) {
        // Remove from full
        let mut prev = &mut self.full_slabs as *mut *mut SlabHeader;
        let mut current = self.full_slabs;
        
        while !current.is_null() {
            if current == slab {
                *prev = (*current).next;
                (*current).next = null_mut();
                break;
            }
            prev = &mut (*current).next;
            current = (*current).next;
        }
        
        // Add to partial
        (*slab).next = self.partial_slabs;
        self.partial_slabs = slab;
    }
    
    /// Move slab to empty list
    unsafe fn move_to_empty(&mut self, slab: *mut SlabHeader) {
        // Remove from partial
        let mut prev = &mut self.partial_slabs as *mut *mut SlabHeader;
        let mut current = self.partial_slabs;
        
        while !current.is_null() {
            if current == slab {
                *prev = (*current).next;
                (*current).next = null_mut();
                break;
            }
            prev = &mut (*current).next;
            current = (*current).next;
        }
        
        // Add to empty
        (*slab).next = self.empty_slabs;
        self.empty_slabs = slab;
    }
}

/// Main slab allocator managing multiple size classes
pub struct SlabAllocator {
    caches: [SlabCache; 8],
}

impl SlabAllocator {
    pub const fn new() -> Self {
        SlabAllocator {
            caches: [
                SlabCache::new(SLAB_SIZES[0]),
                SlabCache::new(SLAB_SIZES[1]),
                SlabCache::new(SLAB_SIZES[2]),
                SlabCache::new(SLAB_SIZES[3]),
                SlabCache::new(SLAB_SIZES[4]),
                SlabCache::new(SLAB_SIZES[5]),
                SlabCache::new(SLAB_SIZES[6]),
                SlabCache::new(SLAB_SIZES[7]),
            ],
        }
    }
    
    /// Allocate from appropriate slab cache
    pub unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        use crate::ffi;
        ffi::serial_print(b"[Slab] Allocating object of size: \0".as_ptr());
        
        // Find appropriate size class
        for cache in &mut self.caches {
            if size <= cache.size {
                ffi::serial_print(b"using cache\n\0".as_ptr());
                let result = cache.alloc();
                if result.is_null() {
                    ffi::serial_print(b"[Slab] Cache alloc failed!\n\0".as_ptr());
                }
                return result;
            }
        }
        
        // Size too large for slab allocator
        ffi::serial_print(b"[Slab] Size too large for slab allocator\n\0".as_ptr());
        null_mut()
    }
    
    /// Free to appropriate slab cache
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize) {
        // Find appropriate size class
        for cache in &mut self.caches {
            if size <= cache.size {
                cache.free(ptr);
                return;
            }
        }
    }
    
    /// Check if size is suitable for slab allocation
    pub fn can_allocate(&self, size: usize) -> bool {
        size <= SLAB_SIZES[SLAB_SIZES.len() - 1]
    }
    
    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        let mut total_alloc = 0;
        let mut total_free = 0;
        
        for cache in &self.caches {
            total_alloc += cache.objects_allocated;
            total_free += cache.objects_freed;
        }
        
        (total_alloc, total_free)
    }
}

unsafe impl Send for SlabAllocator {}
unsafe impl Sync for SlabAllocator {}
