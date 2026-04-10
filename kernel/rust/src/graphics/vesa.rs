//! VESA VBE Graphics Display Implementation
//!
//! Provides a Display implementation for VESA VBE graphics modes,
//! allowing the graphics layer to work with hardware-accelerated displays.

use core::fmt::Debug;

use super::framebuffer::{Framebuffer, FramebufferInfo};
use super::{Display, FramebufferBuffer as FramebufferBufferTrait};
use crate::ffi;

/// Error types for VESA display operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VesaError {
    /// VESA is not available on this system
    VesaNotAvailable,
    /// Failed to set graphics mode
    ModeSetFailed,
    /// Framebuffer not available
    FramebufferNotAvailable,
    /// Invalid framebuffer information
    InvalidFramebufferInfo,
    /// Invalid operation
    InvalidOperation,
}

/// Framebuffer buffer wrapper for VESA
#[derive(Debug)]
pub struct VesaBuffer {
    address: *mut u8,
    pitch: u32,
    size: usize,
}

impl FramebufferBufferTrait for VesaBuffer {
    fn address(&self) -> *mut u8 {
        self.address
    }

    fn pitch(&self) -> u32 {
        self.pitch
    }

    fn size(&self) -> usize {
        self.size
    }
}

/// VESA VBE Graphics Display
///
/// Provides access to VESA graphics modes through the Display trait.
/// Wraps a Framebuffer for hardware access.
pub struct VesaDisplay {
    framebuffer: Framebuffer,
    buffer: VesaBuffer,
    dirty: bool,
}

impl Debug for VesaDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VesaDisplay")
            .field("framebuffer", &self.framebuffer)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl VesaDisplay {
    /// Create a new VESA display
    ///
    /// Initializes VESA, sets a graphics mode, and creates a display wrapper.
    /// Returns None if VESA is not available or mode setup fails.
    pub fn new() -> Option<Self> {
        // Initialize VESA
        ffi::vesa_initialize();

        // Check if VESA is available
        if !ffi::vesa_available() {
            return None;
        }

        // Try to set a graphics mode
        let modes = [0x119u16, 0x118, 0x117, 0x114, 0x110];
        let mut mode_set = false;

        for mode in modes.iter() {
            let (success, _) = ffi::vesa_set_graphics_mode(*mode);
            if success {
                mode_set = true;
                break;
            }
        }

        if !mode_set {
            return None;
        }

        // Get framebuffer address
        let fb_addr = ffi::vesa_framebuffer_addr()?;
        let (width, height) = ffi::vesa_display_resolution();
        let bpp = ffi::vesa_color_depth();
        let scanline_bytes = ffi::vesa_scanline_bytes();

        // Validate resolution
        if width == 0 || height == 0 || bpp == 0 {
            return None;
        }

        // Create framebuffer info
        let (red_mask, green_mask, blue_mask) = match bpp {
            16 => (0xF800, 0x07E0, 0x001F),
            24 => (0xFF0000, 0x00FF00, 0x0000FF),
            32 => (0xFF0000, 0x00FF00, 0x0000FF),
            _ => return None,
        };

        let fb_info = FramebufferInfo::new(
            fb_addr,
            width as u32,
            height as u32,
            scanline_bytes as u32,
            bpp,
            red_mask,
            green_mask,
            blue_mask,
        )
        .ok()?;

        let framebuffer = Framebuffer::new(fb_info).ok()?;
        let fb_size = framebuffer.size().ok()?;

        let buffer = VesaBuffer {
            address: fb_addr as *mut u8,
            pitch: scanline_bytes as u32,
            size: fb_size,
        };

        Some(VesaDisplay {
            framebuffer,
            buffer,
            dirty: true,
        })
    }

    /// Get the underlying framebuffer
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }
}

impl Display for VesaDisplay {
    type Error = VesaError;
    type Buffer = VesaBuffer;

    fn pixel_put(&mut self, x: u32, y: u32, color: u32) {
        if self.framebuffer.put_pixel(x, y, color).is_ok() {
            self.dirty = true;
        }
    }

    fn clear(&mut self, color: u32) {
        if self.framebuffer.clear(color).is_ok() {
            self.dirty = true;
        }
    }

    fn swap_buffer(&mut self) {
        self.dirty = false;
    }

    fn get_resolution(&self) -> (u32, u32) {
        (self.framebuffer.width(), self.framebuffer.height())
    }

    fn get_bits_per_pixel(&self) -> u8 {
        self.framebuffer.bits_per_pixel()
    }

    fn get_buffer(&self) -> &Self::Buffer {
        &self.buffer
    }

    fn fill_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: u32,
    ) -> Result<(), Self::Error> {
        self.framebuffer
            .write_rect(x, y, width, height, color)
            .map_err(|_| VesaError::InvalidOperation)?;
        self.dirty = true;
        Ok(())
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}
