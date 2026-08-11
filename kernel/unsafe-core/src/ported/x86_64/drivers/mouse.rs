use ::core::arch::asm;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type int8_t = i8;
pub type bool_0 = bool;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mouse_event {
    pub dx: int8_t,
    pub dy: int8_t,
    pub wheel: int8_t,
    pub buttons: uint8_t,
    pub flags: uint8_t,
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_LEFT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_RIGHT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_MIDDLE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOUSE_EVENT_FLAG_X_OVERFLOW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MOUSE_EVENT_FLAG_Y_OVERFLOW: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_INPUT_NOT_READY: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_OUTPUT_NOT_READY: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_SET_DEFAULTS: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_SET_DEFAULTS_ACK: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_ENABLE_STREAMING: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const MOUSE_INIT_ERR_ENABLE_STREAMING_ACK: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const PS2_DATA_PORT: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const PS2_STATUS_PORT: ::core::ffi::c_int = 0x64 as ::core::ffi::c_int;
pub const PS2_COMMAND_PORT: ::core::ffi::c_int = 0x64 as ::core::ffi::c_int;
pub const PS2_STATUS_OUTPUT_FULL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PS2_STATUS_INPUT_FULL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PS2_CMD_ENABLE_AUX_DEVICE: ::core::ffi::c_int = 0xa8 as ::core::ffi::c_int;
pub const PS2_CMD_READ_CONFIG: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PS2_CMD_WRITE_CONFIG: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const PS2_CMD_WRITE_TO_AUX: ::core::ffi::c_int = 0xd4 as ::core::ffi::c_int;
pub const PS2_MOUSE_CMD_SET_DEFAULTS: ::core::ffi::c_int = 0xf6 as ::core::ffi::c_int;
pub const PS2_MOUSE_CMD_ENABLE_STREAMING: ::core::ffi::c_int = 0xf4 as ::core::ffi::c_int;
pub const PS2_MOUSE_RESP_ACK: ::core::ffi::c_int = 0xfa as ::core::ffi::c_int;
pub const PS2_MOUSE_RESP_RESEND: ::core::ffi::c_int = 0xfe as ::core::ffi::c_int;
pub const MOUSE_EVENT_BUFFER_SIZE: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
static mut g_mouse_read_pos: uint32_t = 0 as uint32_t;
static mut g_mouse_write_pos: uint32_t = 0 as uint32_t;
static mut g_mouse_events: [mouse_event; 128] = [mouse_event {
    dx: 0,
    dy: 0,
    wheel: 0,
    buttons: 0,
    flags: 0,
}; 128];
static mut g_packet: [uint8_t; 3] = [0; 3];
static mut g_packet_index: uint8_t = 0 as uint8_t;
static mut g_mouse_initialized: bool_0 = false_0 != 0;
static mut g_mouse_init_error: uint8_t = MOUSE_INIT_ERR_NONE as uint8_t;
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("al") value => _, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
}
#[inline]
unsafe extern "C" fn inb(mut port: uint16_t) -> uint8_t {
    let mut value: uint8_t = 0;
    asm!(
        "inb %dx, %al\n", lateout("al") value, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return value;
}
unsafe extern "C" fn ps2_wait_input_ready() -> bool_0 {
    let mut i: uint32_t = 0 as uint32_t;
    while i < 100000 as uint32_t {
        if inb(PS2_STATUS_PORT as uint16_t) as ::core::ffi::c_int & PS2_STATUS_INPUT_FULL
            == 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return false_0 != 0;
}
unsafe extern "C" fn ps2_wait_output_ready() -> bool_0 {
    let mut i: uint32_t = 0 as uint32_t;
    while i < 100000 as uint32_t {
        if inb(PS2_STATUS_PORT as uint16_t) as ::core::ffi::c_int & PS2_STATUS_OUTPUT_FULL
            != 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return false_0 != 0;
}
unsafe extern "C" fn mouse_fail_init(mut error_code: uint8_t) -> bool_0 {
    g_mouse_initialized = false_0 != 0;
    g_mouse_init_error = error_code;
    return false_0 != 0;
}
unsafe extern "C" fn ps2_flush_output() {
    let mut i: uint32_t = 0 as uint32_t;
    while i < MOUSE_EVENT_BUFFER_SIZE as uint32_t {
        if inb(PS2_STATUS_PORT as uint16_t) as ::core::ffi::c_int & PS2_STATUS_OUTPUT_FULL
            == 0 as ::core::ffi::c_int
        {
            break;
        }
        inb(PS2_DATA_PORT as uint16_t);
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn mouse_send_device_command(mut command: uint8_t) -> bool_0 {
    if !ps2_wait_input_ready() {
        return false_0 != 0;
    }
    outb(
        PS2_COMMAND_PORT as uint16_t,
        PS2_CMD_WRITE_TO_AUX as uint8_t,
    );
    if !ps2_wait_input_ready() {
        return false_0 != 0;
    }
    outb(PS2_DATA_PORT as uint16_t, command);
    return true_0 != 0;
}
unsafe extern "C" fn mouse_wait_ack() -> bool_0 {
    let mut i: uint32_t = 0 as uint32_t;
    while i < 32 as uint32_t {
        if !ps2_wait_output_ready() {
            return false_0 != 0;
        }
        let mut response: uint8_t = inb(PS2_DATA_PORT as uint16_t);
        if response as ::core::ffi::c_int == PS2_MOUSE_RESP_ACK {
            return true_0 != 0;
        }
        if response as ::core::ffi::c_int == PS2_MOUSE_RESP_RESEND {
            return false_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return false_0 != 0;
}
unsafe extern "C" fn buffer_put(mut event: mouse_event) {
    let mut next: uint32_t = g_mouse_write_pos
        .wrapping_add(1 as uint32_t)
        .wrapping_rem(MOUSE_EVENT_BUFFER_SIZE as uint32_t);
    if next == g_mouse_read_pos {
        return;
    }
    g_mouse_events[g_mouse_write_pos as usize] = event;
    ::core::ptr::write_volatile(&mut g_mouse_write_pos as *mut uint32_t, next);
}
#[no_mangle]
pub unsafe extern "C" fn mouse_init() -> bool_0 {
    ::core::ptr::write_volatile(&mut g_mouse_read_pos as *mut uint32_t, 0 as uint32_t);
    ::core::ptr::write_volatile(&mut g_mouse_write_pos as *mut uint32_t, 0 as uint32_t);
    g_packet_index = 0 as uint8_t;
    g_mouse_initialized = false_0 != 0;
    g_mouse_init_error = MOUSE_INIT_ERR_NONE as uint8_t;
    ps2_flush_output();
    if !ps2_wait_input_ready() {
        return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY as uint8_t);
    }
    outb(
        PS2_COMMAND_PORT as uint16_t,
        PS2_CMD_ENABLE_AUX_DEVICE as uint8_t,
    );
    if !ps2_wait_input_ready() {
        return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY as uint8_t);
    }
    outb(PS2_COMMAND_PORT as uint16_t, PS2_CMD_READ_CONFIG as uint8_t);
    if !ps2_wait_output_ready() {
        return mouse_fail_init(MOUSE_INIT_ERR_OUTPUT_NOT_READY as uint8_t);
    }
    let mut config: uint8_t = inb(PS2_DATA_PORT as uint16_t);
    config = (config as ::core::ffi::c_uint | (1 as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        as uint8_t;
    config = (config as ::core::ffi::c_int
        & !((1 as ::core::ffi::c_uint) << 5 as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_int)
        as uint8_t;
    if !ps2_wait_input_ready() {
        return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY as uint8_t);
    }
    outb(
        PS2_COMMAND_PORT as uint16_t,
        PS2_CMD_WRITE_CONFIG as uint8_t,
    );
    if !ps2_wait_input_ready() {
        return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY as uint8_t);
    }
    outb(PS2_DATA_PORT as uint16_t, config);
    if !mouse_send_device_command(PS2_MOUSE_CMD_SET_DEFAULTS as uint8_t) {
        return mouse_fail_init(MOUSE_INIT_ERR_SET_DEFAULTS as uint8_t);
    }
    if !mouse_wait_ack() {
        return mouse_fail_init(MOUSE_INIT_ERR_SET_DEFAULTS_ACK as uint8_t);
    }
    if !mouse_send_device_command(PS2_MOUSE_CMD_ENABLE_STREAMING as uint8_t) {
        return mouse_fail_init(MOUSE_INIT_ERR_ENABLE_STREAMING as uint8_t);
    }
    if !mouse_wait_ack() {
        return mouse_fail_init(MOUSE_INIT_ERR_ENABLE_STREAMING_ACK as uint8_t);
    }
    g_mouse_initialized = true_0 != 0;
    g_mouse_init_error = MOUSE_INIT_ERR_NONE as uint8_t;
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mouse_handler() {
    let mut byte: uint8_t = inb(PS2_DATA_PORT as uint16_t);
    if !g_mouse_initialized {
        return;
    }
    if g_packet_index as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && byte as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        return;
    }
    let fresh0 = g_packet_index;
    g_packet_index = g_packet_index.wrapping_add(1);
    g_packet[fresh0 as usize] = byte;
    if (g_packet_index as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
        return;
    }
    g_packet_index = 0 as uint8_t;
    let mut status: uint8_t = g_packet[0 as ::core::ffi::c_int as usize];
    let mut event: mouse_event = mouse_event {
        dx: 0 as int8_t,
        dy: 0,
        wheel: 0,
        buttons: 0,
        flags: 0,
    };
    event.dx = g_packet[1 as ::core::ffi::c_int as usize] as int8_t;
    event.dy = g_packet[2 as ::core::ffi::c_int as usize] as int8_t;
    event.wheel = 0 as int8_t;
    event.buttons = (status as ::core::ffi::c_int
        & (MOUSE_BUTTON_LEFT | MOUSE_BUTTON_RIGHT | MOUSE_BUTTON_MIDDLE))
        as uint8_t;
    event.flags = 0 as uint8_t;
    if status as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        event.flags = (event.flags as ::core::ffi::c_int | MOUSE_EVENT_FLAG_X_OVERFLOW) as uint8_t;
    }
    if status as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        event.flags = (event.flags as ::core::ffi::c_int | MOUSE_EVENT_FLAG_Y_OVERFLOW) as uint8_t;
    }
    buffer_put(event);
}
#[no_mangle]
pub unsafe extern "C" fn mouse_has_data() -> bool_0 {
    return g_mouse_read_pos != g_mouse_write_pos;
}
#[no_mangle]
pub unsafe extern "C" fn mouse_is_initialized() -> bool_0 {
    return g_mouse_initialized;
}
#[no_mangle]
pub unsafe extern "C" fn mouse_last_init_error() -> uint8_t {
    return g_mouse_init_error;
}
#[no_mangle]
pub unsafe extern "C" fn mouse_read_event(
    mut dx: *mut int8_t,
    mut dy: *mut int8_t,
    mut wheel: *mut int8_t,
    mut buttons: *mut uint8_t,
    mut flags: *mut uint8_t,
) -> bool_0 {
    if !mouse_has_data() {
        return false_0 != 0;
    }
    let mut event: mouse_event = g_mouse_events[g_mouse_read_pos as usize];
    ::core::ptr::write_volatile(
        &mut g_mouse_read_pos as *mut uint32_t,
        g_mouse_read_pos
            .wrapping_add(1 as uint32_t)
            .wrapping_rem(MOUSE_EVENT_BUFFER_SIZE as uint32_t),
    );
    if !dx.is_null() {
        *dx = event.dx;
    }
    if !dy.is_null() {
        *dy = event.dy;
    }
    if !wheel.is_null() {
        *wheel = event.wheel;
    }
    if !buttons.is_null() {
        *buttons = event.buttons;
    }
    if !flags.is_null() {
        *flags = event.flags;
    }
    return true_0 != 0;
}
