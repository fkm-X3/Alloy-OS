//! Safe VESA VBE graphics driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/vesa.rs`. Reads the bootloader-provided
//! framebuffer from the multiboot2 info structure and exposes its geometry.
//! All state is captured from the multiboot framebuffer tag (there is no real
//! VBE software interrupt path), so `Vesa::init` is what the boot main calls.
//!
//! The `#[no_mangle]` C-ABI entry points are kept for the ported boot main
//! (`vesa_init_from_multiboot`) and the pre-migration kernel-crate call sites.

use crate::drivers::serial::Serial;
use crate::io::{IoPort, X86IoPort};

const VBE_MODE_MASK: u16 = 0x3fff;

const VBE_DISPI_IOPORT_INDEX: u16 = 0x1ce;
const VBE_DISPI_IOPORT_DATA: u16 = 0x1cf;
const VBE_DISPI_INDEX_CURSOR_X: u16 = 0xa;
const VBE_DISPI_INDEX_CURSOR_Y: u16 = 0xb;
const VBE_DISPI_INDEX_CURSOR_ENABLE: u16 = 0xc;

const VBE_CAP_DAC_SWITCHABLE: u8 = 0x1;
const VBE_CAP_BLANK_SCREEN_VBE: u8 = 0x4;

const MULTIBOOT_TAG_TYPE_END: u32 = 0;
const MULTIBOOT_TAG_TYPE_FRAMEBUFFER: u32 = 8;
const MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT: u8 = 2;

/// Every resolution the driver knows how to report as a VBE mode number.
const SUPPORTED_MODES: [u16; 6] = [
    0x138, // 1024x768x32
    0x133, // 800x600x32
    0x130, // 640x480x32
    0x117, // 1024x768x16
    0x114, // 800x600x16
    0x111, // 640x480x16
];

/// Errors from [`Vesa::set_mode`], matching the ported return codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VesaError {
    NotInitialized = 1,
    UnsupportedMode = 2,
    BootFramebufferMismatch = 3,
}

/// Captured VESA state, filled by [`Vesa::init`].
#[derive(Debug, Clone, Copy)]
pub struct VesaInfo {
    pub available: bool,
    pub initialized: bool,
    pub framebuffer_ready: bool,
    pub vbe_version: u16,
    pub capabilities: u8,
    pub current_mode: u16,
    pub bytes_per_scanline: u16,
    pub x_resolution: u16,
    pub y_resolution: u16,
    pub bits_per_pixel: u8,
    pub linear_framebuffer: u64,
    pub framebuffer_size: u64,
    pub supported_modes: [u16; 128],
    pub num_supported_modes: u8,
}

impl Default for VesaInfo {
    fn default() -> Self {
        VesaInfo {
            available: false,
            initialized: false,
            framebuffer_ready: false,
            vbe_version: 0,
            capabilities: 0,
            current_mode: 0,
            bytes_per_scanline: 0,
            x_resolution: 0,
            y_resolution: 0,
            bits_per_pixel: 0,
            linear_framebuffer: 0,
            framebuffer_size: 0,
            supported_modes: [0; 128],
            num_supported_modes: 0,
        }
    }
}

#[repr(C)]
struct MultibootTag {
    tag_type: u32,
    size: u32,
}

#[repr(C, packed)]
struct MultibootTagFramebufferCommon {
    tag_type: u32,
    size: u32,
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    reserved: u16,
}

static mut G_VESA_STATE: VesaInfo = VesaInfo {
    available: false,
    initialized: false,
    framebuffer_ready: false,
    vbe_version: 0,
    capabilities: 0,
    current_mode: 0,
    bytes_per_scanline: 0,
    x_resolution: 0,
    y_resolution: 0,
    bits_per_pixel: 0,
    linear_framebuffer: 0,
    framebuffer_size: 0,
    supported_modes: [0; 128],
    num_supported_modes: 0,
};

/// Safe VESA facade.
pub struct Vesa;

