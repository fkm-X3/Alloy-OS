//! ARM64 (aarch64) architecture implementation
//!
//! Basic working support for ARM64 architecture.
//! Suitable for QEMU virt machine emulation.
//! Session 3.6: GDT/IDT/syscall/exception handlers moved here from
//! the ported C2Rust modules.

use crate::raw::asm::aarch64 as sysregs;

use super::{Arch, CpuContext};

// ============================================================================
// Arch trait implementation
// ============================================================================

pub struct Aarch64Arch;

impl Arch for Aarch64Arch {
    const NAME: &'static str = "aarch64";
    const POINTER_WIDTH: u32 = 64;
    const PAGE_SIZE: u32 = 4096;

    fn init() {
        let mair = 0xFFu64 | (0x04u64 << 8);
        sysregs::write_mair_el1(mair);

        let tcr = 16u64
            | (1u64 << 8)
            | (1u64 << 10)
            | (3u64 << 12)
            | (0u64 << 14)
            | (0u64 << 32);
        sysregs::write_tcr_el1(tcr);

        let sctlr = sysregs::read_sctlr_el1();
        sysregs::write_sctlr_el1(sctlr | 1 | (1 << 2) | (1 << 12));

        sysregs::dsb_sy();
        sysregs::isb();
    }

    fn halt() {
        sysregs::wfi();
    }

    fn disable_interrupts() {
        sysregs::daifset();
    }

    fn enable_interrupts() {
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
        super::aarch64::gdt_init();
    }

    fn init_idt() {
        super::aarch64::idt_init();
    }

    fn get_fault_address() -> usize {
        sysregs::read_far_el1() as usize
    }

    unsafe fn invalidate_tlb_entry(virt_addr: usize) {
        sysregs::tlbi_vae1(virt_addr as u64);
    }

    unsafe fn switch_page_directory(pd_phys: usize) {
        sysregs::write_ttbr0_el1(pd_phys as u64);
        sysregs::tlbi_vmalle1();
    }
}

// ============================================================================
// Safe boot-init wrappers
// ============================================================================

/// ARM64 has no GDT — this is a no-op.
pub fn gdt_init() {}

/// Initialize VBAR_EL1 (exception vector table) and enable IRQs.
/// Called from `kernel_main`.
pub fn idt_init() {
    unsafe {
        extern "C" {
            static _exception_vectors: u8;
        }
        let vbar = &_exception_vectors as *const u8 as u64;
        sysregs::write_vbar_el1(vbar);
        sysregs::isb();
        sysregs::daifclr();
    }
}

/// ARM64 SVC interface — no MSR setup needed (uses SVC instruction).
/// Called from `kernel_main`.
pub fn syscall_init() {
    crate::drivers::serial::Serial::write_str("[Syscall] ARM64 SVC interface ready\n");
}

// ============================================================================
// Exception handlers (called from exception_vectors.S)
// ============================================================================

/// EL1 synchronous exception handler — infinite loop for unexpected exceptions.
#[no_mangle]
pub unsafe extern "C" fn exception_handler_el1() {
    loop {
        sysregs::wfi();
    }
}

/// EL1 IRQ handler — delegates to the timer driver.
#[no_mangle]
pub unsafe extern "C" fn irq_handler_el1() {
    crate::drivers::timer::timer_handler();
}

/// EL1 page fault handler — invoked from exception_vectors.S.
#[no_mangle]
pub unsafe extern "C" fn page_fault_handler(far: u64, esr: u64) {
    let err_code = (esr & 0xFF) as u32;
    let _ = crate::api::callback::invoke_page_fault(far as usize, err_code);
}

/// EL0 SVC handler — dispatches to the syscall dispatcher via callback.
#[no_mangle]
pub unsafe extern "C" fn svc_handler(
    num: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64,
) -> u32 {
    extern "C" {
        #[link_name = "syscall_dispatcher"]
        fn syscall_dispatcher_0(
            syscall_no: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32, arg4: u32,
            frame: *mut u32,
        ) -> u32;
    }
    syscall_dispatcher_0(
        num as u32, arg0 as u32, arg1 as u32, arg2 as u32, arg3 as u32, arg4 as u32,
        core::ptr::null_mut::<u32>(),
    )
}

