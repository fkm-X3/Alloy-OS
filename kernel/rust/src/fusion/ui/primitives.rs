//! High-level UI Components
//!
//! Provides reusable UI component abstractions (Button, Panel, Label, TextBox)
//! built on top of the rect and text drawing primitives.
//!
//! # Components
//!
//! - **Button**: Interactive button with hover and click detection
//! - **Panel**: Container with optional border and padding
//! - **Label**: Text label with optional background and centering
//! - **TextBox**: Input field with cursor support and text buffer management
//!
//! # Error Handling
//!
//! All drawing operations return `Result<(), PrimitiveError>` to handle
//! surface errors, rendering failures, and out-of-bounds conditions.
//!
//! # Color Format
//!
//! All colors use ARGB8888 format: `0xAARRGGBB`
//! - AA: Alpha (0xFF fully opaque)
//! - RR: Red component
//! - GG: Green component
//! - BB: Blue component
//!
//! # Example
//!
//! ```no_run
//! # use kernel::fusion::surface::Surface;
//! # use kernel::fusion::ui::primitives::*;
//! let mut surface = Surface::new(800, 600)?;
//!
//! // Create a button
//! let mut button = Button::new(100, 100, 120, 40, "Click Me", 0xFF0080FF, 0xFFFFFFFF);
//! button.draw(&mut surface)?;
//!
//! // Create a panel
//! let panel = Panel::new(50, 50, 300, 200, 0xFF1a1a1a)
//!     .with_border(0xFF808080);
//! panel.draw(&mut surface)?;
//!
//! // Create a label
//! let label = Label::new(200, 150, "Hello, Fusion!", 0xFFFFFFFF)
//!     .with_centered(true);
//! label.draw(&mut surface)?;
//!
//! // Create a text input box
//! let mut textbox = TextBox::new(100, 300, 200, 30, 32);
//! textbox.draw(&mut surface)?;
//! # Ok::<(), PrimitiveError>(())
//! ```

use core::fmt::Debug;
use crate::fusion::surface::{Surface, SurfaceError};
use super::rect::{self, RectError};
use super::text::{self, TextError};

/// Error types for UI primitive operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveError {
    /// Surface operation failed
    SurfaceError(SurfaceError),
    /// Rectangle drawing failed
    RectError(RectError),
    /// Text rendering failed
    TextError(TextError),
    /// Operation exceeded bounds
    OutOfBounds,
    /// Rendering operation failed
    RenderError,
}

impl core::fmt::Display for PrimitiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PrimitiveError::SurfaceError(e) => write!(f, "Surface error: {}", e),
            PrimitiveError::RectError(e) => write!(f, "Rectangle error: {}", e),
            PrimitiveError::TextError(e) => write!(f, "Text error: {}", e),
            PrimitiveError::OutOfBounds => write!(f, "Operation out of bounds"),
            PrimitiveError::RenderError => write!(f, "Rendering failed"),
        }
    }
}

impl From<SurfaceError> for PrimitiveError {
    fn from(err: SurfaceError) -> Self {
        PrimitiveError::SurfaceError(err)
    }
}

impl From<RectError> for PrimitiveError {
    fn from(err: RectError) -> Self {
        PrimitiveError::RectError(err)
    }
}

impl From<TextError> for PrimitiveError {
    fn from(err: TextError) -> Self {
        PrimitiveError::TextError(err)
    }
}

/// Interactive button component with hover and click detection.
///
/// A Button combines a filled rectangle background with centered text.
/// It supports two color states: normal and hovered.
///
/// # Fields
///
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Button width in pixels
/// * `height` - Button height in pixels
/// * `text` - Button label text
/// * `normal_bg_color` - Background color when not hovered (ARGB8888)
/// * `hover_bg_color` - Background color when hovered (ARGB8888)
/// * `text_color` - Text color (ARGB8888)
/// * `is_hovered` - Current hover state
/// * `z_order` - Layer ordering (higher draws on top)
#[derive(Debug, Clone)]
pub struct Button {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    text: &'static str,
    normal_bg_color: u32,
    hover_bg_color: u32,
    text_color: u32,
    is_hovered: bool,
    z_order: u32,
}

