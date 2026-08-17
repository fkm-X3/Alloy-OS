//! Inline-asm helpers, feature-gated per architecture.
//!
//! Moved from the inline `core::arch::asm!` snippets that were scattered
//! through the HAL. These are the only inline-asm shims the
//! kernel uses; everything above them (drivers, arch impls, IoPort/Mmio)
//! calls through these helpers.

// ============================================================================
// x86_64
// ============================================================================

#[cfg(feature = "x86_64")]
pub mod x86_64 {
    // --- Interrupt / CPU control ---

    /// Disable interrupts (IF = 0).
    #[inline]
    pub fn cli() {
        unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
    }

    /// Enable interrupts (IF = 1).
    #[inline]
    pub fn sti() {
        unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
    }

    /// Halt until the next interrupt.
    #[inline]
    pub fn hlt() {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }

    /// Disable interrupts and halt.
    #[inline]
    pub fn halt() {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }

    /// Save the interrupt state (RFLAGS) and disable interrupts (IF = 0).
    /// Returns the saved flags for [`restore_irq_state`].
    #[inline]
    pub fn save_irq_state() -> u64 {
        let flags: u64;
        unsafe {
            core::arch::asm!("pushfq", "pop {0}", out(reg) flags, options(preserves_flags));
            core::arch::asm!("cli", options(nomem, nostack));
        }
        flags
    }

    /// Restore a previously saved interrupt state (RFLAGS).
    #[inline]
    pub fn restore_irq_state(flags: u64) {
        unsafe {
            core::arch::asm!("push {0}", "popfq", in(reg) flags, options(nomem, nostack));
        }
    }

    /// Mask IRQs (same as [`cli`]).
    #[inline]
    pub fn disable_irqs() {
        cli();
    }

    /// Unmask IRQs (same as [`sti`]).
    #[inline]
    pub fn enable_irqs() {
        sti();
    }

    /// Wait for the next interrupt (same as [`hlt`]).
    #[inline]
    pub fn wait_for_interrupt() {
        hlt();
    }

    // --- Port I/O ---

    #[inline]
    pub fn outb(port: u16, value: u8) {
        unsafe { core::arch::asm!("out dx, al", in("al") value, in("dx") port); }
    }

    #[inline]
    pub fn outw(port: u16, value: u16) {
        unsafe { core::arch::asm!("out dx, ax", in("ax") value, in("dx") port); }
    }

    #[inline]
    pub fn outl(port: u16, value: u32) {
        unsafe { core::arch::asm!("out dx, eax", in("eax") value, in("dx") port); }
    }

    #[inline]
    pub fn inb(port: u16) -> u8 {
        let value: u8;
        unsafe { core::arch::asm!("in al, dx", in("dx") port, out("al") value); }
        value
    }

    #[inline]
    pub fn inw(port: u16) -> u16 {
        let value: u16;
        unsafe { core::arch::asm!("in ax, dx", in("dx") port, out("ax") value); }
        value
    }

    #[inline]
    pub fn inl(port: u16) -> u32 {
        let value: u32;
        unsafe { core::arch::asm!("in eax, dx", in("dx") port, out("eax") value); }
        value
    }

    // --- CPUID ---

    /// CPU vendor string (leaf 0, "GenuineIntel" / "AuthenticAMD", ...).
    #[inline]
    pub fn cpuid_vendor() -> [u8; 12] {
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

        [
            ebx as u8, (ebx >> 8) as u8, (ebx >> 16) as u8, (ebx >> 24) as u8,
            edx as u8, (edx >> 8) as u8, (edx >> 16) as u8, (edx >> 24) as u8,
            ecx as u8, (ecx >> 8) as u8, (ecx >> 16) as u8, (ecx >> 24) as u8,
        ]
    }

    /// CPU feature flags (leaf 1, EDX).
    #[inline]
    pub fn cpuid_features() -> u32 {
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

    /// Raw CPUID leaf-1 EAX (family/model/stepping encoding).
    #[inline]
    pub fn cpuid_model_info() -> u32 {
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
        eax
    }

    // --- Paging / TLB ---

    /// Read CR2 (page-fault linear address).
    #[inline]
    pub fn read_cr2() -> u64 {
        let cr2: u64;
        unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2); }
        cr2
    }

    /// Read CR3 (current page directory root).
    #[inline]
    pub fn read_cr3() -> u64 {
        let cr3: u64;
        unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3); }
        cr3
    }

    /// Write CR3 (switch page directory root).
    #[inline]
    pub fn write_cr3(value: u64) {
        unsafe { core::arch::asm!("mov cr3, {}", in(reg) value, options(nostack)); }
    }

    /// Invalidate a single TLB entry.
    #[inline]
    pub fn invlpg(virt_addr: usize) {
        unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt_addr as u64, options(nostack)); }
    }
}

// ============================================================================
// aarch64
// ============================================================================

#[cfg(feature = "aarch64")]
pub mod aarch64 {
    /// Wait for interrupt (WFI).
    #[inline]
    pub fn wfi() {
        unsafe { core::arch::asm!("wfi"); }
    }

