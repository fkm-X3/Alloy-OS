//! VESA VBE Graphics Display Implementation
//!
//! Provides a Display implementation for VESA VBE graphics modes,
//! allowing the graphics layer to work with hardware-accelerated displays.

use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::fmt::Debug;

use super::framebuffer::{Framebuffer, FramebufferInfo};
use super::{Display, FramebufferBuffer as FramebufferBufferTrait};
use crate::ffi;

const PAGE_SIZE: u64 = 4096;
const IDENTITY_MAP_LIMIT: u64 = 0x0100_0000;

fn map_framebuffer_for_kernel_access(fb_addr: u64, fb_size: u64) -> Option<u64> {
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
    back_buffer: Vec<u32>,
    dirty: bool,
}

impl Debug for VesaDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VesaDisplay")
            .field("framebuffer", &self.framebuffer)
            .field("back_buffer_pixels", &self.back_buffer.len())
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

        let pixel_count = (width as usize).checked_mul(height as usize)?;
        let back_buffer = vec![0u32; pixel_count];

        Some(VesaDisplay {
            framebuffer,
            buffer,
            back_buffer,
            dirty: true,
        })
    }

    /// Get the underlying framebuffer
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    #[inline]
    fn back_index(&self, x: u32, y: u32) -> Option<usize> {
        let width = self.framebuffer.width();
        let height = self.framebuffer.height();
        if x >= width || y >= height {
            return None;
        }

        let row_start = (y as usize).checked_mul(width as usize)?;
        row_start.checked_add(x as usize)
    }

    fn present_back_buffer(&mut self) -> Result<(), VesaError> {
        let width = self.framebuffer.width() as usize;
        let height = self.framebuffer.height() as usize;
        let pitch = self.framebuffer.pitch() as usize;
        let bpp = self.framebuffer.bits_per_pixel();
        let base = self.framebuffer.as_raw_ptr();

        unsafe {
            match bpp {
                32 => {
                    for row in 0..height {
                        let dst = base.add(row.saturating_mul(pitch)) as *mut u32;
                        let src_offset = row.saturating_mul(width);
                        let src = &self.back_buffer[src_offset..src_offset.saturating_add(width)];
                        core::ptr::copy_nonoverlapping(src.as_ptr(), dst, width);
                    }
                }
                24 => {
                    for row in 0..height {
                        let row_dst = base.add(row.saturating_mul(pitch));
                        let src_offset = row.saturating_mul(width);
                        for col in 0..width {
                            let color = self.back_buffer[src_offset + col];
                            let native = self.framebuffer.convert_color(color);
                            let pixel_dst = row_dst.add(col.saturating_mul(3));
                            *pixel_dst = (native & 0xFF) as u8;
                            *pixel_dst.add(1) = ((native >> 8) & 0xFF) as u8;
                            *pixel_dst.add(2) = ((native >> 16) & 0xFF) as u8;
                        }
                    }
                }
                16 => {
                    for row in 0..height {
                        let dst = base.add(row.saturating_mul(pitch)) as *mut u16;
                        let src_offset = row.saturating_mul(width);
                        for col in 0..width {
                            let color = self.back_buffer[src_offset + col];
                            let native = self.framebuffer.convert_color(color);
                            *dst.add(col) = (native & 0xFFFF) as u16;
                        }
                    }
                }
                8 => {
                    for row in 0..height {
                        let dst = base.add(row.saturating_mul(pitch));
                        let src_offset = row.saturating_mul(width);
                        for col in 0..width {
                            let color = self.back_buffer[src_offset + col];
                            let native = self.framebuffer.convert_color(color);
                            *dst.add(col) = (native & 0xFF) as u8;
                        }
                    }
                }
                _ => return Err(VesaError::InvalidOperation),
            }
        }

        Ok(())
    }
}

impl Display for VesaDisplay {
    type Error = VesaError;
    type Buffer = VesaBuffer;

    fn pixel_put(&mut self, x: u32, y: u32, color: u32) {
        if let Some(index) = self.back_index(x, y) {
            self.back_buffer[index] = color;
            self.dirty = true;
        }
    }

    fn clear(&mut self, color: u32) {
        self.back_buffer.fill(color);
        self.dirty = true;
    }

    fn swap_buffer(&mut self) {
        if self.dirty {
            let _ = self.present_back_buffer();
        }
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
        let fb_width = self.framebuffer.width();
        let fb_height = self.framebuffer.height();
        if x >= fb_width || y >= fb_height {
            return Err(VesaError::InvalidOperation);
        }

        let x_end = x.saturating_add(width).min(fb_width);
        let y_end = y.saturating_add(height).min(fb_height);
        let row_width = (x_end.saturating_sub(x)) as usize;

        for row in y..y_end {
            let row_start = (row as usize)
                .saturating_mul(fb_width as usize)
                .saturating_add(x as usize);
            let row_end = row_start.saturating_add(row_width);
            if row_end <= self.back_buffer.len() {
                self.back_buffer[row_start..row_end].fill(color);
            }
        }

        self.dirty = true;
        Ok(())
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}