impl Button {
    /// Create a new button with specified parameters.
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - Button width in pixels
    /// * `height` - Button height in pixels
    /// * `text` - Button label (must be valid for entire button lifetime)
    /// * `bg_color` - Default background color (ARGB8888)
    /// * `text_color` - Text color (ARGB8888)
    ///
    /// # Returns
    ///
    /// A new Button with hover color set to a lighter version of bg_color.
    pub fn new(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        text: &'static str,
        bg_color: u32,
        text_color: u32,
    ) -> Self {
        let hover_color = Self::lighten_color(bg_color);
        Button {
            x,
            y,
            width,
            height,
            text,
            normal_bg_color: bg_color,
            hover_bg_color: hover_color,
            text_color,
            is_hovered: false,
            z_order: 0,
        }
    }

    /// Draw the button on the specified surface.
    ///
    /// Renders the button as a filled rectangle with centered text.
    /// The background color changes based on hover state.
    ///
    /// # Arguments
    ///
    /// * `surface` - Mutable reference to Surface to draw on
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `PrimitiveError` if rendering fails.
    pub fn draw(&self, surface: &mut Surface) -> Result<(), PrimitiveError> {
        let bg_color = if self.is_hovered {
            self.hover_bg_color
        } else {
            self.normal_bg_color
        };

        // Draw button background
        rect::draw_filled_rect(surface, self.x, self.y, self.width, self.height, bg_color)?;

        // Draw button text centered
        let text_width = text::char_width() * self.text.len() as u32;
        let text_height = text::char_height();

        let text_x = if text_width < self.width {
            self.x + (self.width - text_width) / 2
        } else {
            self.x + 2
        };

        let text_y = if text_height < self.height {
            self.y + (self.height - text_height) / 2
        } else {
            self.y + 2
        };

        text::draw_text(surface, text_x, text_y, self.text, self.text_color)?;

        Ok(())
    }

    /// Update button hover state based on cursor position.
    ///
    /// # Arguments
    ///
    /// * `x` - Cursor X coordinate
    /// * `y` - Cursor Y coordinate
    ///
    /// # Returns
    ///
    /// `true` if hover state changed, `false` otherwise.
    pub fn update_hover(&mut self, x: u32, y: u32) -> bool {
        let was_hovered = self.is_hovered;
        self.is_hovered = self.point_inside(x, y);
        was_hovered != self.is_hovered
    }

    /// Check if a point is inside button bounds (hit test).
    ///
    /// # Arguments
    ///
    /// * `x` - Point X coordinate
    /// * `y` - Point Y coordinate
    ///
    /// # Returns
    ///
    /// `true` if point is within button bounds, `false` otherwise.
    pub fn is_clicked(&self, x: u32, y: u32) -> bool {
        self.point_inside(x, y)
    }

    /// Helper to check if point is within button bounds.
    fn point_inside(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Lighten a color by increasing its brightness.
    fn lighten_color(color: u32) -> u32 {
        let alpha = (color >> 24) & 0xFF;
        let r = ((color >> 16) & 0xFF).saturating_add(0x40).min(0xFF);
        let g = ((color >> 8) & 0xFF).saturating_add(0x40).min(0xFF);
        let b = (color & 0xFF).saturating_add(0x40).min(0xFF);

        (alpha << 24) | (r << 16) | (g << 8) | b
    }

    /// Set button's z-order for layering.
    pub fn set_z_order(&mut self, z_order: u32) {
        self.z_order = z_order;
    }
}

/// Panel component - a container with optional border and padding.
///
/// A Panel provides a rectangular container for organizing UI elements.
/// It supports optional borders and configurable padding for content.
///
/// # Fields
///
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - Panel width in pixels
/// * `height` - Panel height in pixels
/// * `bg_color` - Background fill color (ARGB8888)
/// * `border_color` - Border color (ARGB8888)
/// * `has_border` - Whether to draw a border
/// * `padding` - Internal padding in pixels
/// * `z_order` - Layer ordering
#[derive(Debug, Clone)]
pub struct Panel {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    bg_color: u32,
    border_color: u32,
    has_border: bool,
    padding: u32,
    z_order: u32,
}

impl Panel {
    /// Create a new panel with specified dimensions and background color.
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - Panel width in pixels
    /// * `height` - Panel height in pixels
    /// * `bg_color` - Background color (ARGB8888)
    ///
    /// # Returns
    ///
    /// A new Panel with no border and default padding.
    pub fn new(x: u32, y: u32, width: u32, height: u32, bg_color: u32) -> Self {
        Panel {
            x,
            y,
            width,
            height,
            bg_color,
            border_color: 0xFF000000,
            has_border: false,
            padding: 4,
            z_order: 0,
        }
    }

