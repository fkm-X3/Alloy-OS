use ::core::arch::asm;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type vga_color = ::core::ffi::c_uint;
pub const VGA_COLOR_WHITE: vga_color = 15;
pub const VGA_COLOR_LIGHT_BROWN: vga_color = 14;
pub const VGA_COLOR_LIGHT_MAGENTA: vga_color = 13;
pub const VGA_COLOR_LIGHT_RED: vga_color = 12;
pub const VGA_COLOR_LIGHT_CYAN: vga_color = 11;
pub const VGA_COLOR_LIGHT_GREEN: vga_color = 10;
pub const VGA_COLOR_LIGHT_BLUE: vga_color = 9;
pub const VGA_COLOR_DARK_GREY: vga_color = 8;
pub const VGA_COLOR_LIGHT_GREY: vga_color = 7;
pub const VGA_COLOR_BROWN: vga_color = 6;
pub const VGA_COLOR_MAGENTA: vga_color = 5;
pub const VGA_COLOR_RED: vga_color = 4;
pub const VGA_COLOR_CYAN: vga_color = 3;
pub const VGA_COLOR_GREEN: vga_color = 2;
pub const VGA_COLOR_BLUE: vga_color = 1;
pub const VGA_COLOR_BLACK: vga_color = 0;
pub const VGA_WIDTH: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
pub const VGA_HEIGHT: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
static mut VGA_BUFFER: *mut uint16_t = unsafe { 0xb8000 as ::core::ffi::c_int as *mut uint16_t };
pub const VGA_CTRL_REGISTER: ::core::ffi::c_int = 0x3d4 as ::core::ffi::c_int;
pub const VGA_DATA_REGISTER: ::core::ffi::c_int = 0x3d5 as ::core::ffi::c_int;
pub const VGA_CURSOR_HIGH: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const VGA_CURSOR_LOW: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
static mut cursor_x: uint8_t = 0 as uint8_t;
static mut cursor_y: uint8_t = 0 as uint8_t;
static mut current_color: uint8_t = 0xf as uint8_t;
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("dx") port => _, inlateout("al") value => _,
        options(preserves_flags, att_syntax)
    );
}
#[inline]
unsafe extern "C" fn vga_entry(mut c: ::core::ffi::c_char, mut color: uint8_t) -> uint16_t {
    return (c as uint8_t as uint16_t as ::core::ffi::c_int
        | (color as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
unsafe extern "C" fn update_cursor() {
    let mut pos: uint16_t =
        (cursor_y as ::core::ffi::c_int * VGA_WIDTH + cursor_x as ::core::ffi::c_int) as uint16_t;
    outb(VGA_CTRL_REGISTER as uint16_t, VGA_CURSOR_HIGH as uint8_t);
    outb(
        VGA_DATA_REGISTER as uint16_t,
        (pos as ::core::ffi::c_int >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            as uint8_t,
    );
    outb(VGA_CTRL_REGISTER as uint16_t, VGA_CURSOR_LOW as uint8_t);
    outb(
        VGA_DATA_REGISTER as uint16_t,
        (pos as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
    );
}
unsafe extern "C" fn scroll() {
    let mut y: uint8_t = 0 as uint8_t;
    while (y as ::core::ffi::c_int) < VGA_HEIGHT - 1 as ::core::ffi::c_int {
        let mut x: uint8_t = 0 as uint8_t;
        while (x as ::core::ffi::c_int) < VGA_WIDTH {
            *VGA_BUFFER
                .offset((y as ::core::ffi::c_int * VGA_WIDTH + x as ::core::ffi::c_int) as isize) =
                *VGA_BUFFER.offset(
                    ((y as ::core::ffi::c_int + 1 as ::core::ffi::c_int) * VGA_WIDTH
                        + x as ::core::ffi::c_int) as isize,
                );
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    let mut x_0: uint8_t = 0 as uint8_t;
    while (x_0 as ::core::ffi::c_int) < VGA_WIDTH {
        *VGA_BUFFER.offset(
            ((VGA_HEIGHT - 1 as ::core::ffi::c_int) * VGA_WIDTH + x_0 as ::core::ffi::c_int)
                as isize,
        ) = vga_entry(' ' as i32 as ::core::ffi::c_char, current_color);
        x_0 = x_0.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vga_init() {
    current_color = ((VGA_COLOR_BLACK as ::core::ffi::c_int) << 4 as ::core::ffi::c_int
        | VGA_COLOR_LIGHT_GREY as ::core::ffi::c_int) as uint8_t;
    cursor_x = 0 as uint8_t;
    cursor_y = 0 as uint8_t;
    vga_clear();
}
#[no_mangle]
pub unsafe extern "C" fn vga_clear() {
    let mut y: uint8_t = 0 as uint8_t;
    while (y as ::core::ffi::c_int) < VGA_HEIGHT {
        let mut x: uint8_t = 0 as uint8_t;
        while (x as ::core::ffi::c_int) < VGA_WIDTH {
            *VGA_BUFFER
                .offset((y as ::core::ffi::c_int * VGA_WIDTH + x as ::core::ffi::c_int) as isize) =
                vga_entry(' ' as i32 as ::core::ffi::c_char, current_color);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    cursor_x = 0 as uint8_t;
    cursor_y = 0 as uint8_t;
    update_cursor();
}
#[no_mangle]
pub unsafe extern "C" fn vga_set_color(mut fg: uint8_t, mut bg: uint8_t) {
    current_color = ((bg as ::core::ffi::c_int) << 4 as ::core::ffi::c_int
        | fg as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn vga_set_cursor(mut x: uint8_t, mut y: uint8_t) {
    if (x as ::core::ffi::c_int) < VGA_WIDTH && (y as ::core::ffi::c_int) < VGA_HEIGHT {
        cursor_x = x;
        cursor_y = y;
        update_cursor();
    }
}
#[no_mangle]
pub unsafe extern "C" fn vga_get_cursor_x() -> uint8_t {
    return cursor_x;
}
#[no_mangle]
pub unsafe extern "C" fn vga_get_cursor_y() -> uint8_t {
    return cursor_y;
}
#[no_mangle]
pub unsafe extern "C" fn vga_putchar(mut c: ::core::ffi::c_char) {
    if c as ::core::ffi::c_int == '\n' as i32 {
        cursor_x = 0 as uint8_t;
        cursor_y = cursor_y.wrapping_add(1);
    } else if c as ::core::ffi::c_int == '\r' as i32 {
        cursor_x = 0 as uint8_t;
    } else if c as ::core::ffi::c_int == '\t' as i32 {
        cursor_x = (cursor_x as ::core::ffi::c_int + 8 as ::core::ffi::c_int
            & !(7 as ::core::ffi::c_int)) as uint8_t;
    } else if c as ::core::ffi::c_int == '\u{8}' as i32 {
        if cursor_x as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            cursor_x = cursor_x.wrapping_sub(1);
        }
    } else {
        *VGA_BUFFER.offset(
            (cursor_y as ::core::ffi::c_int * VGA_WIDTH + cursor_x as ::core::ffi::c_int) as isize,
        ) = vga_entry(c, current_color);
        cursor_x = cursor_x.wrapping_add(1);
    }
    if cursor_x as ::core::ffi::c_int >= VGA_WIDTH {
        cursor_x = 0 as uint8_t;
        cursor_y = cursor_y.wrapping_add(1);
    }
    if cursor_y as ::core::ffi::c_int >= VGA_HEIGHT {
        scroll();
        cursor_y = (VGA_HEIGHT - 1 as ::core::ffi::c_int) as uint8_t;
    }
    update_cursor();
}
#[no_mangle]
pub unsafe extern "C" fn vga_print(mut str: *const ::core::ffi::c_char) {
    if str.is_null() {
        return;
    }
    while *str != 0 {
        vga_putchar(*str);
        str = str.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vga_println(mut str: *const ::core::ffi::c_char) {
    vga_print(str);
    vga_putchar('\n' as i32 as ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn vga_print_hex(mut value: uint32_t) {
    vga_print(b"0x\0" as *const u8 as *const ::core::ffi::c_char);
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
    vga_print(&raw mut buffer as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn vga_print_dec(mut value: uint32_t) {
    if value == 0 as uint32_t {
        vga_putchar('0' as i32 as ::core::ffi::c_char);
        return;
    }
    let mut buffer: [::core::ffi::c_char; 12] = [0; 12];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while value > 0 as uint32_t {
        let fresh0 = i;
        i = i + 1;
        buffer[fresh0 as usize] = ('0' as i32 as uint32_t)
            .wrapping_add(value.wrapping_rem(10 as uint32_t))
            as ::core::ffi::c_char;
        value = value.wrapping_div(10 as uint32_t);
    }
    while i > 0 as ::core::ffi::c_int {
        i -= 1;
        vga_putchar(buffer[i as usize]);
    }
}
