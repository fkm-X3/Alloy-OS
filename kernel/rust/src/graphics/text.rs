//! Text rendering utilities for graphics displays.
//!
//! This module provides higher-level text rendering capabilities built on top
//! of the Display trait. It handles font rendering, cursor positioning, and
//! text layout.
//!
//! # Architecture
//!
//! Text rendering involves:
//! - Font data storage and lookup (5x7 bitmap font for ASCII 32-126)
//! - Glyph rendering to pixel data via bit-level rasterization
//! - Text positioning and layout with newline support
//! - Cursor management and color control
//!
//! # Font Details
//!
//! The 5x7 bitmap font stores ASCII characters 32-126 (95 characters).
//! Each character is 7 bytes (one per row, 5 pixels wide, 7 pixels tall).
//! The format uses LSBs as left pixels, so bit 0 represents the leftmost pixel.
//!
//! # Example
//!
//! ```no_run
//! # use kernel::graphics::{Display, text::TextRenderer};
//! # let mut display: &mut dyn Display<Error=(), Buffer=()> = unsafe { &mut *(0 as *mut _) };
//! let mut renderer = TextRenderer::new();
//! renderer.set_color(0xFFFFFFFF, 0xFF000000); // White on black
//! let _ = renderer.render_string(10, 10, "Hello", 0xFFFFFFFF, display);
//! ```

use super::{Display, FramebufferBuffer};
use crate::graphics::font::Font;
use core::fmt::Debug;

/// Error types for text rendering operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRenderError {
    DisplayError,
    OutOfBounds,
}

/// Text rendering context for drawing text on a display.
///
/// Uses the 8x16 bitmap font (Font trait) and renders via the Display trait.
#[derive(Debug, Clone)]
pub struct TextRenderer {
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub fg_color: u32,
    pub bg_color: u32,
    font: crate::graphics::font::BitmapFont8x16,
}

impl TextRenderer {
    pub fn new() -> Self {
        TextRenderer {
            cursor_x: 0,
            cursor_y: 0,
            fg_color: 0xFFFFFFFF,
            bg_color: 0xFF000000,
            font: crate::graphics::font::BitmapFont8x16::new(),
        }
    }

    pub fn set_cursor(&mut self, x: u32, y: u32) {
        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn set_color(&mut self, fg: u32, bg: u32) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    pub fn get_char_width(&self) -> u32 {
        self.font.char_width()
    }

    pub fn get_char_height(&self) -> u32 {
        self.font.char_height()
    }

    pub fn get_line_height(&self) -> u32 {
        self.font.line_height()
    }

    pub fn render_char<B: FramebufferBuffer>(
        &self,
        x: u32,
        y: u32,
        ch: char,
        color: u32,
        display: &mut dyn Display<Error = (), Buffer = B>,
    ) -> Result<(), TextRenderError> {
        let (screen_width, screen_height) = display.get_resolution();

        let width = self.font.char_width();
        let height = self.font.char_height();

        let char_code = ch as u32;
        if !(32..=126).contains(&char_code) {
            return Ok(());
        }

        for row in 0..height {
            if y + row >= screen_height {
                break;
            }
            for col in 0..width {
                if x + col >= screen_width {
                    break;
                }
            }
        }

        let mut buf = alloc::vec![0u32; (width * height) as usize];
        self.font
            .render_glyph(ch, &mut buf, width, height, 0, 0, color, self.bg_color);

        for row in 0..height {
            for col in 0..width {
                let idx = (row * width + col) as usize;
                if idx < buf.len() && buf[idx] != self.bg_color {
                    display.pixel_put(x + col, y + row, buf[idx]);
                }
            }
        }

        Ok(())
    }

    pub fn render_string<B: FramebufferBuffer>(
        &self,
        x: u32,
        y: u32,
        text: &str,
        color: u32,
        display: &mut dyn Display<Error = (), Buffer = B>,
    ) -> Result<(), TextRenderError> {
        let mut current_x = x;
        let mut current_y = y;

        for ch in text.chars() {
            if ch == '\n' {
                current_x = x;
                current_y = current_y.saturating_add(self.get_line_height());
                continue;
            }

            self.render_char(current_x, current_y, ch, color, display)?;
            current_x = current_x.saturating_add(self.get_char_width());
        }

        Ok(())
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}
