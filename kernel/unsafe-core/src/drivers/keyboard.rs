//! Safe PS/2 keyboard driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/keyboard.rs`. The C-ABI entry points
//! `keyboard_init` and `keyboard_handler` are kept here because the ported
//! IDT (`idt.rs`, IRQ1) still calls `keyboard_handler` by symbol; the handler
//! is re-implemented over the same ring buffers the safe type exposes.
//! The serial marker printed during init matches the C driver exactly
//! (`[KBD] Skip init for testing`).
//!
//! The driver keeps two queues: the translated-character ring buffer that the
//! terminal consumes, and a raw make/break scancode ring buffer that feeds
//! Wayland input routing. Both are filled from IRQ context and drained from
//! task context (single-CPU), mirroring the C driver's locking model.

use crate::api::callback::invoke_keyboard_wake;
use crate::drivers::serial::Serial;
use crate::raw::asm::x86_64::inb;

/// Keyboard data port (PS/2).
const KEYBOARD_DATA_PORT: u16 = 0x60;

/// Translated-character ring buffer size.
const KEYBOARD_BUFFER_SIZE: u32 = 256;

/// Raw make/break scancode event ring buffer size.
const SCANCODE_BUFFER_SIZE: u32 = 128;

/// Special-key codes handed to the translated buffer (values >= 128 so they
/// never collide with ASCII).
pub const SPECIAL_KEY_UP: u8 = 128;
pub const SPECIAL_KEY_DOWN: u8 = 129;
pub const SPECIAL_KEY_LEFT: u8 = 130;
pub const SPECIAL_KEY_RIGHT: u8 = 131;
pub const SPECIAL_KEY_HOME: u8 = 132;
pub const SPECIAL_KEY_END: u8 = 133;
pub const SPECIAL_KEY_DELETE: u8 = 134;
pub const SPECIAL_KEY_PGUP: u8 = 135;
pub const SPECIAL_KEY_PGDN: u8 = 136;

/// Scancodes for extended (0xE0-prefixed) navigation keys.
const KEY_UP_ARROW: u8 = 72;
const KEY_DOWN_ARROW: u8 = 80;
const KEY_LEFT_ARROW: u8 = 75;
const KEY_RIGHT_ARROW: u8 = 77;
const KEY_HOME: u8 = 71;
const KEY_END: u8 = 79;
const KEY_DELETE: u8 = 83;
const KEY_PGUP: u8 = 73;
const KEY_PGDN: u8 = 81;

/// Modifier scancodes.
const KEY_LCTRL: u8 = 0x1d;
const KEY_LSHIFT: u8 = 0x2a;
const KEY_RSHIFT: u8 = 0x36;
const KEY_LALT: u8 = 0x38;
const KEY_CAPSLOCK: u8 = 0x3a;

/// A raw make/break scancode event for Wayland input routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// Scancode (0x7F-masked).
    pub code: u8,
    /// `true` on make (press), `false` on break (release).
    pub pressed: bool,
    /// `true` for 0xE0-prefixed keys.
    pub extended: bool,
}

static mut SHIFT_PRESSED: bool = false;
static mut CTRL_PRESSED: bool = false;
static mut ALT_PRESSED: bool = false;
static mut CAPSLOCK_ACTIVE: bool = false;
static mut EXTENDED_SCANCODE: bool = false;

static mut KEYBOARD_BUFFER: [u8; KEYBOARD_BUFFER_SIZE as usize] = [0; KEYBOARD_BUFFER_SIZE as usize];
static mut BUFFER_READ_POS: u32 = 0;
static mut BUFFER_WRITE_POS: u32 = 0;

static mut SCANCODE_BUFFER: [KeyEvent; SCANCODE_BUFFER_SIZE as usize] =
    [KeyEvent { code: 0, pressed: false, extended: false }; SCANCODE_BUFFER_SIZE as usize];
static mut SCANCODE_READ_POS: u32 = 0;
static mut SCANCODE_WRITE_POS: u32 = 0;

