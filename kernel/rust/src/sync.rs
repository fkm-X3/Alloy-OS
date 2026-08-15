//! Synchronization primitives for the kernel
//!
//! Provides interrupt-safe spinlocks and other synchronization tools

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, AtomicBool, Ordering};

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
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        // Acquire spinlock
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        // Memory barrier
        fence(Ordering::Acquire);

        SpinLockGuard { lock: self }
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
    pub fn lock(&self) -> SpinlockIRQGuard<'_, T> {
        // Disable interrupts
        let flags = self.disable_interrupts();

        // Acquire spinlock
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        // Memory barrier
        fence(Ordering::Acquire);

        SpinlockIRQGuard { lock: self, flags }
    }

    /// Disable interrupts and return previous state
    #[cfg(feature = "x86_64")]
    #[inline]
    fn disable_interrupts(&self) -> u32 {
        let rflags: u64;
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {0:r}",
                out(reg) rflags
            );
            core::arch::asm!("cli");
        }
        rflags as u32
    }

    /// Restore interrupt state (x86)
    #[cfg(feature = "x86_64")]
    #[inline]
    fn restore_interrupts(&self, flags: u32) {
        unsafe {
            // Check if interrupts were enabled (IF bit = bit 9)
            if (flags & 0x200) != 0 {
                core::arch::asm!("sti");
            }
        }
    }

    #[cfg(feature = "aarch64")]
    #[inline]
    fn disable_interrupts(&self) -> u32 {
        let flags: u64;
        unsafe {
            core::arch::asm!("mrs {}, daif", out(reg) flags);
            core::arch::asm!("msr daifset, #2"); // Mask IRQ (DAIF immediate bit 1 = I)
        }
        // Return previous IRQ mask state: DAIF bit 7 = I (PSR_I_BIT)
        ((flags >> 7) & 1) as u32
    }

    #[cfg(feature = "aarch64")]
    #[inline]
    fn restore_interrupts(&self, flags: u32) {
        unsafe {
            if (flags & 1) == 0 {
                // Was not masked, so unmask IRQ
                core::arch::asm!("msr daifclr, #2"); // Unmask IRQ
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

    /// Release the lock without restoring interrupt state.
    /// Caller must ensure interrupts are in the desired state (e.g. disabled via cli).
    pub fn release_no_irq_restore(self) {
        fence(Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
        core::mem::forget(self);
    }
}

impl<'a, T> Deref for SpinlockIRQGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinlockIRQGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
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

/// Save the current interrupt state and mask IRQs, returning the previous
/// state so the caller can later restore it with [`irq_restore`].
///
/// Unlike [`irq_disable`], this preserves the prior mask state, so it is safe
/// to use from contexts where IRQs are already masked (e.g. inside an IRQ
/// handler).
#[inline]
pub fn irq_save() -> u64 {
    #[cfg(feature = "x86_64")]
    {
        let flags: u64;
        unsafe {
            core::arch::asm!("pushfq", "pop {0}", out(reg) flags, options(preserves_flags));
            core::arch::asm!("cli", options(nomem, nostack));
        }
        flags
    }
    #[cfg(feature = "aarch64")]
    {
        let flags: u64;
        unsafe {
            core::arch::asm!("mrs {}, daif", out(reg) flags);
            core::arch::asm!("msr daifset, #2");
        }
        flags
    }
}

/// Restore a previously saved interrupt state (see [`irq_save`]).
#[inline]
pub fn irq_restore(flags: u64) {
    #[cfg(feature = "x86_64")]
    unsafe {
        core::arch::asm!("push {0}", "popfq", in(reg) flags, options(nomem, nostack));
    }
    #[cfg(feature = "aarch64")]
    unsafe {
        core::arch::asm!("msr daif, {}", in(reg) flags);
    }
}

/// Disable interrupts (mask IRQ) — arch-specific.
///
/// x86_64: `cli`. aarch64: mask IRQ via DAIF (bit 1), matching the IRQ-only
/// masking used by [`SpinlockIRQ`].
#[inline]
pub fn irq_disable() {
    #[cfg(feature = "x86_64")]
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
    #[cfg(feature = "aarch64")]
    unsafe {
        core::arch::asm!("msr daifset, #2");
    }
}

/// Re-enable interrupts (unmask IRQ) — arch-specific.
#[inline]
pub fn irq_enable() {
    #[cfg(feature = "x86_64")]
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
    #[cfg(feature = "aarch64")]
    unsafe {
        core::arch::asm!("msr daifclr, #2");
    }
}
