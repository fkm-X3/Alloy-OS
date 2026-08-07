//! `#[no_mangle]` globals that C/asm depend on, plus safe accessors.
//!
//! Scaffolding: empty. The translated C already exports the
//! globals it owns (`g_pmm`, `g_vmm`, `kernel_pml4_phys`, ...) directly via
//! `#[no_mangle]` in `ported/`, so this module exists to host any symbols the
//! *boot asm* or the surviving C need that the translation does not provide.
//!
//! The swap fills this in with whatever the link actually
//! demands; until then it stays empty.
