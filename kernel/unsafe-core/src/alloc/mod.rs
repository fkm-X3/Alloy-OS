//! Kernel global allocator.
//!
//! The `unsafe impl GlobalAlloc` that backs `#[global_allocator]` in the
//! kernel crate lives here, behind a two-tier strategy:
//! - [`Slab`] for small objects (<= 1024 bytes)
//! - a free-list [`HeapAllocator`] for larger objects
//!
//! Moved verbatim from the kernel crate's `allocator.rs`; the IRQ-mask lock
//! now uses the [`crate::sync`] primitives. The lock masks IRQs while held
//! because the timer IRQ handler can allocate (the scheduler re-enqueues the
//! preempted task, growing the ready queue), so a plain spinlock would
//! deadlock if an IRQ fired mid-allocation.

pub mod heap;
pub mod slab;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{fence, AtomicBool, Ordering};

use crate::sync::{irq_restore, irq_save};

use heap::HeapAllocator;
pub use slab::Slab;

/// Wrapper to make `UnsafeCell` `Sync` when access is guarded by `ALLOC_LOCK`.
struct AllocCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for AllocCell<T> {}

impl<T> AllocCell<T> {
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// Global lock for the allocator.
///
/// Masks IRQs while held: the timer IRQ handler can allocate (the scheduler
/// re-enqueues the preempted task, growing the ready queue), so a plain
/// spinlock would deadlock if an IRQ fired mid-allocation. `lock()` returns
/// the saved interrupt state, which `unlock()` restores.
static ALLOC_LOCK: AtomicBool = AtomicBool::new(false);

/// Slab allocator for small objects
static SLAB_ALLOCATOR: AllocCell<Slab> = AllocCell(UnsafeCell::new(Slab::new()));

/// Heap allocator for larger objects
static HEAP_ALLOCATOR: AllocCell<HeapAllocator> = AllocCell(UnsafeCell::new(HeapAllocator::new()));

/// Acquire the allocator lock with memory barriers, masking IRQs while held.
/// Returns the previous interrupt state for [`unlock`].
fn lock() -> u64 {
    let flags = irq_save();
    while ALLOC_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
    // Ensure all previous writes are visible
    fence(Ordering::Acquire);
    flags
}

/// Release the allocator lock with memory barriers, restoring the saved IRQ
/// state.
fn unlock(flags: u64) {
    // Ensure all our writes complete before releasing
    fence(Ordering::Release);
    ALLOC_LOCK.store(false, Ordering::Release);
    irq_restore(flags);
}

/// The Alloy kernel allocator with slab and heap tiers.
///
/// Used by the kernel crate's `#[global_allocator]` static; also exposes the
/// combined allocation statistics for the `stats` shell command.
pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let flags = lock();

        let result =
            if unsafe { (*SLAB_ALLOCATOR.get()).can_allocate(layout.size(), layout.align()) } {
                // Use slab allocator for small objects
                unsafe { (*SLAB_ALLOCATOR.get()).alloc(layout.size(), layout.align()) }
            } else {
                // Use heap allocator for larger objects
                unsafe { (*HEAP_ALLOCATOR.get()).alloc(layout) }
            };

        unlock(flags);
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let flags = lock();

        if unsafe { (*SLAB_ALLOCATOR.get()).can_allocate(layout.size(), layout.align()) } {
            unsafe {
                (*SLAB_ALLOCATOR.get()).free(ptr, layout.size(), layout.align());
            }
        } else {
            unsafe {
                (*HEAP_ALLOCATOR.get()).dealloc(ptr, layout);
            }
        }

        unlock(flags);
    }
}

impl KernelAllocator {
    /// Get allocation statistics:
    /// `((slab_allocated, slab_freed), (heap_allocated, heap_freed))`.
    pub fn get_stats() -> ((usize, usize), (usize, usize)) {
        let flags = lock();
        let slab_stats = unsafe { (*SLAB_ALLOCATOR.get()).stats() };
        let heap_stats = unsafe { (*HEAP_ALLOCATOR.get()).stats() };
        unlock(flags);
        (slab_stats, heap_stats)
    }
}

/// Get allocation statistics for the `stats` shell command
/// (`((slab_allocated, slab_freed), (heap_allocated, heap_freed))`).
pub fn get_stats() -> ((usize, usize), (usize, usize)) {
    KernelAllocator::get_stats()
}
