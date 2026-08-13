//! C2Rust-translated kernel C, merged under `#[cfg(target_arch)]`.
//!
//! Merge rule:
//! - files byte-identical in both transpile outputs are kept once in
//!   `common/` (`pmm`),
//! - files that differ only where the C had arch `#if`s become a single file
//!   with `#[cfg(target_arch = "x86_64")]` / `#[cfg(target_arch = "aarch64")]`
//!   on the divergent items (`vmm`, `syscall`, `serial`),
//! - truly arch-specific C becomes `#[cfg]`-gated module pairs pointing into
//!   `x86_64/` and `aarch64/` (`paging`, `cpu`, `timer`, gdt/idt, `boot::main`,
//!   plus the x86-only / aarch64-only driver modules).
//!
//! Everything here is machine output; no idiomatic rewrites.

pub mod arch {
    pub mod syscall {
        include!("common/arch_syscall.rs");
    }

    #[cfg(target_arch = "x86_64")]
    pub mod cpu {
        include!("x86_64/arch/cpu.rs");
    }
    #[cfg(target_arch = "aarch64")]
    pub mod cpu {
        include!("aarch64/arch/cpu.rs");
    }

    #[cfg(target_arch = "x86_64")]
    pub mod x86_64 {
        pub mod gdt {
            include!("x86_64/arch/x86_64/gdt.rs");
        }
        pub mod idt {
            include!("x86_64/arch/x86_64/idt.rs");
        }
    }
    #[cfg(target_arch = "aarch64")]
    pub mod aarch64 {
        pub mod gdt {
            include!("aarch64/arch/aarch64/gdt.rs");
        }
        pub mod idt {
            include!("aarch64/arch/aarch64/idt.rs");
        }
    }
}

pub mod drivers {
    #[cfg(target_arch = "x86_64")]
    pub mod vesa {
        include!("x86_64/drivers/vesa.rs");
    }
    #[cfg(target_arch = "x86_64")]
    pub mod keyboard {
        include!("x86_64/drivers/keyboard.rs");
    }
    #[cfg(target_arch = "x86_64")]
    pub mod mouse {
        include!("x86_64/drivers/mouse.rs");
    }
    #[cfg(target_arch = "x86_64")]
    pub mod pci {
        include!("x86_64/drivers/pci.rs");
    }
    #[cfg(target_arch = "x86_64")]
    pub mod ata {
        include!("x86_64/drivers/ata.rs");
    }
    #[cfg(target_arch = "x86_64")]
    pub mod ahci {
        include!("x86_64/drivers/ahci.rs");
    }
    #[cfg(target_arch = "x86_64")]
    pub mod initrd {
        include!("x86_64/drivers/initrd.rs");
    }
}

// The C2Rust PMM/VMM/paging modules are replaced by the hand-written
// `crate::mem::{pmm,vmm,paging,paging_aarch64}` and deleted
// from the tree. The boot mains and `raw::ffi` still resolve the same
// `#[no_mangle]` symbols (`g_pmm`, `g_vmm`, `g_paging`, `pmm_init`,
// `vmm_init`, `paging_init`, ...) against the new modules.

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
