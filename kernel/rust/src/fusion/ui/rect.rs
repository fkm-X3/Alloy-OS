//! Rectangle Drawing Primitives
//!
//! Provides efficient rectangle drawing functions for the Fusion UI framework.
//! Supports filled rectangles, outlines, and rounded rectangles with various colors and styles.
//!
//! All functions leverage the Surface's built-in clipping to handle out-of-bounds cases gracefully.

use core::fmt::Debug;
use crate::fusion::surface::{Surface, SurfaceError};

/// Error types for rectangle drawing operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectError {
    /// Rectangle dimensions are invalid (zero or overflow)
    InvalidDimensions,
    /// Operation failed due to surface error
    SurfaceError(SurfaceError),
}

impl core::fmt::Display for RectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RectError::InvalidDimensions => write!(f, "Invalid rectangle dimensions"),
            RectError::SurfaceError(e) => write!(f, "Surface error: {}", e),
        }
    }
}

impl From<SurfaceError> for RectError {
    fn from(err: SurfaceError) -> Self {
        RectError::SurfaceError(err)
    }
}

/// Draw a filled rectangle with a solid color.
///
/// # Arguments
///
/// * `surface` - Target surface for drawing
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Rectangle width in pixels
/// * `height` - Rectangle height in pixels
/// * `color` - Fill color in ARGB8888 format (0xAARRGGBB)
///
/// # Returns
///
/// `Ok(())` on success, or `RectError` on failure.
///
/// # Behavior
///
/// Uses the Surface's efficient `fill_rect` method. The rectangle is automatically
/// clipped to surface bounds, so partial overlaps are handled safely.
pub fn draw_filled_rect(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
) -> Result<(), RectError> {
    if width == 0 || height == 0 {
        return Err(RectError::InvalidDimensions);
    }

    surface.fill_rect(x, y, width, height, color)?;
    Ok(())
}

/// Draw a rectangle outline with a specified thickness.
///
/// # Arguments
///
/// * `surface` - Target surface for drawing
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Rectangle width in pixels
/// * `height` - Rectangle height in pixels
/// * `color` - Outline color in ARGB8888 format (0xAARRGGBB)
/// * `thickness` - Outline thickness in pixels
///
/// # Returns
///
/// `Ok(())` on success, or `RectError` on failure.
///
/// # Behavior
///
/// Draws four rectangles (top, bottom, left, right) to form the outline.
/// The thickness is applied inward from the rectangle edges.
/// Clipping is handled automatically by the Surface.
pub fn draw_outline_rect(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
    thickness: u32,
) -> Result<(), RectError> {
    if width == 0 || height == 0 || thickness == 0 {
        return Err(RectError::InvalidDimensions);
    }

    // Clamp thickness to not exceed half of the smallest dimension
    let thickness = core::cmp::min(thickness, core::cmp::min(width, height) / 2);

    // Top edge
    surface.fill_rect(x, y, width, thickness, color)?;

    // Bottom edge
    let bottom_y = y.saturating_add(height.saturating_sub(thickness));
    if bottom_y >= y {
        surface.fill_rect(x, bottom_y, width, thickness, color)?;
    }

    // Left edge (excluding corners already drawn)
    if height > thickness {
        surface.fill_rect(x, y + thickness, thickness, height - 2 * thickness, color)?;
    }

    // Right edge (excluding corners already drawn)
    if height > thickness {
        let right_x = x.saturating_add(width.saturating_sub(thickness));
        if right_x >= x {
            surface.fill_rect(
                right_x,
                y + thickness,
                thickness,
                height - 2 * thickness,
                color,
            )?;
        }
    }

    Ok(())
}

/// Draw a rectangle with rounded corners.
///
/// # Arguments
///
/// * `surface` - Target surface for drawing
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Rectangle width in pixels
/// * `height` - Rectangle height in pixels
/// * `radius` - Corner radius in pixels
/// * `color` - Fill color in ARGB8888 format (0xAARRGGBB)
///
/// # Returns
///
/// `Ok(())` on success, or `RectError` on failure.
///
/// # Behavior
///
/// Draws a filled rectangle with rounded corners using a simple approximation.
/// The algorithm:
/// 1. Fills the main rectangular body (height - 2*radius)
/// 2. Draws corner regions with clipping based on distance from corner centers
///
/// The clipping is handled automatically by checking pixel distance from corner centers.
pub fn draw_rounded_rect(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: u32,
) -> Result<(), RectError> {
    if width == 0 || height == 0 {
        return Err(RectError::InvalidDimensions);
    }

    // Clamp radius to half of the smallest dimension
    let radius = core::cmp::min(radius, core::cmp::min(width, height) / 2);

    if radius == 0 {
        // Degenerate to filled rectangle
        return draw_filled_rect(surface, x, y, width, height, color);
    }

    // Draw main body (center strip)
    if height > 2 * radius {
        surface.fill_rect(x, y + radius, width, height - 2 * radius, color)?;
    }

    // Draw top and bottom strips with corners
    draw_corner_strip(surface, x, y, width, radius, color, false)?; // top
    draw_corner_strip(surface, x, y + height - radius, width, radius, color, true)?; // bottom

    Ok(())
}

