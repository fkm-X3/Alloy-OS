/// Global allocator implementation for Rust kernel
/// 
/// This allocator bridges to the C++ VMM (Virtual Memory Manager)
/// to provide heap allocation for Rust code.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use crate::ffi;

/// Alloy kernel allocator backed by VMM
pub struct AllocatorVMM;

unsafe impl GlobalAlloc for AllocatorVMM {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Align up to page size if needed
        let size = layout.size();
        let align = layout.align();
        
        // For now, use VMM which allocates in pages (4KB)
        // We'll use a simple strategy: allocate at least one page
        let page_size = 4096;
        let pages_needed = (size + page_size - 1) / page_size;
        let alloc_size = pages_needed * page_size;
        
        // VMM flags: Present | Writable
        let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE;
        
        let ptr = ffi::vmm_alloc_region(alloc_size as u32, flags);
        
        if ptr.is_null() {
            return null_mut();
        }
        
        // Check alignment
        let addr = ptr as usize;
        if addr % align != 0 {
            // Free the misaligned allocation
            ffi::vmm_free_region(ptr, alloc_size as u32);
            return null_mut();
        }
        
        ptr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let page_size = 4096;
        let pages_needed = (size + page_size - 1) / page_size;
        let alloc_size = pages_needed * page_size;
        
        ffi::vmm_free_region(ptr as *mut core::ffi::c_void, alloc_size as u32);
    }
}

/// Global allocator instance
#[global_allocator]
static ALLOCATOR: AllocatorVMM = AllocatorVMM;

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: failed to allocate {} bytes with {} byte alignment", 
           layout.size(), layout.align());
}
