//! x86_64 architecture implementation
//!
//! Full 64-bit x86 architecture support.
//! Session 3.6: GDT, IDT, syscall MSRs, exception/IRQ handlers, and
//! CPU detection — all moved here from the ported C2Rust modules.
//! Exception/IRQ handlers are `#[no_mangle]` for the asm ISR/IRQ stubs.

use core::arch::asm;

use crate::raw::asm::x86_64;

use super::{Arch, CpuContext};

// ============================================================================
// Arch trait implementation
// ============================================================================

pub struct X86_64Arch;

impl Arch for X86_64Arch {
    const NAME: &'static str = "x86_64";
    const POINTER_WIDTH: u32 = 64;
    const PAGE_SIZE: u32 = 4096;

    fn init() {}

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
        super::x86_64::gdt_init();
    }

    fn init_idt() {
        super::x86_64::idt_init();
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

// ============================================================================
// Safe boot-init wrappers
// ============================================================================

/// Initialize the GDT, TSS, and segment descriptors.
/// Called from `kernel_main` (the asm entry point).
pub fn gdt_init() {
    unsafe {
        extern "C" {
            static mut kernel_stack_top: u64;
        }

        gdtp.limit = (core::mem::size_of::<GdtEntry>() * 7 - 1) as u16;
        gdtp.base = &raw mut gdt as u64;

        gdt_set_gate(0, 0, 0, 0, 0);        // null
        gdt_set_gate(1, 0, 0, 0x9a, 0x20);  // kernel code
        gdt_set_gate(2, 0, 0, 0x92, 0);      // kernel data
        gdt_set_gate(3, 0, 0, 0xf2, 0);      // user data
        gdt_set_gate(4, 0, 0, 0xfa, 0x20);  // user code

        crate::raw::string::memset(
            &raw mut kernel_tss as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<Tss>() as crate::raw::string::size_t,
        );
        kernel_tss.0.rsp0 = kernel_stack_top;
        kernel_tss.0.iopb_offset = core::mem::size_of::<Tss>() as u16;
        tss_set_gate(5, &raw mut kernel_tss as u64, (core::mem::size_of::<Tss>() - 1) as u32);

        extern "C" {
            fn gdt_flush(gdt_ptr: u64);
        }
        gdt_flush(&raw mut gdtp as u64);
        asm!("ltr %ax", inlateout("ax") 0x28u16 => _, options(preserves_flags, att_syntax));
    }
}

/// Update TSS RSP0 — called after kernel stack is established.
pub fn tss_update_rsp0(rsp0: u64) {
    unsafe {
        kernel_tss.0.rsp0 = rsp0;
    }
}

/// Initialize the PIC, IDT entries, and enable interrupts.
/// Called from `kernel_main`.
pub fn idt_init() {
    unsafe {
        idtp.limit = (core::mem::size_of::<IdtEntry>() * 256 - 1) as u16;
        idtp.base = &raw mut idt as u64;

        for i in 0..256 {
            idt_set_gate(i as u8, 0, 0, 0);
        }

        pic_remap();

        // ISRs 0–31 (CPU exceptions)
        idt_set_gate(0, isr_addr(0), 0x8, 0x8e);
        idt_set_gate(1, isr_addr(1), 0x8, 0x8e);
        idt_set_gate(2, isr_addr(2), 0x8, 0x8e);
        idt_set_gate(3, isr_addr(3), 0x8, 0x8e);
        idt_set_gate(4, isr_addr(4), 0x8, 0x8e);
        idt_set_gate(5, isr_addr(5), 0x8, 0x8e);
        idt_set_gate(6, isr_addr(6), 0x8, 0x8e);
        idt_set_gate(7, isr_addr(7), 0x8, 0x8e);
        idt_set_gate(8, isr_addr(8), 0x8, 0x8e);
        idt_set_gate(9, isr_addr(9), 0x8, 0x8e);
        idt_set_gate(10, isr_addr(10), 0x8, 0x8e);
        idt_set_gate(11, isr_addr(11), 0x8, 0x8e);
        idt_set_gate(12, isr_addr(12), 0x8, 0x8e);
        idt_set_gate(13, isr_addr(13), 0x8, 0x8e);
        idt_set_gate(14, isr_addr(14), 0x8, 0x8e);
        idt_set_gate(15, isr_addr(15), 0x8, 0x8e);
        idt_set_gate(16, isr_addr(16), 0x8, 0x8e);
        idt_set_gate(17, isr_addr(17), 0x8, 0x8e);
        idt_set_gate(18, isr_addr(18), 0x8, 0x8e);
        idt_set_gate(19, isr_addr(19), 0x8, 0x8e);
        idt_set_gate(20, isr_addr(20), 0x8, 0x8e);
        idt_set_gate(21, isr_addr(21), 0x8, 0x8e);
        idt_set_gate(22, isr_addr(22), 0x8, 0x8e);
        idt_set_gate(23, isr_addr(23), 0x8, 0x8e);
        idt_set_gate(24, isr_addr(24), 0x8, 0x8e);
        idt_set_gate(25, isr_addr(25), 0x8, 0x8e);
        idt_set_gate(26, isr_addr(26), 0x8, 0x8e);
        idt_set_gate(27, isr_addr(27), 0x8, 0x8e);
        idt_set_gate(28, isr_addr(28), 0x8, 0x8e);
        idt_set_gate(29, isr_addr(29), 0x8, 0x8e);
        idt_set_gate(30, isr_addr(30), 0x8, 0x8e);
        idt_set_gate(31, isr_addr(31), 0x8, 0x8e);

        // IRQs 0–15 → IDT 32–47
        idt_set_gate(32, irq_addr(0), 0x8, 0x8e);
        idt_set_gate(33, irq_addr(1), 0x8, 0x8e);
        idt_set_gate(34, irq_addr(2), 0x8, 0x8e);
        idt_set_gate(35, irq_addr(3), 0x8, 0x8e);
        idt_set_gate(36, irq_addr(4), 0x8, 0x8e);
        idt_set_gate(37, irq_addr(5), 0x8, 0x8e);
        idt_set_gate(38, irq_addr(6), 0x8, 0x8e);
        idt_set_gate(39, irq_addr(7), 0x8, 0x8e);
        idt_set_gate(40, irq_addr(8), 0x8, 0x8e);
        idt_set_gate(41, irq_addr(9), 0x8, 0x8e);
        idt_set_gate(42, irq_addr(10), 0x8, 0x8e);
        idt_set_gate(43, irq_addr(11), 0x8, 0x8e);
        idt_set_gate(44, irq_addr(12), 0x8, 0x8e);
        idt_set_gate(45, irq_addr(13), 0x8, 0x8e);
        idt_set_gate(46, irq_addr(14), 0x8, 0x8e);
        idt_set_gate(47, irq_addr(15), 0x8, 0x8e);

        // Syscall entry at INT 0x80 (DPL=3 for user-mode access)
        idt_set_gate(0x80, syscall_entry_addr(), 0x8, 0xee);

        extern "C" {
            fn idt_flush(idt_ptr: u64);
        }
        idt_flush(&raw mut idtp as u64);
        asm!("sti", options(preserves_flags, att_syntax));
    }
}

/// Program the x86_64 MSRs for the `syscall`/`sysret` mechanism.
/// Called from `kernel_main`.
pub fn syscall_init() {
    crate::drivers::serial::Serial::write_str("[Syscall] Initializing x86_64 syscall interface\n");

    unsafe {
        extern "C" {
            static mut kernel_stack_top: u64;
            fn syscall_entry();
        }
        kernel_stack_top = &raw mut kernel_stack_top_alias as u64;

        crate::drivers::serial::Serial::write_str("[Syscall] Kernel stack top: 0x");
        crate::drivers::serial::Serial::write_hex64(kernel_stack_top);
        crate::drivers::serial::Serial::write_str("\n");

        // STAR MSR: kernel CS/SS = 0x08/0x10, user CS/SS = 0x20/0x18
        let star: u64 = (0x8u64 << 32) | (0x10u64 << 48);
        let star_low = (star & 0xFFFF_FFFF) as u32;
        let star_high = ((star >> 32) & 0xFFFF_FFFF) as u32;
        asm!("wrmsr",
            inlateout("cx") 0xC000_0081u32 => _,
            inlateout("ax") star_low => _,
            inlateout("dx") star_high => _,
            options(preserves_flags, att_syntax));

        // LSTAR MSR: syscall entry point
        let lstar = syscall_entry as usize as u64;
        let lstar_low = (lstar & 0xFFFF_FFFF) as u32;
        let lstar_high = ((lstar >> 32) & 0xFFFF_FFFF) as u32;
        asm!("wrmsr",
            inlateout("dx") lstar_high => _,
            inlateout("cx") 0xC000_0082u32 => _,
            inlateout("ax") lstar_low => _,
            options(preserves_flags, att_syntax));

        // SFMASK MSR: mask IF and TF on syscall entry
        let sf_mask: u64 = 0x300;
        let sf_low = (sf_mask & 0xFFFF_FFFF) as u32;
        let sf_high = ((sf_mask >> 32) & 0xFFFF_FFFF) as u32;
        asm!("wrmsr",
            inlateout("cx") 0xC000_0084u32 => _,
            inlateout("ax") sf_low => _,
            inlateout("dx") sf_high => _,
            options(preserves_flags, att_syntax));

        // KERNEL_GS_BASE MSR: point to syscall GS save area
        let gs_base = &raw mut syscall_gs_save_area as usize as u64;
        let kgs_low = (gs_base & 0xFFFF_FFFF) as u32;
        let kgs_high = ((gs_base >> 32) & 0xFFFF_FFFF) as u32;
        asm!("wrmsr",
            inlateout("ax") kgs_low => _,
            inlateout("cx") 0xC000_0102u32 => _,
            inlateout("dx") kgs_high => _,
            options(preserves_flags, att_syntax));

        g_kernel_gs_base = gs_base;
    }

    crate::drivers::serial::Serial::write_str("[Syscall] x86_64 syscall MSRs configured\n");
}

// ============================================================================
// GDT implementation (from ported gdt.rs)
// ============================================================================

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub access: u8,
    pub granularity: u8,
    pub base_high: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct GdtPtr {
    pub limit: u16,
    pub base: u64,
}

#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct Tss(pub TssInner);

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TssInner {
    pub reserved0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved2: u64,
    pub reserved3: u16,
    pub iopb_offset: u16,
}

#[no_mangle]
static mut gdt: [GdtEntry; 7] = [GdtEntry {
    limit_low: 0, base_low: 0, base_middle: 0, access: 0, granularity: 0, base_high: 0,
}; 7];

#[no_mangle]
static mut gdtp: GdtPtr = GdtPtr { limit: 0, base: 0 };

#[no_mangle]
static mut kernel_tss: Tss = Tss(TssInner {
    reserved0: 0, rsp0: 0, rsp1: 0, rsp2: 0, reserved1: 0,
    ist1: 0, ist2: 0, ist3: 0, ist4: 0, ist5: 0, ist6: 0, ist7: 0,
    reserved2: 0, reserved3: 0, iopb_offset: 0,
});

unsafe fn gdt_set_gate(num: i32, base: u64, limit: u64, access: u8, gran: u8) {
    let e = &mut gdt[num as usize];
    e.base_low = (base & 0xFFFF) as u16;
    e.base_middle = ((base >> 16) & 0xFF) as u8;
    e.base_high = ((base >> 24) & 0xFF) as u8;
    e.limit_low = (limit & 0xFFFF) as u16;
    e.granularity = (((limit >> 16) & 0xF) | ((gran as u32 & 0xF0) as u64)) as u8;
    e.access = access;
}

unsafe fn tss_set_gate(num: i32, base: u64, limit: u32) {
    let e = &mut gdt[num as usize];
    e.limit_low = (limit & 0xFFFF) as u16;
    e.base_low = (base & 0xFFFF) as u16;
    e.base_middle = ((base >> 16) & 0xFF) as u8;
    e.access = 0x89;
    e.granularity = ((limit >> 16) & 0xF) as u8;
    e.base_high = ((base >> 24) & 0xFF) as u8;

    let high = &mut gdt[(num + 1) as usize];
    high.limit_low = ((base >> 32) & 0xFFFF) as u16;
    high.base_low = ((base >> 48) & 0xFFFF) as u16;
    high.base_middle = 0;
    high.access = 0;
    high.granularity = 0;
    high.base_high = 0;
}

// ============================================================================
// IDT implementation (from ported idt.rs)
// ============================================================================

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtEntry {
    pub base_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub flags: u8,
    pub base_mid: u16,
    pub base_high: u32,
    pub reserved: u32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtPtr {
    pub limit: u16,
    pub base: u64,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct InterruptFrame {
    pub gs: u64,
    pub fs: u64,
    pub es: u64,
    pub ds: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub int_no: u64,
    pub err_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[no_mangle]
static mut idt: [IdtEntry; 256] = [IdtEntry {
    base_low: 0, selector: 0, ist: 0, flags: 0, base_mid: 0, base_high: 0, reserved: 0,
}; 256];

#[no_mangle]
static mut idtp: IdtPtr = IdtPtr { limit: 0, base: 0 };

unsafe fn idt_set_gate(num: u8, base: u64, selector: u16, flags: u8) {
    let e = &mut idt[num as usize];
    e.base_low = (base & 0xFFFF) as u16;
    e.base_mid = ((base >> 16) & 0xFFFF) as u16;
    e.base_high = ((base >> 32) & 0xFFFF_FFFF) as u32;
    e.selector = selector;
    e.ist = 0;
    e.flags = flags;
    e.reserved = 0;
}

/// Convert an ISR/IRQ function pointer to a u64 for IDT gate setup.
unsafe fn isr_addr(n: usize) -> u64 {
    extern "C" {
        fn isr0(); fn isr1(); fn isr2(); fn isr3(); fn isr4(); fn isr5(); fn isr6(); fn isr7();
        fn isr8(); fn isr9(); fn isr10(); fn isr11(); fn isr12(); fn isr13(); fn isr14();
        fn isr15(); fn isr16(); fn isr17(); fn isr18(); fn isr19(); fn isr20(); fn isr21();
        fn isr22(); fn isr23(); fn isr24(); fn isr25(); fn isr26(); fn isr27(); fn isr28();
        fn isr29(); fn isr30(); fn isr31();
    }
    let table: [unsafe extern "C" fn(); 32] = [
        isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7,
        isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15,
        isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23,
        isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31,
    ];
    table[n] as usize as u64
}

unsafe fn irq_addr(n: usize) -> u64 {
    extern "C" {
        fn irq0(); fn irq1(); fn irq2(); fn irq3(); fn irq4(); fn irq5(); fn irq6(); fn irq7();
        fn irq8(); fn irq9(); fn irq10(); fn irq11(); fn irq12(); fn irq13(); fn irq14();
        fn irq15();
    }
    let table: [unsafe extern "C" fn(); 16] = [
        irq0, irq1, irq2, irq3, irq4, irq5, irq6, irq7,
        irq8, irq9, irq10, irq11, irq12, irq13, irq14, irq15,
    ];
    table[n] as usize as u64
}

unsafe fn syscall_entry_addr() -> u64 {
    extern "C" { fn syscall_entry(); }
    syscall_entry as usize as u64
}

// PIC constants
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

unsafe fn pic_remap() {
    let mask1 = x86_64::inb(PIC1_DATA);
    let mask2 = x86_64::inb(PIC2_DATA);

    x86_64::outb(PIC1_COMMAND, 0x11); // ICW1_INIT | ICW1_ICW4
    x86_64::outb(PIC2_COMMAND, 0x11);
    x86_64::outb(PIC1_DATA, 32);  // PIC1 offset
    x86_64::outb(PIC2_DATA, 40);  // PIC2 offset
    x86_64::outb(PIC1_DATA, 4);   // PIC1 cascade
    x86_64::outb(PIC2_DATA, 2);   // PIC2 cascade
    x86_64::outb(PIC1_DATA, 0x01); // ICW4_8086
    x86_64::outb(PIC2_DATA, 0x01);

    // Unmask IRQ0 (timer), IRQ1 (keyboard), IRQ2 (cascade), IRQ12 (mouse)
    x86_64::outb(PIC1_DATA, mask1 & !((1 << 0) | (1 << 1) | (1 << 2)) as u8);
    x86_64::outb(PIC2_DATA, mask2 & !(1 << 4) as u8);
}

// ============================================================================
// Exception / IRQ handlers (called from asm ISR/IRQ stubs)
// ============================================================================

static mut EXCEPTION_MESSAGES: [*const core::ffi::c_char; 19] = [
    b"Division By Zero\0" as *const u8 as *const core::ffi::c_char,
    b"Debug\0" as *const u8 as *const core::ffi::c_char,
    b"Non Maskable Interrupt\0" as *const u8 as *const core::ffi::c_char,
    b"Breakpoint\0" as *const u8 as *const core::ffi::c_char,
    b"Into Detected Overflow\0" as *const u8 as *const core::ffi::c_char,
    b"Out of Bounds\0" as *const u8 as *const core::ffi::c_char,
    b"Invalid Opcode\0" as *const u8 as *const core::ffi::c_char,
    b"No Coprocessor\0" as *const u8 as *const core::ffi::c_char,
    b"Double Fault\0" as *const u8 as *const core::ffi::c_char,
    b"Coprocessor Segment Overrun\0" as *const u8 as *const core::ffi::c_char,
    b"Bad TSS\0" as *const u8 as *const core::ffi::c_char,
    b"Segment Not Present\0" as *const u8 as *const core::ffi::c_char,
    b"Stack Fault\0" as *const u8 as *const core::ffi::c_char,
    b"General Protection Fault\0" as *const u8 as *const core::ffi::c_char,
    b"Page Fault\0" as *const u8 as *const core::ffi::c_char,
    b"Unknown Interrupt\0" as *const u8 as *const core::ffi::c_char,
    b"Coprocessor Fault\0" as *const u8 as *const core::ffi::c_char,
    b"Alignment Check\0" as *const u8 as *const core::ffi::c_char,
    b"Machine Check\0" as *const u8 as *const core::ffi::c_char,
];

// ISR diagnostic globals — written by the asm ISR stubs, read by exception_handler.
#[no_mangle]
pub static mut g_isr_diag_rsp: u64 = 0;
#[no_mangle]
pub static mut g_isr_diag_cr3: u64 = 0;
#[no_mangle]
pub static mut g_isr_diag_int_no: u64 = 0;
#[no_mangle]
pub static mut g_isr_diag_rip: u64 = 0;
#[no_mangle]
pub static mut g_isr_diag_cs: u64 = 0;
#[no_mangle]
pub static mut g_isr_diag_err_code: u64 = 0;
#[no_mangle]
pub static mut g_isr_diag_cr2: u64 = 0;

/// x86_64 exception handler — called from ISR0–ISR31 asm stubs.
#[no_mangle]
pub unsafe extern "C" fn exception_handler(frame: *mut InterruptFrame) {
    let int_no = (*frame).int_no;
    let err_code = (*frame).err_code;

    extern "C" {
        static mut g_ctx_switch_diag: u32;
    }
    let diag = g_ctx_switch_diag;
    if diag != 0 {
        crate::drivers::serial::Serial::write_str("\n[CTX_DIAG] Last checkpoint: 0x");
        crate::drivers::serial::Serial::write_hex(diag);
        let chars: [core::ffi::c_char; 5] = [
            ((diag >> 24) & 0xFF) as core::ffi::c_char,
            ((diag >> 16) & 0xFF) as core::ffi::c_char,
            ((diag >> 8) & 0xFF) as core::ffi::c_char,
            (diag & 0xFF) as core::ffi::c_char,
            0,
        ];
        crate::drivers::serial::Serial::write_str(" (");
        crate::drivers::serial::Serial::write_str(core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(&chars[0] as *const core::ffi::c_char as *const u8, 4),
        ));
        crate::drivers::serial::Serial::write_str(")\n");
        g_ctx_switch_diag = 0;
    }

    let isr_rsp = g_isr_diag_rsp;
    let isr_cr3 = g_isr_diag_cr3;
    let isr_int_no = g_isr_diag_int_no;
    let isr_rip = g_isr_diag_rip;
    let isr_cs = g_isr_diag_cs;
    let isr_err = g_isr_diag_err_code;
    let isr_cr2 = g_isr_diag_cr2;

    if isr_rsp != 0 || isr_cr3 != 0 {
        crate::drivers::serial::Serial::write_str("[ISR_DIAG] RSP=0x");
        crate::drivers::serial::Serial::write_hex64(isr_rsp);
        crate::drivers::serial::Serial::write_str(" CR3=0x");
        crate::drivers::serial::Serial::write_hex64(isr_cr3);
        crate::drivers::serial::Serial::write_str(" INT=0x");
        crate::drivers::serial::Serial::write_hex(isr_int_no as u32);
        crate::drivers::serial::Serial::write_str(" ERR=0x");
        crate::drivers::serial::Serial::write_hex(isr_err as u32);
        crate::drivers::serial::Serial::write_str(" RIP=0x");
        crate::drivers::serial::Serial::write_hex64(isr_rip);
        crate::drivers::serial::Serial::write_str(" CS=0x");
        crate::drivers::serial::Serial::write_hex64(isr_cs);
        crate::drivers::serial::Serial::write_str(" CR2=0x");
        crate::drivers::serial::Serial::write_hex64(isr_cr2);
        crate::drivers::serial::Serial::write_str("\n");
    }

    if int_no == 14 {
        // Page fault
        let mut fault_addr: u64 = 0;
        asm!("mov %cr2, {0}", lateout(reg) fault_addr, options(preserves_flags, att_syntax));

        let user_cr3 = crate::mem::paging::g_saved_user_cr3;
        let kern_cr3 = crate::raw::ffi::paging_get_kernel_directory_phys() as u64;
        let from_user = user_cr3 != kern_cr3;

        crate::drivers::serial::Serial::write_str("\n!!! PAGE FAULT at 0x");
        crate::drivers::serial::Serial::write_hex64(fault_addr);
        crate::drivers::serial::Serial::write_str(" Error Code: ");
        crate::drivers::serial::Serial::write_hex(err_code as u32);
        crate::drivers::serial::Serial::write_str("\n  UserCR3=0x");
        crate::drivers::serial::Serial::write_hex64(user_cr3);
        crate::drivers::serial::Serial::write_str(" KernCR3=0x");
        crate::drivers::serial::Serial::write_hex64(kern_cr3);
        crate::drivers::serial::Serial::write_str(" RIP=0x");
        crate::drivers::serial::Serial::write_hex64((*frame).rip);
        crate::drivers::serial::Serial::write_str(" RSP=0x");
        crate::drivers::serial::Serial::write_hex64((*frame).rsp);
        crate::drivers::serial::Serial::write_str("\n");

        // Identity-map page table diagnostics
        let pml4_id: *mut u64 = 0x1000 as *mut u64;
        let pdpt_id: *mut u64 = 0x2000 as *mut u64;
        let pd_id: *mut u64 = 0x3000 as *mut u64;
        let win_pt: *mut u64 = 0x10_0000 as *mut u64;
        let acc_pt: *mut u64 = 0x10_2000 as *mut u64;
        let pd_idx = (fault_addr >> 21) & 0x1FF;

        crate::drivers::serial::Serial::write_str("  IDmap: PML4[0]=0x");
        crate::drivers::serial::Serial::write_hex64(*pml4_id);
        crate::drivers::serial::Serial::write_str(" PDPT[0]=0x");
        crate::drivers::serial::Serial::write_hex64(*pdpt_id);
        crate::drivers::serial::Serial::write_str(" PD[10]=0x");
        crate::drivers::serial::Serial::write_hex64(*pd_id.add(10));
        crate::drivers::serial::Serial::write_str(" PD[12]=0x");
        crate::drivers::serial::Serial::write_hex64(*pd_id.add(12));
        crate::drivers::serial::Serial::write_str(" winPT[0]=0x");
        crate::drivers::serial::Serial::write_hex64(*win_pt);
        crate::drivers::serial::Serial::write_str(" accPT[0]=0x");
        crate::drivers::serial::Serial::write_hex64(*acc_pt);
        crate::drivers::serial::Serial::write_str(" PD[");
        crate::drivers::serial::Serial::write_hex(pd_idx as u32);
        crate::drivers::serial::Serial::write_str("]=0x");
        crate::drivers::serial::Serial::write_hex64(*pd_id.add(pd_idx as usize));
        crate::drivers::serial::Serial::write_str("\n");

        extern "C" {
            fn paging_dump_user_pt(cr3: u64, fault_addr: u64);
        }
        paging_dump_user_pt(user_cr3, fault_addr);

        // COW fault resolution
        if err_code & 0x7 == 0x7 {
            if crate::raw::ffi::paging_handle_cow_fault(user_cr3 as usize, fault_addr as usize) != 0 {
                crate::drivers::serial::Serial::write_str("[COW] Resolved COW page fault at 0x");
                crate::drivers::serial::Serial::write_hex64(fault_addr);
                crate::drivers::serial::Serial::write_str("\n");
                return;
            }
        }

        // Demand-map for kernel pages
        if err_code & 0x4 == 0 && fault_addr >= 0x20_0000 {
            let ok;
            if from_user {
                extern "C" {
                    fn paging_demand_map_kernel_page(fault_addr: u64, user_cr3: u64) -> bool;
                }
                ok = paging_demand_map_kernel_page(fault_addr, user_cr3);
            } else {
                extern "C" {
                    fn paging_demand_alloc_kernel_page(fault_addr: u64) -> bool;
                }
                ok = paging_demand_alloc_kernel_page(fault_addr);
            }
            if ok {
                return;
            }
        }

        // User-mode page fault → invoke callback (terminates the task)
        if err_code & 0x4 != 0 {
            crate::drivers::serial::Serial::write_str(
                "User-mode page fault, terminating current task...\n",
            );
            let _ = crate::api::callback::invoke_page_fault(fault_addr as usize, err_code as u32);
            return;
        }
    }

    // Unhandled exception
    crate::drivers::serial::Serial::write_str("\n!!! EXCEPTION: ");
    if (int_no as usize) < 19 {
        crate::drivers::serial::Serial::write_str(core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(
                EXCEPTION_MESSAGES[int_no as usize] as *const u8,
                core::ffi::CStr::from_ptr(EXCEPTION_MESSAGES[int_no as usize])
                    .to_bytes()
                    .len(),
            ),
        ));
    } else {
        crate::drivers::serial::Serial::write_str("Unknown Exception");
    }
    crate::drivers::serial::Serial::write_str(" (");
    crate::drivers::serial::Serial::write_hex(int_no as u32);
    crate::drivers::serial::Serial::write_str(")\nError Code: ");
    crate::drivers::serial::Serial::write_hex(err_code as u32);
    crate::drivers::serial::Serial::write_str("\nRIP: 0x");
    crate::drivers::serial::Serial::write_hex64((*frame).rip);
    crate::drivers::serial::Serial::write_str("\nCS:  0x");
    crate::drivers::serial::Serial::write_hex64((*frame).cs);
    crate::drivers::serial::Serial::write_str("\nRSP: 0x");
    crate::drivers::serial::Serial::write_hex64((*frame).rsp);
    crate::drivers::serial::Serial::write_str("\nSS:  0x");
    crate::drivers::serial::Serial::write_hex64((*frame).ss);
    crate::drivers::serial::Serial::write_str("\nSystem halted.\n");
    loop {
        x86_64::halt();
    }
}

/// x86_64 IRQ handler — called from IRQ0–IRQ15 asm stubs.
#[no_mangle]
pub unsafe extern "C" fn irq_handler(frame: *mut InterruptFrame) {
    let irq_no = (*frame).int_no;

    // Ack the PIC before dispatching — the timer handler may switch tasks
    // and never return, so the EOI must already be sent or the cascade stalls.
    if irq_no >= 8 {
        x86_64::outb(PIC2_COMMAND, 0x20);
    }
    x86_64::outb(PIC1_COMMAND, 0x20);

    match irq_no {
        0 => crate::drivers::timer::timer_handler(),
        1 => crate::drivers::keyboard::keyboard_handler(),
        12 => crate::drivers::mouse::mouse_handler(),
        _ => {}
    }
}

/// System uptime in milliseconds — called from asm context-switch diagnostics.
#[no_mangle]
pub unsafe extern "C" fn get_system_uptime_ms() -> u64 {
    crate::drivers::timer::timer_get_uptime_ms_ffi()
}

// ============================================================================
// Syscall frame (x86_64-specific, read from GS save area)
// ============================================================================

/// Layout of the GS save area written by `syscall_entry.asm` before the
/// dispatcher runs.
///
/// ```text
///   [0]  user RSP         [3]  return RIP (RCX)     [6] rbp
///   [1]  syscall number   [4]  saved RFLAGS (R11)   [7] r12  [8] r13
///   [2]  user CR3         [5]  rbx                  [9] r14  [10] r15
/// ```
static mut syscall_gs_save_area: [u64; 11] = [0; 11];

/// Snapshot of the x86_64 syscall frame of the currently running syscall.
/// Valid only from within syscall context.
#[derive(Debug, Clone, Copy)]
pub struct UserSyscallFrame {
    pub user_rsp: u64,
    pub syscall_no: u32,
    pub user_cr3: u64,
    pub rip: u64,
    pub rflags: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

/// Snapshot the x86_64 syscall frame of the currently running syscall.
pub fn current_user_syscall_frame() -> UserSyscallFrame {
    let a = unsafe { &raw const syscall_gs_save_area } as *const u64;
    unsafe {
        UserSyscallFrame {
            user_rsp: core::ptr::read(a),
            syscall_no: core::ptr::read(a.add(1)) as u32,
            user_cr3: core::ptr::read(a.add(2)),
            rip: core::ptr::read(a.add(3)),
            rflags: core::ptr::read(a.add(4)),
            rbx: core::ptr::read(a.add(5)),
            rbp: core::ptr::read(a.add(6)),
            r12: core::ptr::read(a.add(7)),
            r13: core::ptr::read(a.add(8)),
            r14: core::ptr::read(a.add(9)),
            r15: core::ptr::read(a.add(10)),
        }
    }
}

#[no_mangle]
pub static mut g_kernel_gs_base: u64 = 0;

extern "C" {
    #[link_name = "kernel_stack_top_alias"]
    static mut kernel_stack_top_alias: u64;
}

// ============================================================================
// CpuContext implementation
// ============================================================================

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

// ============================================================================
// Paging types and constants
// ============================================================================

pub mod paging {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Pml4Entry {
        pub entries: [u64; 512],
    }
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PdptEntry {
        pub entries: [u64; 512],
    }
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageDirectoryEntry {
        pub entries: [u64; 512],
    }
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableEntry {
        pub entries: [u64; 512],
    }

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

pub mod segments {
    pub const KERNEL_CODE: u16 = 0x08;
    pub const KERNEL_DATA: u16 = 0x10;
    pub const USER_CODE: u16 = 0x20;
    pub const USER_DATA: u16 = 0x18;
    pub const TSS: u16 = 0x28;
}
