//! Terminal Surface Integration - Wrapper for rendering terminal UI to framebuffer
//!
//! Provides a Surface abstraction that allows terminal rendering directly
//! to kernel framebuffer memory. Supports character-based dimensions that
//! map to pixel coordinates using a 5x7 bitmap font.

use alloc::vec::Vec;
use crate::terminal::Terminal;
use super::backend::FusionError;

// Font metrics: 5x7 characters with 9-pixel line height
const CHAR_WIDTH_PIXELS: u32 = 5;
const CHAR_HEIGHT_PIXELS: u32 = 7;
const LINE_HEIGHT_PIXELS: u32 = 9;

/// Simple surface wrapper for pixel buffer
#[derive(Debug)]
pub struct Surface {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
}

impl Surface {
    fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Surface {
            pixels: alloc::vec![0u32; size],
            width,
            height,
        }
    }

    pub fn get_buffer(&self) -> &[u32] {
        &self.pixels
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color;
            }
        }
    }
}

/// Terminal Surface - framebuffer wrapper for UI rendering
///
/// Provides an interface for rendering terminal and UI elements to a fixed framebuffer.
/// Maps character-based coordinates to pixel coordinates using 5x7 bitmap font metrics.
#[derive(Debug)]
pub struct TerminalSurface {
    width_pixels: u32,
    height_pixels: u32,
    pixels: Vec<u32>,
    surface: Surface,
    _terminal: *mut Terminal,
}

impl TerminalSurface {
    /// Create a new terminal surface from character dimensions
    ///
    /// Converts character dimensions (e.g., 80x25) to pixel dimensions using
    /// 5x7 font metrics with 9-pixel line height.
    pub fn new(terminal: &mut Terminal, width_chars: u32, height_chars: u32) -> Result<Self, FusionError> {
        // Convert character dimensions to pixels
        let width_pixels = width_chars * CHAR_WIDTH_PIXELS;
        let height_pixels = height_chars * LINE_HEIGHT_PIXELS;

        if width_pixels == 0 || height_pixels == 0 {
            return Err(FusionError::InvalidDimensions);
        }

        let pixel_count = (width_pixels * height_pixels) as usize;
        
        Ok(TerminalSurface {
            width_pixels,
            height_pixels,
            pixels: alloc::vec![0x000000u32; pixel_count],
            surface: Surface::new(width_pixels, height_pixels),
            _terminal: terminal as *mut Terminal,
        })
    }

    /// Get surface dimensions in pixels
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width_pixels, self.height_pixels)
    }

    /// Get surface dimensions (compatibility method)
    pub fn get_surface_dimensions(&self) -> (u32, u32) {
        (self.width_pixels, self.height_pixels)
    }

    /// Get mutable pixel buffer
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    /// Get pixel buffer
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Set a pixel to the given color
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        self.surface.set_pixel(x, y, color);
        if x < self.width_pixels && y < self.height_pixels {
            let idx = (y * self.width_pixels + x) as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color;
            }
        }
    }

    /// Get underlying surface (compatibility method)
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Mark surface as dirty (no-op for now)
    pub fn mark_full_dirty(&mut self) {
        // Placeholder - would normally track dirty regions
    }

    /// Render surface (syncs pixels to display)
    pub fn render(&mut self) -> Result<(), ()> {
        // Sync pixels to surface for display_server compatibility
        if self.surface.pixels.len() == self.pixels.len() {
            self.surface.pixels.copy_from_slice(&self.pixels);
        }
        Ok(())
    }

    /// Clear surface with color
    pub fn clear(&mut self, color: u32) {
        for pixel in &mut self.pixels {
            *pixel = color;
        }
    }

    /// Handle terminal input
    pub fn handle_input(&mut self, key: u8) -> Result<(), ()> {
        if !self._terminal.is_null() {
            unsafe {
                (*self._terminal).handle_input(key);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Fill a rectangle with color
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        for row in 0..height {
            if y + row >= self.height_pixels {
                break;
            }
            for col in 0..width {
                if x + col >= self.width_pixels {
                    break;
                }
                let idx = ((y + row) * self.width_pixels + (x + col)) as usize;
                if idx < self.pixels.len() {
                    self.pixels[idx] = color;
                }
            }
        }
    }
}
