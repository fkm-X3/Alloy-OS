//! Callback registration for the syscall / timer / page-fault paths.
//!
//! The ported syscall dispatcher, timer IRQ, and page-fault handler live in
//! `unsafe-core` (they run in syscall/IRQ/exception context and touch
//! registers), but they never call back into the kernel crate by symbol.
//! Instead the kernel crate registers plain function pointers here at init
//! (`rust_main`); the ported code invokes them through [`dispatch_syscall`],
//! [`invoke_timer_tick`], and [`invoke_page_fault`].
//!
//! This keeps the dependency direction strictly one-way — `unsafe-core`
//! never depends on the kernel crate — and removes the `rust_sys_*` /
//! `rust_timer_tick` / `rust_handle_page_fault` C↔Rust call loops that used
//! to make the two crates re-entrant.
//!
//! The tables are plain `static mut` arrays written once during boot, before
//! any userland or IRQ can observe them.

/// A registered syscall handler. Takes the up-to-5 syscall arguments and
/// returns the raw result to hand back to userland.
pub type SyscallHandler = fn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32) -> u32;

/// Number of syscall slots (all current syscall numbers are < 32).
const SYSCALL_SLOTS: usize = 32;

static mut SYSCALL_TABLE: [Option<SyscallHandler>; SYSCALL_SLOTS] = [None; SYSCALL_SLOTS];

/// Registered timer-tick handler (scheduler preemption), if any.
static mut TIMER_TICK_HANDLER: Option<fn()> = None;

/// Registered keyboard-wake handler (wakes tasks blocked on input), if any.
static mut KEYBOARD_WAKE_HANDLER: Option<fn()> = None;

/// Registered mouse-wake handler (wakes tasks blocked on input), if any.
static mut MOUSE_WAKE_HANDLER: Option<fn()> = None;

/// Registered page-fault handler (task termination policy), if any.
static mut PAGE_FAULT_HANDLER: Option<fn(usize, u32) -> FaultAction> = None;

/// What the kernel chose to do about a page fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    /// The fault was resolved; execution may resume at the faulting
    /// instruction.
    Resumed,
    /// The fault could not be resolved; the kernel terminated the faulting
    /// task.
    Terminate,
}

/// A table of syscall handlers, filled by the kernel crate at boot.
pub struct SyscallTable;

impl SyscallTable {
    /// Register `handler` for syscall number `no`.
    ///
    /// Returns `false` (and does not register) when `no` is out of range
    /// (`>= 32`).
    pub fn register(no: u32, handler: SyscallHandler) -> bool {
        if no as usize >= SYSCALL_SLOTS {
            return false;
        }
        unsafe {
            *core::ptr::addr_of_mut!(SYSCALL_TABLE[no as usize]) = Some(handler);
        }
        true
    }
}

/// Register the timer-tick handler. Invoked from IRQ context on every tick.
pub fn set_timer_tick_handler(handler: fn()) {
    unsafe { TIMER_TICK_HANDLER = Some(handler); }
}

/// Register the keyboard-wake handler. Invoked from IRQ context on every
/// buffered keypress, so blocked readers can be woken.
pub fn set_keyboard_wake_handler(handler: fn()) {
    unsafe { KEYBOARD_WAKE_HANDLER = Some(handler); }
}

/// Register the mouse-wake handler. Invoked from IRQ context on every
/// buffered mouse event, so blocked readers can be woken.
pub fn set_mouse_wake_handler(handler: fn()) {
    unsafe { MOUSE_WAKE_HANDLER = Some(handler); }
}

/// Register the page-fault handler. Invoked from exception context.
pub fn set_page_fault_handler(handler: fn(usize, u32) -> FaultAction) {
    unsafe { PAGE_FAULT_HANDLER = Some(handler); }
}

/// Result of consulting the syscall table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyscallDispatch {
    /// A handler was registered and returned `u32`.
    Handled(u32),
    /// No handler is registered for that number.
    Unhandled,
}

/// Invoke the registered syscall handler for `no`, if any.
pub(crate) fn dispatch_syscall(
    no: u32,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    a4: u32,
) -> SyscallDispatch {
    if no as usize >= SYSCALL_SLOTS {
        return SyscallDispatch::Unhandled;
    }
    let slot: Option<SyscallHandler> =
        unsafe { *core::ptr::addr_of!(SYSCALL_TABLE[no as usize]) };
    match slot {
        Some(handler) => SyscallDispatch::Handled(handler(a0, a1, a2, a3, a4)),
        None => SyscallDispatch::Unhandled,
    }
}

/// Invoke the registered timer-tick handler, if any.
pub(crate) fn invoke_timer_tick() {
    if let Some(handler) = unsafe { TIMER_TICK_HANDLER } {
        handler();
    }
}

/// Invoke the registered keyboard-wake handler, if any.
pub(crate) fn invoke_keyboard_wake() {
    if let Some(handler) = unsafe { KEYBOARD_WAKE_HANDLER } {
        handler();
    }
}

/// Invoke the registered mouse-wake handler, if any.
pub(crate) fn invoke_mouse_wake() {
    if let Some(handler) = unsafe { MOUSE_WAKE_HANDLER } {
        handler();
    }
}

/// Invoke the registered page-fault handler, if any. Returns the handler's
/// action, or [`FaultAction::Terminate`] when none is registered.
pub(crate) fn invoke_page_fault(addr: usize, err_code: u32) -> FaultAction {
    match unsafe { PAGE_FAULT_HANDLER } {
        Some(handler) => handler(addr, err_code),
        None => FaultAction::Terminate,
    }
}
