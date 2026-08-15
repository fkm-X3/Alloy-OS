//! Safe PS/2 mouse driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/mouse.rs`. The C-ABI entry points
//! `mouse_init` and `mouse_handler` are kept here because the ported IDT
//! (`idt.rs`, IRQ12) still calls `mouse_handler` by symbol. Packet decoding
//! and the event ring buffer are implemented once, behind the safe `Mouse`
//! facade; `mouse_handler` is the thin IRQ wrapper over it.

use crate::api::callback::invoke_mouse_wake;
use crate::raw::asm::x86_64::{inb, outb};

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

const PS2_STATUS_OUTPUT_FULL: u8 = 0x01;
const PS2_STATUS_INPUT_FULL: u8 = 0x02;

const PS2_CMD_ENABLE_AUX_DEVICE: u8 = 0xA8;
const PS2_CMD_READ_CONFIG: u8 = 0x20;
const PS2_CMD_WRITE_CONFIG: u8 = 0x60;
const PS2_CMD_WRITE_TO_AUX: u8 = 0xD4;

const PS2_MOUSE_CMD_SET_DEFAULTS: u8 = 0xF6;
const PS2_MOUSE_CMD_ENABLE_STREAMING: u8 = 0xF4;
const PS2_MOUSE_RESP_ACK: u8 = 0xFA;
const PS2_MOUSE_RESP_RESEND: u8 = 0xFE;

const MOUSE_EVENT_BUFFER_SIZE: u32 = 128;

/// A decoded PS/2 mouse packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    /// Signed X delta.
    pub dx: i8,
    /// Signed Y delta.
    pub dy: i8,
    /// Signed wheel delta (unsupported on standard PS/2: always 0).
    pub wheel: i8,
    /// Pressed button mask (`MOUSE_BUTTON_*`).
    pub buttons: u8,
    /// Overflow flags (`MOUSE_EVENT_FLAG_*`).
    pub flags: u8,
}

pub const MOUSE_BUTTON_LEFT: u8 = 0x01;
pub const MOUSE_BUTTON_RIGHT: u8 = 0x02;
pub const MOUSE_BUTTON_MIDDLE: u8 = 0x04;

pub const MOUSE_EVENT_FLAG_X_OVERFLOW: u8 = 0x01;
pub const MOUSE_EVENT_FLAG_Y_OVERFLOW: u8 = 0x02;

pub const MOUSE_INIT_ERR_NONE: u8 = 0;
pub const MOUSE_INIT_ERR_INPUT_NOT_READY: u8 = 1;
pub const MOUSE_INIT_ERR_OUTPUT_NOT_READY: u8 = 2;
pub const MOUSE_INIT_ERR_SET_DEFAULTS: u8 = 3;
pub const MOUSE_INIT_ERR_SET_DEFAULTS_ACK: u8 = 4;
pub const MOUSE_INIT_ERR_ENABLE_STREAMING: u8 = 5;
pub const MOUSE_INIT_ERR_ENABLE_STREAMING_ACK: u8 = 6;

static mut MOUSE_READ_POS: u32 = 0;
static mut MOUSE_WRITE_POS: u32 = 0;
static mut MOUSE_EVENTS: [MouseEvent; MOUSE_EVENT_BUFFER_SIZE as usize] =
    [MouseEvent { dx: 0, dy: 0, wheel: 0, buttons: 0, flags: 0 }; MOUSE_EVENT_BUFFER_SIZE as usize];
static mut PACKET: [u8; 3] = [0; 3];
static mut PACKET_INDEX: u8 = 0;
static mut MOUSE_INITIALIZED: bool = false;
static mut MOUSE_INIT_ERROR: u8 = MOUSE_INIT_ERR_NONE;

fn ps2_wait_input_ready() -> bool {
    for _ in 0..100_000u32 {
        if inb(PS2_STATUS_PORT) & PS2_STATUS_INPUT_FULL == 0 {
            return true;
        }
    }
    false
}

fn ps2_wait_output_ready() -> bool {
    for _ in 0..100_000u32 {
        if inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL != 0 {
            return true;
        }
    }
    false
}

fn mouse_fail_init(error_code: u8) -> bool {
    unsafe {
        MOUSE_INITIALIZED = false;
        MOUSE_INIT_ERROR = error_code;
    }
    false
}

fn ps2_flush_output() {
    for _ in 0..MOUSE_EVENT_BUFFER_SIZE {
        if inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL == 0 {
            break;
        }
        let _ = inb(PS2_DATA_PORT);
    }
}

fn mouse_send_device_command(command: u8) -> bool {
    if !ps2_wait_input_ready() {
        return false;
    }
    outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_TO_AUX);
    if !ps2_wait_input_ready() {
        return false;
    }
    outb(PS2_DATA_PORT, command);
    true
}

fn mouse_wait_ack() -> bool {
    for _ in 0..32u32 {
        if !ps2_wait_output_ready() {
            return false;
        }
        let response = inb(PS2_DATA_PORT);
        if response == PS2_MOUSE_RESP_ACK {
            return true;
        }
        if response == PS2_MOUSE_RESP_RESEND {
            return false;
        }
    }
    false
}

/// Safe PS/2 mouse facade.
pub struct Mouse;

