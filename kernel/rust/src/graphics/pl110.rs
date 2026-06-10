use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;

use super::framebuffer::{Framebuffer, FramebufferInfo};
use super::{Display, FramebufferBuffer as FramebufferBufferTrait};
use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pl110Error {
    Pl110NotAvailable,
    FramebufferNotAvailable,
    InvalidFramebufferInfo,
    InvalidOperation,
}

pub struct Pl110Buffer {
    address: *mut u8,
    pitch: u32,
    size: usize,
}

impl Debug for Pl110Buffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Pl110Buffer")
            .field("address", &self.address)
            .field("pitch", &self.pitch)
            .field("size", &self.size)
            .finish()
    }
}

impl FramebufferBufferTrait for Pl110Buffer {
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

pub struct Pl110Display {
    framebuffer: Framebuffer,
    buffer: Pl110Buffer,
    back_buffer: Vec<u32>,
    dirty: bool,
}

impl Debug for Pl110Display {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Pl110Display")
            .field("framebuffer", &self.framebuffer)
            .field("back_buffer_pixels", &self.back_buffer.len())
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl Pl110Display {
    pub fn new() -> Option<Self> {
        if unsafe { ffi::pl110_is_available() } == 0 {
            return None;
        }

        let fb_addr = unsafe { ffi::pl110_get_framebuffer() };
        if fb_addr == 0 {
            return None;
        }

        let mut width: u16 = 0;
        let mut height: u16 = 0;
        unsafe {
            ffi::pl110_get_resolution(&mut width, &mut height);
        }

        let bpp = unsafe { ffi::pl110_get_bits_per_pixel() };

        if width == 0 || height == 0 || bpp == 0 {
            return None;
        }

        let scanline_bytes = (width as u32).saturating_mul((bpp as u32) / 8);
        let fb_size = scanline_bytes.saturating_mul(height as u32);

        let (red_mask, green_mask, blue_mask) = match bpp {
            16 => (0xF800, 0x07E0, 0x001F),
            _ => return None,
        };

        let fb_info = FramebufferInfo::new(
            fb_addr,
            width as u32,
            height as u32,
            scanline_bytes,
            bpp,
            red_mask,
            green_mask,
            blue_mask,
        )
        .ok()?;

        let framebuffer = Framebuffer::new(fb_info).ok()?;
        let mapped_size = framebuffer.size().ok()?;

        let buffer = Pl110Buffer {
            address: fb_addr as *mut u8,
            pitch: scanline_bytes,
            size: mapped_size,
        };

        let pixel_count = (width as usize).checked_mul(height as usize)?;
        let back_buffer = vec![0u32; pixel_count];

        Some(Pl110Display {
            framebuffer,
            buffer,
            back_buffer,
            dirty: true,
        })
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

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    fn present_back_buffer(&mut self) -> Result<(), Pl110Error> {
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
                _ => return Err(Pl110Error::InvalidOperation),
            }
        }

        Ok(())
    }
}

impl Display for Pl110Display {
    type Error = Pl110Error;
    type Buffer = Pl110Buffer;

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
            return Err(Pl110Error::InvalidOperation);
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
