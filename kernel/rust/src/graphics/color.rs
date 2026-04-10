//! Color system for graphics display.
//!
//! This module provides comprehensive color type definitions, conversions, and palette
//! management for use with the Display trait. It abstracts different color formats
//! (8-bit, 16-bit, 24-bit, 32-bit) into a unified interface.
//!
//! # Color Formats Supported
//!
//! - ARGB8888 (32-bit): Internal representation with per-channel alpha
//! - RGB888 (24-bit): 8 bits per channel
//! - RGB565 (16-bit): 5:6:5 format for embedded graphics
//! - Indexed color (8-bit): VGA 16-color and extended palettes
//!
//! # Blending
//!
//! The module provides alpha blending with proper color space handling for correct
//! visual results.

/// Represents an ARGB color value.
///
/// Colors are stored as 32-bit values with 8 bits each for:
/// - Alpha (transparency, MSB)
/// - Red
/// - Green
/// - Blue (LSB)
///
/// Format: 0xAARRGGBB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub value: u32,
}

impl Color {
    /// Create a color from ARGB components.
    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Color {
            value: ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }

    /// Create a color from RGB components (alpha = fully opaque).
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color {
            value: 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }

    /// Create a color from a 32-bit ARGB value.
    pub const fn from_argb32(argb: u32) -> Self {
        Color { value: argb }
    }

    /// Get the alpha component (transparency).
    pub const fn alpha(self) -> u8 {
        (self.value >> 24) as u8
    }

    /// Get the red component.
    pub const fn red(self) -> u8 {
        (self.value >> 16) as u8
    }

    /// Get the green component.
    pub const fn green(self) -> u8 {
        (self.value >> 8) as u8
    }

    /// Get the blue component.
    pub const fn blue(self) -> u8 {
        self.value as u8
    }

    /// Set the alpha component, returning a new color.
    pub const fn set_alpha(self, alpha: u8) -> Self {
        Color {
            value: (self.value & 0x00FFFFFF) | ((alpha as u32) << 24),
        }
    }

    /// Set the red component, returning a new color.
    pub const fn set_red(self, red: u8) -> Self {
        Color {
            value: (self.value & 0xFF00FFFF) | ((red as u32) << 16),
        }
    }

    /// Set the green component, returning a new color.
    pub const fn set_green(self, green: u8) -> Self {
        Color {
            value: (self.value & 0xFFFF00FF) | ((green as u32) << 8),
        }
    }

    /// Set the blue component, returning a new color.
    pub const fn set_blue(self, blue: u8) -> Self {
        Color {
            value: (self.value & 0xFFFFFF00) | (blue as u32),
        }
    }

    /// Convert to RGB565 format (16-bit: 5 bits red, 6 bits green, 5 bits blue).
    pub const fn to_rgb565(self) -> u16 {
        let r = (self.red() >> 3) & 0x1F;
        let g = (self.green() >> 2) & 0x3F;
        let b = (self.blue() >> 3) & 0x1F;
        ((r as u16) << 11) | ((g as u16) << 5) | (b as u16)
    }

    /// Convert to RGB888 format (24-bit: 8 bits each for R, G, B).
    pub const fn to_rgb888(self) -> u32 {
        (self.value >> 8) & 0x00FFFFFF
    }

    /// Convert to ARGB8888 format (32-bit: A, R, G, B).
    pub const fn to_argb8888(self) -> u32 {
        self.value
    }

    /// Convert RGB888 value to Color (ARGB8888 with full opacity).
    pub const fn from_rgb888(rgb: u32) -> Self {
        Color {
            value: 0xFF000000 | (rgb & 0x00FFFFFF),
        }
    }

    /// Convert RGB565 value to Color (ARGB8888 with full opacity).
    pub const fn from_rgb565(rgb565: u16) -> Self {
        let r = (((rgb565 >> 11) & 0x1F) as u32) << 3;
        let g = (((rgb565 >> 5) & 0x3F) as u32) << 2;
        let b = ((rgb565 & 0x1F) as u32) << 3;
        Color {
            value: 0xFF000000 | (r << 16) | (g << 8) | b,
        }
    }

    /// Standard color constants
    pub const BLACK: Self = Color { value: 0xFF000000 };
    pub const WHITE: Self = Color { value: 0xFFFFFFFF };
    pub const RED: Self = Color { value: 0xFFFF0000 };
    pub const GREEN: Self = Color { value: 0xFF00FF00 };
    pub const BLUE: Self = Color { value: 0xFF0000FF };
    pub const YELLOW: Self = Color { value: 0xFFFFFF00 };
    pub const CYAN: Self = Color { value: 0xFF00FFFF };
    pub const MAGENTA: Self = Color { value: 0xFFFF00FF };

