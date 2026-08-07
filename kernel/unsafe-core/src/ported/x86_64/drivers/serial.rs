use ::core::arch::asm;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("al") value => _, inlateout("dx") port => _,
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
unsafe extern "C" fn serial_transmit_empty() -> ::core::ffi::c_int {
    return inb((0x3f8 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int;
}
unsafe extern "C" fn serial_putchar(mut c: ::core::ffi::c_char) {
    while serial_transmit_empty() == 0 as ::core::ffi::c_int {}
    outb(0x3f8 as uint16_t, c as uint8_t);
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
