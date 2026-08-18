//! C2Rust-translated kernel C, merged under `#[cfg(target_arch)]`.
//!
//! Session 3.6: arch (gdt/idt/cpu/syscall) modules moved to
//! [`crate::arch`]. Only the boot entry points remain here — they are
//! `#[no_mangle] extern "C"` because the asm loader calls them.

pub mod boot {
    #[cfg(target_arch = "x86_64")]
    pub mod main {
        include!("x86_64/boot/main.rs");
    }
    #[cfg(target_arch = "aarch64")]
    pub mod main {
        include!("aarch64/boot/main_aarch64.rs");
    }
}
