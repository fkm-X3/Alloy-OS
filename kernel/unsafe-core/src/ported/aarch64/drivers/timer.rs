use ::core::arch::asm;
extern "C" {
    fn rust_timer_tick();
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
}
pub type uint32_t = u32;
pub type uint64_t = u64;
#[no_mangle]
pub static mut g_timer_ticks: uint64_t = 0 as uint64_t;
static mut g_timer_frequency: uint32_t = 0 as uint32_t;
static mut g_timer_freq_hz: uint64_t = 0 as uint64_t;
#[inline]
unsafe extern "C" fn read_cntfrq_el0() -> uint64_t {
    let mut val: uint64_t = 0;
    asm!("mrs {0}, S3_3_C14_C0_0\n", lateout(reg) val, options(preserves_flags));
    return val;
}
#[inline]
unsafe extern "C" fn read_cntpct_el0() -> uint64_t {
    let mut val: uint64_t = 0;
    asm!("mrs {0}, S3_3_C14_C0_1\n", lateout(reg) val, options(preserves_flags));
    return val;
}
#[inline]
unsafe extern "C" fn write_cntp_cval_el1(mut val: uint64_t) {
    asm!("msr S3_3_C14_C2_2, {0}\n", inlateout(reg) val => _, options(preserves_flags));
}
#[inline]
unsafe extern "C" fn write_cntp_ctl_el1(mut val: uint64_t) {
    asm!("msr S3_3_C14_C2_1, {0}\n", inlateout(reg) val => _, options(preserves_flags));
}
pub const GICD_BASE: *mut uint32_t = 0x8000000 as ::core::ffi::c_int as *mut uint32_t;
pub const GICC_BASE: *mut uint32_t = 0x8010000 as ::core::ffi::c_int as *mut uint32_t;
pub const GICD_CTLR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const GICD_ISENABLER: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const GICD_IPRIORITYR: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const GICC_CTLR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const GICC_PMR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GICC_EOIR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GIC_PPI_PHYS_TIMER: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn gic_init() {
    ::core::ptr::write_volatile(
        GICD_BASE.offset((GICD_CTLR / 4 as ::core::ffi::c_int) as isize),
        0 as uint32_t,
    );
    ::core::ptr::write_volatile(
        GICD_BASE.offset(
            ((GICD_IPRIORITYR
                + GIC_PPI_PHYS_TIMER / 4 as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
                / 4 as ::core::ffi::c_int) as isize,
        ),
        0x80808080 as ::core::ffi::c_uint as uint32_t,
    );
    ::core::ptr::write_volatile(
        GICD_BASE.offset(
            ((GICD_ISENABLER
                + GIC_PPI_PHYS_TIMER / 32 as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
                / 4 as ::core::ffi::c_int) as isize,
        ),
        ((1 as ::core::ffi::c_int) << GIC_PPI_PHYS_TIMER % 32 as ::core::ffi::c_int) as uint32_t,
    );
    ::core::ptr::write_volatile(
        GICD_BASE.offset((GICD_CTLR / 4 as ::core::ffi::c_int) as isize),
        1 as uint32_t,
    );
    ::core::ptr::write_volatile(
        GICC_BASE.offset((GICC_CTLR / 4 as ::core::ffi::c_int) as isize),
        1 as uint32_t,
    );
    ::core::ptr::write_volatile(
        GICC_BASE.offset((GICC_PMR / 4 as ::core::ffi::c_int) as isize),
        0xff as uint32_t,
    );
    serial_print(b"[Timer] GICv2 initialized\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn timer_init_ffi(mut frequency: uint32_t) {
    serial_print(
        b"[Timer] Initializing ARM Generic Timer\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    g_timer_frequency = frequency;
    g_timer_freq_hz = read_cntfrq_el0();
    if g_timer_freq_hz == 0 as uint64_t {
        g_timer_freq_hz = 62500000 as uint64_t;
    }
    serial_print(
        b"[Timer] System counter frequency: \0" as *const u8 as *const ::core::ffi::c_char,
    );
    serial_print_hex(g_timer_freq_hz as uint32_t);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    gic_init();
    let mut period: uint64_t = g_timer_freq_hz.wrapping_div(frequency as uint64_t);
    let mut now: uint64_t = read_cntpct_el0();
    write_cntp_cval_el1(now.wrapping_add(period));
    write_cntp_ctl_el1(1 as uint64_t);
    serial_print(
        b"[Timer] ARM Generic Timer initialized\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn timer_handler() {
    ::core::ptr::write_volatile(
        GICC_BASE.offset((GICC_EOIR / 4 as ::core::ffi::c_int) as isize),
        GIC_PPI_PHYS_TIMER as uint32_t,
    );
    let mut period: uint64_t = g_timer_freq_hz.wrapping_div(g_timer_frequency as uint64_t);
    let mut now: uint64_t = read_cntpct_el0();
    write_cntp_cval_el1(now.wrapping_add(period));
    ::core::ptr::write_volatile(
        &mut g_timer_ticks as *mut uint64_t,
        ::core::ptr::read_volatile::<uint64_t>(&g_timer_ticks as *const uint64_t).wrapping_add(1),
    );
    rust_timer_tick();
}
#[no_mangle]
pub unsafe extern "C" fn timer_get_ticks_ffi() -> uint64_t {
    return g_timer_ticks;
}
#[no_mangle]
pub unsafe extern "C" fn timer_get_uptime_ms_ffi() -> uint64_t {
    if g_timer_freq_hz == 0 as uint64_t {
        return 0 as uint64_t;
    }
    return read_cntpct_el0()
        .wrapping_mul(1000 as uint64_t)
        .wrapping_div(g_timer_freq_hz);
}
#[no_mangle]
pub unsafe extern "C" fn timer_get_frequency_ffi() -> uint32_t {
    return g_timer_frequency;
}
