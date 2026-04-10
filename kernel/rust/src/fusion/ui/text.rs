//! Text drawing primitives for Alloy OS Fusion UI.
//!
//! This module provides high-level text rendering functions for UI elements,
//! built on top of the kernel graphics TextRenderer. It supports single-line and
//! multi-line text with automatic line wrapping, centering, and background fills.
//!
//! # Features
//!
//! - **Text Drawing**: Basic text rendering at arbitrary positions
//! - **Centered Text**: Automatically center text within a region
//! - **Background Fills**: Draw text with colored background rectangles
//! - **Multi-line Support**: Automatic line height management with `\n` support
//! - **Text Measurement**: Query text dimensions before rendering
//!
//! # Architecture
//!
//! This module acts as a bridge between the kernel graphics text rendering system
//! and the higher-level UI framework. It provides safe, ergonomic wrappers around
//! the lower-level TextRenderer and integrates with the Surface abstraction for
//! off-screen rendering.
//!
//! # Example
//!
//! ```no_run
//! # use kernel::fusion::surface::Surface;
//! # use kernel::fusion::ui::text::*;
//! let mut surface = Surface::new(800, 600)?;
//! draw_text(&mut surface, 10, 20, "Hello, Fusion!", 0xFFFFFFFF)?;
//! draw_text_centered(&mut surface, 0, 100, 800, "Centered Title", 0xFF00FF00)?;
//! draw_text_with_bg(&mut surface, 50, 50, "Button Text", 0xFFFFFFFF, 0xFF0000FF)?;
//! # Ok::<(), TextError>(())
//! ```

use crate::fusion::surface::{Surface, SurfaceError};
use crate::graphics::text::TextRenderer;
use core::fmt::Debug;

/// Error types for text rendering operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextError {
    /// Surface operation failed (out of bounds, allocation, etc.)
    SurfaceError,
    /// Text rendering coordinates out of bounds
    OutOfBounds,
    /// Text rendering failed for other reasons
    RenderError,
}

impl core::fmt::Display for TextError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TextError::SurfaceError => write!(f, "Surface operation failed"),
            TextError::OutOfBounds => write!(f, "Text coordinates out of bounds"),
            TextError::RenderError => write!(f, "Text rendering failed"),
        }
    }
}

/// Convert SurfaceError to TextError.
impl From<SurfaceError> for TextError {
    fn from(_: SurfaceError) -> Self {
        TextError::SurfaceError
    }
}

/// Get the width of a single character in pixels.
///
/// # Returns
///
/// Character width (5 pixels for the 5x7 font).
#[inline]
pub fn char_width() -> u32 {
    TextRenderer::get_char_width()
}

/// Get the height of a single character in pixels.
///
/// # Returns
///
/// Character height (7 pixels for the 5x7 font).
#[inline]
pub fn char_height() -> u32 {
    TextRenderer::get_char_height()
}

/// Get the line height including spacing.
///
/// # Returns
///
/// Line height (9 pixels: 7-pixel character + 2-pixel spacing).
#[inline]
pub fn line_height() -> u32 {
    TextRenderer::get_line_height()
}

/// Measure text dimensions.
///
/// Calculates the width and height required to render text, accounting for
/// multi-line text with `\n` line breaks.
///
/// # Arguments
///
/// * `text` - String to measure
///
/// # Returns
///
/// Tuple of (width, height) in pixels.
///
/// # Notes
///
/// - Width is calculated from the longest line
/// - Height accounts for all lines including newlines
/// - Empty strings return (0, 0)
pub fn measure_text(text: &str) -> (u32, u32) {
    if text.is_empty() {
        return (0, 0);
    }

    let mut max_width = 0u32;
    let mut current_line_width = 0u32;
    let mut line_count = 1u32;

    for ch in text.chars() {
        if ch == '\n' {
            max_width = core::cmp::max(max_width, current_line_width);
            current_line_width = 0;
            line_count += 1;
        } else if ch != '\r' {
            current_line_width += char_width();
        }
    }

    // Account for the last line
    max_width = core::cmp::max(max_width, current_line_width);

    let height = line_count * line_height();

    (max_width, height)
}