impl Vesa {
    /// Initialize VESA state from the multiboot2 framebuffer tag.
    /// Idempotent: the boot main calls this with the real `multiboot_addr`;
    /// later calls (with 0, e.g. from the graphics layer) are ignored.
    pub fn init(multiboot_addr: u32) {
        unsafe {
            if G_VESA_STATE.initialized {
                return;
            }
            G_VESA_STATE = VesaInfo::default();
            G_VESA_STATE.initialized = true;
            G_VESA_STATE.supported_modes[..SUPPORTED_MODES.len()].copy_from_slice(&SUPPORTED_MODES);
            G_VESA_STATE.num_supported_modes = SUPPORTED_MODES.len() as u8;
            G_VESA_STATE.vbe_version = 0x300;
            G_VESA_STATE.capabilities = VBE_CAP_DAC_SWITCHABLE | VBE_CAP_BLANK_SCREEN_VBE;
        }
        Serial::write_str("[VESA] Initializing VBE detection...\n");
        if !load_multiboot_framebuffer(multiboot_addr) {
            Serial::write_str("[VESA] No valid multiboot framebuffer metadata; graphics unavailable\n");
            return;
        }
        unsafe {
            G_VESA_STATE.available = true;
        }
        Serial::write_str("[VESA] VESA VBE initialized - version=0x");
        Serial::write_hex(unsafe { G_VESA_STATE.vbe_version } as u32);
        Serial::write_str("[VESA] Supported modes: count=");
        Serial::write_hex(unsafe { G_VESA_STATE.num_supported_modes } as u32);
        Serial::write_str("[VESA] Framebuffer: addr=0x");
        Serial::write_hex(unsafe { G_VESA_STATE.linear_framebuffer } as u32);
        Serial::write_str(" width=0x");
        Serial::write_hex(unsafe { G_VESA_STATE.x_resolution } as u32);
        Serial::write_str(" height=0x");
        Serial::write_hex(unsafe { G_VESA_STATE.y_resolution } as u32);
        Serial::write_str(" bpp=0x");
        Serial::write_hex(unsafe { G_VESA_STATE.bits_per_pixel } as u32);
        Serial::write_str("\n");
    }

    /// Whether a usable framebuffer was captured.
    pub fn available() -> bool {
        unsafe { G_VESA_STATE.available && G_VESA_STATE.framebuffer_ready }
    }

    /// Framebuffer physical address, or 0 if not ready.
    pub fn framebuffer_addr() -> u64 {
        unsafe { G_VESA_STATE.linear_framebuffer }
    }

    /// Framebuffer size in bytes.
    pub fn framebuffer_size() -> u64 {
        unsafe { G_VESA_STATE.framebuffer_size }
    }

    /// Framebuffer resolution `(width, height)`, or `(0, 0)` if not ready.
    pub fn resolution() -> (u16, u16) {
        unsafe { (G_VESA_STATE.x_resolution, G_VESA_STATE.y_resolution) }
    }

    /// Color depth in bits per pixel, or 0 if not ready.
    pub fn bits_per_pixel() -> u8 {
        unsafe { G_VESA_STATE.bits_per_pixel }
    }

    /// Bytes per scanline, or 0 if not ready.
    pub fn bytes_per_scanline() -> u16 {
        unsafe { G_VESA_STATE.bytes_per_scanline }
    }

    /// The VBE mode number matching the active framebuffer, if any.
    pub fn current_mode() -> Option<u16> {
        let mode = unsafe { G_VESA_STATE.current_mode };
        if mode == 0 {
            None
        } else {
            Some(mode)
        }
    }

    /// Hardware capabilities flags.
    pub fn capabilities() -> u8 {
        unsafe { G_VESA_STATE.capabilities }
    }

