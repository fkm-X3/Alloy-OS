//! Synchronization primitives for the kernel.
//!
//! Safe guards over the raw interrupt-mask + spinlock asm in [`crate::raw::asm`]:
//! - `SpinLock<T>`: plain spinlock. Safe only when the guarded data is never
//!   touched from interrupt context (or while IRQs are already masked).
//! - `SpinLockIrq<T>`: spinlock that masks IRQs while held, so an IRQ handler
//!   that takes the same lock (e.g. the allocator called from the timer tick)
//!   cannot deadlock the holder.
//! - `irq_save`/`irq_restore`/`irq_disable`/`irq_enable`: the raw interrupt
//!   mask primitives the allocator and scheduler use directly.
//!
//! aarch64 DAIF note: IRQ masking is the DAIF immediate I bit (`daifset #2` /
//! `daifclr #2`), and `irq_save`/`irq_restore` carry the full DAIF register so
//! the pre-mask state (including the I bit) is preserved exactly. This keeps
//! the IRQ-only masking semantics the C-era `disable_interrupts` adopted for
//! aarch64 (PSR I bit, not the x86 IF-equivalent bit position).

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, AtomicBool, Ordering};

use crate::raw::asm;

/// A plain spinlock.
///
/// Use this for data that is only accessed with interrupts already disabled,
/// or that interrupt handlers never touch.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    /// Create a new unlocked spinlock.
    pub const fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquire the lock, spinning until it is free.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        fence(Ordering::Acquire);
        SpinLockGuard { lock: self }
    }
}

/// Guard for a plain [`SpinLock`]; releases the lock on drop.
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
        fence(Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
    }
}

// Safety: the guard gives exclusive access to `T`; the lock is shareable.
unsafe impl<T> Sync for SpinLock<T> where T: Send {}
unsafe impl<T> Send for SpinLock<T> where T: Send {}

/// An interrupt-safe spinlock: masks IRQs while the lock is held.
///
/// Prevents deadlocks when the same lock is acquired from interrupt context
/// (e.g. the allocator being called from the timer IRQ handler).
pub struct SpinLockIrq<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinLockIrq<T> {
    /// Create a new unlocked, IRQ-masking spinlock.
    pub const fn new(data: T) -> Self {
        SpinLockIrq {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquire the lock, masking IRQs for the duration of the guard.
    pub fn lock(&self) -> SpinLockIrqGuard<'_, T> {
        let flags = asm::save_irq_state();
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        fence(Ordering::Acquire);
        SpinLockIrqGuard { lock: self, flags }
    }
}

/// Guard for an interrupt-safe [`SpinLockIrq`].
///
/// Restores the pre-lock interrupt state and releases the lock on drop.
pub struct SpinLockIrqGuard<'a, T> {
    lock: &'a SpinLockIrq<T>,
    flags: u64,
}

impl<'a, T> SpinLockIrqGuard<'a, T> {
    /// Release the lock WITHOUT restoring the interrupt state.
    ///
    /// Caller must ensure interrupts are left in a known state (e.g. the
    /// context switch that is about to load the next task's saved mask). The
    /// guard is consumed so its drop cannot double-release.
    pub fn release_no_irq_restore(self) {
        fence(Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
        core::mem::forget(self);
    }
}

impl<'a, T> Deref for SpinLockIrqGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockIrqGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockIrqGuard<'a, T> {
    fn drop(&mut self) {
        fence(Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
        asm::restore_irq_state(self.flags);
    }
}

// Safety: interrupts are masked while the lock is held, so a task can never
// be preempted mid-guard by a handler that takes the same lock.
unsafe impl<T> Sync for SpinLockIrq<T> where T: Send {}
unsafe impl<T> Send for SpinLockIrq<T> where T: Send {}

/// Save the current interrupt mask state and mask IRQs, returning the previous
/// state so the caller can restore it with [`irq_restore`].
///
/// Unlike [`irq_disable`], this preserves the prior mask state, so it is safe
/// to use from contexts where IRQs are already masked (e.g. inside an IRQ
/// handler).
#[inline]
pub fn irq_save() -> u64 {
    asm::save_irq_state()
}

/// Restore a previously saved interrupt mask state (see [`irq_save`]).
#[inline]
pub fn irq_restore(flags: u64) {
    asm::restore_irq_state(flags);
}

/// Mask IRQs (x86_64 `cli`; aarch64 DAIF I bit). No return value.
#[inline]
pub fn irq_disable() {
    asm::disable_irqs();
}

/// Unmask IRQs (x86_64 `sti`; aarch64 DAIF I bit).
#[inline]
pub fn irq_enable() {
    asm::enable_irqs();
}
