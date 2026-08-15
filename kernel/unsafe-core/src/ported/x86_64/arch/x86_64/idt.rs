use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
    fn serial_print_hex64(value: uint64_t);
    fn idt_flush(idt_ptr: uint64_t);
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
    fn irq0();
    fn irq1();
    fn irq2();
    fn irq3();
    fn irq4();
    fn irq5();
    fn irq6();
    fn irq7();
    fn irq8();
    fn irq9();
    fn irq10();
    fn irq11();
    fn irq12();
    fn irq13();
    fn irq14();
    fn irq15();
    fn syscall_entry();
    fn paging_handle_cow_fault(pd_phys: uintptr_t, fault_addr: uintptr_t) -> uint8_t;
    fn paging_demand_map_kernel_page(fault_addr: uint64_t, user_cr3: uint64_t) -> bool_0;
    fn paging_get_kernel_directory_phys() -> uintptr_t;
    static mut g_saved_user_cr3: uint64_t;
    static mut g_ctx_switch_diag: uint32_t;
    fn keyboard_handler();
    fn mouse_handler();
    fn timer_handler();
    fn timer_get_uptime_ms_ffi() -> uint64_t;
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type bool_0 = bool;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct idt_entry {
    pub base_low: uint16_t,
    pub selector: uint16_t,
    pub ist: uint8_t,
    pub flags: uint8_t,
    pub base_mid: uint16_t,
    pub base_high: uint32_t,
    pub reserved: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct idt_ptr {
    pub limit: uint16_t,
    pub base: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct interrupt_frame {
    pub gs: uint64_t,
    pub fs: uint64_t,
    pub es: uint64_t,
    pub ds: uint64_t,
    pub r15: uint64_t,
    pub r14: uint64_t,
    pub r13: uint64_t,
    pub r12: uint64_t,
    pub r11: uint64_t,
    pub r10: uint64_t,
    pub r9: uint64_t,
    pub r8: uint64_t,
    pub rbp: uint64_t,
    pub rdi: uint64_t,
    pub rsi: uint64_t,
    pub rdx: uint64_t,
    pub rcx: uint64_t,
    pub rbx: uint64_t,
    pub rax: uint64_t,
    pub int_no: uint64_t,
    pub err_code: uint64_t,
    pub rip: uint64_t,
    pub cs: uint64_t,
    pub rflags: uint64_t,
    pub rsp: uint64_t,
    pub ss: uint64_t,
}
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut idt: [idt_entry; 256] = [idt_entry {
    base_low: 0,
    selector: 0,
    ist: 0,
    flags: 0,
    base_mid: 0,
    base_high: 0,
    reserved: 0,
}; 256];
#[no_mangle]
pub static mut idtp: idt_ptr = idt_ptr { limit: 0, base: 0 };
unsafe extern "C" fn idt_set_gate(
    mut num: uint8_t,
    mut base: uint64_t,
    mut selector: uint16_t,
    mut flags: uint8_t,
) {
    idt[num as usize].base_low = (base & 0xffff as uint64_t) as uint16_t;
    idt[num as usize].base_mid =
        (base >> 16 as ::core::ffi::c_int & 0xffff as uint64_t) as uint16_t;
    idt[num as usize].base_high =
        (base >> 32 as ::core::ffi::c_int & 0xffffffff as uint64_t) as uint32_t;
    idt[num as usize].selector = selector;
    idt[num as usize].ist = 0 as uint8_t;
    idt[num as usize].flags = flags;
    idt[num as usize].reserved = 0 as uint32_t;
}
pub const PIC1_COMMAND: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PIC1_DATA: ::core::ffi::c_int = 0x21 as ::core::ffi::c_int;
pub const PIC2_COMMAND: ::core::ffi::c_int = 0xa0 as ::core::ffi::c_int;
pub const PIC2_DATA: ::core::ffi::c_int = 0xa1 as ::core::ffi::c_int;
pub const ICW1_INIT: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const ICW1_ICW4: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ICW4_8086: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("dx") port => _, inlateout("al") value => _,
        options(preserves_flags, att_syntax)
    );
}
#[inline]
unsafe extern "C" fn inb(mut port: uint16_t) -> uint8_t {
    let mut ret: uint8_t = 0;
    asm!(
        "inb %dx, %al\n", lateout("al") ret, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return ret;
}
unsafe extern "C" fn pic_remap() {
    let mut mask1: uint8_t = inb(PIC1_DATA as uint16_t);
    let mut mask2: uint8_t = inb(PIC2_DATA as uint16_t);
    outb(PIC1_COMMAND as uint16_t, (ICW1_INIT | ICW1_ICW4) as uint8_t);
    outb(PIC2_COMMAND as uint16_t, (ICW1_INIT | ICW1_ICW4) as uint8_t);
    outb(PIC1_DATA as uint16_t, 32 as uint8_t);
    outb(PIC2_DATA as uint16_t, 40 as uint8_t);
    outb(PIC1_DATA as uint16_t, 4 as uint8_t);
    outb(PIC2_DATA as uint16_t, 2 as uint8_t);
    outb(PIC1_DATA as uint16_t, ICW4_8086 as uint8_t);
    outb(PIC2_DATA as uint16_t, ICW4_8086 as uint8_t);
    mask1 = (mask1 as ::core::ffi::c_int
        & !((1 as ::core::ffi::c_uint) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_uint) << 2 as ::core::ffi::c_int) as uint8_t
            as ::core::ffi::c_int) as uint8_t;
    mask2 = (mask2 as ::core::ffi::c_int
        & !((1 as ::core::ffi::c_uint) << 4 as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_int)
        as uint8_t;
    outb(PIC1_DATA as uint16_t, mask1);
    outb(PIC2_DATA as uint16_t, mask2);
}
#[no_mangle]
pub unsafe extern "C" fn init_idt() {
    idtp.limit = (::core::mem::size_of::<idt_entry>() as usize)
        .wrapping_mul(256 as usize)
        .wrapping_sub(1 as usize) as uint16_t;
    idtp.base = &raw mut idt as uint64_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        idt_set_gate(i as uint8_t, 0 as uint64_t, 0 as uint16_t, 0 as uint8_t);
        i += 1;
    }
    pic_remap();
    idt_set_gate(
        0 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr0)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        1 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr1)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        2 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr2)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        3 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr3)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        4 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr4)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        5 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr5)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        6 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr6)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        7 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr7)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        8 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr8)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        9 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr9)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        10 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr10)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        11 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr11)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        12 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr12)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        13 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr13)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        14 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr14)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        15 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr15)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        16 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr16)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        17 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr17)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        18 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr18)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        19 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr19)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        20 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr20)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        21 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr21)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        22 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr22)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        23 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr23)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        24 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr24)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        25 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr25)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        26 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr26)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        27 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr27)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        28 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr28)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        29 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr29)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        30 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr30)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        31 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(isr31)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        32 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq0)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        33 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq1)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        34 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq2)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        35 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq3)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        36 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq4)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        37 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq5)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        38 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq6)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        39 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq7)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        40 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq8)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        41 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq9)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        42 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq10)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        43 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq11)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        44 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq12)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        45 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq13)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        46 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq14)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        47 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(irq15)),
        0x8 as uint16_t,
        0x8e as uint8_t,
    );
    idt_set_gate(
        0x80 as uint8_t,
        ::core::mem::transmute::<Option<unsafe extern "C" fn() -> ()>, uint64_t>(Some(
            syscall_entry,
        )),
        0x8 as uint16_t,
        0xee as uint8_t,
    );
    idt_flush(&raw mut idtp as uint64_t);
    asm!("sti\n", options(preserves_flags, att_syntax));
}
static mut exception_messages: [*const ::core::ffi::c_char; 19] = [
    b"Division By Zero\0" as *const u8 as *const ::core::ffi::c_char,
    b"Debug\0" as *const u8 as *const ::core::ffi::c_char,
    b"Non Maskable Interrupt\0" as *const u8 as *const ::core::ffi::c_char,
    b"Breakpoint\0" as *const u8 as *const ::core::ffi::c_char,
    b"Into Detected Overflow\0" as *const u8 as *const ::core::ffi::c_char,
    b"Out of Bounds\0" as *const u8 as *const ::core::ffi::c_char,
    b"Invalid Opcode\0" as *const u8 as *const ::core::ffi::c_char,
    b"No Coprocessor\0" as *const u8 as *const ::core::ffi::c_char,
    b"Double Fault\0" as *const u8 as *const ::core::ffi::c_char,
    b"Coprocessor Segment Overrun\0" as *const u8 as *const ::core::ffi::c_char,
    b"Bad TSS\0" as *const u8 as *const ::core::ffi::c_char,
    b"Segment Not Present\0" as *const u8 as *const ::core::ffi::c_char,
    b"Stack Fault\0" as *const u8 as *const ::core::ffi::c_char,
    b"General Protection Fault\0" as *const u8 as *const ::core::ffi::c_char,
    b"Page Fault\0" as *const u8 as *const ::core::ffi::c_char,
    b"Unknown Interrupt\0" as *const u8 as *const ::core::ffi::c_char,
    b"Coprocessor Fault\0" as *const u8 as *const ::core::ffi::c_char,
    b"Alignment Check\0" as *const u8 as *const ::core::ffi::c_char,
    b"Machine Check\0" as *const u8 as *const ::core::ffi::c_char,
];
#[no_mangle]
pub static mut g_isr_diag_rsp: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut g_isr_diag_cr3: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut g_isr_diag_int_no: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut g_isr_diag_rip: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut g_isr_diag_cs: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut g_isr_diag_err_code: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut g_isr_diag_cr2: uint64_t = 0 as uint64_t;
#[no_mangle]
pub unsafe extern "C" fn exception_handler(mut frame: *mut interrupt_frame) {
    let mut int_no: uint64_t = (*frame).int_no;
    let mut err_code: uint64_t = (*frame).err_code;
    let mut diag: uint32_t = g_ctx_switch_diag;
    if diag != 0 as uint32_t {
        serial_print(
            b"\n[CTX_DIAG] Last checkpoint: 0x\0" as *const u8 as *const ::core::ffi::c_char,
        );
        serial_print_hex(diag);
        serial_print(b" (\0" as *const u8 as *const ::core::ffi::c_char);
        let chars: [::core::ffi::c_char; 5] = [
            (diag >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char,
            (diag >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char,
            (diag >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char,
            (diag & 0xff as uint32_t) as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ];
        serial_print(&raw const chars as *const ::core::ffi::c_char);
        serial_print(b")\n\0" as *const u8 as *const ::core::ffi::c_char);
        g_ctx_switch_diag = 0 as uint32_t;
    }
    let mut isr_rsp: uint64_t = g_isr_diag_rsp;
    let mut isr_cr3: uint64_t = g_isr_diag_cr3;
    let mut isr_int_no: uint64_t = g_isr_diag_int_no;
    let mut isr_rip: uint64_t = g_isr_diag_rip;
    let mut isr_cs: uint64_t = g_isr_diag_cs;
    let mut isr_err: uint64_t = g_isr_diag_err_code;
    let mut isr_cr2: uint64_t = g_isr_diag_cr2;
    if isr_rsp != 0 as uint64_t || isr_cr3 != 0 as uint64_t {
        serial_print(b"[ISR_DIAG] RSP=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(isr_rsp);
        serial_print(b" CR3=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(isr_cr3);
        serial_print(b" INT=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex(isr_int_no as uint32_t);
        serial_print(b" ERR=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex(isr_err as uint32_t);
        serial_print(b" RIP=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(isr_rip);
        serial_print(b" CS=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(isr_cs);
        serial_print(b" CR2=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(isr_cr2);
        serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if int_no == 14 as uint64_t {
        let mut fault_addr: uint64_t = 0;
        asm!(
            "mov %cr2, {0}\n", lateout(reg) fault_addr, options(preserves_flags,
            att_syntax)
        );
        let mut user_cr3: uint64_t = g_saved_user_cr3;
        let mut kern_cr3: uint64_t = paging_get_kernel_directory_phys() as uint64_t;
        let mut from_user: ::core::ffi::c_int = (user_cr3 != kern_cr3) as ::core::ffi::c_int;
        serial_print(b"\n!!! PAGE FAULT at 0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(fault_addr);
        serial_print(b" Error Code: \0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex(err_code as uint32_t);
        serial_print(b"\n  UserCR3=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(user_cr3);
        serial_print(b" KernCR3=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(kern_cr3);
        serial_print(b" RIP=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64((*frame).rip);
        serial_print(b" RSP=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64((*frame).rsp);
        serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
        let mut pml4_id: *mut uint64_t = 0x1000 as ::core::ffi::c_ulonglong as *mut uint64_t;
        let mut pdpt_id: *mut uint64_t = 0x2000 as ::core::ffi::c_ulonglong as *mut uint64_t;
        let mut pd_id: *mut uint64_t = 0x3000 as ::core::ffi::c_ulonglong as *mut uint64_t;
        let mut win_pt: *mut uint64_t = 0x100000 as ::core::ffi::c_ulonglong as *mut uint64_t;
        let mut acc_pt: *mut uint64_t = 0x102000 as ::core::ffi::c_ulonglong as *mut uint64_t;
        let mut pd_idx: uint64_t = fault_addr >> 21 as ::core::ffi::c_int & 0x1ff as uint64_t;
        serial_print(b"  IDmap: PML4[0]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*pml4_id.offset(0 as ::core::ffi::c_int as isize));
        serial_print(b" PDPT[0]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*pdpt_id.offset(0 as ::core::ffi::c_int as isize));
        serial_print(b" PD[10]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*pd_id.offset(10 as ::core::ffi::c_int as isize));
        serial_print(b" PD[12]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*pd_id.offset(12 as ::core::ffi::c_int as isize));
        serial_print(b" winPT[0]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*win_pt.offset(0 as ::core::ffi::c_int as isize));
        serial_print(b" accPT[0]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*acc_pt.offset(0 as ::core::ffi::c_int as isize));
        serial_print(b" PD[\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex(pd_idx as uint32_t);
        serial_print(b"]=0x\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex64(*pd_id.offset(pd_idx as isize));
        serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
        extern "C" {
            #[link_name = "paging_dump_user_pt"]
            fn paging_dump_user_pt_0(cr3: uint64_t, fault_addr_0: uint64_t);
        }
        paging_dump_user_pt_0(user_cr3, fault_addr);
        if err_code & 0x7 as uint64_t == 0x7 as uint64_t {
            if paging_handle_cow_fault(user_cr3 as uintptr_t, fault_addr as uintptr_t) != 0 {
                serial_print(
                    b"[COW] Resolved COW page fault at 0x\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                serial_print_hex64(fault_addr);
                serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
                return;
            }
        }
        if err_code & 0x4 as uint64_t == 0 && fault_addr >= 0x200000 as uint64_t {
            let mut ok: bool_0 = false_0 != 0;
            if from_user != 0 {
                ok = paging_demand_map_kernel_page(fault_addr, user_cr3);
            } else {
                extern "C" {
                    #[link_name = "paging_demand_alloc_kernel_page"]
                    fn paging_demand_alloc_kernel_page_0(fault_addr_0: uint64_t) -> bool_0;
                }
                ok = paging_demand_alloc_kernel_page_0(fault_addr);
            }
            if ok {
                return;
            }
        }
        if err_code & 0x4 as uint64_t != 0 {
            serial_print(
                b"User-mode page fault, terminating current task...\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            let _ = crate::api::callback::invoke_page_fault(
                fault_addr as uintptr_t,
                err_code as uint32_t,
            );
            return;
        }
    }
    serial_print(b"\n!!! EXCEPTION: \0" as *const u8 as *const ::core::ffi::c_char);
    if int_no < 19 as uint64_t {
        serial_print(exception_messages[int_no as usize]);
    } else {
        serial_print(b"Unknown Exception\0" as *const u8 as *const ::core::ffi::c_char);
    }
    serial_print(b" (\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(int_no as uint32_t);
    serial_print(b")\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"Error Code: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(err_code as uint32_t);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"RIP: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64((*frame).rip);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"CS:  0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64((*frame).cs);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"RSP: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64((*frame).rsp);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"SS:  0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64((*frame).ss);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"System halted.\n\0" as *const u8 as *const ::core::ffi::c_char);
    loop {
        asm!("cli; hlt\n", options(preserves_flags, att_syntax));
    }
}
#[no_mangle]
pub unsafe extern "C" fn irq_handler(mut frame: *mut interrupt_frame) {
    let mut irq_no: uint64_t = (*frame).int_no;
    // Ack the PIC before dispatching: the timer handler may switch tasks and
    // never return, so the EOI must already be sent or the cascade stalls.
    if irq_no >= 8 as uint64_t {
        outb(PIC2_COMMAND as uint16_t, 0x20 as uint8_t);
    }
    outb(PIC1_COMMAND as uint16_t, 0x20 as uint8_t);
    match irq_no {
        0 => {
            timer_handler();
        }
        1 => {
            keyboard_handler();
        }
        12 => {
            mouse_handler();
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_system_uptime_ms() -> uint64_t {
    return timer_get_uptime_ms_ffi();
}