    /// Add a border to the panel (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `border_color` - Border color (ARGB8888)
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_border(mut self, border_color: u32) -> Self {
        self.has_border = true;
        self.border_color = border_color;
        self
    }

    /// Set padding for the content area (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `padding` - Padding in pixels
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Draw the panel on the specified surface.
    ///
    /// Renders the panel background and optional border.
    ///
    /// # Arguments
    ///
    /// * `surface` - Mutable reference to Surface to draw on
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `PrimitiveError` if rendering fails.
    pub fn draw(&self, surface: &mut Surface) -> Result<(), PrimitiveError> {
        // Draw background
        rect::draw_filled_rect(
            surface,
            self.x,
            self.y,
            self.width,
            self.height,
            self.bg_color,
        )?;

        // Draw border if enabled
        if self.has_border {
            rect::draw_outline_rect(
                surface,
                self.x,
                self.y,
                self.width,
                self.height,
                self.border_color,
                1,
            )?;
        }

        Ok(())
    }

    /// Get the content area (interior region for child elements).
    ///
    /// # Returns
    ///
    /// Tuple of (x, y, width, height) representing the usable content area.
    pub fn get_content_area(&self) -> (u32, u32, u32, u32) {
        let content_x = self.x + self.padding;
        let content_y = self.y + self.padding;
        let content_width = self.width.saturating_sub(2 * self.padding);
        let content_height = self.height.saturating_sub(2 * self.padding);

        (content_x, content_y, content_width, content_height)
    }

    /// Set panel's z-order for layering.
    pub fn set_z_order(&mut self, z_order: u32) {
        self.z_order = z_order;
    }
}

/// Label component - text display with optional background and centering.
///
/// A Label is a simple text display element that can optionally have a
/// background rectangle and be centered within its region.
///
/// # Fields
///
/// * `x` - X coordinate for text
/// * `y` - Y coordinate for text
/// * `text` - Label text content
/// * `text_color` - Text color (ARGB8888)
/// * `bg_color` - Background color (optional, ARGB8888)
/// * `centered` - Whether text should be centered
/// * `z_order` - Layer ordering
#[derive(Debug, Clone)]
pub struct Label {
    x: u32,
    y: u32,
    text: &'static str,
    text_color: u32,
    bg_color: Option<u32>,
    centered: bool,
    z_order: u32,
}

impl Label {
    /// Create a new label with specified position, text, and color.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate for text
    /// * `y` - Y coordinate for text
    /// * `text` - Label text (must be valid for entire label lifetime)
    /// * `text_color` - Text color (ARGB8888)
    ///
    /// # Returns
    ///
    /// A new Label with no background.
    pub fn new(x: u32, y: u32, text: &'static str, text_color: u32) -> Self {
        Label {
            x,
            y,
            text,
            text_color,
            bg_color: None,
            centered: false,
            z_order: 0,
        }
    }

    /// Add a background to the label (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `bg_color` - Background color (ARGB8888)
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_background(mut self, bg_color: u32) -> Self {
        self.bg_color = Some(bg_color);
        self
    }