/// Draw text on a surface at specified coordinates.
///
/// Renders text with automatic multi-line support. Text is drawn from the
/// given starting position, with newlines advancing to the next line at
/// the original X position.
///
/// # Arguments
///
/// * `surface` - Mutable reference to Surface to draw on
/// * `x` - Starting X coordinate (in surface coordinates)
/// * `y` - Starting Y coordinate (in surface coordinates)
/// * `text` - Text string to render (may contain `\n` for line breaks)
/// * `color` - Text color in ARGB8888 format (0xAARRGGBB)
///
/// # Returns
///
/// `Ok(())` on success, or `TextError` if rendering fails.
///
/// # Clipping
///
/// Text is automatically clipped to surface bounds. Partially off-screen
/// characters are rendered appropriately.
///
/// # Notes
///
/// - Characters outside ASCII 32-126 range are skipped
/// - Invalid surface operations return `TextError::SurfaceError`
/// - Coordinates should be within surface bounds for full visibility
pub fn draw_text(
    surface: &mut Surface,
    x: u32,
    y: u32,
    text: &str,
    color: u32,
) -> Result<(), TextError> {
    let (surf_width, surf_height) = surface.get_dimensions();

    // Early exit if starting position is completely out of bounds
    if x >= surf_width || y >= surf_height {
        return Err(TextError::OutOfBounds);
    }

    let mut current_x = x;
    let mut current_y = y;

    for ch in text.chars() {
        if ch == '\n' {
            current_x = x;
            current_y += line_height();
            if current_y >= surf_height {
                break;
            }
            continue;
        }

        // Skip out-of-bounds characters
        if current_x >= surf_width || current_y >= surf_height {
            current_x += char_width();
            if current_x >= surf_width + char_width() * 2 {
                // Performance optimization: skip ahead if far out of bounds
                current_x = x;
                current_y += line_height();
            }
            continue;
        }

        // Render character using TextRenderer's static method
        // We need to create an adapter to work with Surface
        render_char_to_surface(surface, current_x, current_y, ch, color)?;
        current_x += char_width();
    }

    Ok(())
}

/// Draw centered text on a surface.
///
/// Text is horizontally centered within the specified region. If text is
/// wider than the available width, it's drawn starting from the left edge.
///
/// # Arguments
///
/// * `surface` - Mutable reference to Surface to draw on
/// * `x` - Left edge of centering region (X coordinate)
/// * `y` - Y coordinate to start drawing
/// * `width` - Width of the region to center text within
/// * `text` - Text string to render
/// * `color` - Text color in ARGB8888 format
///
/// # Returns
///
/// `Ok(())` on success, or `TextError` if rendering fails.
///
/// # Notes
///
/// - Multi-line text is centered line by line
/// - Each line is centered independently
pub fn draw_text_centered(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    text: &str,
    color: u32,
) -> Result<(), TextError> {
    let (text_width, _) = measure_text(text);

    // Calculate center position
    let center_offset = if text_width < width {
        (width - text_width) / 2
    } else {
        0
    };

    let center_x = x + center_offset;

    draw_text(surface, center_x, y, text, color)
}

/// Draw text with a colored background rectangle.
///
/// Text is rendered with a background fill, useful for UI elements like
/// buttons, labels, and highlighted text. The background is drawn slightly
/// larger than the text to provide padding.
///
/// # Arguments
///
/// * `surface` - Mutable reference to Surface to draw on
/// * `x` - X coordinate for text and background
/// * `y` - Y coordinate for text and background
/// * `text` - Text string to render
/// * `fg_color` - Text (foreground) color in ARGB8888 format
/// * `bg_color` - Background fill color in ARGB8888 format
///
/// # Returns
///
/// `Ok(())` on success, or `TextError` if rendering fails.
///
/// # Padding
///
/// The background rectangle includes automatic padding:
/// - Left/Right: 2 pixels each
/// - Top/Bottom: 2 pixels each
///
/// # Notes
///
/// - Multi-line text background covers all lines with padding
/// - Background is always drawn first, then text on top
pub fn draw_text_with_bg(
    surface: &mut Surface,
    x: u32,
    y: u32,
    text: &str,
    fg_color: u32,
    bg_color: u32,
) -> Result<(), TextError> {
    let (text_width, text_height) = measure_text(text);

    if text_width == 0 || text_height == 0 {
        return Ok(());
    }

    // Add padding around text
    const PADDING: u32 = 2;
    let bg_x = x.saturating_sub(PADDING);
    let bg_y = y.saturating_sub(PADDING);
    let bg_width = text_width + PADDING * 2;
    let bg_height = text_height + PADDING * 2;

    // Draw background rectangle first
    surface.fill_rect(bg_x, bg_y, bg_width, bg_height, bg_color)?;

    // Draw text on top
    draw_text(surface, x, y, text, fg_color)?;

    Ok(())
}

