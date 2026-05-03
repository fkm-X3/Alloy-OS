//! Iced Framebuffer Renderer - Converts Iced UI to framebuffer pixels
//!
//! Provides a simple renderer that can draw Iced components directly to
//! framebuffer pixel buffers. Uses basic primitives for text and shapes.

use crate::fusion::terminal::TerminalSurface;

/// Simple color representation (ARGB8888)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create color from ARGB
    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Create color from RGB (fully opaque)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    /// Convert to u32 pixel format
    pub fn to_pixel(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Common colors
    pub fn black() -> Color {
        Color { r: 0, g: 0, b: 0, a: 255 }
    }

    pub fn white() -> Color {
        Color { r: 255, g: 255, b: 255, a: 255 }
    }

    pub fn red() -> Color {
        Color { r: 255, g: 0, b: 0, a: 255 }
    }

    pub fn green() -> Color {
        Color { r: 0, g: 255, b: 0, a: 255 }
    }

    pub fn blue() -> Color {
        Color { r: 0, g: 0, b: 255, a: 255 }
    }

    pub fn cyan() -> Color {
        Color { r: 0, g: 255, b: 255, a: 255 }
    }

    pub fn magenta() -> Color {
        Color { r: 255, g: 0, b: 255, a: 255 }
    }

    pub fn yellow() -> Color {
        Color { r: 255, g: 255, b: 0, a: 255 }
    }

    pub fn light_gray() -> Color {
        Color { r: 200, g: 200, b: 200, a: 255 }
    }

    pub fn dark_gray() -> Color {
        Color { r: 64, g: 64, b: 64, a: 255 }
    }
}

/// Simple framebuffer renderer for UI drawing
pub struct FramebufferRenderer {
    width: u32,
    height: u32,
    pixels: alloc::vec::Vec<u32>,
}

impl FramebufferRenderer {
    /// Create a new renderer for a given surface size
    pub fn new(width: u32, height: u32) -> Result<Self, crate::fusion::backend::FusionError> {
        if width == 0 || height == 0 {
            return Err(crate::fusion::backend::FusionError::InvalidDimensions);
        }
        
        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(crate::fusion::backend::FusionError::InvalidDimensions)?;

        Ok(FramebufferRenderer {
            width,
            height,
            pixels: alloc::vec![0u32; pixel_count],
        })
    }

    /// Get surface dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get pixel buffer
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Get mutable pixel buffer
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    /// Clear the entire surface with a color
    pub fn clear(&mut self, color: Color) {
        let pixel = color.to_pixel();
        for p in &mut self.pixels {
            *p = pixel;
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        let pixel = color.to_pixel();
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
                    self.pixels[idx] = pixel;
                }
            }
        }
    }

    /// Draw a rectangle outline
    pub fn stroke_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color, thickness: u32) {
        let pixel = color.to_pixel();
        
        // Top and bottom edges
        for i in 0..width {
            for t in 0..thickness {
                self.set_pixel(x + i, y + t, pixel); // Top
                if y + height > t {
                    self.set_pixel(x + i, y + height - t - 1, pixel); // Bottom
                }
            }
        }

        // Left and right edges
        for i in 0..height {
            for t in 0..thickness {
                self.set_pixel(x + t, y + i, pixel); // Left
                if x + width > t {
                    self.set_pixel(x + width - t - 1, y + i, pixel); // Right
                }
            }
        }
    }

    /// Set a single pixel
    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color;
            }
        }
    }

    /// Draw a horizontal line
    pub fn h_line(&mut self, x1: u32, x2: u32, y: u32, color: Color, thickness: u32) {
        for i in x1..=x2 {
            for t in 0..thickness {
                self.set_pixel(i, y + t, color.to_pixel());
            }
        }
    }

    /// Draw a vertical line
    pub fn v_line(&mut self, x: u32, y1: u32, y2: u32, color: Color, thickness: u32) {
        for i in y1..=y2 {
            for t in 0..thickness {
                self.set_pixel(x + t, i, color.to_pixel());
            }
        }
    }

    /// Draw a circle (Bresenham's algorithm)
    pub fn circle(&mut self, cx: u32, cy: u32, radius: u32, color: Color, filled: bool) {
        let pixel = color.to_pixel();
        
        if filled {
            // Filled circle using scanline
            for y in 0..=(2 * radius) {
                let dy = (y as i32 - radius as i32).abs();
                if dy > radius as i32 {
                    continue;
                }
                
                // Integer square root approximation (no_std compatible)
                let dy2 = dy as u32;
                let r2 = radius * radius;
                let dy2_sq = dy2 * dy2;
                let dx_sq = r2.saturating_sub(dy2_sq);
                let dx = integer_sqrt(dx_sq);
                
                for x in 0..=dx {
                    let px = cx.saturating_add(radius).saturating_sub(dx).saturating_add(x);
                    let py = cy.saturating_add(y).saturating_sub(radius);
                    if px < self.width && py < self.height {
                        self.set_pixel(px, py, pixel);
                    }
                    
                    let px2 = cx.saturating_add(radius).saturating_sub(dx).saturating_add(x);
                    let py2 = cy.saturating_add(radius).saturating_sub(y);
                    if px2 < self.width && py2 < self.height {
                        self.set_pixel(px2, py2, pixel);
                    }
                }
            }
        } else {
            // Circle outline using Bresenham
            let mut x = radius as i32;
            let mut y = 0i32;
            let mut d = 3 - 2 * radius as i32;

            while x >= y {
                // Draw 8 symmetric points
                let points = [
                    (cx as i32 + x, cy as i32 + y),
                    (cx as i32 + y, cy as i32 + x),
                    (cx as i32 - y, cy as i32 + x),
                    (cx as i32 - x, cy as i32 + y),
                    (cx as i32 - x, cy as i32 - y),
                    (cx as i32 - y, cy as i32 - x),
                    (cx as i32 + y, cy as i32 - x),
                    (cx as i32 + x, cy as i32 - y),
                ];

                for (px, py) in &points {
                    if *px >= 0 && (*px as u32) < self.width && *py >= 0 && (*py as u32) < self.height {
                        self.set_pixel(*px as u32, *py as u32, pixel);
                    }
                }

                if d < 0 {
                    d = d + 4 * y + 6;
                } else {
                    d = d + 4 * (y - x) + 10;
                    x -= 1;
                }
                y += 1;
            }
        }
    }
}

/// Integer square root for no_std compatibility
#[inline]
fn integer_sqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

impl Default for FramebufferRenderer {
    fn default() -> Self {
        Self::new(1024, 768).expect("Failed to create default framebuffer renderer")
    }
}
