//! `#[no_mangle]` globals that C/asm depend on, plus safe accessors.
//!
//! Session 1.2 (the swap): the translated `ported/` modules export every
//! global they own directly via `#[no_mangle]` (`g_pmm`, `g_vmm`, `g_paging`,
//! `kernel_pml4_phys`, `g_current_user_cr3`, `g_saved_user_cr3`,
//! `g_kernel_gs_base`, `g_isr_diag_*`, `g_timer_ticks`, ...), so no duplicate
//! definitions live here — a second `#[no_mangle]` of the same name would be
//! a *different* variable and silently diverge from the one the ported code
//! reads and writes.
//!
//! This module exists to host safe accessors for those globals once the
//! Phase 3 API boundary replaces direct raw reads/writes. Until then it stays
//! empty.