/// Helper function to draw rounded corners for top or bottom strips.
///
/// This efficiently draws corner regions by checking if each pixel is within
/// the rounded corner distance from the corner center.
fn draw_corner_strip(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    radius: u32,
    color: u32,
    is_bottom: bool,
) -> Result<(), RectError> {
    let (surf_width, surf_height) = surface.get_dimensions();

    // Center coordinates of the rounded corners
    let left_center_x = x + radius;
    let right_center_x = x + width - radius;

    for row in y..core::cmp::min(y + radius, surf_height) {
        let dist_y = if is_bottom {
            row.saturating_sub(y)
        } else {
            radius.saturating_sub(row.saturating_sub(y))
        };

        for col in x..core::cmp::min(x + width, surf_width) {
            let in_corner = if col < left_center_x {
                // Left corner
                let dist_x = left_center_x.saturating_sub(col);
                dist_x * dist_x + dist_y * dist_y <= radius * radius
            } else if col >= right_center_x {
                // Right corner
                let dist_x = col.saturating_sub(right_center_x - 1);
                dist_x * dist_x + dist_y * dist_y <= radius * radius
            } else {
                // Straight middle section
                true
            };

            if in_corner {
                let _ = surface.put_pixel(col, row, color);
            }
        }
    }

    Ok(())
}

/// Draw a rectangle with a vertical color gradient.
///
/// # Arguments
///
/// * `surface` - Target surface for drawing
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Rectangle width in pixels
/// * `height` - Rectangle height in pixels
/// * `color_start` - Color at top in ARGB8888 format
/// * `color_end` - Color at bottom in ARGB8888 format
///
/// # Returns
///
/// `Ok(())` on success, or `RectError` on failure.
///
/// # Behavior
///
/// Interpolates colors linearly from top to bottom. Each row is filled with
/// a single blended color based on its position between start and end.
pub fn draw_gradient_rect_vertical(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color_start: u32,
    color_end: u32,
) -> Result<(), RectError> {
    if width == 0 || height == 0 {
        return Err(RectError::InvalidDimensions);
    }

    let (surf_width, surf_height) = surface.get_dimensions();
    let x_end = core::cmp::min(x + width, surf_width);
    let y_end = core::cmp::min(y + height, surf_height);

    // Extract color components
    let start_a = (color_start >> 24) & 0xFF;
    let start_r = (color_start >> 16) & 0xFF;
    let start_g = (color_start >> 8) & 0xFF;
    let start_b = color_start & 0xFF;

    let end_a = (color_end >> 24) & 0xFF;
    let end_r = (color_end >> 16) & 0xFF;
    let end_g = (color_end >> 8) & 0xFF;
    let end_b = color_end & 0xFF;

    for row in y..y_end {
        let progress = if height > 1 {
            (row - y) as u32 * 256 / (height - 1)
        } else {
            0
        };

        // Interpolate each component
        let a = lerp_u8(start_a, end_a, progress);
        let r = lerp_u8(start_r, end_r, progress);
        let g = lerp_u8(start_g, end_g, progress);
        let b = lerp_u8(start_b, end_b, progress);

        let interpolated = (a << 24) | (r << 16) | (g << 8) | b;

        surface.fill_rect(x, row, width.min(x_end - x), 1, interpolated)?;
    }

    Ok(())
}

