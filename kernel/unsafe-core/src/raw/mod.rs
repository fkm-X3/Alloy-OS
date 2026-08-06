//! Raw, unsafe-only primitives.
//!
//! - `ffi.rs` — extern "C" blocks moved verbatim from `hal/src/ffi.rs` (Phase 1).
//! - `asm.rs` — cli/sti/hlt/wfi, inb/outb, cpuid, invlpg, rdmsr/wrmsr, cr3,
//!   mrs/msr, tlbi/dsb/isb (feature-gated per arch).
//!
//! Later phases will add:
//! - `symbols.rs` — `#[no_mangle]` globals C/asm depend on + safe accessors.
//! - `compat.rs` — drop-in C-ABI shims so unported C/asm still link.

pub mod asm;
pub mod ffi;
