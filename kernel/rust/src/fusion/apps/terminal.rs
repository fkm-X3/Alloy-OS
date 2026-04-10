//! Terminal Application for Fusion
//!
//! Provides a launchable terminal application that runs within Fusion.
//! The terminal manages its own surface for rendering text and handles
//! keyboard input for user interaction.
//!
//! # Architecture
//!
//! TerminalApp manages:
//! - A drawable surface for terminal content
//! - Window position and size
//! - Visibility and focus state
//! - Integration with Compositor for rendering
//! - Keyboard input processing

use crate::fusion::surface::Surface;
use super::{AppError, ApplicationLifecycle};
use alloc::vec::Vec;

/// Default terminal width in characters
const TERMINAL_WIDTH: u32 = 80;
/// Default terminal height in characters
const TERMINAL_HEIGHT: u32 = 24;
/// Character cell width in pixels (8x16 font)
const CHAR_WIDTH: u32 = 8;
/// Character cell height in pixels (8x16 font)
const CHAR_HEIGHT: u32 = 16;

/// Terminal application - first launchable app in Fusion
///
/// Manages a terminal window with text rendering capabilities.
/// Can be launched/closed dynamically and processes keyboard input.
///
/// # Example
///
/// ```no_run
/// # use kernel::fusion::apps::TerminalApp;
/// let mut terminal = TerminalApp::new()?;
/// // launch() requires mutable reference to Compositor
/// # Ok::<(), Box<dyn core::fmt::Debug>>(())
/// ```
#[derive(Debug)]
pub struct TerminalApp {
    /// Surface for rendering terminal content
    surface: Option<Surface>,
    /// Surface ID assigned by compositor (-1 if not launched)
    surface_id: Option<u32>,
    /// Window position X
    x: i32,
    /// Window position Y
    y: i32,
    /// Window width in pixels
    width: u32,
    /// Window height in pixels
    height: u32,
    /// Visibility flag
    visible: bool,
    /// Focus flag
    focused: bool,
    /// Text buffer for terminal content
    text_buffer: Vec<u8>,
    /// Current cursor position in text buffer
    cursor_pos: usize,
    /// Maximum buffer capacity
    max_capacity: usize,
}

impl TerminalApp {
    /// Create a new terminal application
    ///
    /// Initializes a terminal window with default dimensions (80x24 chars).
    /// The terminal is not yet visible or running - call `launch()` to start it.
    ///
    /// # Returns
    ///
    /// `Ok(TerminalApp)` with initialized state, or `Err` if allocation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::fusion::apps::TerminalApp;
    /// let terminal = TerminalApp::new()?;
    /// # Ok::<(), kernel::fusion::apps::AppError>(())
    /// ```
    pub fn new() -> Result<Self, AppError> {
        // Calculate dimensions in pixels
        let width = TERMINAL_WIDTH * CHAR_WIDTH;
        let height = TERMINAL_HEIGHT * CHAR_HEIGHT;

        // Validate dimensions
        if width == 0 || height == 0 {
            return Err(AppError::InvalidDimensions);
        }

        // Initialize text buffer with capacity for terminal content
        let max_capacity = (TERMINAL_WIDTH * TERMINAL_HEIGHT) as usize;

        Ok(TerminalApp {
            surface: None,
            surface_id: None,
            x: 0,
            y: 0,
            width,
            height,
            visible: false,
            focused: false,
            text_buffer: Vec::new(),
            cursor_pos: 0,
            max_capacity,
        })
    }

    /// Move the terminal window to a new position
    ///
    /// # Arguments
    ///
    /// * `x` - New X coordinate
    /// * `y` - New Y coordinate
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;