    /// Enable or disable text centering (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `centered` - Whether to center the text
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    /// Draw the label on the specified surface.
    ///
    /// # Arguments
    ///
    /// * `surface` - Mutable reference to Surface to draw on
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `PrimitiveError` if rendering fails.
    pub fn draw(&self, surface: &mut Surface) -> Result<(), PrimitiveError> {
        match self.bg_color {
            Some(bg_color) => {
                text::draw_text_with_bg(
                    surface,
                    self.x,
                    self.y,
                    self.text,
                    self.text_color,
                    bg_color,
                )?;
            }
            None => {
                if self.centered {
                    let (surf_width, _) = surface.get_dimensions();
                    text::draw_text_centered(
                        surface,
                        self.x,
                        self.y,
                        surf_width.saturating_sub(self.x),
                        self.text,
                        self.text_color,
                    )?;
                } else {
                    text::draw_text(surface, self.x, self.y, self.text, self.text_color)?;
                }
            }
        }

        Ok(())
    }

    /// Set label's z-order for layering.
    pub fn set_z_order(&mut self, z_order: u32) {
        self.z_order = z_order;
    }
}

/// TextBox component - text input field with cursor support.
///
/// A TextBox provides a text input area with a configurable maximum length.
/// It supports character insertion, deletion, and cursor positioning.
///
/// # Fields
///
/// * `x` - Top-left X coordinate
/// * `y` - Top-left Y coordinate
/// * `width` - TextBox width in pixels
/// * `height` - TextBox height in pixels
/// * `text` - Current text buffer
/// * `cursor_pos` - Current cursor position in characters
/// * `bg_color` - Background fill color (ARGB8888)
/// * `border_color` - Border color (ARGB8888)
/// * `text_color` - Text color (ARGB8888)
/// * `max_length` - Maximum number of characters
#[derive(Debug, Clone)]
pub struct TextBox {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    text: [u8; 256],
    text_len: u32,
    cursor_pos: u32,
    bg_color: u32,
    border_color: u32,
    text_color: u32,
    max_length: u32,
}

impl TextBox {
    /// Create a new text input box with specified dimensions and max length.
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - TextBox width in pixels
    /// * `height` - TextBox height in pixels
    /// * `max_length` - Maximum number of characters (capped at 256)
    ///
    /// # Returns
    ///
    /// A new TextBox with empty text buffer.
    pub fn new(x: u32, y: u32, width: u32, height: u32, max_length: u32) -> Self {
        let max_len = core::cmp::min(max_length, 256);
        TextBox {
            x,
            y,
            width,
            height,
            text: [0u8; 256],
            text_len: 0,
            cursor_pos: 0,
            bg_color: 0xFF1a1a1a,
            border_color: 0xFF808080,
            text_color: 0xFFFFFFFF,
            max_length: max_len,
        }
    }

    /// Draw the text box on the specified surface.
    ///
    /// Renders the text box as a filled rectangle with border and text content.
    ///
    /// # Arguments
    ///
    /// * `surface` - Mutable reference to Surface to draw on
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `PrimitiveError` if rendering fails.
    pub fn draw(&self, surface: &mut Surface) -> Result<(), PrimitiveError> {
        // Draw background
        rect::draw_filled_rect(
            surface,
            self.x,
            self.y,
            self.width,
            self.height,
            self.bg_color,
        )?;

        // Draw border
        rect::draw_outline_rect(
            surface,
            self.x,
            self.y,
            self.width,
            self.height,
            self.border_color,
            1,
        )?;

        // Draw text if any
        if self.text_len > 0 {
            let text_x = self.x + 4;
            let text_y = self.y + 4;

            // Convert buffer to string slice
            let text_slice = core::str::from_utf8(&self.text[..self.text_len as usize])
                .unwrap_or("");

            text::draw_text(surface, text_x, text_y, text_slice, self.text_color)?;

            // Draw cursor
            let cursor_x = text_x + (self.cursor_pos * text::char_width());
            let cursor_y = text_y;
            rect::draw_filled_rect(
                surface,
                cursor_x,
                cursor_y,
                1,
                text::char_height(),
                self.text_color,
            )?;
        }

        Ok(())
    }

