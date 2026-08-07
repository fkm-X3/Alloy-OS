use ::core::arch::asm;
extern "C" {
    fn rust_timer_tick();
    fn serial_print(str: *const ::core::ffi::c_char);
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub const PIT_CHANNEL_0: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PIT_COMMAND: ::core::ffi::c_int = 0x43 as ::core::ffi::c_int;
pub const PIT_BASE_FREQ: ::core::ffi::c_int = 1193180 as ::core::ffi::c_int;
pub const PIT_CMD_BINARY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PIT_CMD_MODE3: ::core::ffi::c_int = 0x6 as ::core::ffi::c_int;
pub const PIT_CMD_RW_BOTH: ::core::ffi::c_int = 0x30 as ::core::ffi::c_int;
pub const PIT_CMD_CHANNEL0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PIT_CMD_INIT: ::core::ffi::c_int =
    PIT_CMD_CHANNEL0 | PIT_CMD_RW_BOTH | PIT_CMD_MODE3 | PIT_CMD_BINARY;
#[no_mangle]
pub static mut g_timer_ticks: uint64_t = 0 as uint64_t;
static mut g_timer_frequency: uint32_t = 0 as uint32_t;
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("al") value => _, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
}
#[no_mangle]
pub unsafe extern "C" fn timer_init_ffi(mut frequency: uint32_t) {
    serial_print(b"[Timer] Initializing PIT timer\n\0" as *const u8 as *const ::core::ffi::c_char);
    g_timer_frequency = frequency;
    let mut divisor: uint32_t = (PIT_BASE_FREQ as uint32_t).wrapping_div(frequency);
    if divisor > 65535 as uint32_t {
        divisor = 65535 as uint32_t;
    }
    outb(PIT_COMMAND as uint16_t, PIT_CMD_INIT as uint8_t);
    outb(
        PIT_CHANNEL_0 as uint16_t,
        (divisor & 0xff as uint32_t) as uint8_t,
    );
    outb(
        PIT_CHANNEL_0 as uint16_t,
        (divisor >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t,
    );
    serial_print(b"[Timer] PIT initialized\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn timer_handler() {
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
    if g_timer_frequency == 0 as uint32_t {
        return 0 as uint64_t;
    }
    return g_timer_ticks
        .wrapping_mul(1000 as uint64_t)
        .wrapping_div(g_timer_frequency as uint64_t);
}
#[no_mangle]
pub unsafe extern "C" fn timer_get_frequency_ffi() -> uint32_t {
    return g_timer_frequency;
}
