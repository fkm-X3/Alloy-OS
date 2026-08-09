//! ARM64 (aarch64) architecture implementation (minimal working)
//!
//! Basic working support for ARM64 architecture.
//! Suitable for QEMU virt machine emulation.
//! Moved from `hal/src/arch/aarch64/mod.rs`.

use crate::raw::asm::aarch64 as sysregs;

use super::{Arch, CpuContext};

pub struct Aarch64Arch;

impl Arch for Aarch64Arch {
    const NAME: &'static str = "aarch64";
    const POINTER_WIDTH: u32 = 64;
    const PAGE_SIZE: u32 = 4096;

    fn init() {
        // Basic aarch64 initialization
        // Set up MAIR for memory attributes
        // MAIR_EL1: Normal cacheable (index 0), Device-nGnRE (index 1)
        let mair = (0xFFu64) | ((0x04u64) << 8);
        sysregs::write_mair_el1(mair);

        // TCR_EL1: 4KB granule, 48-bit VA, 48-bit PA
        // T0SZ = 64 - 48 = 16, IRGN0 = 1 (Normal WB RA WA), ORGN0 = 1, SH0 = 3 (Inner shareable)
        // IPS = 0 (32-bit PA for simplicity), TG0 = 0 (4KB), T0SZ = 16
        let tcr = (16u64)          // T0SZ = 16 (48-bit VA)
            | (1u64 << 8)          // IRGN0 = Normal WB RA WA
            | (1u64 << 10)         // ORGN0 = Normal WB RA WA
            | (3u64 << 12)         // SH0 = Inner shareable
            | (0u64 << 14)         // TG0 = 4KB
            | (0u64 << 32);        // IPS = 32-bit PA
        sysregs::write_tcr_el1(tcr);

        // Enable MMU: M=1, C=1, I=1
        let sctlr = sysregs::read_sctlr_el1();
        sysregs::write_sctlr_el1(sctlr | 1 | (1 << 2) | (1 << 12));

        sysregs::dsb_sy();
        sysregs::isb();
    }

    fn halt() {
        sysregs::wfi();
    }

    fn disable_interrupts() {
        // Disable IRQ and FIQ (DAIF register)
        sysregs::daifset();
    }

    fn enable_interrupts() {
        // Enable IRQ and FIQ (DAIF register)
        sysregs::daifclr();
    }

    fn get_vendor(buffer: &mut [u8]) {
        let midr = sysregs::read_midr_el1();
        let implementer = (midr >> 16) & 0xFF;
        let vendor_str: &[u8] = match implementer {
            0x41 => b"ARM Limited",
            0x42 => b"Broadcom  ",
            0x43 => b"Cavium    ",
            0x44 => b"DEC       ",
            0x4E => b"NVIDIA    ",
            0x51 => b"Qualcomm  ",
            0x53 => b"Samsung   ",
            0xC0 => b"Ampere    ",
            _ => b"Unknown   ",
        };
        let len = core::cmp::min(vendor_str.len(), buffer.len());
        buffer[..len].copy_from_slice(&vendor_str[..len]);
    }

    fn get_features() -> u32 {
        // Read ID_AA64ISAR0_EL1 for feature info
        sysregs::read_id_aa64isar0_el1() as u32
    }

    fn get_model_info() -> (u32, u32, u32) {
        let midr = sysregs::read_midr_el1();
        let variant = ((midr >> 20) & 0xF) as u32;
        let partnum = ((midr >> 4) & 0xFFF) as u32;
        let revision = (midr & 0xF) as u32;
        (variant, partnum, revision)
    }

    unsafe fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext) {
        extern "C" {
            fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext);
        }
        context_switch(old_ctx, new_ctx);
    }

    fn init_gdt() {
        // ARM64 doesn't use GDT - uses translation tables instead
        // This is a no-op for ARM
    }

    fn init_idt() {
        // ARM64 uses VBAR_EL1 (Vector Base Address Register)
        // Exception vectors must be 2KB aligned
        extern "C" {
            static _exception_vectors: u8;
        }
        let vbar = unsafe { &_exception_vectors as *const u8 as u64 };
        sysregs::write_vbar_el1(vbar);
    }

    fn get_fault_address() -> usize {
        sysregs::read_far_el1() as usize
    }

    unsafe fn invalidate_tlb_entry(virt_addr: usize) {
        sysregs::tlbi_vae1(virt_addr as u64);
    }

    unsafe fn switch_page_directory(pd_phys: usize) {
        // PD must be 4KB aligned, lower bits are attributes
        sysregs::write_ttbr0_el1(pd_phys as u64);
        sysregs::tlbi_vmalle1();
    }
}

/// ARM64 translation table structures
pub mod paging {
    /// 4KB page, 4-level translation (L0-L3)
    /// L0: 512 entries, maps 512GB regions
    /// L1: 512 entries, maps 1GB regions
    /// L2: 512 entries, maps 2MB blocks
    /// L3: 512 entries, maps 4KB pages