    /// Insert a character at the current cursor position.
    ///
    /// # Arguments
    ///
    /// * `ch` - Character to insert
    ///
    /// # Returns
    ///
    /// `true` if character was inserted, `false` if buffer is full.
    pub fn insert_char(&mut self, ch: char) -> bool {
        if self.text_len >= self.max_length {
            return false;
        }

        let byte = ch as u8;
        let cursor_pos = core::cmp::min(self.cursor_pos as usize, self.text_len as usize);

        // Shift characters right to make space
        for i in (cursor_pos..self.text_len as usize).rev() {
            self.text[i + 1] = self.text[i];
        }

        // Insert character
        self.text[cursor_pos] = byte;
        self.text_len += 1;
        self.cursor_pos += 1;

        true
    }

    /// Delete character before the cursor (backspace).
    ///
    /// # Returns
    ///
    /// `true` if a character was deleted, `false` if cursor is at beginning.
    pub fn backspace(&mut self) -> bool {
        if self.cursor_pos == 0 || self.text_len == 0 {
            return false;
        }

        let cursor_pos = self.cursor_pos as usize - 1;

        // Shift characters left
        for i in cursor_pos..self.text_len as usize - 1 {
            self.text[i] = self.text[i + 1];
        }

        self.text[self.text_len as usize - 1] = 0;
        self.text_len -= 1;
        self.cursor_pos -= 1;

        true
    }

    /// Get the current text content as a string slice.
    ///
    /// # Returns
    ///
    /// String slice containing current text.
    pub fn get_text(&self) -> &str {
        core::str::from_utf8(&self.text[..self.text_len as usize]).unwrap_or("")
    }

    /// Clear all text from the buffer.
    pub fn clear(&mut self) {
        self.text = [0u8; 256];
        self.text_len = 0;
        self.cursor_pos = 0;
    }

    /// Set cursor position.
    ///
    /// # Arguments
    ///
    /// * `pos` - New cursor position (clamped to valid range)
    pub fn set_cursor_pos(&mut self, pos: u32) {
        self.cursor_pos = core::cmp::min(pos, self.text_len);
    }

    /// Get current cursor position.
    pub fn get_cursor_pos(&self) -> u32 {
        self.cursor_pos
    }

    /// Set background color.
    pub fn set_bg_color(&mut self, color: u32) {
        self.bg_color = color;
    }

    /// Set text color.
    pub fn set_text_color(&mut self, color: u32) {
        self.text_color = color;
    }

    /// Set border color.
    pub fn set_border_color(&mut self, color: u32) {
        self.border_color = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new(10, 10, 100, 40, "Test", 0xFF0000FF, 0xFFFFFFFF);
        assert_eq!(button.x, 10);
        assert_eq!(button.y, 10);
        assert_eq!(button.width, 100);
        assert_eq!(button.height, 40);
    }

    #[test]
    fn test_button_hover_detection() {
        let mut button = Button::new(10, 10, 100, 40, "Test", 0xFF0000FF, 0xFFFFFFFF);
        assert!(!button.is_hovered);

        // Update hover with point inside button
        let changed = button.update_hover(50, 25);
        assert!(changed);
        assert!(button.is_hovered);

        // Update hover with point outside button
        let changed = button.update_hover(150, 50);
        assert!(changed);
        assert!(!button.is_hovered);
    }

    #[test]
    fn test_button_click_detection() {
        let button = Button::new(10, 10, 100, 40, "Test", 0xFF0000FF, 0xFFFFFFFF);

        // Point inside button
        assert!(button.is_clicked(50, 25));

        // Point outside button
        assert!(!button.is_clicked(150, 50));
        assert!(!button.is_clicked(5, 25));
    }