const SCANCODE_TO_ASCII: [u8; 128] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 8, 0,
    9, b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 10, 0, 0,
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\', b'z', b'x',
    b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 0, b'*', 0, b' ', 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SCANCODE_TO_ASCII_SHIFT: [u8; 128] = [
    0, 27, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', 8, 0,
    9, b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', 10, 0, 0,
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', 0, b'|', b'Z', b'X',
    b'C', b'V', b'B', b'N', b'M', b'<', b'>', b'?', 0, b'*', 0, b' ', 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Safe PS/2 keyboard facade.
pub struct Keyboard;

impl Keyboard {
    /// Initialize the keyboard controller state. Idempotent-safe only if
    /// called before IRQ1 starts delivering scancodes; matches the C driver.
    pub fn init() {
        unsafe {
            BUFFER_READ_POS = 0;
            BUFFER_WRITE_POS = 0;
            SCANCODE_READ_POS = 0;
            SCANCODE_WRITE_POS = 0;
            SHIFT_PRESSED = false;
            CTRL_PRESSED = false;
            ALT_PRESSED = false;
            CAPSLOCK_ACTIVE = false;
            EXTENDED_SCANCODE = false;
        }
        Serial::write_str("[KBD] Skip init for testing\n");
    }

    /// `true` when a translated character is buffered.
    pub fn has_key() -> bool {
        unsafe { BUFFER_READ_POS != BUFFER_WRITE_POS }
    }

    /// Pop the next translated character, or `None` if the buffer is empty.
    pub fn read() -> Option<u8> {
        unsafe {
            if BUFFER_READ_POS == BUFFER_WRITE_POS {
                return None;
            }
            let c = KEYBOARD_BUFFER[BUFFER_READ_POS as usize];
            BUFFER_READ_POS = (BUFFER_READ_POS + 1) % KEYBOARD_BUFFER_SIZE;
            Some(c)
        }
    }

    /// `true` when a raw make/break scancode event is buffered.
    pub fn has_scancode() -> bool {
        unsafe { SCANCODE_READ_POS != SCANCODE_WRITE_POS }
    }

    /// Pop the next raw make/break scancode event, or `None` if empty.
    pub fn read_scancode() -> Option<KeyEvent> {
        unsafe {
            if SCANCODE_READ_POS == SCANCODE_WRITE_POS {
                return None;
            }
            let event = SCANCODE_BUFFER[SCANCODE_READ_POS as usize];
            SCANCODE_READ_POS = (SCANCODE_READ_POS + 1) % SCANCODE_BUFFER_SIZE;
            Some(event)
        }
    }
}

fn buffer_put(c: u8) {
    unsafe {
        let next_pos = (BUFFER_WRITE_POS + 1) % KEYBOARD_BUFFER_SIZE;
        if next_pos != BUFFER_READ_POS {
            KEYBOARD_BUFFER[BUFFER_WRITE_POS as usize] = c;
            BUFFER_WRITE_POS = next_pos;
        }
    }
}

fn scancode_put(event: KeyEvent) {
    unsafe {
        let next_pos = (SCANCODE_WRITE_POS + 1) % SCANCODE_BUFFER_SIZE;
        if next_pos != SCANCODE_READ_POS {
            SCANCODE_BUFFER[SCANCODE_WRITE_POS as usize] = event;
            SCANCODE_WRITE_POS = next_pos;
        }
    }
}

/// `keyboard_init()`: reset keyboard state (IRQ1 safety).
#[no_mangle]
pub extern "C" fn keyboard_init() {
    Keyboard::init();
}

/// `keyboard_handler()`: IRQ1 dispatcher — read a scancode, record the raw
/// make/break event for input routing, then translate it for the terminal.
#[no_mangle]
pub extern "C" fn keyboard_handler() {
    let mut scancode = inb(KEYBOARD_DATA_PORT);

    if scancode == 0xE0 {
        unsafe { EXTENDED_SCANCODE = true; }
        return;
    }

    let key_released = scancode & 0x80 != 0;
    scancode &= 0x7F;

    let extended = unsafe { EXTENDED_SCANCODE };
    unsafe { EXTENDED_SCANCODE = false; }

    scancode_put(KeyEvent { code: scancode, pressed: !key_released, extended });

    if extended {
        if key_released {
            return;
        }
        let special_key: u8 = match scancode {
            KEY_UP_ARROW => SPECIAL_KEY_UP,
            KEY_DOWN_ARROW => SPECIAL_KEY_DOWN,
            KEY_LEFT_ARROW => SPECIAL_KEY_LEFT,
            KEY_RIGHT_ARROW => SPECIAL_KEY_RIGHT,
            KEY_HOME => SPECIAL_KEY_HOME,
            KEY_END => SPECIAL_KEY_END,
            KEY_DELETE => SPECIAL_KEY_DELETE,
            KEY_PGUP => SPECIAL_KEY_PGUP,
            KEY_PGDN => SPECIAL_KEY_PGDN,
            _ => return,
        };
        buffer_put(special_key);
        invoke_keyboard_wake();
        return;
    }

    unsafe {
        if scancode == KEY_LSHIFT || scancode == KEY_RSHIFT {
            SHIFT_PRESSED = !key_released;
            return;
        }
        if scancode == KEY_LCTRL {
            CTRL_PRESSED = !key_released;
            return;
        }
        if scancode == KEY_LALT {
            ALT_PRESSED = !key_released;
            return;
        }
        if scancode == KEY_CAPSLOCK && !key_released {
            CAPSLOCK_ACTIVE = !CAPSLOCK_ACTIVE;
            return;
        }
    }
    if key_released {
        return;
    }

    let mut ascii = unsafe {
        if SHIFT_PRESSED {
            SCANCODE_TO_ASCII_SHIFT[scancode as usize]
        } else {
            SCANCODE_TO_ASCII[scancode as usize]
        }
    };

    unsafe {
        if CAPSLOCK_ACTIVE && ascii >= b'a' && ascii <= b'z' {
            ascii -= 32;
        } else if CAPSLOCK_ACTIVE && ascii >= b'A' && ascii <= b'Z' && SHIFT_PRESSED {
            ascii += 32;
        }
    }

    if ascii != 0 {
        buffer_put(ascii);
        invoke_keyboard_wake();
    }
}
