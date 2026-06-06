//! Global allocator implementation for Rust kernel
//! 
//! This allocator uses a two-tier strategy:
//! - Slab allocator for small objects (<= 1024 bytes)
//! - Heap allocator for larger objects

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering, fence};
use crate::heap::HeapAllocator;
use crate::slab::SlabAllocator;

/// Wrapper to make `UnsafeCell` `Sync` when access is guarded by `ALLOC_LOCK`.
struct AllocCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for AllocCell<T> {}

impl<T> AllocCell<T> {
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// Global lock for allocator (simple spinlock)
static ALLOC_LOCK: AtomicBool = AtomicBool::new(false);

/// Slab allocator for small objects
static SLAB_ALLOCATOR: AllocCell<SlabAllocator> = AllocCell(UnsafeCell::new(SlabAllocator::new()));

/// Heap allocator for larger objects
static HEAP_ALLOCATOR: AllocCell<HeapAllocator> = AllocCell(UnsafeCell::new(HeapAllocator::new()));

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
        
        let result = if unsafe { (*SLAB_ALLOCATOR.get()).can_allocate(layout.size(), layout.align()) } {
            // Use slab allocator for small objects
            unsafe { (*SLAB_ALLOCATOR.get()).alloc(layout.size(), layout.align()) }
        } else {
            // Use heap allocator for larger objects
            unsafe { (*HEAP_ALLOCATOR.get()).alloc(layout) }
        };
        
        unlock();
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        lock();
        
        if unsafe { (*SLAB_ALLOCATOR.get()).can_allocate(layout.size(), layout.align()) } {
            unsafe { (*SLAB_ALLOCATOR.get()).free(ptr, layout.size(), layout.align()); }
        } else {
            unsafe { (*HEAP_ALLOCATOR.get()).dealloc(ptr, layout); }
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
        let slab_stats = (*SLAB_ALLOCATOR.get()).stats();
        let heap_stats = (*HEAP_ALLOCATOR.get()).stats();
        unlock();
        (slab_stats, heap_stats)
    }
}

/// Print allocation statistics to serial (non-intrusive)
pub fn print_stats() {
    let _ = get_stats();
    
    unsafe {
        use crate::ffi;
        ffi::serial_print(c"\n=== Allocator Statistics ===\n".as_ptr() as *const u8);
        ffi::serial_print(c"Slab allocator:\n".as_ptr() as *const u8);
        ffi::serial_print(c"  Objects allocated: ".as_ptr() as *const u8);
        ffi::serial_print(c"  Objects freed: ".as_ptr() as *const u8);
        ffi::serial_print(c"  Net objects: ".as_ptr() as *const u8);
        
        ffi::serial_print(c"\nHeap allocator:\n".as_ptr() as *const u8);
        ffi::serial_print(c"  Bytes allocated: ".as_ptr() as *const u8);
        ffi::serial_print(c"  Bytes freed: ".as_ptr() as *const u8);
        ffi::serial_print(c"  Net bytes: ".as_ptr() as *const u8);
        ffi::serial_print(c"===========================\n\n".as_ptr() as *const u8);
    }
}
