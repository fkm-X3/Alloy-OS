/// Synchronization primitives for the kernel
/// 
/// Provides interrupt-safe spinlocks and other synchronization tools

use core::sync::atomic::{AtomicBool, Ordering, fence};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Simple spinlock (without interrupt handling)
/// Use this for data that's only accessed with interrupts already disabled
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    /// Create a new spinlock
    pub const fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Acquire lock
    pub fn lock(&self) -> SpinLockGuard<T> {
        // Acquire spinlock
        while self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        
        // Memory barrier
        fence(Ordering::Acquire);
        
        SpinLockGuard {
            lock: self,
        }
    }
}

/// Guard for simple spinlock
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        // Memory barrier before releasing
        fence(Ordering::Release);
        
        // Release lock
        self.lock.locked.store(false, Ordering::Release);
    }
}

// Safety: SpinLock can be shared between threads
unsafe impl<T> Sync for SpinLock<T> where T: Send {}
unsafe impl<T> Send for SpinLock<T> where T: Send {}

/// Interrupt-safe spinlock that disables IRQs while held
/// 
/// This prevents deadlocks when the same lock is acquired from
/// interrupt context (e.g., allocator called from IRQ handler)
pub struct SpinlockIRQ<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinlockIRQ<T> {
    /// Create a new interrupt-safe spinlock
    pub const fn new(data: T) -> Self {
        SpinlockIRQ {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Acquire lock (disables interrupts)
    /// Returns previous interrupt state and lock guard
    pub fn lock(&self) -> SpinlockIRQGuard<T> {
        // Disable interrupts
        let flags = self.disable_interrupts();
        
        // Acquire spinlock
        while self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        
        // Memory barrier
        fence(Ordering::Acquire);
        
        SpinlockIRQGuard {
            lock: self,
            flags,
        }
    }
    
    /// Disable interrupts and return previous state
    #[inline]
    fn disable_interrupts(&self) -> u32 {
        let flags: u32;
        unsafe {
            // Read EFLAGS
            core::arch::asm!(
                "pushfd",
                "pop {0}",
                out(reg) flags
            );
            
            // Disable interrupts (CLI)
            core::arch::asm!("cli");
        }
        flags
    }
    
    /// Restore interrupt state
    #[inline]
    fn restore_interrupts(&self, flags: u32) {
        unsafe {
            // Check if interrupts were enabled (IF bit = bit 9)
            if (flags & 0x200) != 0 {
                core::arch::asm!("sti");
            }
        }
    }
}

/// Guard for interrupt-safe spinlock
/// Automatically releases lock and restores interrupts when dropped
pub struct SpinlockIRQGuard<'a, T> {
    lock: &'a SpinlockIRQ<T>,
    flags: u32,
}

impl<'a, T> SpinlockIRQGuard<'a, T> {
    /// Get mutable reference to protected data
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinlockIRQGuard<'a, T> {
    fn drop(&mut self) {
        // Memory barrier before releasing
        fence(Ordering::Release);
        
        // Release lock
        self.lock.locked.store(false, Ordering::Release);
        
        // Restore interrupts
        self.lock.restore_interrupts(self.flags);
    }
}

// Safety: SpinlockIRQ can be shared between threads (we disable IRQs)
unsafe impl<T> Sync for SpinlockIRQ<T> where T: Send {}
unsafe impl<T> Send for SpinlockIRQ<T> where T: Send {}