impl Mouse {
    /// Enable the PS/2 auxiliary device and put the mouse in streaming mode.
    /// Returns `false` (and records the failure code via [`Self::init_error`])
    /// when the controller does not respond.
    pub fn init() -> bool {
        unsafe {
            MOUSE_READ_POS = 0;
            MOUSE_WRITE_POS = 0;
            PACKET_INDEX = 0;
            MOUSE_INITIALIZED = false;
            MOUSE_INIT_ERROR = MOUSE_INIT_ERR_NONE;
        }
        ps2_flush_output();

        if !ps2_wait_input_ready() {
            return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY);
        }
        outb(PS2_COMMAND_PORT, PS2_CMD_ENABLE_AUX_DEVICE);

        if !ps2_wait_input_ready() {
            return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY);
        }
        outb(PS2_COMMAND_PORT, PS2_CMD_READ_CONFIG);
        if !ps2_wait_output_ready() {
            return mouse_fail_init(MOUSE_INIT_ERR_OUTPUT_NOT_READY);
        }
        let mut config = inb(PS2_DATA_PORT);
        config |= 1u8 << 1;
        config &= !(1u8 << 5);

        if !ps2_wait_input_ready() {
            return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY);
        }
        outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_CONFIG);
        if !ps2_wait_input_ready() {
            return mouse_fail_init(MOUSE_INIT_ERR_INPUT_NOT_READY);
        }
        outb(PS2_DATA_PORT, config);

        if !mouse_send_device_command(PS2_MOUSE_CMD_SET_DEFAULTS) {
            return mouse_fail_init(MOUSE_INIT_ERR_SET_DEFAULTS);
        }
        if !mouse_wait_ack() {
            return mouse_fail_init(MOUSE_INIT_ERR_SET_DEFAULTS_ACK);
        }

        if !mouse_send_device_command(PS2_MOUSE_CMD_ENABLE_STREAMING) {
            return mouse_fail_init(MOUSE_INIT_ERR_ENABLE_STREAMING);
        }
        if !mouse_wait_ack() {
            return mouse_fail_init(MOUSE_INIT_ERR_ENABLE_STREAMING_ACK);
        }

        unsafe {
            MOUSE_INITIALIZED = true;
            MOUSE_INIT_ERROR = MOUSE_INIT_ERR_NONE;
        }
        true
    }

    /// `true` when the mouse is initialized and streaming.
    pub fn ready() -> bool {
        unsafe { MOUSE_INITIALIZED }
    }

    /// Last init failure code (`MOUSE_INIT_ERR_*`), or
    /// [`MOUSE_INIT_ERR_NONE`] when initialized or never attempted.
    pub fn init_error() -> u8 {
        unsafe { MOUSE_INIT_ERROR }
    }

    /// `true` when a decoded mouse event is buffered.
    pub fn has_event() -> bool {
        unsafe { MOUSE_READ_POS != MOUSE_WRITE_POS }
    }

    /// Pop the next decoded mouse event, or `None` if the buffer is empty.
    pub fn read() -> Option<MouseEvent> {
        unsafe {
            if MOUSE_READ_POS == MOUSE_WRITE_POS {
                return None;
            }
            let event = MOUSE_EVENTS[MOUSE_READ_POS as usize];
            MOUSE_READ_POS = (MOUSE_READ_POS + 1) % MOUSE_EVENT_BUFFER_SIZE;
            Some(event)
        }
    }
}

fn buffer_put(event: MouseEvent) {
    unsafe {
        let next = (MOUSE_WRITE_POS + 1) % MOUSE_EVENT_BUFFER_SIZE;
        if next == MOUSE_READ_POS {
            return;
        }
        MOUSE_EVENTS[MOUSE_WRITE_POS as usize] = event;
        MOUSE_WRITE_POS = next;
    }
}

fn decode_packet() {
    unsafe {
        let status = PACKET[0];
        let event = MouseEvent {
            dx: PACKET[1] as i8,
            dy: PACKET[2] as i8,
            wheel: 0,
            buttons: status & (MOUSE_BUTTON_LEFT | MOUSE_BUTTON_RIGHT | MOUSE_BUTTON_MIDDLE),
            flags: if status & 0x40 != 0 { MOUSE_EVENT_FLAG_X_OVERFLOW } else { 0 }
                | if status & 0x80 != 0 { MOUSE_EVENT_FLAG_Y_OVERFLOW } else { 0 },
        };
        buffer_put(event);
    }
    invoke_mouse_wake();
}

/// `mouse_init()`: initialize the PS/2 mouse controller (C contract).
#[no_mangle]
pub extern "C" fn mouse_init() -> bool {
    Mouse::init()
}

/// `mouse_handler()`: IRQ12 dispatcher — read a packet byte and decode a full
/// 3-byte PS/2 packet into a `MouseEvent` when one is complete.
#[no_mangle]
pub extern "C" fn mouse_handler() {
    let byte = inb(PS2_DATA_PORT);
    unsafe {
        if !MOUSE_INITIALIZED {
            return;
        }
        if PACKET_INDEX == 0 && byte & 0x08 == 0 {
            return;
        }
        PACKET[PACKET_INDEX as usize] = byte;
        PACKET_INDEX += 1;
        if PACKET_INDEX < 3 {
            return;
        }
        PACKET_INDEX = 0;
    }
    decode_packet();
}
