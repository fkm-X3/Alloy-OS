use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type bool_0 = bool;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const KEY_LCTRL: ::core::ffi::c_int = 0x1d as ::core::ffi::c_int;
pub const KEY_LSHIFT: ::core::ffi::c_int = 0x2a as ::core::ffi::c_int;
pub const KEY_RSHIFT: ::core::ffi::c_int = 0x36 as ::core::ffi::c_int;
pub const KEY_LALT: ::core::ffi::c_int = 0x38 as ::core::ffi::c_int;
pub const KEY_CAPSLOCK: ::core::ffi::c_int = 0x3a as ::core::ffi::c_int;
pub const KEY_HOME: ::core::ffi::c_int = 71;
pub const KEY_UP_ARROW: ::core::ffi::c_int = 72;
pub const KEY_PGUP: ::core::ffi::c_int = 73;
pub const KEY_LEFT_ARROW: ::core::ffi::c_int = 75;
pub const KEY_RIGHT_ARROW: ::core::ffi::c_int = 77;
pub const KEY_END: ::core::ffi::c_int = 79;
pub const KEY_DOWN_ARROW: ::core::ffi::c_int = 80;
pub const KEY_PGDN: ::core::ffi::c_int = 81;
pub const KEY_DELETE: ::core::ffi::c_int = 83;
pub const SPECIAL_KEY_UP: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
pub const SPECIAL_KEY_DOWN: ::core::ffi::c_int = 129 as ::core::ffi::c_int;
pub const SPECIAL_KEY_LEFT: ::core::ffi::c_int = 130 as ::core::ffi::c_int;
pub const SPECIAL_KEY_RIGHT: ::core::ffi::c_int = 131 as ::core::ffi::c_int;
pub const SPECIAL_KEY_HOME: ::core::ffi::c_int = 132 as ::core::ffi::c_int;
pub const SPECIAL_KEY_END: ::core::ffi::c_int = 133 as ::core::ffi::c_int;
pub const SPECIAL_KEY_DELETE: ::core::ffi::c_int = 134 as ::core::ffi::c_int;
pub const SPECIAL_KEY_PGUP: ::core::ffi::c_int = 135 as ::core::ffi::c_int;
pub const SPECIAL_KEY_PGDN: ::core::ffi::c_int = 136 as ::core::ffi::c_int;
pub const KEYBOARD_BUFFER_SIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const KEYBOARD_DATA_PORT: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn inb(mut port: uint16_t) -> uint8_t {
    let mut ret: uint8_t = 0;
    asm!(
        "inb %dx, %al\n", lateout("al") ret, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return ret;
}
static mut shift_pressed: bool_0 = false_0 != 0;
static mut ctrl_pressed: bool_0 = false_0 != 0;
static mut alt_pressed: bool_0 = false_0 != 0;
static mut capslock_active: bool_0 = false_0 != 0;
static mut extended_scancode: bool_0 = false_0 != 0;
static mut keyboard_buffer: [::core::ffi::c_char; 256] = [0; 256];
static mut buffer_read_pos: uint32_t = 0 as uint32_t;
static mut buffer_write_pos: uint32_t = 0 as uint32_t;
static mut scancode_to_ascii: [::core::ffi::c_char; 128] = [
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    27 as ::core::ffi::c_int as ::core::ffi::c_char,
    '1' as i32 as ::core::ffi::c_char,
    '2' as i32 as ::core::ffi::c_char,
    '3' as i32 as ::core::ffi::c_char,
    '4' as i32 as ::core::ffi::c_char,
    '5' as i32 as ::core::ffi::c_char,
    '6' as i32 as ::core::ffi::c_char,
    '7' as i32 as ::core::ffi::c_char,
    '8' as i32 as ::core::ffi::c_char,
    '9' as i32 as ::core::ffi::c_char,
    '0' as i32 as ::core::ffi::c_char,
    '-' as i32 as ::core::ffi::c_char,
    '=' as i32 as ::core::ffi::c_char,
    '\u{8}' as i32 as ::core::ffi::c_char,
    '\t' as i32 as ::core::ffi::c_char,
    'q' as i32 as ::core::ffi::c_char,
    'w' as i32 as ::core::ffi::c_char,
    'e' as i32 as ::core::ffi::c_char,
    'r' as i32 as ::core::ffi::c_char,
    't' as i32 as ::core::ffi::c_char,
    'y' as i32 as ::core::ffi::c_char,
    'u' as i32 as ::core::ffi::c_char,
    'i' as i32 as ::core::ffi::c_char,
    'o' as i32 as ::core::ffi::c_char,
    'p' as i32 as ::core::ffi::c_char,
    '[' as i32 as ::core::ffi::c_char,
    ']' as i32 as ::core::ffi::c_char,
    '\n' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    'a' as i32 as ::core::ffi::c_char,
    's' as i32 as ::core::ffi::c_char,
    'd' as i32 as ::core::ffi::c_char,
    'f' as i32 as ::core::ffi::c_char,
    'g' as i32 as ::core::ffi::c_char,
    'h' as i32 as ::core::ffi::c_char,
    'j' as i32 as ::core::ffi::c_char,
    'k' as i32 as ::core::ffi::c_char,
    'l' as i32 as ::core::ffi::c_char,
    ';' as i32 as ::core::ffi::c_char,
    '\'' as i32 as ::core::ffi::c_char,
    '`' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    '\\' as i32 as ::core::ffi::c_char,
    'z' as i32 as ::core::ffi::c_char,
    'x' as i32 as ::core::ffi::c_char,
    'c' as i32 as ::core::ffi::c_char,
    'v' as i32 as ::core::ffi::c_char,
    'b' as i32 as ::core::ffi::c_char,
    'n' as i32 as ::core::ffi::c_char,
    'm' as i32 as ::core::ffi::c_char,
    ',' as i32 as ::core::ffi::c_char,
    '.' as i32 as ::core::ffi::c_char,
    '/' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    '*' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    ' ' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
];
static mut scancode_to_ascii_shift: [::core::ffi::c_char; 128] = [
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    27 as ::core::ffi::c_int as ::core::ffi::c_char,
    '!' as i32 as ::core::ffi::c_char,
    '@' as i32 as ::core::ffi::c_char,
    '#' as i32 as ::core::ffi::c_char,
    '$' as i32 as ::core::ffi::c_char,
    '%' as i32 as ::core::ffi::c_char,
    '^' as i32 as ::core::ffi::c_char,
    '&' as i32 as ::core::ffi::c_char,
    '*' as i32 as ::core::ffi::c_char,
    '(' as i32 as ::core::ffi::c_char,
    ')' as i32 as ::core::ffi::c_char,
    '_' as i32 as ::core::ffi::c_char,
    '+' as i32 as ::core::ffi::c_char,
    '\u{8}' as i32 as ::core::ffi::c_char,
    '\t' as i32 as ::core::ffi::c_char,
    'Q' as i32 as ::core::ffi::c_char,
    'W' as i32 as ::core::ffi::c_char,
    'E' as i32 as ::core::ffi::c_char,
    'R' as i32 as ::core::ffi::c_char,
    'T' as i32 as ::core::ffi::c_char,
    'Y' as i32 as ::core::ffi::c_char,
    'U' as i32 as ::core::ffi::c_char,
    'I' as i32 as ::core::ffi::c_char,
    'O' as i32 as ::core::ffi::c_char,
    'P' as i32 as ::core::ffi::c_char,
    '{' as i32 as ::core::ffi::c_char,
    '}' as i32 as ::core::ffi::c_char,
    '\n' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    'A' as i32 as ::core::ffi::c_char,
    'S' as i32 as ::core::ffi::c_char,
    'D' as i32 as ::core::ffi::c_char,
    'F' as i32 as ::core::ffi::c_char,
    'G' as i32 as ::core::ffi::c_char,
    'H' as i32 as ::core::ffi::c_char,
    'J' as i32 as ::core::ffi::c_char,
    'K' as i32 as ::core::ffi::c_char,
    'L' as i32 as ::core::ffi::c_char,
    ':' as i32 as ::core::ffi::c_char,
    '"' as i32 as ::core::ffi::c_char,
    '~' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    '|' as i32 as ::core::ffi::c_char,
    'Z' as i32 as ::core::ffi::c_char,
    'X' as i32 as ::core::ffi::c_char,
    'C' as i32 as ::core::ffi::c_char,
    'V' as i32 as ::core::ffi::c_char,
    'B' as i32 as ::core::ffi::c_char,
    'N' as i32 as ::core::ffi::c_char,
    'M' as i32 as ::core::ffi::c_char,
    '<' as i32 as ::core::ffi::c_char,
    '>' as i32 as ::core::ffi::c_char,
    '?' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    '*' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    ' ' as i32 as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
];
unsafe extern "C" fn buffer_put(mut c: ::core::ffi::c_char) {
    let mut next_pos: uint32_t = buffer_write_pos
        .wrapping_add(1 as uint32_t)
        .wrapping_rem(KEYBOARD_BUFFER_SIZE as uint32_t);
    if next_pos != buffer_read_pos {
        keyboard_buffer[buffer_write_pos as usize] = c;
        ::core::ptr::write_volatile(&mut buffer_write_pos as *mut uint32_t, next_pos);
    }
}
#[no_mangle]
pub unsafe extern "C" fn keyboard_init() {
    ::core::ptr::write_volatile(&mut buffer_read_pos as *mut uint32_t, 0 as uint32_t);
    ::core::ptr::write_volatile(&mut buffer_write_pos as *mut uint32_t, 0 as uint32_t);
    shift_pressed = false_0 != 0;
    ctrl_pressed = false_0 != 0;
    alt_pressed = false_0 != 0;
    capslock_active = false_0 != 0;
    extended_scancode = false_0 != 0;
    serial_print(b"[KBD] Skip init for testing\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn keyboard_handler() {
    let mut scancode: uint8_t = inb(KEYBOARD_DATA_PORT as uint16_t);
    if scancode as ::core::ffi::c_int == 0xe0 as ::core::ffi::c_int {
        extended_scancode = true_0 != 0;
        return;
    }
    let mut key_released: bool_0 =
        scancode as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    scancode = (scancode as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if extended_scancode {
        extended_scancode = false_0 != 0;
        if key_released {
            return;
        }
        let mut special_key: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
        match scancode as ::core::ffi::c_int {
            KEY_UP_ARROW => {
                special_key = SPECIAL_KEY_UP as ::core::ffi::c_char;
            }
            KEY_DOWN_ARROW => {
                special_key = SPECIAL_KEY_DOWN as ::core::ffi::c_char;
            }
            KEY_LEFT_ARROW => {
                special_key = SPECIAL_KEY_LEFT as ::core::ffi::c_char;
            }
            KEY_RIGHT_ARROW => {
                special_key = SPECIAL_KEY_RIGHT as ::core::ffi::c_char;
            }
            KEY_HOME => {
                special_key = SPECIAL_KEY_HOME as ::core::ffi::c_char;
            }
            KEY_END => {
                special_key = SPECIAL_KEY_END as ::core::ffi::c_char;
            }
            KEY_DELETE => {
                special_key = SPECIAL_KEY_DELETE as ::core::ffi::c_char;
            }
            KEY_PGUP => {
                special_key = SPECIAL_KEY_PGUP as ::core::ffi::c_char;
            }
            KEY_PGDN => {
                special_key = SPECIAL_KEY_PGDN as ::core::ffi::c_char;
            }
            _ => return,
        }
        if special_key as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            buffer_put(special_key);
        }
        return;
    }
    if scancode as ::core::ffi::c_int == KEY_LSHIFT || scancode as ::core::ffi::c_int == KEY_RSHIFT
    {
        shift_pressed = !key_released;
        return;
    }
    if scancode as ::core::ffi::c_int == KEY_LCTRL {
        ctrl_pressed = !key_released;
        return;
    }
    if scancode as ::core::ffi::c_int == KEY_LALT {
        alt_pressed = !key_released;
        return;
    }
    if scancode as ::core::ffi::c_int == KEY_CAPSLOCK && !key_released {
        capslock_active = !capslock_active;
        return;
    }
    if key_released {
        return;
    }
    let mut ascii: ::core::ffi::c_char = 0;
    if shift_pressed {
        ascii = scancode_to_ascii_shift[scancode as usize];
    } else {
        ascii = scancode_to_ascii[scancode as usize];
    }
    if capslock_active as ::core::ffi::c_int != 0
        && ascii as ::core::ffi::c_int >= 'a' as i32
        && ascii as ::core::ffi::c_int <= 'z' as i32
    {
        ascii = (ascii as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as ::core::ffi::c_char;
    } else if capslock_active as ::core::ffi::c_int != 0
        && ascii as ::core::ffi::c_int >= 'A' as i32
        && ascii as ::core::ffi::c_int <= 'Z' as i32
        && shift_pressed as ::core::ffi::c_int != 0
    {
        ascii = (ascii as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as ::core::ffi::c_char;
    }
    if ascii as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        buffer_put(ascii);
    }
}
#[no_mangle]
pub unsafe extern "C" fn keyboard_has_data() -> bool_0 {
    return buffer_read_pos != buffer_write_pos;
}
#[no_mangle]
pub unsafe extern "C" fn keyboard_get_char() -> ::core::ffi::c_char {
    while !keyboard_has_data() {
        asm!("hlt\n", options(preserves_flags, att_syntax));
    }
    let mut c: ::core::ffi::c_char = keyboard_buffer[buffer_read_pos as usize];
    ::core::ptr::write_volatile(
        &mut buffer_read_pos as *mut uint32_t,
        buffer_read_pos
            .wrapping_add(1 as uint32_t)
            .wrapping_rem(KEYBOARD_BUFFER_SIZE as uint32_t),
    );
    return c;
}
