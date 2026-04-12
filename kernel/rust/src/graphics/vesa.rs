//! VESA VBE Graphics Display Implementation
//!
//! Provides a Display implementation for VESA VBE graphics modes,
//! allowing the graphics layer to work with hardware-accelerated displays.

use core::ffi::c_void;
use core::fmt::Debug;

use super::framebuffer::{Framebuffer, FramebufferInfo};
use super::{Display, FramebufferBuffer as FramebufferBufferTrait};
use crate::ffi;

const PAGE_SIZE: u32 = 4096;
const IDENTITY_MAP_LIMIT: u32 = 0x0100_0000;

fn map_framebuffer_for_kernel_access(fb_addr: u32, fb_size: u32) -> Option<u32> {
    if fb_size == 0 {
        return None;
    }

    let page_mask = PAGE_SIZE - 1;
    let start_page = fb_addr & !page_mask;
    let start_offset = fb_addr.wrapping_sub(start_page);
    let mapped_span = start_offset.checked_add(fb_size)?;
    let page_count = (mapped_span.saturating_add(page_mask)) / PAGE_SIZE;
    let end_page = start_page.checked_add(page_count.checked_mul(PAGE_SIZE)?)?;

    if end_page <= IDENTITY_MAP_LIMIT {
        return Some(fb_addr);
    }

    let mut page = start_page;
    while page < end_page {
        let virt = page as usize as *mut c_void;
        let phys = page as usize as *mut c_void;
        let mapped = unsafe { ffi::vmm_map(virt, phys, ffi::PAGE_PRESENT | ffi::PAGE_WRITE) };
        if !mapped {
            return None;
        }
        page = page.checked_add(PAGE_SIZE)?;
    }

    Some(fb_addr)
}

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
    /// Initializes VESA and creates a display wrapper for the active framebuffer.
    /// Returns None if VESA metadata or framebuffer mapping is unavailable.
    pub fn new() -> Option<Self> {
        // Initialize VESA
        ffi::vesa_initialize();

        // Check if VESA is available
        if !ffi::vesa_available() {
            return None;
        }

        // Get framebuffer address
        let fb_addr = ffi::vesa_framebuffer_addr()?;
        let (width, height) = ffi::vesa_display_resolution();
        let bpp = ffi::vesa_color_depth();
        let scanline_bytes = ffi::vesa_scanline_bytes();
        let fb_size = ffi::vesa_buffer_size();

        // Validate resolution
        if width == 0 || height == 0 || bpp == 0 || scanline_bytes == 0 || fb_size == 0 {
            return None;
        }

        let mapped_fb_addr = map_framebuffer_for_kernel_access(fb_addr, fb_size)?;

        // Create framebuffer info
        let (red_mask, green_mask, blue_mask) = match bpp {
            16 => (0xF800, 0x07E0, 0x001F),
            24 => (0xFF0000, 0x00FF00, 0x0000FF),
            32 => (0xFF0000, 0x00FF00, 0x0000FF),
            _ => return None,
        };

        let fb_info = FramebufferInfo::new(
            mapped_fb_addr,
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
        let mapped_size = framebuffer.size().ok()?;

        let buffer = VesaBuffer {
            address: mapped_fb_addr as *mut u8,
            pitch: scanline_bytes as u32,
            size: mapped_size,
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
