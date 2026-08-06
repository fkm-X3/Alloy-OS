//! x86_64 architecture implementation
//!
//! Full 64-bit x86 architecture support.
//! Moved from `hal/src/arch/x86_64/mod.rs` in Phase 1.

use crate::raw::asm::x86_64;

use super::{Arch, CpuContext};

pub struct X86_64Arch;

impl Arch for X86_64Arch {
    const NAME: &'static str = "x86_64";
    const POINTER_WIDTH: u32 = 64;
    const PAGE_SIZE: u32 = 4096;

    fn init() {
        // Architecture init is handled by the C boot code (boot_x86_64.asm)
        // which sets up long mode paging, GDT, IDT
    }

    fn halt() {
        x86_64::halt();
    }

    fn disable_interrupts() {
        x86_64::cli();
    }

    fn enable_interrupts() {
        x86_64::sti();
    }

    fn get_vendor(buffer: &mut [u8]) {
        let vendor_bytes: [u8; 12] = x86_64::cpuid_vendor();

        let len = core::cmp::min(vendor_bytes.len(), buffer.len());
        buffer[..len].copy_from_slice(&vendor_bytes[..len]);
    }

    fn get_features() -> u32 {
        x86_64::cpuid_features()
    }

    fn get_model_info() -> (u32, u32, u32) {
        let eax = x86_64::cpuid_model_info();

        let base_family = (eax >> 8) & 0xF;
        let ext_family = (eax >> 20) & 0xFF;
        let family = if base_family == 0xF { base_family + ext_family } else { base_family };

        let base_model = (eax >> 4) & 0xF;
        let ext_model = (eax >> 16) & 0xF;
        let model = if base_family == 0xF || base_family == 0x6 {
            (ext_model << 4) | base_model
        } else {
            base_model
        };

        let stepping = eax & 0xF;
        (family, model, stepping)
    }

    unsafe fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext) {
        extern "C" {
            fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext);
        }
        context_switch(old_ctx, new_ctx);
    }

    fn init_gdt() {
        // Handled by C kernel code (arch/x86_64/gdt.c)
    }

    fn init_idt() {
        // Handled by C kernel code (arch/x86_64/idt.c)
    }

    fn get_fault_address() -> usize {
        x86_64::read_cr2() as usize
    }

    unsafe fn invalidate_tlb_entry(virt_addr: usize) {
        x86_64::invlpg(virt_addr);
    }

    unsafe fn switch_page_directory(pd_phys: usize) {
        x86_64::write_cr3(pd_phys as u64);
    }
}

/// x86_64-specific page table structures
pub mod paging {
    /// Page map level 4 entry
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Pml4Entry {
        pub entries: [u64; 512],
    }

    /// Page directory pointer table entry
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PdptEntry {
        pub entries: [u64; 512],
    }

    /// Page directory entry (64-bit)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageDirectoryEntry {
        pub entries: [u64; 512],
    }

    /// Page table entry (64-bit)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableEntry {
        pub entries: [u64; 512],
    }

    /// PTE flags for x86_64
    pub const PTE_PRESENT: u64 = 1 << 0;
    pub const PTE_WRITABLE: u64 = 1 << 1;
    pub const PTE_USER: u64 = 1 << 2;
    pub const PTE_WRITE_THROUGH: u64 = 1 << 3;
    pub const PTE_CACHE_DISABLE: u64 = 1 << 4;
    pub const PTE_ACCESSED: u64 = 1 << 5;
    pub const PTE_DIRTY: u64 = 1 << 6;
    pub const PTE_PS: u64 = 1 << 7;
    pub const PTE_GLOBAL: u64 = 1 << 8;
    pub const PTE_NX: u64 = 1 << 63;
}

/// x86_64 segment selectors
pub mod segments {
    pub const KERNEL_CODE: u16 = 0x08;
    pub const KERNEL_DATA: u16 = 0x10;
    pub const USER_CODE: u16 = 0x20;  /* GDT[4] = user code (SYSRET CS) */
    pub const USER_DATA: u16 = 0x18;  /* GDT[3] = user data (SYSRET SS) */
    pub const TSS: u16 = 0x28;
}

impl super::CpuContext {
    /// Create a new CPU context with sensible defaults.
    pub fn new() -> Self {
        let kernel_cr3 = unsafe { crate::raw::ffi::paging_get_kernel_directory_phys() };
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0,
            cs: 0x08, ds: 0x10, es: 0x10, fs: 0x10, gs: 0x10, ss: 0x10,
            rflags: 0x202,
            cr3: kernel_cr3 as u64,
            fs_base: 0,
        }
    }

    /// Set the initial entry point, stack pointer, and argument for a task.
    pub fn set_entry(&mut self, entry: u64, stack_top: u64, arg: u64) {
        self.rip = entry;
        // context_switch does: mov rsp,[ctx.rsp]; push RIP; ret
        // which leaves RSP = ctx.rsp at function entry.  x86_64 ABI
        // requires RSP % 16 == 8 at entry (call pushes 8-byte RA).
        self.rsp = stack_top - 8;
        self.rbp = stack_top;
        self.rdi = arg;
    }
}

impl Default for super::CpuContext {
    fn default() -> Self {
        Self::new()
    }
}