    /// Translation table entry (64-bit descriptor)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct TranslationTableEntry {
        pub entries: [u64; 512],
    }

    /// Descriptor type bits
    pub const DESC_VALID: u64 = 1 << 0;
    pub const DESC_BLOCK: u64 = 0;     // Block entry (L0-L2)
    pub const DESC_TABLE: u64 = 3;     // Table entry (L0-L2) or page (L3)
    pub const DESC_PAGE: u64 = 3;      // Page entry (L3)

    /// Attribute index (MAIR)
    pub const ATTR_INDEX_NORMAL: u64 = 0 << 2;
    pub const ATTR_INDEX_DEVICE: u64 = 1 << 2;

    /// Access permissions (AP)
    pub const AP_RW_ALL: u64 = 0 << 6;    // PL1 RW, PL0 none
    pub const AP_RW_ALL_PL0_RO: u64 = 1 << 6; // PL1 RW, PL0 RO
    pub const AP_RO_ALL: u64 = 2 << 6;    // PL1 RO, PL0 none
    pub const AP_RO_ALL_PL0_RO: u64 = 3 << 6; // PL1 RO, PL0 RO

    /// Shareability
    pub const SH_OUTER: u64 = 2 << 8;
    pub const SH_INNER: u64 = 3 << 8;

    /// Execute never
    pub const UXN: u64 = 1 << 54;
    pub const PXN: u64 = 1 << 53;

    impl TranslationTableEntry {
        pub const fn new() -> Self {
            Self {
                entries: [0; 512],
            }
        }

        /// Create a block entry (maps large region)
        pub fn set_block(&mut self, index: usize, phys_base: u64, attr: u64) {
            self.entries[index] = DESC_VALID | DESC_BLOCK | (phys_base & 0x0000_FFFF_FFFF_F000)
                | attr | ATTR_INDEX_NORMAL | AP_RW_ALL | SH_INNER;
        }

        /// Create a table entry (points to next level)
        pub fn set_table(&mut self, index: usize, next_table_phys: u64) {
            self.entries[index] = DESC_VALID | DESC_TABLE | (next_table_phys & 0x0000_FFFF_FFFF_F000);
        }

        /// Create a page entry (4KB page at L3)
        pub fn set_page(&mut self, index: usize, phys_base: u64, attr: u64) {
            self.entries[index] = DESC_VALID | DESC_PAGE | (phys_base & 0x0000_FFFF_FFFF_F000)
                | attr | ATTR_INDEX_NORMAL | AP_RW_ALL | SH_INNER;
        }

        /// Check if entry is valid
        pub fn is_valid(&self, index: usize) -> bool {
            self.entries[index] & DESC_VALID != 0
        }
    }
}

/// ARM64 exception types
pub mod exceptions {
    pub const SYNC_FROM_EL0: u64 = 0x000;
    pub const IRQ_FROM_EL0: u64 = 0x080;
    pub const FIQ_FROM_EL0: u64 = 0x100;
    pub const SERR_FROM_EL0: u64 = 0x180;
    pub const SYNC_FROM_EL1: u64 = 0x200;
    pub const IRQ_FROM_EL1: u64 = 0x280;
    pub const FIQ_FROM_EL1: u64 = 0x300;
    pub const SERR_FROM_EL1: u64 = 0x380;

    /// Exception syndrome register (ESR_EL1) classes
    pub const ESR_EC_UNKNOWN: u32 = 0x00;
    pub const ESR_EC_WFI_WFE: u32 = 0x01;
    pub const ESR_EC_SVC64: u32 = 0x15;  // SVC instruction in AArch64
    pub const ESR_EC_SMC64: u32 = 0x16;
    pub const ESR_EC_DATA_ABORT_LOWER: u32 = 0x24;
    pub const ESR_EC_DATA_ABORT_CURR: u32 = 0x25;
    pub const ESR_EC_INST_ABORT_LOWER: u32 = 0x20;
    pub const ESR_EC_INST_ABORT_CURR: u32 = 0x21;
}

/// GIC (Generic Interrupt Controller) base addresses for QEMU virt
pub mod gic {
    /// GICv2 distributor base (QEMU virt machine)
    pub const GICD_BASE: u64 = 0x0800_0000;
    /// GICv2 CPU interface base (QEMU virt machine)
    pub const GICC_BASE: u64 = 0x0801_0000;
}

impl super::CpuContext {
    /// Create a new CPU context with sensible defaults.
    pub fn new() -> Self {
        Self {
            x19: 0, x20: 0, x21: 0, x22: 0,
            x23: 0, x24: 0, x25: 0, x26: 0,
            x27: 0, x28: 0, fp: 0, lr: 0,
            sp: 0, elr: 0, spsr: 0, ttbr0: 0, sp0: 0,
        }
    }
}

impl Default for super::CpuContext {
    fn default() -> Self {
        Self::new()
    }
}
