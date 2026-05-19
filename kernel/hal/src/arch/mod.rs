//! Architecture-specific implementations

#[cfg(feature = "i686")]
pub mod i686;

#[cfg(feature = "x86_64")]
pub mod x86_64;

#[cfg(feature = "aarch64")]
pub mod aarch64;

/// Core architecture operations trait
pub trait Arch {
    /// Architecture name
    const NAME: &'static str;

    /// Pointer width in bits
    const POINTER_WIDTH: u32;

    /// Page size in bytes
    const PAGE_SIZE: u32;

    /// Initialize architecture-specific features
    fn init();

    /// Halt the CPU
    fn halt();

    /// Disable interrupts
    fn disable_interrupts();

    /// Enable interrupts
    fn enable_interrupts();

    /// Get CPU vendor string
    fn get_vendor(buffer: &mut [u8]);

    /// Get CPU features bitmask
    fn get_features() -> u32;

    /// Get CPU family, model, stepping
    fn get_model_info() -> (u32, u32, u32);

    /// Context switch between two CPU contexts
    unsafe fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext);

    /// Initialize GDT (or equivalent)
    fn init_gdt();

    /// Initialize IDT (or equivalent interrupt table)
    fn init_idt();

    /// Get the fault address (CR2 on x86, FAR_EL1 on ARM)
    fn get_fault_address() -> usize;

    /// Invalidate a single TLB entry
    unsafe fn invalidate_tlb_entry(virt_addr: usize);

    /// Switch page directory / translation table base
    unsafe fn switch_page_directory(pd_phys: usize);
}

/// CPU context for task switching - architecture-specific
/// Only one architecture feature should be enabled at a time

#[cfg(all(feature = "i686", not(any(feature = "x86_64", feature = "aarch64"))))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub eip: u32,
    pub cs: u32,
    pub ds: u32,
    pub es: u32,
    pub fs: u32,
    pub gs: u32,
    pub ss: u32,
    pub eflags: u32,
    pub cr3: u32,
}

#[cfg(all(feature = "i686", not(any(feature = "x86_64", feature = "aarch64"))))]
impl Default for CpuContext {
    fn default() -> Self {
        Self {
            eax: 0, ebx: 0, ecx: 0, edx: 0,
            esi: 0, edi: 0, ebp: 0, esp: 0,
            eip: 0, cs: 0, ds: 0, es: 0,
            fs: 0, gs: 0, ss: 0, eflags: 0, cr3: 0,
        }
    }
}

#[cfg(all(feature = "x86_64", not(any(feature = "i686", feature = "aarch64"))))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub cs: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
    pub ss: u64,
    pub rflags: u64,
    pub cr3: u64,
}

#[cfg(all(feature = "x86_64", not(any(feature = "i686", feature = "aarch64"))))]
impl Default for CpuContext {
    fn default() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, cs: 0, ds: 0, es: 0,
            fs: 0, gs: 0, ss: 0, rflags: 0, cr3: 0,
        }
    }
}

#[cfg(all(feature = "aarch64", not(any(feature = "i686", feature = "x86_64"))))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub fp: u64,
    pub lr: u64,
    pub sp: u64,
    pub elr: u64,
    pub spsr: u64,
    pub ttbr0: u64,
}

#[cfg(all(feature = "aarch64", not(any(feature = "i686", feature = "x86_64"))))]
impl Default for CpuContext {
    fn default() -> Self {
        Self {
            x19: 0, x20: 0, x21: 0, x22: 0,
            x23: 0, x24: 0, x25: 0, x26: 0,
            x27: 0, x28: 0, fp: 0, lr: 0,
            sp: 0, elr: 0, spsr: 0, ttbr0: 0,
        }
    }
}

/// Common CPU info structure
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub vendor: [u8; 16],
    pub features: u32,
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
}

impl CpuInfo {
    pub fn new<A: Arch>() -> Self {
        let mut vendor = [0u8; 16];
        A::get_vendor(&mut vendor);
        let features = A::get_features();
        let (family, model, stepping) = A::get_model_info();

        Self {
            vendor,
            features,
            family,
            model,
            stepping,
        }
    }

    pub fn vendor_str(&self) -> &str {
        let len = self.vendor.iter().position(|&b| b == 0).unwrap_or(self.vendor.len());
        core::str::from_utf8(&self.vendor[..len]).unwrap_or("Unknown")
    }
}