/// System uptime in milliseconds — called from asm context-switch diagnostics.
#[no_mangle]
pub unsafe extern "C" fn get_system_uptime_ms() -> u64 {
    crate::drivers::timer::timer_get_uptime_ms_ffi()
}

// ============================================================================
// CpuContext implementation
// ============================================================================

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

// ============================================================================
// Translation table types and constants
// ============================================================================

pub mod paging {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct TranslationTableEntry {
        pub entries: [u64; 512],
    }

    pub const DESC_VALID: u64 = 1 << 0;
    pub const DESC_BLOCK: u64 = 0;
    pub const DESC_TABLE: u64 = 3;
    pub const DESC_PAGE: u64 = 3;

    pub const ATTR_INDEX_NORMAL: u64 = 0 << 2;
    pub const ATTR_INDEX_DEVICE: u64 = 1 << 2;

    pub const AP_RW_ALL: u64 = 0 << 6;
    pub const AP_RW_ALL_PL0_RO: u64 = 1 << 6;
    pub const AP_RO_ALL: u64 = 2 << 6;
    pub const AP_RO_ALL_PL0_RO: u64 = 3 << 6;

    pub const SH_OUTER: u64 = 2 << 8;
    pub const SH_INNER: u64 = 3 << 8;

    pub const UXN: u64 = 1 << 54;
    pub const PXN: u64 = 1 << 53;

    impl TranslationTableEntry {
        pub const fn new() -> Self {
            Self { entries: [0; 512] }
        }

        pub fn set_block(&mut self, index: usize, phys_base: u64, attr: u64) {
            self.entries[index] = DESC_VALID | DESC_BLOCK
                | (phys_base & 0x0000_FFFF_FFFF_F000)
                | attr | ATTR_INDEX_NORMAL | AP_RW_ALL | SH_INNER;
        }

        pub fn set_table(&mut self, index: usize, next_table_phys: u64) {
            self.entries[index] = DESC_VALID | DESC_TABLE
                | (next_table_phys & 0x0000_FFFF_FFFF_F000);
        }

        pub fn set_page(&mut self, index: usize, phys_base: u64, attr: u64) {
            self.entries[index] = DESC_VALID | DESC_PAGE
                | (phys_base & 0x0000_FFFF_FFFF_F000)
                | attr | ATTR_INDEX_NORMAL | AP_RW_ALL | SH_INNER;
        }

        pub fn is_valid(&self, index: usize) -> bool {
            self.entries[index] & DESC_VALID != 0
        }
    }
}

pub mod exceptions {
    pub const SYNC_FROM_EL0: u64 = 0x000;
    pub const IRQ_FROM_EL0: u64 = 0x080;
    pub const FIQ_FROM_EL0: u64 = 0x100;
    pub const SERR_FROM_EL0: u64 = 0x180;
    pub const SYNC_FROM_EL1: u64 = 0x200;
    pub const IRQ_FROM_EL1: u64 = 0x280;
    pub const FIQ_FROM_EL1: u64 = 0x300;
    pub const SERR_FROM_EL1: u64 = 0x380;

    pub const ESR_EC_UNKNOWN: u32 = 0x00;
    pub const ESR_EC_WFI_WFE: u32 = 0x01;
    pub const ESR_EC_SVC64: u32 = 0x15;
    pub const ESR_EC_SMC64: u32 = 0x16;
    pub const ESR_EC_DATA_ABORT_LOWER: u32 = 0x24;
    pub const ESR_EC_DATA_ABORT_CURR: u32 = 0x25;
    pub const ESR_EC_INST_ABORT_LOWER: u32 = 0x20;
    pub const ESR_EC_INST_ABORT_CURR: u32 = 0x21;
}

pub mod gic {
    pub const GICD_BASE: u64 = 0x0800_0000;
    pub const GICC_BASE: u64 = 0x0801_0000;
}