/// Draw a rectangle with a horizontal color gradient.
///
/// # Arguments
///
/// * `surface` - Target surface for drawing
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Rectangle width in pixels
/// * `height` - Rectangle height in pixels
/// * `color_left` - Color at left in ARGB8888 format
/// * `color_right` - Color at right in ARGB8888 format
///
/// # Returns
///
/// `Ok(())` on success, or `RectError` on failure.
///
/// # Behavior
///
/// Interpolates colors linearly from left to right. Each column is filled with
/// a single blended color based on its position between start and end.
pub fn draw_gradient_rect_horizontal(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color_left: u32,
    color_right: u32,
) -> Result<(), RectError> {
    if width == 0 || height == 0 {
        return Err(RectError::InvalidDimensions);
    }

    let (surf_width, surf_height) = surface.get_dimensions();
    let x_end = core::cmp::min(x + width, surf_width);
    let y_end = core::cmp::min(y + height, surf_height);

    // Extract color components
    let left_a = (color_left >> 24) & 0xFF;
    let left_r = (color_left >> 16) & 0xFF;
    let left_g = (color_left >> 8) & 0xFF;
    let left_b = color_left & 0xFF;

    let right_a = (color_right >> 24) & 0xFF;
    let right_r = (color_right >> 16) & 0xFF;
    let right_g = (color_right >> 8) & 0xFF;
    let right_b = color_right & 0xFF;

    for col in x..x_end {
        let progress = if width > 1 {
            (col - x) as u32 * 256 / (width - 1)
        } else {
            0
        };

        // Interpolate each component
        let a = lerp_u8(left_a, right_a, progress);
        let r = lerp_u8(left_r, right_r, progress);
        let g = lerp_u8(left_g, right_g, progress);
        let b = lerp_u8(left_b, right_b, progress);

        let interpolated = (a << 24) | (r << 16) | (g << 8) | b;

        surface.fill_rect(col, y, 1, height.min(y_end - y), interpolated)?;
    }

    Ok(())
}

/// Linear interpolation helper for color component blending (0-255 range).
///
/// # Arguments
///
/// * `start` - Start value (0-255)
/// * `end` - End value (0-255)
/// * `progress` - Interpolation factor (0-256, where 256 = 100%)
///
/// # Returns
///
/// Interpolated value in 0-255 range.
#[inline]
fn lerp_u8(start: u32, end: u32, progress: u32) -> u32 {
    let inv_progress = 256 - progress;
    ((start * inv_progress + end * progress) / 256) & 0xFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_filled_rect_valid() {
        let mut surface = Surface::new(100, 100).unwrap();
        let result = draw_filled_rect(&mut surface, 10, 10, 50, 50, 0xFFFF0000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_draw_filled_rect_zero_dimensions() {
        let mut surface = Surface::new(100, 100).unwrap();
        assert_eq!(
            draw_filled_rect(&mut surface, 10, 10, 0, 50, 0xFFFF0000),
            Err(RectError::InvalidDimensions)
        );
        assert_eq!(
            draw_filled_rect(&mut surface, 10, 10, 50, 0, 0xFFFF0000),
            Err(RectError::InvalidDimensions)
        );
    }

    #[test]
    fn test_draw_outline_rect_valid() {
        let mut surface = Surface::new(100, 100).unwrap();
        let result = draw_outline_rect(&mut surface, 10, 10, 50, 50, 0xFFFF0000, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_draw_outline_rect_zero_thickness() {
        let mut surface = Surface::new(100, 100).unwrap();
        assert_eq!(
            draw_outline_rect(&mut surface, 10, 10, 50, 50, 0xFFFF0000, 0),
            Err(RectError::InvalidDimensions)
        );
    }

    #[test]
    fn test_draw_rounded_rect_valid() {
        let mut surface = Surface::new(100, 100).unwrap();
        let result = draw_rounded_rect(&mut surface, 10, 10, 50, 50, 5, 0xFFFF0000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_draw_rounded_rect_zero_radius() {
        let mut surface = Surface::new(100, 100).unwrap();
        let result = draw_rounded_rect(&mut surface, 10, 10, 50, 50, 0, 0xFFFF0000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gradient_vertical_valid() {
        let mut surface = Surface::new(100, 100).unwrap();
        let result = draw_gradient_rect_vertical(
            &mut surface,
            10,
            10,
            50,
            50,
            0xFFFF0000,
            0xFF0000FF,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_gradient_horizontal_valid() {
        let mut surface = Surface::new(100, 100).unwrap();
        let result = draw_gradient_rect_horizontal(
            &mut surface,
            10,
            10,
            50,
            50,
            0xFFFF0000,
            0xFF0000FF,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_lerp_u8() {
        assert_eq!(lerp_u8(0, 255, 0), 0);
        assert_eq!(lerp_u8(0, 255, 256), 255);
        assert_eq!(lerp_u8(0, 255, 128), 127); // Approximately half
    }
}
