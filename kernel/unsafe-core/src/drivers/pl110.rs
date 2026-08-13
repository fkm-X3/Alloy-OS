//! Safe PL110 LCD controller driver (aarch64).
//!
//! Replaces `ported/aarch64/drivers/pl110.rs`. Provides the `Pl110` facade
//! over the QEMU `virt` PL110 memory-mapped registers. The C-ABI entry
//! points are kept because the aarch64 boot main and the
//! kernel graphics module still reference the `pl110_*` symbols.

use crate::io::{DefaultMmio, Mmio};

const PL110_BASE: usize = 0x1E20_0000;

// Register offsets.
const LCDTIMING0: usize = 0x00;
const LCDTIMING1: usize = 0x04;
const LCDTIMING2: usize = 0x08;
const LCDTIMING3: usize = 0x0c;
const LCDUPBASE: usize = 0x10;
const LCDCONTROL: usize = 0x18;
const LCDICR: usize = 0x28;

// LCDCONTROL bits.
const LCDCTL_ENABLE: u32 = 1 << 0;
const LCDCTL_LCDPWR: u32 = 1 << 11;
const LCDCTL_LCDBPP16: u32 = 1 << 1;
const LCDCTL_TFT: u32 = 1 << 5;

static mut framebuffer_phys: u32 = 0;
static mut fb_width: u32 = 1024;
static mut fb_height: u32 = 768;
static mut fb_bpp: u8 = 16;
static mut pl110_initialized: i32 = 0;

#[inline]
fn mmio_write(offset: usize, value: u32) {
    unsafe { <DefaultMmio as Mmio>::write32(PL110_BASE + offset, value) };
}

/// Safe PL110 framebuffer controller facade (aarch64 only).
pub struct Pl110;

impl Pl110 {
    /// Initialize the controller with the given physical framebuffer address
    /// and resolution. `fb_addr` must point to at least `width*height*2`
    /// bytes of physical memory (16 bpp). Matches the C `pl110_init`.
    pub fn init(fb_addr: u32, width: u16, height: u16) {
        let width_u32 = width as u32;
        let height_u32 = height as u32;
        unsafe {
            fb_width = width_u32;
            fb_height = height_u32;
            fb_bpp = 16;
            framebuffer_phys = fb_addr;
        }

        mmio_write(LCDCONTROL, 0);

        let ppl = width_u32.wrapping_sub(1); // Pixels per line - 1
        let hsw: u32 = 40; // Horizontal sync width
        let hfp: u32 = 160; // Horizontal front porch
        let hbp: u32 = 160; // Horizontal back porch
        mmio_write(LCDTIMING0, (hsw << 24) | (ppl << 2));

        let lpp = height_u32.wrapping_sub(1); // Lines per panel - 1
        let vsw: u32 = 6; // Vertical sync width
        let vfp: u32 = 12; // Vertical front porch
        let vbp: u32 = 24; // Vertical back porch
        mmio_write(LCDTIMING1, (vsw << 24) | (lpp << 2));
        mmio_write(LCDTIMING2, (vbp << 8) | vfp);
        mmio_write(LCDTIMING3, (hbp << 8) | hfp);

        mmio_write(LCDUPBASE, fb_addr);

        mmio_write(
            LCDCONTROL,
            LCDCTL_TFT | LCDCTL_LCDBPP16 | LCDCTL_ENABLE | LCDCTL_LCDPWR,
        );
        mmio_write(LCDICR, 0xffff_ffff);

        unsafe { pl110_initialized = 1; }
    }

    /// Whether the controller has been initialized.
    pub fn is_available() -> bool {
        unsafe { pl110_initialized != 0 }
    }

    /// Physical address of the framebuffer (0 if not initialized).
    pub fn framebuffer_addr() -> u32 {
        if !Self::is_available() {
            return 0;
        }
        unsafe { framebuffer_phys }
    }

    /// Current framebuffer resolution `(width, height)` in pixels.
    pub fn resolution() -> (u32, u32) {
        unsafe { (fb_width, fb_height) }
    }

    /// Bits per pixel of the framebuffer (always 16).
    pub fn bits_per_pixel() -> u8 {
        unsafe { fb_bpp }
    }

    /// Write a single 16-bit pixel at `(x, y)` (bounds-checked; no-op when
    /// the controller is uninitialized or `(x, y)` is out of range).
    pub fn set_pixel(x: u16, y: u16, color: u16) {
        if !Self::is_available() {
            return;
        }
        let (width, height) = Self::resolution();
        let x = x as u32;
        let y = y as u32;
        if x >= width || y >= height {
            return;
        }
        let fb = unsafe { framebuffer_phys } as usize as *mut u16;
        unsafe {
            core::ptr::write_volatile(fb.add((y * width + x) as usize), color);
        }
    }

    /// Fill a rectangle `(x, y, w, h)` with a 16-bit color (clipped to the
    /// framebuffer bounds; no-op when the controller is uninitialized).
    pub fn fill_rect(x: u16, y: u16, w: u16, h: u16, color: u16) {
        if !Self::is_available() {
            return;
        }
        let (width, height) = Self::resolution();
        let fb = unsafe { framebuffer_phys } as usize as *mut u16;
        let mut row = y as u32;
        while row < (y as u32).wrapping_add(h as u32) && row < height {
            let mut col = x as u32;
            while col < (x as u32).wrapping_add(w as u32) && col < width {
                unsafe {
                    core::ptr::write_volatile(fb.add((row * width + col) as usize), color);
                }
                col = col.wrapping_add(1);
            }
            row = row.wrapping_add(1);
        }
    }
}

// ============================================================================
// C-ABI entry points kept for surviving callers (aarch64 boot main,
// kernel graphics).
// ============================================================================

/// `pl110_init(fb_addr, width, height)`.
#[no_mangle]
pub extern "C" fn pl110_init(fb_addr: u32, width: u16, height: u16) {
    Pl110::init(fb_addr, width, height);
}

/// `pl110_is_available() -> i32`.
#[no_mangle]
pub extern "C" fn pl110_is_available() -> i32 {
    if Pl110::is_available() { 1 } else { 0 }
}

/// `pl110_get_framebuffer() -> u32`.
#[no_mangle]
pub extern "C" fn pl110_get_framebuffer() -> u32 {
    Pl110::framebuffer_addr()
}

/// `pl110_get_resolution(width, height)`.
#[no_mangle]
pub unsafe extern "C" fn pl110_get_resolution(width: *mut u32, height: *mut u32) {
    let (w, h) = Pl110::resolution();
    if !width.is_null() {
        *width = w;
    }
    if !height.is_null() {
        *height = h;
    }
}

/// `pl110_get_bits_per_pixel() -> u8`.
#[no_mangle]
pub extern "C" fn pl110_get_bits_per_pixel() -> u8 {
    Pl110::bits_per_pixel()
}

/// `pl110_set_pixel(x, y, color)`.
#[no_mangle]
pub extern "C" fn pl110_set_pixel(x: u16, y: u16, color: u16) {
    Pl110::set_pixel(x, y, color);
}

/// `pl110_fill_rect(x, y, w, h, color)`.
#[no_mangle]
pub extern "C" fn pl110_fill_rect(x: u16, y: u16, w: u16, h: u16, color: u16) {
    Pl110::fill_rect(x, y, w, h, color);
}
