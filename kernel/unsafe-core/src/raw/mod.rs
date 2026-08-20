//! Raw, unsafe-only primitives.
//!
//! - `ffi.rs` — extern "C" blocks moved verbatim from `hal/src/ffi.rs`.
//! - `asm.rs` — cli/sti/hlt/wfi, inb/outb, cpuid, invlpg, rdmsr/wrmsr, cr3,
//!   mrs/msr, tlbi/dsb/isb (feature-gated per arch).
//! - `string.rs` — freestanding `memcpy`/`memset`/`size_t` shims.
//! - `symbols.rs` — `#[no_mangle]` globals C/asm depend on + safe accessors
//!   (scaffolding, empty).

pub mod asm;
pub mod ffi;
pub mod string;
pub mod symbols;