    /// Gray constants
    pub const DARK_GRAY: Self = Color { value: 0xFF404040 };
    pub const GRAY: Self = Color { value: 0xFF808080 };
    pub const LIGHT_GRAY: Self = Color { value: 0xFFC0C0C0 };
    pub const SILVER: Self = Color { value: 0xFFC0C0C0 };

    /// Web colors
    pub const NAVY: Self = Color { value: 0xFF000080 };
    pub const TEAL: Self = Color { value: 0xFF008080 };
    pub const MAROON: Self = Color { value: 0xFF800000 };
    pub const PURPLE: Self = Color { value: 0xFF800080 };
    pub const OLIVE: Self = Color { value: 0xFF808000 };
    pub const LIME: Self = Color { value: 0xFF00FF00 };
    pub const AQUA: Self = Color { value: 0xFF00FFFF };
    pub const FUCHSIA: Self = Color { value: 0xFFFF00FF };
    pub const ORANGE: Self = Color { value: 0xFFFFA500 };
    pub const BROWN: Self = Color { value: 0xFFA52A2A };
    pub const PINK: Self = Color { value: 0xFFFFC0CB };
}

/// VGA 16-color palette (standard PC palette).
pub const VGA_PALETTE: [Color; 16] = [
    Color { value: 0xFF000000 }, // 0: Black
    Color { value: 0xFF000080 }, // 1: Blue
    Color { value: 0xFF008000 }, // 2: Green
    Color { value: 0xFF008080 }, // 3: Cyan
    Color { value: 0xFF800000 }, // 4: Red
    Color { value: 0xFF800080 }, // 5: Magenta
    Color { value: 0xFF808000 }, // 6: Brown
    Color { value: 0xFFC0C0C0 }, // 7: Light Gray
    Color { value: 0xFF808080 }, // 8: Dark Gray
    Color { value: 0xFF0000FF }, // 9: Light Blue
    Color { value: 0xFF00FF00 }, // 10: Light Green
    Color { value: 0xFF00FFFF }, // 11: Light Cyan
    Color { value: 0xFFFF0000 }, // 12: Light Red
    Color { value: 0xFFFF00FF }, // 13: Light Magenta
    Color { value: 0xFFFFFF00 }, // 14: Yellow
    Color { value: 0xFFFFFFFF }, // 15: White
];

/// Extended 256-color palette (8-bit indexed color).
/// This is a basic palette; can be customized per application needs.
pub struct ColorPalette;

impl ColorPalette {
    /// Get a color from the 16-color VGA palette.
    ///
    /// # Arguments
    ///
    /// * `index` - VGA palette index (0-15)
    ///
    /// # Panics
    ///
    /// Panics if index is greater than 15.
    pub const fn get_vga_color(index: u8) -> Color {
        VGA_PALETTE[index as usize]
    }

    /// Get a color from the extended 256-color palette.
    ///
    /// This palette is organized as:
    /// - 0-15: VGA standard colors
    /// - 16-231: 6x6x6 RGB cube (216 colors)
    /// - 232-255: Grayscale ramp (24 grays)
    pub fn get_palette_entry(index: u8) -> Color {
        match index {
            // VGA 16 colors
            0..=15 => VGA_PALETTE[index as usize],
            // 6x6x6 RGB cube (216 colors, indices 16-231)
            16..=231 => {
                let cube_index = (index - 16) as u32;
                let r = ((cube_index / 36) * 51) as u8;
                let g = (((cube_index / 6) % 6) * 51) as u8;
                let b = ((cube_index % 6) * 51) as u8;
                Color::from_rgb(r, g, b)
            }
            // Grayscale ramp (24 grays, indices 232-255)
            232..=255 => {
                let gray_index = (index - 232) as u32;
                let level = (8 + gray_index * 10) as u8;
                Color::from_rgb(level, level, level)
            }
        }
    }

    /// Find the closest VGA palette index for a given color.
    pub fn find_closest_vga_index(color: Color) -> u8 {
        let mut closest_index = 0;
        let mut min_distance = u32::MAX;

        for (i, &palette_color) in VGA_PALETTE.iter().enumerate() {
            let distance = color_distance(color, palette_color);
            if distance < min_distance {
                min_distance = distance;
                closest_index = i as u8;
            }
        }

        closest_index
    }
}

/// Calculate the Euclidean distance between two colors in RGB space.
/// Used for palette matching.
fn color_distance(c1: Color, c2: Color) -> u32 {
    let dr = (c1.red() as i32) - (c2.red() as i32);
    let dg = (c1.green() as i32) - (c2.green() as i32);
    let db = (c1.blue() as i32) - (c2.blue() as i32);
    ((dr * dr) + (dg * dg) + (db * db)) as u32
}

