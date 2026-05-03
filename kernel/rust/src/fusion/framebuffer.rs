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

/// Simple framebuffer renderer for Iced UI
pub struct FramebufferRenderer {
    surface: TerminalSurface,
}

impl FramebufferRenderer {
    /// Create a new renderer for a given surface size
    pub fn new(width: u32, height: u32) -> Result<Self, crate::fusion::backend::FusionError> {
        let surface = TerminalSurface::new(width, height);
        Ok(FramebufferRenderer { surface })
    }

    /// Get mutable reference to underlying surface
    pub fn surface_mut(&mut self) -> &mut TerminalSurface {
        &mut self.surface
    }

    /// Get reference to underlying surface
    pub fn surface(&self) -> &TerminalSurface {
        &self.surface
    }

    /// Clear the entire surface with a color
    pub fn clear(&mut self, color: Color) {
        self.surface.clear(color.to_pixel());
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        self.surface.fill_rect(x, y, width, height, color.to_pixel());
    }

    /// Draw a rectangle outline
    pub fn stroke_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color, thickness: u32) {
        let pixel = color.to_pixel();
        
        // Top and bottom edges
        for i in 0..width {
            for t in 0..thickness {
                self.surface.set_pixel(x + i, y + t, pixel); // Top
                if y + height > t {
                    self.surface.set_pixel(x + i, y + height - t - 1, pixel); // Bottom
                }
            }
        }

        // Left and right edges
        for i in 0..height {
            for t in 0..thickness {
                self.surface.set_pixel(x + t, y + i, pixel); // Left
                if x + width > t {
                    self.surface.set_pixel(x + width - t - 1, y + i, pixel); // Right
                }
            }
        }
    }

    /// Draw a horizontal line
    pub fn h_line(&mut self, x1: u32, x2: u32, y: u32, color: Color, thickness: u32) {
        for i in x1..=x2 {
            for t in 0..thickness {
                self.surface.set_pixel(i, y + t, color.to_pixel());
            }
        }
    }

    /// Draw a vertical line
    pub fn v_line(&mut self, x: u32, y1: u32, y2: u32, color: Color, thickness: u32) {
        for i in y1..=y2 {
            for t in 0..thickness {
                self.surface.set_pixel(x + t, i, color.to_pixel());
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
                let dx = ((radius as i32 * radius as i32 - dy * dy) as f32).sqrt() as u32;
                
                for x in 0..=dx {
                    if cx + radius + x < 2048 && cy + y < 2048 {
                        self.surface.set_pixel(cx + radius - dx + x, cy + y - radius, pixel);
                    }
                    if cx + radius + x < 2048 && cy + radius > y {
                        self.surface.set_pixel(cx + radius - dx + x, cy + radius - y, pixel);
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
                if cx + x as u32 < 2048 && cy + y as u32 < 2048 {
                    self.surface.set_pixel(cx + x as u32, cy + y as u32, pixel);
                }
                if cx + y as u32 < 2048 && cy + x as u32 < 2048 {
                    self.surface.set_pixel(cx + y as u32, cy + x as u32, pixel);
                }
                if cy + x as u32 < 2048 && x >= 0 {
                    self.surface.set_pixel(cx - x as u32, cy + y as u32, pixel);
                }
                if cy + y as u32 < 2048 && y >= 0 {
                    self.surface.set_pixel(cx - y as u32, cy + x as u32, pixel);
                }
                if cy + x as u32 < 2048 && x >= 0 {
                    self.surface.set_pixel(cx + x as u32, cy - y as u32, pixel);
                }
                if cy + y as u32 < 2048 && y >= 0 {
                    self.surface.set_pixel(cx + y as u32, cy - x as u32, pixel);
                }
                if cy + x as u32 < 2048 && x >= 0 {
                    self.surface.set_pixel(cx - x as u32, cy - y as u32, pixel);
                }
                if cy + y as u32 < 2048 && y >= 0 {
                    self.surface.set_pixel(cx - y as u32, cy - x as u32, pixel);
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

    /// Get framebuffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        self.surface.dimensions()
    }

    /// Get pixel buffer
    pub fn pixels(&self) -> &[u32] {
        self.surface.pixels()
    }

    /// Get mutable pixel buffer for direct access
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        self.surface.pixels_mut()
    }
}

impl Default for FramebufferRenderer {
    fn default() -> Self {
        Self::new(1024, 768).expect("Failed to create default framebuffer renderer")
    }
}