        // Update surface position if it exists
        if let Some(surface) = &mut self.surface {
            surface.set_position(x, y);
        }
    }

    /// Resize the terminal window
    ///
    /// Note: Resizing is limited by Surface implementation.
    /// Currently this is informational only.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), AppError> {
        if width == 0 || height == 0 {
            return Err(AppError::InvalidDimensions);
        }

        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Set terminal visibility
    ///
    /// # Arguments
    ///
    /// * `visible` - Visibility state
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;

        if let Some(surface) = &mut self.surface {
            surface.set_visible(visible);
        }
    }

    /// Set terminal focus
    ///
    /// # Arguments
    ///
    /// * `focused` - Focus state
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Write text to terminal
    ///
    /// Appends text to the terminal buffer. If buffer is full, oldest content is lost.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to write
    pub fn write(&mut self, text: &[u8]) -> Result<(), AppError> {
        for &byte in text {
            if self.text_buffer.len() >= self.max_capacity {
                // Remove oldest line when buffer full
                if let Some(newline_pos) = self.text_buffer.iter().position(|&b| b == b'\n') {
                    self.text_buffer.drain(0..=newline_pos);
                    self.cursor_pos = self.cursor_pos.saturating_sub(newline_pos + 1);
                } else {
                    self.text_buffer.clear();
                    self.cursor_pos = 0;
                }
            }

            self.text_buffer.push(byte);
            self.cursor_pos = self.text_buffer.len();
        }

        Ok(())
    }

    /// Clear terminal content
    pub fn clear(&mut self) -> Result<(), AppError> {
        self.text_buffer.clear();
        self.cursor_pos = 0;

        if let Some(surface) = &mut self.surface {
            surface.clear(0xFF000000)
                .map_err(|_| AppError::SurfaceError)?;
        }

        Ok(())
    }

    /// Get current terminal dimensions in characters
    pub fn get_dimensions(&self) -> (u32, u32) {
        (TERMINAL_WIDTH, TERMINAL_HEIGHT)
    }

    /// Get window size in pixels
    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get window position
    pub fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// Get terminal buffer as string slice
    pub fn get_buffer(&self) -> &[u8] {
        &self.text_buffer
    }
}

impl ApplicationLifecycle for TerminalApp {
    /// Launch terminal in compositor
    ///
    /// Creates a surface for the terminal and adds it to rendering pipeline.
    /// Must be called before terminal can be displayed.
    ///
    /// # Returns
    ///
    /// `Ok(surface_id)` on success, or `Err` if already launched or creation fails.
    fn launch(&mut self) -> Result<u32, AppError> {
        // Check if already running
        if self.surface_id.is_some() {
            return Err(AppError::AlreadyRunning);
        }

        // Create surface for terminal
        let mut surface = Surface::new(self.width, self.height)
            .map_err(|_| AppError::SurfaceError)?;

        // Position and make visible
        surface.set_position(self.x, self.y);
        surface.set_visible(true);

        // Assign surface ID (use position in this simple implementation)
        let surface_id = 1;

        self.surface = Some(surface);
        self.surface_id = Some(surface_id);
        self.visible = true;

        Ok(surface_id)
    }

    /// Close terminal and remove from compositor
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err` if not running.
    fn close(&mut self) -> Result<(), AppError> {
        if self.surface_id.is_none() {
            return Err(AppError::NotRunning);
        }

        self.surface = None;
        self.surface_id = None;
        self.visible = false;

        Ok(())
    }

    /// Handle keyboard input
    ///
    /// Processes keyboard input and updates terminal state.
    /// Currently accepts characters and special keys.
    ///
    /// # Arguments
    ///
    /// * `key` - Key code
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err` if not running.
    fn handle_input(&mut self, key: u32) -> Result<(), AppError> {
        if self.surface_id.is_none() {
            return Err(AppError::NotRunning);
        }

        if !self.focused {
            return Ok(());
        }

        // Handle special keys
        match key {
            0x08 => {
                // Backspace
                if self.cursor_pos > 0 {
                    self.text_buffer.pop();
                    self.cursor_pos -= 1;
                }
            }
            0x0D => {
                // Enter/Return
                self.text_buffer.push(b'\n');
                self.cursor_pos += 1;
            }
            0x09 => {
                // Tab
                for _ in 0..4 {
                    if self.text_buffer.len() < self.max_capacity {
                        self.text_buffer.push(b' ');
                        self.cursor_pos += 1;
                    }
                }
            }
            c if c >= 32 && c < 127 => {
                // Printable ASCII
                if self.text_buffer.len() < self.max_capacity {
                    self.text_buffer.push(c as u8);
                    self.cursor_pos += 1;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Update terminal display
    ///
    /// Renders text buffer to surface. This would typically be called
    /// each frame or after text changes.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err` if not running or rendering fails.
    fn update(&mut self) -> Result<(), AppError> {
        if let Some(surface) = &mut self.surface {
            // Clear surface to black
            surface.clear(0xFF000000)
                .map_err(|_| AppError::DisplayError)?;

            // Render text buffer to surface
            // Note: Full text rendering requires font support
            // For now, we maintain the buffer state
            // Future: Implement with glyph rendering
        }

        Ok(())
    }

    /// Check if terminal is active
    fn is_active(&self) -> bool {
        self.surface_id.is_some() && self.visible
    }

    /// Get application name
    fn name(&self) -> &str {
        "Terminal"
    }
}
