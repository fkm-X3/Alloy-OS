//! Terminal Surface Integration - Wrapper for rendering terminal UI to framebuffer
//!
//! Provides a Surface abstraction that allows Iced UI components to render
//! directly to kernel framebuffer memory.

use alloc::vec::Vec;

/// Terminal Surface - framebuffer wrapper for UI rendering
///
/// Provides an interface for rendering Iced UI elements to a fixed framebuffer.
/// Supports text rendering, rectangles, and color compositing.
#[derive(Debug)]
pub struct TerminalSurface {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl TerminalSurface {
    /// Create a new terminal surface
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        
        TerminalSurface {
            width,
            height,
            pixels: alloc::vec![0x000000u32; pixel_count],
        }
    }

    /// Get surface dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get mutable pixel buffer
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    /// Get pixel buffer
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Clear surface with color
    pub fn clear(&mut self, color: u32) {
        for pixel in &mut self.pixels {
            *pixel = color;
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        for row in 0..height {
            if y + row >= self.height {
                break;
            }
            for col in 0..width {
                if x + col >= self.width {
                    break;
                }
                let idx = ((y + row) * self.width + (x + col)) as usize;
                if idx < self.pixels.len() {
                    self.pixels[idx] = color;
                }
            }
        }
    }

    /// Draw a line (simple horizontal or vertical)
    pub fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
        if y1 == y2 {
            // Horizontal line
            let start_x = x1.min(x2);
            let end_x = x1.max(x2);
            for x in start_x..=end_x {
                if x < self.width && y1 < self.height {
                    let idx = (y1 * self.width + x) as usize;
                    if idx < self.pixels.len() {
                        self.pixels[idx] = color;
                    }
                }
            }
        } else if x1 == x2 {
            // Vertical line
            let start_y = y1.min(y2);
            let end_y = y1.max(y2);
            for y in start_y..=end_y {
                if x1 < self.width && y < self.height {
                    let idx = (y * self.width + x1) as usize;
                    if idx < self.pixels.len() {
                        self.pixels[idx] = color;
                    }
                }
            }
        }
    }

    /// Set a single pixel
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color;
            }
        }
    }

    /// Blend a color onto the surface (simple alpha blending)
    pub fn blend_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32, alpha: u8) {
        let alpha = alpha as u32;
        let inv_alpha = 255 - alpha;

        for row in 0..height {
            if y + row >= self.height {
                break;
            }
            for col in 0..width {
                if x + col >= self.width {
                    break;
                }
                let idx = ((y + row) * self.width + (x + col)) as usize;
                if idx < self.pixels.len() {
                    let existing = self.pixels[idx];
                    
                    // Extract RGB components
                    let r1 = (existing >> 16) & 0xFF;
                    let g1 = (existing >> 8) & 0xFF;
                    let b1 = existing & 0xFF;

                    let r2 = (color >> 16) & 0xFF;
                    let g2 = (color >> 8) & 0xFF;
                    let b2 = color & 0xFF;

                    // Blend
                    let r = ((r1 * inv_alpha + r2 * alpha) / 255) & 0xFF;
                    let g = ((g1 * inv_alpha + g2 * alpha) / 255) & 0xFF;
                    let b = ((b1 * inv_alpha + b2 * alpha) / 255) & 0xFF;

                    self.pixels[idx] = (r << 16) | (g << 8) | b;
                }
            }
        }
    }
}

impl Default for TerminalSurface {
    fn default() -> Self {
        Self::new(1024, 768)
    }
}
