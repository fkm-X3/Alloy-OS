//! Raw, unsafe-only primitives.
//!
//! What will populate this module:
//! - `ffi.rs` — extern "C" blocks moved verbatim from `hal/src/ffi.rs`.
//! - `asm.rs` — cli/sti/hlt/wfi, inb/outb, cpuid, invlpg, rdmsr/wrmsr, cr3,
//!   mrs/msr, tlbi/dsb/isb (feature-gated per arch).
//! - `symbols.rs` — `#[no_mangle]` globals C/asm depend on + safe accessors.
//! - `compat.rs` — drop-in C-ABI shims so unported C/asm still link.