    /// Request a mode; only succeeds if it is both advertised and matches the
    /// boot framebuffer dimensions (there is no real mode switch).
    pub fn set_mode(mode: u16) -> Result<(), VesaError> {
        unsafe {
            if !G_VESA_STATE.initialized {
                Serial::write_str("[VESA] Error: VESA not initialized\n");
                return Err(VesaError::NotInitialized);
            }
            if !G_VESA_STATE.available || !G_VESA_STATE.framebuffer_ready {
                Serial::write_str("[VESA] Error: Bootloader framebuffer is unavailable\n");
                return Err(VesaError::BootFramebufferMismatch);
            }
        }
        let mode_number = mode & VBE_MODE_MASK;
        let supported = unsafe {
            G_VESA_STATE.supported_modes[..G_VESA_STATE.num_supported_modes as usize]
                .iter()
                .any(|&m| m & VBE_MODE_MASK == mode_number)
        };
        if !supported {
            Serial::write_str("[VESA] Error: Mode ");
            Serial::write_hex(mode_number as u32);
            Serial::write_str(" not supported\n");
            return Err(VesaError::UnsupportedMode);
        }
        let detected = unsafe {
            mode_for_dimensions(
                G_VESA_STATE.x_resolution,
                G_VESA_STATE.y_resolution,
                G_VESA_STATE.bits_per_pixel,
            )
        };
        if detected == 0 || detected != mode_number {
            Serial::write_str("[VESA] Error: Requested mode does not match active boot framebuffer\n");
            return Err(VesaError::BootFramebufferMismatch);
        }
        unsafe {
            G_VESA_STATE.current_mode = mode_number;
        }
        Serial::write_str("[VESA] Mode set: 0x");
        Serial::write_hex(mode_number as u32);
        Serial::write_str(" (width=");
        Serial::write_hex(unsafe { G_VESA_STATE.x_resolution } as u32);
        Serial::write_str(", height=");
        Serial::write_hex(unsafe { G_VESA_STATE.y_resolution } as u32);
        Serial::write_str(", bpp=");
        Serial::write_hex(unsafe { G_VESA_STATE.bits_per_pixel } as u32);
        Serial::write_str(")\n");
        Ok(())
    }

    /// Whether the VBE DISPI hardware cursor responds (probed through the
    /// Bochs VBE index/data ports).
    pub fn cursor_available() -> bool {
        if !Self::available() {
            return false;
        }
        let saved = vbe_read_register(VBE_DISPI_INDEX_CURSOR_X);
        vbe_write_register(VBE_DISPI_INDEX_CURSOR_X, 0xaaaa);
        let test = vbe_read_register(VBE_DISPI_INDEX_CURSOR_X);
        vbe_write_register(VBE_DISPI_INDEX_CURSOR_X, saved);
        if test == 0xaaaa {
            Serial::write_str("[VESA] Hardware cursor available\n");
            true
        } else {
            Serial::write_str("[VESA] Hardware cursor not available (VBE doesn't support it)\n");
            false
        }
    }

    /// Enable or disable the VBE DISPI hardware cursor.
    pub fn cursor_enable(enable: bool) {
        vbe_write_register(VBE_DISPI_INDEX_CURSOR_ENABLE, enable as u16);
    }

    /// Position the VBE DISPI hardware cursor.
    pub fn cursor_set_position(x: u16, y: u16) {
        vbe_write_register(VBE_DISPI_INDEX_CURSOR_X, x);
        vbe_write_register(VBE_DISPI_INDEX_CURSOR_Y, y);
    }
}

/// Map (width, height, bpp) to the canonical VBE mode number, or 0.
fn mode_for_dimensions(width: u16, height: u16, bpp: u8) -> u16 {
    match (width, height, bpp) {
        (1024, 768, 16) => 0x117,
        (800, 600, 16) => 0x114,
        (640, 480, 16) => 0x111,
        (1024, 768, 32) => 0x138,
        (800, 600, 32) => 0x133,
        (640, 480, 32) => 0x130,
        _ => 0,
    }
}

