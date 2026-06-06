//! i686 (32-bit x86) architecture implementation
//!
//! This is the main supported architecture for Alloy OS.

use super::{Arch, CpuContext};

pub struct I686Arch;

impl Arch for I686Arch {
    const NAME: &'static str = "i686";
    const POINTER_WIDTH: u32 = 32;
    const PAGE_SIZE: u32 = 4096;

    fn init() {
        // i686 initialization
        // GDT and IDT are initialized from C++ boot code
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

        // CPUID leaf 0 returns vendor string in EBX, EDX, ECX
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
        // Family: bits [11:8] + extended family [27:20]
        let base_family = (eax >> 8) & 0xF;
        let ext_family = (eax >> 20) & 0xFF;
        let family = if base_family == 0xF {
            base_family + ext_family
        } else {
            base_family
        };

        // Model: bits [7:4] + extended model [19:16]
        let base_model = (eax >> 4) & 0xF;
        let ext_model = (eax >> 16) & 0xF;
        let model = if base_family == 0xF || base_family == 0x6 {
            (ext_model << 4) | base_model
        } else {
            base_model
        };

        // Stepping: bits [3:0]
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
        extern "C" {
            fn init_gdt();
        }
        unsafe {
            init_gdt();
        }
    }

    fn init_idt() {
        extern "C" {
            fn init_idt();
        }
        unsafe {
            init_idt();
        }
    }

    fn get_fault_address() -> usize {
        let cr2: u32;
        unsafe {
            core::arch::asm!("mov {0:e}, cr2", out(reg) cr2);
        }
        cr2 as usize
    }

    unsafe fn invalidate_tlb_entry(virt_addr: usize) {
        core::arch::asm!("invlpg [{0:e}]", in(reg) virt_addr as u32);
    }

    unsafe fn switch_page_directory(pd_phys: usize) {
        core::arch::asm!("mov cr3, {0:e}", in(reg) pd_phys as u32);
    }
}

/// i686-specific page table structures
pub mod paging {
    /// Page directory entry flags
    pub const PTE_PRESENT: u32 = 1 << 0;
    pub const PTE_WRITABLE: u32 = 1 << 1;
    pub const PTE_USER: u32 = 1 << 2;
    pub const PTE_WRITE_THROUGH: u32 = 1 << 3;
    pub const PTE_CACHE_DISABLE: u32 = 1 << 4;
    pub const PTE_ACCESSED: u32 = 1 << 5;
    pub const PTE_DIRTY: u32 = 1 << 6;
    pub const PTE_PS: u32 = 1 << 7;
    pub const PTE_GLOBAL: u32 = 1 << 8;

    /// Page directory entry (32-bit)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageDirectoryEntry {
        pub entries: [u32; 1024],
    }

    /// Page table entry (32-bit)
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableEntry {
        pub entries: [u32; 1024],
    }

    impl PageDirectoryEntry {
        pub const fn new() -> Self {
            Self {
                entries: [0; 1024],
            }
        }
    }

    impl PageTableEntry {
        pub const fn new() -> Self {
            Self {
                entries: [0; 1024],
            }
        }
    }
}

/// i686 segment selectors
pub mod segments {
    pub const KERNEL_CODE: u16 = 0x08;
    pub const KERNEL_DATA: u16 = 0x10;
    pub const USER_CODE: u16 = 0x18;
    pub const USER_DATA: u16 = 0x20;
}

use crate::interrupt::{InterruptController, Pic8259};
use crate::memory::{MemoryManager, Pmm};
use crate::serial::{SerialPort, Uart16550};
use crate::time::{Timer, Pit};
use crate::HalPlatform;

/// i686 HAL platform implementation using C-backed memory manager
pub struct I686Platform;

impl HalPlatform for I686Platform {
    type Arch = I686Arch;
    type InterruptCtrl = Pic8259;
    type MemManager = Pmm;
    type Serial = Uart16550;
    type Timer = Pit;

    fn init_early() -> Self::Serial {
        let mut serial = Uart16550::new();
        serial.init(0x3F8, 115200);
        serial
    }

    fn init_interrupts() -> Self::InterruptCtrl {
        let mut pic = Pic8259::new(0x20, 0x28);
        pic.init();
        pic
    }

    fn init_memory() -> Self::MemManager {
        let mut pmm = Pmm::new();
        pmm.init(0);
        pmm
    }

    fn init_timer(frequency: u32) -> Self::Timer {
        let mut pit = Pit::new();
        pit.init(frequency);
        pit
    }
}
