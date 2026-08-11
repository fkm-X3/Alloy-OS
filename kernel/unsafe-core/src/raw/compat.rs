//! Drop-in C-ABI compat shims.
//!
//! the swap: the C2Rust output in [`crate::ported`] already
//! emits every externally-referenced symbol under its original C name —
//! functions (`kernel_main`, `init_serial`, `serial_print`, `syscall_dispatcher`,
//! `exception_handler`, `irq_handler`, ...) and globals (`g_pmm`, `g_vmm`,
//! `g_paging`, `kernel_pml4_phys`, `g_current_user_cr3`, `g_saved_user_cr3`,
//! `g_kernel_gs_base`, `g_isr_diag_*`) all carry `#[no_mangle]`. The boot asm,
//! the arch asm, and the safe kernel's `raw::ffi` externs therefore keep
//! resolving against the translated code exactly as they did against the C.
//!
//! The symbols the *asm* owns (`kernel_stack_top`, `kernel_stack_bottom`,
//! `kernel_stack_top_alias`, `g_ctx_switch_diag`, `gdt_flush`, `context_switch`,
//! `save_context`, `load_context`, `start`, ...) and the linker script owns
//! (`_kernel_start`, `_kernel_end`) are provided by those objects unchanged.
//!
//! This module stays as the single place to add any `#[no_mangle]` wrapper a
//! future outside consumer needs that the translation does not emit. Nothing
//! is needed today.