/// Convert sRGB color component to linear RGB for accurate blending.
///
/// sRGB uses gamma correction (γ ≈ 2.2) for perceptual color space.
/// For accurate blending, we convert to linear space, blend, then convert back.
/// Uses a simplified approximation suitable for no_std environments.
fn srgb_to_linear(value: u8) -> f32 {
    let v = (value as f32) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        // Simplified approximation for no_std: (v + 0.055) / 1.055 * (v + 0.055) / 1.055
        let normalized = (v + 0.055) / 1.055;
        normalized * normalized // Approximates ^(1/2.2) ≈ ^0.45, so squared for ^0.9
    }
}

/// Convert linear RGB to sRGB color component.
fn linear_to_srgb(linear: f32) -> u8 {
    let v = if linear <= 0.0031308 {
        linear * 12.92
    } else {
        // Simplified approximation: use sqrt as an approximation for 1/2.4 power
        1.055 * linear_sqrt(linear) - 0.055
    };
    let clamped = if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v };
    (clamped * 255.0) as u8
}

/// Fast integer-based square root approximation for no_std.
fn linear_sqrt(x: f32) -> f32 {
    // Newton's method for square root
    if x <= 0.0 {
        return 0.0;
    }
    let mut guess = x;
    for _ in 0..3 {
        guess = (guess + x / guess) / 2.0;
    }
    guess
}

/// Blend two colors using alpha blending with proper gamma correction.
///
/// Uses the formula: `result = foreground * alpha + background * (1 - alpha)`
/// with proper sRGB color space handling for perceptually correct results.
///
/// # Arguments
///
/// * `foreground` - The top color
/// * `background` - The bottom color
/// * `alpha` - Blending factor (0-255, where 255 is fully opaque foreground)
///
/// # Returns
///
/// The blended color with full opacity (alpha = 255)
pub fn blend(foreground: Color, background: Color, alpha: u8) -> Color {
    let alpha_f = (alpha as f32) / 255.0;
    let inv_alpha = 1.0 - alpha_f;

    // Convert to linear RGB for blending
    let fr = srgb_to_linear(foreground.red());
    let fg = srgb_to_linear(foreground.green());
    let fb = srgb_to_linear(foreground.blue());

    let br = srgb_to_linear(background.red());
    let bg = srgb_to_linear(background.green());
    let bb = srgb_to_linear(background.blue());

    // Blend in linear space
    let out_r = (fr * alpha_f) + (br * inv_alpha);
    let out_g = (fg * alpha_f) + (bg * inv_alpha);
    let out_b = (fb * alpha_f) + (bb * inv_alpha);

    // Convert back to sRGB
    Color::from_rgb(
        linear_to_srgb(out_r),
        linear_to_srgb(out_g),
        linear_to_srgb(out_b),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_components() {
        let color = Color::from_argb(200, 100, 150, 50);
        assert_eq!(color.alpha(), 200);
        assert_eq!(color.red(), 100);
        assert_eq!(color.green(), 150);
        assert_eq!(color.blue(), 50);
    }

    #[test]
    fn test_color_setters() {
        let color = Color::from_rgb(100, 150, 200);
        let modified = color.set_red(255).set_blue(0);
        assert_eq!(modified.red(), 255);
        assert_eq!(modified.green(), 150);
        assert_eq!(modified.blue(), 0);
    }

    #[test]
    fn test_rgb565_conversion() {
        // Pure red in RGB888 -> RGB565
        let red = Color::from_rgb(255, 0, 0);
        let rgb565 = red.to_rgb565();
        assert_eq!(rgb565, 0xF800); // 11111 00000 00000

        // Convert back
        let back = Color::from_rgb565(0xF800);
        assert_eq!(back.red(), 248); // Close to 255 (due to 5-bit precision)
    }

    #[test]
    fn test_vga_palette() {
        let black = ColorPalette::get_vga_color(0);
        assert_eq!(black, Color::BLACK);

        let white = ColorPalette::get_vga_color(15);
        assert_eq!(white, Color::WHITE);
    }

    #[test]
    fn test_extended_palette() {
        // VGA colors in extended palette
        let black = ColorPalette::get_palette_entry(0);
        assert_eq!(black, Color::BLACK);

        // RGB cube should have predictable colors
        let color = ColorPalette::get_palette_entry(16); // First RGB cube entry
        assert!(color.red() == 0 && color.green() == 0 && color.blue() == 0);
    }

    #[test]
    fn test_blend() {
        // Blending white over black with 50% alpha should give gray
        let blended = blend(Color::WHITE, Color::BLACK, 128);
        let gray_val = blended.red();
        assert!(gray_val > 100 && gray_val < 160); // Approximately 127-128 in linear space
    }
}
