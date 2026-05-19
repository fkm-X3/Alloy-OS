//! x86_64 architecture implementation (placeholder)
//!
//! This is a placeholder for future x86_64 (64-bit) support.
//! The current OS primarily targets i686 (32-bit x86).

use super::{Arch, CpuContext};

pub struct X86_64Arch;

impl Arch for X86_64Arch {
    const NAME: &'static str = "x86_64";
    const POINTER_WIDTH: u32 = 64;
    const PAGE_SIZE: u32 = 4096;

    fn init() {
        // TODO: Enter long mode, set up 64-bit GDT/IDT, enable PAE
    }

    fn halt() {
        unsafe {
            core::arch::asm!("hlt");
        }
    }

    fn disable_interrupts() {
        unsafe {
            core::arch::asm!("cli");
        }
    }

    fn enable_interrupts() {
        unsafe {
            core::arch::asm!("sti");
        }
    }

    fn get_vendor(buffer: &mut [u8]) {
        let ebx: u32;
        let edx: u32;
        let ecx: u32;

        unsafe {
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "mov {0:e}, ebx",
                "pop rbx",
                out(reg) ebx,
                in("eax") 0,
                out("ecx") ecx,
                out("edx") edx,
            );
        }

        let vendor_bytes: [u8; 12] = [
            ebx as u8,
            (ebx >> 8) as u8,
            (ebx >> 16) as u8,
            (ebx >> 24) as u8,
            edx as u8,
            (edx >> 8) as u8,
            (edx >> 16) as u8,
            (edx >> 24) as u8,
            ecx as u8,
            (ecx >> 8) as u8,
            (ecx >> 16) as u8,
            (ecx >> 24) as u8,
        ];

        let len = core::cmp::min(vendor_bytes.len(), buffer.len());
        buffer[..len].copy_from_slice(&vendor_bytes[..len]);
    }

    fn get_features() -> u32 {
        let edx: u32;
        unsafe {
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "pop rbx",
                in("eax") 1,
                in("ecx") 0,
                lateout("edx") edx,
                lateout("eax") _,
                lateout("ecx") _,
            );
        }
        edx
    }

    fn get_model_info() -> (u32, u32, u32) {
        let eax: u32;
        unsafe {
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "pop rbx",
                in("eax") 1,
                in("ecx") 0,
                lateout("eax") eax,
                lateout("ecx") _,
                lateout("edx") _,
            );
        }
        let base_family = (eax >> 8) & 0xF;
        let ext_family = (eax >> 20) & 0xFF;
        let family = if base_family == 0xF {
            base_family + ext_family
        } else {
            base_family
        };

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

    unsafe fn context_switch(_old_ctx: *mut CpuContext, _new_ctx: *mut CpuContext) {
        // TODO: Implement 64-bit context switch
        // Save/restore: RAX-R15, RBP, RSP, RIP, CS/DS/ES/FS/GS/SS, RFLAGS, CR3
        // Use syscall/sysret for fast syscalls
    }

    fn init_gdt() {
        // TODO: Set up 64-bit GDT with TSS
        // Need: NULL, Kernel Code (64-bit), Kernel Data, User Code, User Data, TSS
    }

    fn init_idt() {
        // TODO: Set up 64-bit IDT
        // 16-byte entries with 64-bit offset
    }

    fn get_fault_address() -> usize {
        let cr2: u64;
        unsafe {
            core::arch::asm!("mov {}, cr2", out(reg) cr2);
        }
        cr2 as usize
    }

    unsafe fn invalidate_tlb_entry(virt_addr: usize) {
        core::arch::asm!("invlpg [{}]", in(reg) virt_addr as u64);
    }

    unsafe fn switch_page_directory(pd_phys: usize) {
        core::arch::asm!("mov cr3, {}", in(reg) pd_phys as u64);
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
    pub const PTE_NX: u64 = 1 << 63; // No-execute bit
}

/// x86_64 segment selectors
pub mod segments {
    pub const KERNEL_CODE: u16 = 0x08;
    pub const KERNEL_DATA: u16 = 0x10;
    pub const USER_CODE: u16 = 0x18;
    pub const USER_DATA: u16 = 0x20;
    pub const TSS: u16 = 0x28;
}