/// Capture the framebuffer tag from the multiboot2 info structure.
fn load_multiboot_framebuffer(multiboot_addr: u32) -> bool {
    if multiboot_addr == 0 {
        return false;
    }
    let mut tag_addr = (multiboot_addr as usize) + 8;
    loop {
        let tag = unsafe { &*(tag_addr as *const MultibootTag) };
        if tag.tag_type == MULTIBOOT_TAG_TYPE_END {
            break;
        }
        if tag.tag_type == MULTIBOOT_TAG_TYPE_FRAMEBUFFER {
            let fb = unsafe { &*(tag_addr as *const MultibootTagFramebufferCommon) };
            if fb.framebuffer_type == MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT {
                Serial::write_str("[VESA] Multiboot framebuffer is text mode\n");
                return false;
            }
            if fb.framebuffer_addr == 0
                || fb.framebuffer_pitch == 0
                || fb.framebuffer_width == 0
                || fb.framebuffer_height == 0
                || fb.framebuffer_bpp == 0
                || fb.framebuffer_width > 0xffff
                || fb.framebuffer_height > 0xffff
                || fb.framebuffer_pitch > 0xffff
            {
                Serial::write_str("[VESA] Invalid multiboot framebuffer metadata\n");
                return false;
            }
            unsafe {
                G_VESA_STATE.linear_framebuffer = fb.framebuffer_addr;
                G_VESA_STATE.bytes_per_scanline = fb.framebuffer_pitch as u16;
                G_VESA_STATE.x_resolution = fb.framebuffer_width as u16;
                G_VESA_STATE.y_resolution = fb.framebuffer_height as u16;
                G_VESA_STATE.bits_per_pixel = fb.framebuffer_bpp;
                G_VESA_STATE.framebuffer_size =
                    (G_VESA_STATE.bytes_per_scanline as u64) * (G_VESA_STATE.y_resolution as u64);
                G_VESA_STATE.current_mode = mode_for_dimensions(
                    G_VESA_STATE.x_resolution,
                    G_VESA_STATE.y_resolution,
                    G_VESA_STATE.bits_per_pixel,
                );
                G_VESA_STATE.framebuffer_ready = true;
            }
            return true;
        }
        tag_addr += (tag.size as usize + 7) & !7;
    }
    false
}

/// Read a VBE DISPI register through the index/data port pair.
fn vbe_read_register(index: u16) -> u16 {
    unsafe {
        X86IoPort::outw(VBE_DISPI_IOPORT_INDEX, index);
        X86IoPort::inw(VBE_DISPI_IOPORT_DATA)
    }
}

/// Write a VBE DISPI register through the index/data port pair.
fn vbe_write_register(index: u16, value: u16) {
    unsafe {
        X86IoPort::outw(VBE_DISPI_IOPORT_INDEX, index);
        X86IoPort::outw(VBE_DISPI_IOPORT_DATA, value);
    }
}

// --- C-ABI shims kept for the ported boot main and pre-migration callers ---

#[no_mangle]
pub extern "C" fn vesa_init_from_multiboot(multiboot_addr: u32) {
    Vesa::init(multiboot_addr);
}

#[no_mangle]
pub extern "C" fn vesa_init() {
    Vesa::init(0);
}

#[no_mangle]
pub extern "C" fn vesa_set_mode(mode: u16) -> u16 {
    match Vesa::set_mode(mode) {
        Ok(()) => 0,
        Err(e) => e as u16,
    }
}

#[no_mangle]
pub extern "C" fn vesa_is_available() -> u8 {
    if unsafe { G_VESA_STATE.available } {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn vesa_get_capabilities() -> u8 {
    if unsafe { !G_VESA_STATE.available } {
        0
    } else {
        unsafe { G_VESA_STATE.capabilities }
    }
}

#[no_mangle]
pub extern "C" fn vesa_get_framebuffer() -> u64 {
    Vesa::framebuffer_addr()
}

#[no_mangle]
pub extern "C" fn vesa_get_resolution(width: *mut u16, height: *mut u16) {
    unsafe {
        if width.is_null() || height.is_null() {
            return;
        }
        let (w, h) = Vesa::resolution();
        *width = w;
        *height = h;
    }
}

#[no_mangle]
pub extern "C" fn vesa_get_mode(mode: *mut u16) -> u16 {
    unsafe {
        if mode.is_null() || !Vesa::available() {
            return 1;
        }
        *mode = G_VESA_STATE.current_mode;
        if G_VESA_STATE.current_mode == 0 {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vesa_get_bits_per_pixel() -> u8 {
    Vesa::bits_per_pixel()
}

#[no_mangle]
pub extern "C" fn vesa_get_bytes_per_scanline() -> u16 {
    Vesa::bytes_per_scanline()
}

#[no_mangle]
pub extern "C" fn vesa_get_framebuffer_size() -> u64 {
    Vesa::framebuffer_size()
}

#[no_mangle]
pub extern "C" fn vesa_cursor_is_available() -> u8 {
    Vesa::cursor_available() as u8
}

#[no_mangle]
pub extern "C" fn vesa_cursor_enable(enable: u8) {
    Vesa::cursor_enable(enable != 0);
}

#[no_mangle]
pub extern "C" fn vesa_cursor_set_position(x: u16, y: u16) {
    Vesa::cursor_set_position(x, y);
}