    /// Disable IRQ and FIQ (DAIF).
    #[inline]
    pub fn daifset() {
        unsafe { core::arch::asm!("msr daifset, #0b0011"); }
    }

    /// Enable IRQ and FIQ (DAIF).
    #[inline]
    pub fn daifclr() {
        unsafe { core::arch::asm!("msr daifclr, #0b0011"); }
    }

    /// Mask IRQs only (DAIF immediate I bit), leaving FIQ untouched. Matches
    /// the IRQ-only masking used by the kernel's `SpinLockIrq`/`irq_save`.
    #[inline]
    pub fn disable_irqs() {
        unsafe { core::arch::asm!("msr daifset, #2"); }
    }

    /// Unmask IRQs only (DAIF immediate I bit).
    #[inline]
    pub fn enable_irqs() {
        unsafe { core::arch::asm!("msr daifclr, #2"); }
    }

    /// Save the interrupt state (DAIF) and mask IRQs. Returns the saved DAIF
    /// for [`restore_irq_state`].
    #[inline]
    pub fn save_irq_state() -> u64 {
        let flags: u64;
        unsafe {
            core::arch::asm!("mrs {0}, daif", out(reg) flags);
            core::arch::asm!("msr daifset, #2");
        }
        flags
    }

    /// Restore a previously saved interrupt state (DAIF).
    #[inline]
    pub fn restore_irq_state(flags: u64) {
        unsafe { core::arch::asm!("msr daif, {0}", in(reg) flags); }
    }

    /// Wait for the next interrupt (same as [`wfi`]).
    #[inline]
    pub fn wait_for_interrupt() {
        wfi();
    }

    #[inline]
    pub fn read_midr_el1() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, midr_el1", out(reg) val); }
        val
    }

    #[inline]
    pub fn read_cntfrq_el0() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, S3_3_C14_C0_0", out(reg) val); }
        val
    }

    #[inline]
    pub fn read_cntpct_el0() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, S3_3_C14_C0_1", out(reg) val); }
        val
    }

    #[inline]
    pub fn read_far_el1() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, far_el1", out(reg) val); }
        val
    }

    #[inline]
    pub fn read_ttbr0_el1() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) val); }
        val
    }

    #[inline]
    pub fn write_ttbr0_el1(val: u64) {
        unsafe { core::arch::asm!("msr ttbr0_el1, {}", in(reg) val); }
    }

    #[inline]
    pub fn write_tcr_el1(val: u64) {
        unsafe { core::arch::asm!("msr tcr_el1, {}", in(reg) val); }
    }

    #[inline]
    pub fn write_mair_el1(val: u64) {
        unsafe { core::arch::asm!("msr mair_el1, {}", in(reg) val); }
    }

    #[inline]
    pub fn write_sctlr_el1(val: u64) {
        unsafe { core::arch::asm!("msr sctlr_el1, {}", in(reg) val); }
    }

    #[inline]
    pub fn read_sctlr_el1() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, sctlr_el1", out(reg) val); }
        val
    }

    #[inline]
    pub fn write_vbar_el1(val: u64) {
        unsafe { core::arch::asm!("msr vbar_el1, {}", in(reg) val); }
    }

    #[inline]
    pub fn read_id_aa64isar0_el1() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, id_aa64isar0_el1", out(reg) val); }
        val
    }

    #[inline]
    pub fn tlbi_vmalle1() {
        unsafe { core::arch::asm!("tlbi vmalle1"); }
        unsafe { core::arch::asm!("dsb sy"); }
        unsafe { core::arch::asm!("isb"); }
    }

    #[inline]
    pub fn tlbi_vae1(virt: u64) {
        unsafe { core::arch::asm!("tlbi vae1, {}", in(reg) virt >> 12); }
        unsafe { core::arch::asm!("dsb sy"); }
        unsafe { core::arch::asm!("isb"); }
    }

    #[inline]
    pub fn dsb_sy() {
        unsafe { core::arch::asm!("dsb sy"); }
    }

    #[inline]
    pub fn isb() {
        unsafe { core::arch::asm!("isb"); }
    }

    /// Write the generic-timer compare-value register (S3_3_C14_C2_0).
    #[inline]
    pub fn write_timer_cval(val: u64) {
        unsafe { core::arch::asm!("msr S3_3_C14_C2_0, {}", in(reg) val); }
    }

    /// Write the generic-timer control register (S3_3_C14_C2_1).
    #[inline]
    pub fn write_timer_ctl(val: u64) {
        unsafe { core::arch::asm!("msr S3_3_C14_C2_1, {}", in(reg) val); }
    }
}

// ============================================================================
// Arch-uniform aliases
//
// The interrupt guard and other safe facades call these names without
// naming the arch module. Only one module exists per target, so the glob
// re-exports never collide.
// ============================================================================

#[cfg(feature = "x86_64")]
pub use self::x86_64::*;
#[cfg(feature = "aarch64")]
pub use self::aarch64::*;