    #[test]
    fn test_panel_creation() {
        let panel = Panel::new(20, 20, 200, 150, 0xFF1a1a1a);
        assert_eq!(panel.x, 20);
        assert_eq!(panel.y, 20);
        assert_eq!(panel.width, 200);
        assert_eq!(panel.height, 150);
        assert!(!panel.has_border);
    }

    #[test]
    fn test_panel_with_border() {
        let panel = Panel::new(20, 20, 200, 150, 0xFF1a1a1a).with_border(0xFF808080);
        assert!(panel.has_border);
        assert_eq!(panel.border_color, 0xFF808080);
    }

    #[test]
    fn test_panel_content_area() {
        let panel = Panel::new(20, 20, 200, 150, 0xFF1a1a1a).with_padding(10);
        let (x, y, width, height) = panel.get_content_area();
        assert_eq!(x, 30);
        assert_eq!(y, 30);
        assert_eq!(width, 180);
        assert_eq!(height, 130);
    }

    #[test]
    fn test_label_creation() {
        let label = Label::new(10, 10, "Hello", 0xFFFFFFFF);
        assert_eq!(label.x, 10);
        assert_eq!(label.y, 10);
        assert_eq!(label.text, "Hello");
        assert!(!label.centered);
        assert!(label.bg_color.is_none());
    }

    #[test]
    fn test_label_with_background() {
        let label = Label::new(10, 10, "Hello", 0xFFFFFFFF).with_background(0xFF0000FF);
        assert!(label.bg_color.is_some());
        assert_eq!(label.bg_color.unwrap(), 0xFF0000FF);
    }

    #[test]
    fn test_label_with_centered() {
        let label = Label::new(10, 10, "Hello", 0xFFFFFFFF).with_centered(true);
        assert!(label.centered);
    }

    #[test]
    fn test_textbox_creation() {
        let textbox = TextBox::new(10, 10, 200, 30, 32);
        assert_eq!(textbox.x, 10);
        assert_eq!(textbox.y, 10);
        assert_eq!(textbox.width, 200);
        assert_eq!(textbox.height, 30);
        assert_eq!(textbox.max_length, 32);
        assert_eq!(textbox.text_len, 0);
        assert_eq!(textbox.cursor_pos, 0);
    }

    #[test]
    fn test_textbox_insert_char() {
        let mut textbox = TextBox::new(10, 10, 200, 30, 32);

        assert!(textbox.insert_char('H'));
        assert!(textbox.insert_char('i'));
        assert_eq!(textbox.text_len, 2);
        assert_eq!(textbox.cursor_pos, 2);
        assert_eq!(textbox.get_text(), "Hi");
    }

    #[test]
    fn test_textbox_backspace() {
        let mut textbox = TextBox::new(10, 10, 200, 30, 32);
        textbox.insert_char('H');
        textbox.insert_char('i');

        assert!(textbox.backspace());
        assert_eq!(textbox.text_len, 1);
        assert_eq!(textbox.cursor_pos, 1);
        assert_eq!(textbox.get_text(), "H");

        assert!(textbox.backspace());
        assert_eq!(textbox.text_len, 0);
    }

    #[test]
    fn test_textbox_max_length() {
        let mut textbox = TextBox::new(10, 10, 200, 30, 3);

        assert!(textbox.insert_char('A'));
        assert!(textbox.insert_char('B'));
        assert!(textbox.insert_char('C'));
        assert!(!textbox.insert_char('D')); // Should fail - max length reached
        assert_eq!(textbox.text_len, 3);
    }

    #[test]
    fn test_textbox_clear() {
        let mut textbox = TextBox::new(10, 10, 200, 30, 32);
        textbox.insert_char('H');
        textbox.insert_char('i');

        textbox.clear();
        assert_eq!(textbox.text_len, 0);
        assert_eq!(textbox.cursor_pos, 0);
        assert_eq!(textbox.get_text(), "");
    }

    #[test]
    fn test_lighten_color() {
        let color = 0xFF0000FF;
        let lighter = Button::lighten_color(color);
        assert_eq!(lighter & 0xFF000000, 0xFF000000); // Alpha unchanged
    }
}
