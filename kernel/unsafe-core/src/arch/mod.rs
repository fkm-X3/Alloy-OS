//! Architecture-specific implementations
//!
//! Moved from `hal/src/arch/mod.rs`; the HAL re-exports this
//! module.
//!
//! This module provides the **safe arch API** that the kernel and boot
//! sequence use.  The implementations live in the arch-specific submodules
//! and call into `crate::raw::asm` / `crate::raw::ffi` for the actual
//! hardware operations.
//!
//! Session 3.6: GDT/IDT/syscall/context-switch moved here from the
//! ported C2Rust modules.  Exception/IRQ handlers remain `#[no_mangle]`
//! for the asm stubs but are now authored (not translated) Rust.

#[cfg(feature = "x86_64")]
pub mod x86_64;

#[cfg(feature = "aarch64")]
pub mod aarch64;

/// Boot entry points (`kernel_main`) called from asm. Relocated here from
/// the deleted `ported` module so the asm stubs can still find them.
pub mod boot {
    #[cfg(target_arch = "x86_64")]
    pub mod main {
        include!("boot/x86_64/main.rs");
    }
    #[cfg(target_arch = "aarch64")]
    pub mod main {
        include!("boot/aarch64/main_aarch64.rs");
    }
}

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

/// Safe context switch. Takes `&mut` references guaranteeing exclusive
/// ownership — no aliasing is possible. The asm `context_switch` reads
/// `old` and writes both `old` (saves) and `new` (restores).
pub fn context_switch(old: &mut CpuContext, new: &mut CpuContext) {
    unsafe {
        crate::raw::ffi::context_switch(old as *mut CpuContext, new as *mut CpuContext);
    }
}

/// Save the current CPU context into `ctx`. Returns normally — the
/// caller continues from this point when the task is later restored.
pub fn save_context(ctx: &mut CpuContext) {
    unsafe {
        crate::raw::ffi::save_context(ctx as *mut CpuContext);
    }
}

/// Restore `ctx` and never return. Jumps to the saved RIP/LR (or
/// erets to ELR for a fresh task). Declared `-> !` so the compiler
/// emits any guard drops BEFORE the switch.
///
/// # Safety contract
/// `ctx` must point to a valid `CpuContext` that was previously saved
/// via [`save_context`] and resides in memory that will outlive this
/// call (typically a static or leaked allocation). The scheduler's
/// static `current_task` satisfies this.
pub fn load_context(ctx: *mut CpuContext) -> ! {
    unsafe {
        crate::raw::ffi::load_context(ctx);
    }
}

/// Halt the CPU until the next interrupt (safe wrapper).
#[cfg(feature = "x86_64")]
pub fn cpu_halt() {
    unsafe { core::arch::asm!("hlt") };
}

/// Halt the CPU until the next interrupt (safe wrapper).
#[cfg(feature = "aarch64")]
pub fn cpu_halt() {
    unsafe { core::arch::asm!("wfi") };
}

/// Enable interrupts and halt (safe wrapper, x86 idle pattern).
#[cfg(feature = "x86_64")]
pub fn cpu_sti_hlt() {
    unsafe { core::arch::asm!("sti; hlt", options(nomem, nostack)) };
}

/// Register snapshot for panic diagnostics.
#[derive(Debug, Clone, Copy, Default)]
pub struct PanicRegs {
    pub rsp: u64,
    pub rbp: u64,
    pub rflags: u64,
}

/// Capture key registers for panic diagnostics (x86_64).
#[cfg(feature = "x86_64")]
pub fn capture_panic_regs() -> PanicRegs {
    let rsp: u64;
    let rbp: u64;
    let rflags: u64;
    unsafe {
        core::arch::asm!(
            "mov {0:r}, rsp",
            "mov {1:r}, rbp",
            out(reg) rsp,
            out(reg) rbp,
        );
        core::arch::asm!(
            "pushfq",
            "pop {0:r}",
            out(reg) rflags,
        );
    }
    PanicRegs { rsp, rbp, rflags }
}

/// Syscall number constants — shared across architectures.
pub mod syscall_no {
    pub const SYS_EXIT: u32 = 0;
    pub const SYS_YIELD: u32 = 1;
    pub const SYS_GETPID: u32 = 2;
    pub const SYS_SLEEP: u32 = 3;
    pub const SYS_OPEN: u32 = 4;
    pub const SYS_READ: u32 = 5;
    pub const SYS_WRITE: u32 = 6;
    pub const SYS_CLOSE: u32 = 7;
    pub const SYS_DUP: u32 = 8;
    pub const SYS_LSEEK: u32 = 9;
    pub const SYS_PIPE: u32 = 10;
    pub const SYS_EXECVE: u32 = 11;
    pub const SYS_SOCKET: u32 = 12;
    pub const SYS_BIND: u32 = 13;
    pub const SYS_LISTEN: u32 = 14;
    pub const SYS_ACCEPT: u32 = 15;
    pub const SYS_CONNECT: u32 = 16;
    pub const SYS_CLOSE_SOCKET: u32 = 17;
    pub const SYS_HAS_PENDING_CONNECTIONS: u32 = 18;
    pub const SYS_FORK: u32 = 20;
    pub const SYS_CLONE: u32 = 21;
    pub const SYS_WAITPID: u32 = 22;
    pub const SYS_SOCKET_READ: u32 = 23;
    pub const SYS_SOCKET_WRITE: u32 = 24;
    pub const SYS_DUP2: u32 = 29;
    pub const SYS_KILL: u32 = 30;
}

/// Central syscall dispatcher — called from `syscall_entry` (asm) and
/// `svc_handler` (aarch64). Delegates to the kernel crate's registered
/// handlers via the callback API.
#[no_mangle]
pub unsafe extern "C" fn syscall_dispatcher(
    syscall_no: u32,
    arg0: u32,
    arg1: u32,
    arg2: u32,
    arg3: u32,
    arg4: u32,
) -> u32 {
    match crate::api::callback::dispatch_syscall(syscall_no, arg0, arg1, arg2, arg3, arg4) {
        crate::api::callback::SyscallDispatch::Handled(result) => result,
        crate::api::callback::SyscallDispatch::Unhandled => {
            crate::drivers::serial::Serial::write_str(
                "[Syscall] Unknown syscall number: 0x",
            );
            // serial_print_hex is still extern "C" from ported — but we can
            // just format here instead.  For now keep the raw symbol.
            extern "C" { fn serial_print_hex(v: u32); }
            serial_print_hex(syscall_no);
            crate::drivers::serial::Serial::write_str("\n");
            u32::MAX
        }
    }
}

/// Safe x86_64 userspace syscall invocation (`int 0x80`).
///
/// The asm is encapsulated here so the kernel crate can trigger syscalls
/// from safe code. The register layout matches the `syscall_entry` asm stub.
#[cfg(feature = "x86_64")]
pub fn syscall_invoke(num: u32, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "push rbx",
            "mov ebx, {0:e}",
            "int 0x80",
            "pop rbx",
            in(reg) arg0,
            inlateout("eax") num => result,
            in("ecx") arg1,
            in("edx") arg2,
            lateout("ecx") _,
            lateout("edx") _,
            options(preserves_flags),
        );
    }
    result
}

/// CPU context for task switching - architecture-specific
/// Only one architecture feature should be enabled at a time

#[cfg(not(feature = "aarch64"))]
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
    pub fs_base: u64,
}

#[cfg(feature = "aarch64")]
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
    pub sp0: u64,
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