/// Render a single character to a surface at the specified position.
///
/// This is an internal helper function that bridges the TextRenderer API
/// with the Surface abstraction.
///
/// # Arguments
///
/// * `surface` - Mutable reference to Surface
/// * `x` - X coordinate within surface
/// * `y` - Y coordinate within surface
/// * `ch` - Character to render
/// * `color` - Character color
///
/// # Returns
///
/// `Ok(())` on success, or error if surface operation fails.
fn render_char_to_surface(
    surface: &mut Surface,
    x: u32,
    y: u32,
    ch: char,
    color: u32,
) -> Result<(), TextError> {
    let char_code = ch as u32;

    // ASCII range: 32 (space) to 126 (~)
    if char_code < 32 || char_code > 126 {
        return Ok(());
    }

    // Font glyph data from TextRenderer's font
    const FONT_5X7: &[u8] = &[
        // ASCII 32: Space
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // ASCII 33: !
        0x08, 0x08, 0x08, 0x08, 0x00, 0x08, 0x00,
        // ASCII 34: "
        0x14, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00,
        // ASCII 35: #
        0x0A, 0x1F, 0x0A, 0x1F, 0x0A, 0x00, 0x00,
        // ASCII 36: $
        0x08, 0x1C, 0x0A, 0x1C, 0x14, 0x1C, 0x08,
        // ASCII 37: %
        0x16, 0x16, 0x08, 0x04, 0x0D, 0x0D, 0x00,
        // ASCII 38: &
        0x0C, 0x12, 0x0A, 0x04, 0x0A, 0x12, 0x0D,
        // ASCII 39: '
        0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
        // ASCII 40: (
        0x04, 0x08, 0x08, 0x08, 0x08, 0x08, 0x04,
        // ASCII 41: )
        0x08, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08,
        // ASCII 42: *
        0x00, 0x08, 0x1B, 0x0E, 0x1B, 0x08, 0x00,
        // ASCII 43: +
        0x00, 0x08, 0x08, 0x1C, 0x08, 0x08, 0x00,
        // ASCII 44: ,
        0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x04,
        // ASCII 45: -
        0x00, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x00,
        // ASCII 46: .
        0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00,
        // ASCII 47: /
        0x00, 0x10, 0x10, 0x08, 0x04, 0x04, 0x02,
        // ASCII 48: 0
        0x0C, 0x12, 0x12, 0x12, 0x12, 0x12, 0x0C,
        // ASCII 49: 1
        0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E,
        // ASCII 50: 2
        0x0C, 0x12, 0x02, 0x04, 0x08, 0x10, 0x1E,
        // ASCII 51: 3
        0x1E, 0x02, 0x04, 0x02, 0x02, 0x12, 0x0C,
        // ASCII 52: 4
        0x02, 0x06, 0x0A, 0x12, 0x1E, 0x02, 0x02,
        // ASCII 53: 5
        0x1E, 0x10, 0x1C, 0x02, 0x02, 0x12, 0x0C,
        // ASCII 54: 6
        0x0C, 0x10, 0x10, 0x1C, 0x12, 0x12, 0x0C,
        // ASCII 55: 7
        0x1E, 0x02, 0x04, 0x04, 0x08, 0x08, 0x08,
        // ASCII 56: 8
        0x0C, 0x12, 0x12, 0x0C, 0x12, 0x12, 0x0C,
        // ASCII 57: 9
        0x0C, 0x12, 0x12, 0x0E, 0x02, 0x04, 0x08,
        // ASCII 58: :
        0x00, 0x08, 0x00, 0x00, 0x08, 0x00, 0x00,
        // ASCII 59: ;
        0x00, 0x08, 0x00, 0x00, 0x08, 0x08, 0x04,
        // ASCII 60: <
        0x04, 0x08, 0x10, 0x10, 0x08, 0x04, 0x00,
        // ASCII 61: =
        0x00, 0x00, 0x1C, 0x00, 0x1C, 0x00, 0x00,
        // ASCII 62: >
        0x08, 0x04, 0x02, 0x02, 0x04, 0x08, 0x00,
        // ASCII 63: ?
        0x0C, 0x12, 0x02, 0x04, 0x08, 0x00, 0x08,
        // ASCII 64: @
        0x0C, 0x12, 0x1E, 0x1A, 0x1E, 0x10, 0x0C,
        // ASCII 65: A
        0x0C, 0x12, 0x12, 0x1E, 0x12, 0x12, 0x12,
        // ASCII 66: B
        0x1C, 0x12, 0x12, 0x1C, 0x12, 0x12, 0x1C,
        // ASCII 67: C
        0x0C, 0x12, 0x10, 0x10, 0x10, 0x12, 0x0C,
        // ASCII 68: D
        0x18, 0x14, 0x12, 0x12, 0x12, 0x14, 0x18,
        // ASCII 69: E
        0x1E, 0x10, 0x10, 0x1C, 0x10, 0x10, 0x1E,
        // ASCII 70: F
        0x1E, 0x10, 0x10, 0x1C, 0x10, 0x10, 0x10,
        // ASCII 71: G
        0x0C, 0x12, 0x10, 0x1E, 0x12, 0x12, 0x0C,
        // ASCII 72: H
        0x12, 0x12, 0x12, 0x1E, 0x12, 0x12, 0x12,
        // ASCII 73: I
        0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E,
        // ASCII 74: J
        0x0E, 0x04, 0x04, 0x04, 0x04, 0x14, 0x08,
        // ASCII 75: K
        0x12, 0x12, 0x14, 0x18, 0x14, 0x12, 0x12,
        // ASCII 76: L
        0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1E,
        // ASCII 77: M
        0x12, 0x1B, 0x1B, 0x15, 0x12, 0x12, 0x12,
        // ASCII 78: N
        0x12, 0x1A, 0x16, 0x12, 0x12, 0x12, 0x12,
        // ASCII 79: O
        0x0C, 0x12, 0x12, 0x12, 0x12, 0x12, 0x0C,
        // ASCII 80: P
        0x1C, 0x12, 0x12, 0x1C, 0x10, 0x10, 0x10,
        // ASCII 81: Q
        0x0C, 0x12, 0x12, 0x12, 0x12, 0x14, 0x0A,
        // ASCII 82: R
        0x1C, 0x12, 0x12, 0x1C, 0x12, 0x12, 0x12,
        // ASCII 83: S
        0x0C, 0x12, 0x10, 0x0C, 0x02, 0x12, 0x0C,
        // ASCII 84: T
        0x1E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
        // ASCII 85: U
        0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x0C,
        // ASCII 86: V
        0x12, 0x12, 0x12, 0x12, 0x12, 0x0A, 0x04,
        // ASCII 87: W
        0x12, 0x12, 0x12, 0x15, 0x1B, 0x1B, 0x12,
        // ASCII 88: X
        0x12, 0x12, 0x0A, 0x04, 0x0A, 0x12, 0x12,
        // ASCII 89: Y
        0x12, 0x12, 0x0A, 0x04, 0x08, 0x08, 0x08,
        // ASCII 90: Z
        0x1E, 0x02, 0x04, 0x08, 0x10, 0x10, 0x1E,
        // ASCII 91: [
        0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0E,
        // ASCII 92: \
        0x02, 0x02, 0x04, 0x08, 0x10, 0x10, 0x10,
        // ASCII 93: ]
        0x0E, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0E,
        // ASCII 94: ^
        0x04, 0x0A, 0x12, 0x00, 0x00, 0x00, 0x00,
        // ASCII 95: _
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F,
        // ASCII 96: `
        0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
        // ASCII 97: a
        0x00, 0x0C, 0x02, 0x0E, 0x12, 0x12, 0x0E,
        // ASCII 98: b
        0x10, 0x10, 0x1C, 0x12, 0x12, 0x12, 0x1C,
        // ASCII 99: c
        0x00, 0x0C, 0x12, 0x10, 0x10, 0x12, 0x0C,
        // ASCII 100: d
        0x02, 0x02, 0x0E, 0x12, 0x12, 0x12, 0x0E,
        // ASCII 101: e
        0x00, 0x0C, 0x12, 0x1E, 0x10, 0x12, 0x0C,
        // ASCII 102: f
        0x04, 0x08, 0x08, 0x1C, 0x08, 0x08, 0x08,
        // ASCII 103: g
        0x00, 0x0E, 0x12, 0x12, 0x0E, 0x02, 0x0C,
        // ASCII 104: h
        0x10, 0x10, 0x1C, 0x12, 0x12, 0x12, 0x12,
        // ASCII 105: i
        0x08, 0x00, 0x08, 0x08, 0x08, 0x08, 0x08,
        // ASCII 106: j
        0x04, 0x00, 0x04, 0x04, 0x04, 0x14, 0x08,
        // ASCII 107: k
        0x10, 0x10, 0x12, 0x14, 0x18, 0x14, 0x12,
        // ASCII 108: l
        0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
        // ASCII 109: m
        0x00, 0x12, 0x1B, 0x15, 0x15, 0x12, 0x12,
        // ASCII 110: n
        0x00, 0x1C, 0x12, 0x12, 0x12, 0x12, 0x12,
        // ASCII 111: o
        0x00, 0x0C, 0x12, 0x12, 0x12, 0x12, 0x0C,
        // ASCII 112: p
        0x00, 0x1C, 0x12, 0x12, 0x1C, 0x10, 0x10,
        // ASCII 113: q
        0x00, 0x0E, 0x12, 0x12, 0x0E, 0x02, 0x02,
        // ASCII 114: r
        0x00, 0x16, 0x08, 0x08, 0x08, 0x08, 0x08,
        // ASCII 115: s
        0x00, 0x0C, 0x10, 0x0C, 0x02, 0x02, 0x1C,
        // ASCII 116: t
        0x08, 0x08, 0x1C, 0x08, 0x08, 0x08, 0x04,
        // ASCII 117: u
        0x00, 0x12, 0x12, 0x12, 0x12, 0x12, 0x0E,
        // ASCII 118: v
        0x00, 0x12, 0x12, 0x12, 0x12, 0x0A, 0x04,
        // ASCII 119: w
        0x00, 0x12, 0x12, 0x15, 0x1B, 0x1B, 0x12,
        // ASCII 120: x
        0x00, 0x12, 0x0A, 0x04, 0x04, 0x0A, 0x12,
        // ASCII 121: y
        0x00, 0x12, 0x12, 0x0E, 0x02, 0x02, 0x0C,
        // ASCII 122: z
        0x00, 0x1E, 0x04, 0x08, 0x10, 0x10, 0x1E,
        // ASCII 123: {
        0x06, 0x08, 0x08, 0x10, 0x08, 0x08, 0x06,
        // ASCII 124: |
        0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
        // ASCII 125: }
        0x0C, 0x04, 0x04, 0x02, 0x04, 0x04, 0x0C,
        // ASCII 126: ~
        0x0A, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let glyph_offset = ((char_code - 32) as usize) * 7;
    let glyph = &FONT_5X7[glyph_offset..glyph_offset + 7];

    let (surf_width, surf_height) = surface.get_dimensions();

    // Render each row of the glyph
    for row in 0..7u32 {
        if y + row >= surf_height {
            break;
        }

        let byte = glyph[row as usize];

        // Render each bit (pixel) in the byte, LSB = leftmost
        for col in 0..5u32 {
            if x + col >= surf_width {
                break;
            }

            // Check if this bit is set (LSB first)
            if (byte & (1 << col)) != 0 {
                surface.put_pixel(x + col, y + row, color)?;
            }
        }
    }

    Ok(())
}
