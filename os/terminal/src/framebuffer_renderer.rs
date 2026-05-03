//! Framebuffer renderer for headless UI rendering
//!
//! Provides a simple pixel-based renderer that can output Iced UI components
//! to a framebuffer buffer for use in both desktop and kernel environments.

use std::f32::consts::PI;

/// Simple ARGB8888 color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_u32(pixel: u32) -> Self {
        Color {
            a: (pixel >> 24) as u8,
            r: (pixel >> 16) as u8,
            g: (pixel >> 8) as u8,
            b: pixel as u8,
        }
    }

    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const DARK_GRAY: Color = Color { r: 64, g: 64, b: 64, a: 255 };
}

/// Framebuffer for pixel rendering
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Framebuffer {
            width,
            height,
            pixels: vec![0u32; size],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    fn pixel_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if let Some(idx) = self.pixel_index(x, y) {
            self.pixels[idx] = color.to_u32();
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        self.pixel_index(x, y).map(|idx| Color::from_u32(self.pixels[idx]))
    }

    /// Blend a color into existing pixel with alpha
    pub fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        if let Some(idx) = self.pixel_index(x, y) {
            let dst = Color::from_u32(self.pixels[idx]);
            let a = color.a as f32 / 255.0;
            let ia = 1.0 - a;

            let r = (color.r as f32 * a + dst.r as f32 * ia) as u8;
            let g = (color.g as f32 * a + dst.g as f32 * ia) as u8;
            let b = (color.b as f32 * a + dst.b as f32 * ia) as u8;

            self.pixels[idx] = Color::from_rgb(r, g, b).to_u32();
        }
    }

    pub fn clear(&mut self, color: Color) {
        let pixel = color.to_u32();
        for p in &mut self.pixels {
            *p = pixel;
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x + dx, y + dy, color);
            }
        }
    }

    pub fn stroke_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color, thickness: u32) {
        // Top and bottom
        for i in 0..w {
            for t in 0..thickness {
                self.set_pixel(x + i, y + t, color);
                if y + h > t {
                    self.set_pixel(x + i, y + h - t - 1, color);
                }
            }
        }
        // Left and right
        for i in 0..h {
            for t in 0..thickness {
                self.set_pixel(x + t, y + i, color);
                if x + w > t {
                    self.set_pixel(x + w - t - 1, y + i, color);
                }
            }
        }
    }

    pub fn circle(&mut self, cx: u32, cy: u32, radius: u32, color: Color, filled: bool) {
        if filled {
            for y in 0..=(2 * radius) {
                let dy = (y as i32 - radius as i32).abs();
                if dy > radius as i32 {
                    continue;
                }
                let dx = ((radius as i32 * radius as i32 - dy * dy) as f32).sqrt() as u32;

                for x in 0..=dx {
                    let px1 = cx.saturating_add(radius).saturating_sub(dx).saturating_add(x);
                    let px2 = cx.saturating_add(radius).saturating_sub(dx).saturating_add(x);
                    let py1 = cy.saturating_add(y).saturating_sub(radius);
                    let py2 = cy.saturating_add(radius).saturating_sub(y);

                    if px1 < self.width && py1 < self.height {
                        self.set_pixel(px1, py1, color);
                    }
                    if px2 < self.width && py2 < self.height {
                        self.set_pixel(px2, py2, color);
                    }
                }
            }
        } else {
            // Bresenham circle outline
            let mut x = radius as i32;
            let mut y = 0i32;
            let mut d = 3 - 2 * radius as i32;

            while x >= y {
                let points = [
                    (x, y),
                    (y, x),
                    (-y, x),
                    (-x, y),
                    (-x, -y),
                    (-y, -x),
                    (y, -x),
                    (x, -y),
                ];

                for (dx, dy) in &points {
                    let px = (cx as i32 + dx) as u32;
                    let py = (cy as i32 + dy) as u32;
                    if px < self.width && py < self.height {
                        self.set_pixel(px, py, color);
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

    pub fn draw_line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = (y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = (dx - dy) as i32;
        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                self.set_pixel(x as u32, y as u32, color);
            }

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -(dy as i32) {
                err -= dy as i32;
                x += sx;
            }
            if e2 < dx as i32 {
                err += dx as i32;
                y += sy;
            }
        }
    }

    /// Draw a filled triangle
    pub fn fill_triangle(
        &mut self,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        color: Color,
    ) {
        let min_y = y0.min(y1).min(y2);
        let max_y = y0.max(y1).max(y2);

        for y in min_y..=max_y {
            let y_f = y as f32;

            let (x_start, x_end) = {
                let mut xs = vec![];
                self.line_intersection(x0, y0, x1, y1, y_f, &mut xs);
                self.line_intersection(x1, y1, x2, y2, y_f, &mut xs);
                self.line_intersection(x2, y2, x0, y0, y_f, &mut xs);

                if xs.is_empty() {
                    continue;
                }
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                (xs[0] as u32, xs[xs.len() - 1] as u32)
            };

            for x in x_start..=x_end {
                self.set_pixel(x, y, color);
            }
        }
    }

    fn line_intersection(&self, x0: u32, y0: u32, x1: u32, y1: u32, y: f32, result: &mut Vec<f32>) {
        let y0_f = y0 as f32;
        let y1_f = y1 as f32;

        if (y >= y0_f && y <= y1_f) || (y <= y0_f && y >= y1_f) {
            if (y1_f - y0_f).abs() < 0.001 {
                result.push(x0 as f32);
            } else {
                let t = (y - y0_f) / (y1_f - y0_f);
                let x = x0 as f32 + t * (x1 as f32 - x0 as f32);
                result.push(x);
            }
        }
    }

    /// Draw rotated rectangle (for rotated SVG images)
    pub fn draw_rotated_rect(
        &mut self,
        cx: u32,
        cy: u32,
        w: u32,
        h: u32,
        angle_deg: f32,
        color: Color,
    ) {
        let angle_rad = angle_deg * PI / 180.0;
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let w_f = w as f32 / 2.0;
        let h_f = h as f32 / 2.0;

        let corners = [
            (-w_f, -h_f),
            (w_f, -h_f),
            (w_f, h_f),
            (-w_f, h_f),
        ];

        let mut rotated = Vec::new();
        for (x, y) in corners {
            let rx = x * cos_a - y * sin_a;
            let ry = x * sin_a + y * cos_a;
            rotated.push(((cx as f32 + rx) as u32, (cy as f32 + ry) as u32));
        }

        // Draw filled rotated rect as triangles
        if rotated.len() == 4 {
            self.fill_triangle(rotated[0].0, rotated[0].1, rotated[1].0, rotated[1].1, rotated[2].0, rotated[2].1, color);
            self.fill_triangle(rotated[0].0, rotated[0].1, rotated[2].0, rotated[2].1, rotated[3].0, rotated[3].1, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let fb = Framebuffer::new(800, 600);
        assert_eq!(fb.width(), 800);
        assert_eq!(fb.height(), 600);
    }

    #[test]
    fn test_pixel_access() {
        let mut fb = Framebuffer::new(10, 10);
        let color = Color::RED;
        fb.set_pixel(5, 5, color);
        assert_eq!(fb.get_pixel(5, 5), Some(color));
    }
}
