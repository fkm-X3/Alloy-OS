/// Global allocator implementation for Rust kernel
/// 
/// This allocator uses a two-tier strategy:
/// - Slab allocator for small objects (<= 1024 bytes)
/// - Heap allocator for larger objects

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::heap::HeapAllocator;
use crate::slab::SlabAllocator;

/// Global lock for allocator (simple spinlock)
static ALLOC_LOCK: AtomicBool = AtomicBool::new(false);

/// Slab allocator for small objects
static mut SLAB_ALLOCATOR: SlabAllocator = SlabAllocator::new();

/// Heap allocator for larger objects
static mut HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

/// Acquire allocator lock
fn lock() {
    while ALLOC_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}

/// Release allocator lock
fn unlock() {
    ALLOC_LOCK.store(false, Ordering::Release);
}

/// Alloy kernel allocator with slab and heap tiers
pub struct AllocatorVMM;

unsafe impl GlobalAlloc for AllocatorVMM {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Debug logging for allocations
        use crate::ffi;
        ffi::serial_print(b"[Allocator] Alloc request: size=\0".as_ptr());
        
        lock();
        
        let result = if SLAB_ALLOCATOR.can_allocate(layout.size()) {
            // Use slab allocator for small objects
            ffi::serial_print(b"slab\0".as_ptr());
            let ptr = SLAB_ALLOCATOR.alloc(layout.size());
            ffi::serial_print(b" -> \0".as_ptr());
            if ptr.is_null() {
                ffi::serial_print(b"NULL\n\0".as_ptr());
            } else {
                ffi::serial_print(b"OK\n\0".as_ptr());
            }
            ptr
        } else {
            // Use heap allocator for larger objects
            ffi::serial_print(b"heap\0".as_ptr());
            let ptr = HEAP_ALLOCATOR.alloc(layout);
            ffi::serial_print(b" -> \0".as_ptr());
            if ptr.is_null() {
                ffi::serial_print(b"NULL\n\0".as_ptr());
            } else {
                ffi::serial_print(b"OK\n\0".as_ptr());
            }
            ptr
        };
        
        unlock();
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        use crate::ffi;
        ffi::serial_print(b"[Allocator] Dealloc request\n\0".as_ptr());
        
        lock();
        
        if SLAB_ALLOCATOR.can_allocate(layout.size()) {
            SLAB_ALLOCATOR.free(ptr, layout.size());
        } else {
            HEAP_ALLOCATOR.dealloc(ptr, layout);
        }
        
        unlock();
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

/// Get allocation statistics
pub fn get_stats() -> ((usize, usize), (usize, usize)) {
    unsafe {
        lock();
        let slab_stats = SLAB_ALLOCATOR.stats();
        let heap_stats = HEAP_ALLOCATOR.stats();
        unlock();
        (slab_stats, heap_stats)
    }
}
