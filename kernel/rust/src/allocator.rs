/// Global allocator implementation for Rust kernel
/// 
/// This allocator uses a two-tier strategy:
/// - Slab allocator for small objects (<= 1024 bytes)
/// - Heap allocator for larger objects

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering, fence};
use crate::heap::HeapAllocator;
use crate::slab::SlabAllocator;

/// Global lock for allocator (simple spinlock)
static ALLOC_LOCK: AtomicBool = AtomicBool::new(false);

/// Slab allocator for small objects
static mut SLAB_ALLOCATOR: SlabAllocator = SlabAllocator::new();

/// Heap allocator for larger objects
static mut HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

/// Acquire allocator lock with memory barriers
fn lock() {
    while ALLOC_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
    // Ensure all previous writes are visible
    fence(Ordering::Acquire);
}

/// Release allocator lock with memory barriers
fn unlock() {
    // Ensure all our writes complete before releasing
    fence(Ordering::Release);
    ALLOC_LOCK.store(false, Ordering::Release);
}

/// Alloy kernel allocator with slab and heap tiers
pub struct AllocatorVMM;

unsafe impl GlobalAlloc for AllocatorVMM {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        lock();
        
        let result = if SLAB_ALLOCATOR.can_allocate(layout.size()) {
            // Use slab allocator for small objects
            SLAB_ALLOCATOR.alloc(layout.size())
        } else {
            // Use heap allocator for larger objects
            HEAP_ALLOCATOR.alloc(layout)
        };
        
        unlock();
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
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

/// Print allocation statistics to serial (non-intrusive)
pub fn print_stats() {
    let ((slab_alloc, slab_free), (heap_alloc, heap_free)) = get_stats();
    
    unsafe {
        use crate::ffi;
        ffi::serial_print(b"\n=== Allocator Statistics ===\n\0".as_ptr());
        ffi::serial_print(b"Slab allocator:\n\0".as_ptr());
        ffi::serial_print(b"  Objects allocated: \0".as_ptr());
        ffi::serial_print(b"  Objects freed: \0".as_ptr());
        ffi::serial_print(b"  Net objects: \0".as_ptr());
        
        ffi::serial_print(b"\nHeap allocator:\n\0".as_ptr());
        ffi::serial_print(b"  Bytes allocated: \0".as_ptr());
        ffi::serial_print(b"  Bytes freed: \0".as_ptr());
        ffi::serial_print(b"  Net bytes: \0".as_ptr());
        ffi::serial_print(b"===========================\n\n\0".as_ptr());
    }
}
