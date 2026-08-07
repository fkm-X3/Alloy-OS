use ::core::arch::asm;
#[cfg(target_arch = "x86_64")]
pub type uint8_t = u8;
#[cfg(target_arch = "x86_64")]
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[cfg(target_arch = "aarch64")]
pub type uintptr_t = usize;
#[cfg(target_arch = "aarch64")]
pub const PL011_BASE: ::core::ffi::c_int = 0x9000000 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTDR: ::core::ffi::c_int = PL011_BASE + 0 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTFR: ::core::ffi::c_int = PL011_BASE + 0x18 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTIBRD: ::core::ffi::c_int = PL011_BASE + 0x24 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTFBRD: ::core::ffi::c_int = PL011_BASE + 0x28 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTLCR_H: ::core::ffi::c_int = PL011_BASE + 0x2c as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTCR: ::core::ffi::c_int = PL011_BASE + 0x30 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTIFLS: ::core::ffi::c_int = PL011_BASE + 0x34 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTIMSC: ::core::ffi::c_int = PL011_BASE + 0x38 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const UARTICR: ::core::ffi::c_int = PL011_BASE + 0x44 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const TXFF: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
#[cfg(target_arch = "aarch64")]
pub const BUSY: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("al") value => _, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
}
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe extern "C" fn inb(mut port: uint16_t) -> uint8_t {
    let mut ret: uint8_t = 0;
    asm!(
        "inb %dx, %al\n", lateout("al") ret, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return ret;
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe extern "C" fn mmio_write32(mut addr: uintptr_t, mut value: uint32_t) {
    let mut ptr: *mut uint32_t = addr as *mut uint32_t;
    ::core::ptr::write_volatile(ptr, value);
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe extern "C" fn mmio_read32(mut addr: uintptr_t) -> uint32_t {
    let mut ptr: *mut uint32_t = addr as *mut uint32_t;
    return *ptr;
}
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe extern "C" fn dsb() {
    asm!("dsb sy\n", options(preserves_flags));
}
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn init_serial() {
    outb(
        (0x3f8 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16_t,
        0 as uint8_t,
    );
    outb(
        (0x3f8 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        0x80 as uint8_t,
    );
    outb(0x3f8 as uint16_t, 0x3 as uint8_t);
    outb(
        (0x3f8 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16_t,
        0 as uint8_t,
    );
    outb(
        (0x3f8 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        0x3 as uint8_t,
    );
    outb(
        (0x3f8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        0xc7 as uint8_t,
    );
    outb(
        (0x3f8 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        0xb as uint8_t,
    );
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn init_serial() {
    mmio_write32(UARTCR as uintptr_t, 0 as uint32_t);
    dsb();
    while mmio_read32(UARTFR as uintptr_t) & BUSY as uint32_t != 0 {}
    dsb();
    mmio_write32(UARTIBRD as uintptr_t, 13 as uint32_t);
    mmio_write32(UARTFBRD as uintptr_t, 1 as uint32_t);
    dsb();
    mmio_write32(UARTLCR_H as uintptr_t, 0x70 as uint32_t);
    dsb();
    mmio_write32(UARTIFLS as uintptr_t, 0x12 as uint32_t);
    dsb();
    mmio_write32(UARTIMSC as uintptr_t, 0 as uint32_t);
    dsb();
    mmio_write32(UARTICR as uintptr_t, 0x3ff as uint32_t);
    dsb();
    mmio_write32(UARTCR as uintptr_t, 0x301 as uint32_t);
    dsb();
}
#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn serial_transmit_empty() -> ::core::ffi::c_int {
    return inb((0x3f8 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int;
}
#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn serial_putchar(mut c: ::core::ffi::c_char) {
    while serial_transmit_empty() == 0 as ::core::ffi::c_int {}
    outb(0x3f8 as uint16_t, c as uint8_t);
}
#[cfg(target_arch = "aarch64")]
unsafe extern "C" fn serial_putchar(mut c: ::core::ffi::c_char) {
    while mmio_read32(UARTFR as uintptr_t) & TXFF as uint32_t != 0 {}
    mmio_write32(UARTDR as uintptr_t, c as uint32_t);
}
#[no_mangle]
pub unsafe extern "C" fn serial_print(mut str: *const ::core::ffi::c_char) {
    if str.is_null() {
        return;
    }
    while *str != 0 {
        if *str as ::core::ffi::c_int == '\n' as i32 {
            serial_putchar('\r' as i32 as ::core::ffi::c_char);
        }
        let fresh0 = str;
        str = str.offset(1);
        serial_putchar(*fresh0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn serial_print_hex(mut value: uint32_t) {
    let mut hex_chars: [::core::ffi::c_char; 17] =
        ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"0123456789ABCDEF\0");
    let mut buffer: [::core::ffi::c_char; 9] = [0; 9];
    buffer[8 as ::core::ffi::c_int as usize] = '\0' as i32 as ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        buffer[i as usize] = hex_chars[(value & 0xf as uint32_t) as usize];
        value >>= 4 as ::core::ffi::c_int;
        i -= 1;
    }
    serial_print(&raw mut buffer as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn serial_print_hex64(mut value: uint64_t) {
    let mut hex_chars: [::core::ffi::c_char; 17] =
        ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"0123456789ABCDEF\0");
    let mut buffer: [::core::ffi::c_char; 17] = [0; 17];
    buffer[16 as ::core::ffi::c_int as usize] = '\0' as i32 as ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        buffer[i as usize] = hex_chars[(value & 0xf as uint64_t) as usize];
        value >>= 4 as ::core::ffi::c_int;
        i -= 1;
    }
    serial_print(&raw mut buffer as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn serial_print_hex_with_prefix(
    mut prefix: *const ::core::ffi::c_char,
    mut value: uint32_t,
) {
    serial_print(prefix);
    serial_print(b"0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(value);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
}
