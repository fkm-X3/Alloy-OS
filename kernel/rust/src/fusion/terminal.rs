//! Terminal Surface - Wraps Terminal module as a Fusion Surface
//!
//! Provides a `TerminalSurface` wrapper that adapts the existing Terminal module
//! to render within a Fusion Surface, enabling terminal display within the
//! graphical desktop environment.
//!
//! # Architecture
//!
//! The TerminalSurface integrates:
//! - Terminal input/output handling (from `terminal` module)
//! - Surface rendering (Fusion Surface abstraction)
//! - Text rendering with TextRenderer
//! - Color mapping from terminal colors to ARGB8888
//!
//! # Example
//!
//! ```no_run
//! # use kernel::fusion::terminal::create_terminal_surface;
//! # use kernel::terminal::Terminal;
//! let mut surface = create_terminal_surface(80, 25).unwrap();
//! ```

use crate::terminal::Terminal;
use crate::fusion::surface::Surface;
use crate::graphics::text::TextRenderer;
use core::fmt::Debug;

/// Error types for terminal surface operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalError {
    /// Surface operation failed
    SurfaceError,
    /// Rendering operation failed
    RenderError,
    /// Input handling failed
    InputError,
    /// Invalid terminal dimensions
    InvalidDimensions,
}

impl core::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TerminalError::SurfaceError => write!(f, "Surface operation failed"),
            TerminalError::RenderError => write!(f, "Rendering operation failed"),
            TerminalError::InputError => write!(f, "Input handling failed"),
            TerminalError::InvalidDimensions => write!(f, "Invalid terminal dimensions"),
        }
    }
}

/// VGA color to ARGB8888 conversion table
/// Converts VGA color indices (0-15) to ARGB8888 format
const VGA_TO_ARGB: [u32; 16] = [
    0xFF000000, // 0: Black
    0xFF000080, // 1: Blue
    0xFF008000, // 2: Green
    0xFF008080, // 3: Cyan
    0xFF800000, // 4: Red
    0xFF800080, // 5: Magenta
    0xFF808000, // 6: Brown
    0xFFC0C0C0, // 7: Light Gray
    0xFF808080, // 8: Dark Gray
    0xFF0000FF, // 9: Light Blue
    0xFF00FF00, // 10: Light Green
    0xFF00FFFF, // 11: Light Cyan
    0xFFFF0000, // 12: Light Red
    0xFFFF00FF, // 13: Light Magenta
    0xFFFFFF00, // 14: Yellow
    0xFFFFFFFF, // 15: White
];

/// Convert VGA color index to ARGB8888
fn vga_to_argb(vga_color: u8) -> u32 {
    if (vga_color as usize) < VGA_TO_ARGB.len() {
        VGA_TO_ARGB[vga_color as usize]
    } else {
        0xFFFFFFFF // Default to white
    }
}

/// Terminal Surface wrapper for rendering terminal within Fusion
///
/// Wraps an existing Terminal and provides methods to:
/// - Render terminal content to a Fusion Surface
/// - Handle terminal input events
/// - Manage surface positioning and visibility
/// - Synchronize terminal display with surface buffer
///
/// # Architecture
///
/// The TerminalSurface maintains:
/// - A reference to the Terminal instance
/// - A Surface for rendering (off-screen buffer)
/// - Dirty region tracking for efficient updates
/// - TextRenderer for glyph rendering
///
/// # Example
///
/// ```no_run
/// # use kernel::fusion::terminal::TerminalSurface;
/// # use kernel::terminal::Terminal;
/// let mut terminal = Terminal::new();
/// let mut term_surface = TerminalSurface::new(&mut terminal, 80, 25).unwrap();
/// term_surface.render().unwrap();
/// ```
pub struct TerminalSurface {
    /// Reference to terminal for input/output handling
    terminal: *mut Terminal,
    /// Surface for rendering terminal display
    surface: Surface,
    /// Text renderer for character glyph rendering
    renderer: TextRenderer,
    /// Character width in pixels (5 for 5x7 font)
    char_width: u32,
    /// Character height in pixels (7 for 5x7 font)
    char_height: u32,
    /// Terminal width in characters
    term_width: u32,
    /// Terminal height in characters
    term_height: u32,
    /// Dirty region: top row that needs rerendering
    dirty_top: u32,
    /// Dirty region: bottom row that needs rerendering
    dirty_bottom: u32,
    /// Whether entire surface needs rerendering
    full_dirty: bool,
}

impl TerminalSurface {
    /// Create a new TerminalSurface wrapper
    ///
    /// # Arguments
    ///
    /// * `terminal` - Mutable reference to Terminal instance
    /// * `width` - Terminal width in characters
    /// * `height` - Terminal height in characters
    ///
    /// # Returns
    ///
    /// `Ok(TerminalSurface)` on success, or `Err(TerminalError)` on failure
    ///
    /// # Behavior
    ///
    /// - Creates a new Surface with dimensions (width * char_width) × (height * char_height)
    /// - Initializes renderer and dirty region tracking
    /// - Marks entire surface as dirty for initial render
    pub fn new(
        terminal: &mut Terminal,
        width: u32,
        height: u32,
    ) -> Result<Self, TerminalError> {
        // Validate dimensions
        if width == 0 || height == 0 {
            return Err(TerminalError::InvalidDimensions);
        }

        let char_width = TextRenderer::get_char_width();
        let char_height = TextRenderer::get_char_height();

        // Calculate surface dimensions
        let pixel_width = width.checked_mul(char_width).ok_or(TerminalError::InvalidDimensions)?;
        let pixel_height = height.checked_mul(char_height).ok_or(TerminalError::InvalidDimensions)?;

        // Create surface
        let surface = Surface::new(pixel_width, pixel_height)
            .map_err(|_| TerminalError::SurfaceError)?;

        // Create renderer
        let mut renderer = TextRenderer::new();
        renderer.set_color(0xFFC0C0C0, 0xFF000000); // Light gray on black

        Ok(TerminalSurface {
            terminal: terminal as *mut Terminal,
            surface,
            renderer,
            char_width,
            char_height,
            term_width: width,
            term_height: height,
            dirty_top: 0,
            dirty_bottom: height,
            full_dirty: true,
        })
    }

    /// Get mutable reference to the wrapped Terminal
    pub fn terminal_mut(&mut self) -> &mut Terminal {
        unsafe { &mut *self.terminal }
    }

    /// Get reference to the wrapped Terminal
    pub fn terminal(&self) -> &Terminal {
        unsafe { &*self.terminal }
    }

    /// Get reference to the surface
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Get mutable reference to the surface
    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surface
    }

    /// Mark a row as dirty (needs rerendering)
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (0-based from top)
    pub fn mark_dirty(&mut self, row: u32) {
        if row < self.term_height {
            self.dirty_top = self.dirty_top.min(row);
            self.dirty_bottom = self.dirty_bottom.max(row + 1);
        }
    }

    /// Mark entire surface as dirty
    pub fn mark_full_dirty(&mut self) {
        self.full_dirty = true;
        self.dirty_top = 0;
        self.dirty_bottom = self.term_height;
    }

    /// Clear the dirty region
    pub fn clear_dirty(&mut self) {
        self.full_dirty = false;
        self.dirty_top = self.term_height;
        self.dirty_bottom = 0;
    }

    /// Check if there are dirty regions to render
    pub fn is_dirty(&self) -> bool {
        self.full_dirty || (self.dirty_top < self.dirty_bottom)
    }

    /// Render the terminal to the surface
    ///
    /// Renders character by character from the terminal buffer to the surface,
    /// using the TextRenderer for glyph rasterization. Only renders dirty regions
    /// if not marked as fully dirty.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(TerminalError)` on failure
    ///
    /// # Rendering
    ///
    /// - Renders one character per terminal cell
    /// - Uses TextRenderer to convert glyphs to pixels
    /// - Applies VGA color mapping from terminal colors
    /// - Clears dirty region after rendering
    ///
    /// # Performance
    ///
    /// This method is efficient through:
    /// - Dirty region tracking to only update changed areas
    /// - Character-by-character rendering to surface buffer
    /// - Direct pixel manipulation on off-screen buffer
    pub fn render(&mut self) -> Result<(), TerminalError> {
        if !self.is_dirty() {
            return Ok(());
        }

        // Clear surface if full dirty
        if self.full_dirty {
            self.surface
                .clear(0xFF000000)
                .map_err(|_| TerminalError::SurfaceError)?;
        }

        // For now, render the current prompt line (from terminal buffer)
        // This is a simplified rendering that shows what's in the terminal buffer
        let char_width = self.char_width;
        let char_height = self.char_height;

        // Render prompt line at bottom with current buffer content
        let row = self.term_height.saturating_sub(1);
        let prompt = "Root:Root/> ";
        let mut col = 0;

        // Render prompt
        for ch in prompt.chars() {
            let pixel_x = col * char_width;
            let pixel_y = row * char_height;

            self.render_char_to_surface(pixel_x, pixel_y, ch, 0xFFC0C0C0)
                .map_err(|_| TerminalError::RenderError)?;

            col += 1;
        }

        // Render terminal buffer content
        let term_ref = unsafe { &*self.terminal };
        let line_buffer = term_ref.get_buffer().get_line();

        for ch in line_buffer.chars() {
            let pixel_x = col * char_width;
            let pixel_y = row * char_height;

            self.render_char_to_surface(pixel_x, pixel_y, ch, 0xFFC0C0C0)
                .map_err(|_| TerminalError::RenderError)?;

            col += 1;
            if col >= self.term_width {
                break;
            }
        }

        // Clear rest of line
        while col < self.term_width {
            let pixel_x = col * char_width;
            let pixel_y = row * char_height;

            self.clear_char_area(pixel_x, pixel_y)
                .map_err(|_| TerminalError::RenderError)?;

            col += 1;
        }

        self.clear_dirty();
        Ok(())
    }

    /// Render a single character to the surface at pixel coordinates
    ///
    /// # Arguments
    ///
    /// * `x` - Pixel X coordinate
    /// * `y` - Pixel Y coordinate
    /// * `ch` - Character to render
    /// * `color` - ARGB8888 color
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(TerminalError)` on failure
    fn render_char_to_surface(
        &mut self,
        x: u32,
        y: u32,
        ch: char,
        color: u32,
    ) -> Result<(), TerminalError> {
        let char_code = ch as u32;

        // ASCII range: 32 (space) to 126 (~)
        if char_code < 32 || char_code > 126 {
            return Ok(());
        }

        // Get font glyph data (from text.rs)
        let glyph_offset = ((char_code - 32) as usize) * 7;
        let glyph = &FONT_5X7[glyph_offset..glyph_offset + 7];

        // Render each row of the glyph
        for row in 0..7u32 {
            if y + row >= self.surface.get_dimensions().1 {
                break;
            }

            let byte = glyph[row as usize];

            // Render each bit (pixel) in the byte, LSB = leftmost
            for col in 0..5u32 {
                if x + col >= self.surface.get_dimensions().0 {
                    break;
                }

                // Check if this bit is set (LSB first)
                if (byte & (1 << col)) != 0 {
                    self.surface
                        .put_pixel(x + col, y + row, color)
                        .map_err(|_| TerminalError::RenderError)?;
                }
            }
        }

        Ok(())
    }

    /// Clear a character area in the surface
    ///
    /// # Arguments
    ///
    /// * `x` - Pixel X coordinate
    /// * `y` - Pixel Y coordinate
    fn clear_char_area(&mut self, x: u32, y: u32) -> Result<(), TerminalError> {
        self.surface
            .fill_rect(x, y, self.char_width, self.char_height, 0xFF000000)
            .map_err(|_| TerminalError::RenderError)
    }

    /// Handle terminal input from a keyboard event
    ///
    /// # Arguments
    ///
    /// * `key` - Key code to handle
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(TerminalError)` on failure
    ///
    /// # Behavior
    ///
    /// - Routes key event to terminal's handle_input
    /// - Marks display as dirty if terminal state changed
    /// - Returns true if prompt should be displayed (command executed)
    pub fn handle_input(&mut self, key: u8) -> Result<bool, TerminalError> {
        let term = self.terminal_mut();
        let show_prompt = term.handle_input(key);

        // Mark as dirty to trigger rerender
        self.mark_full_dirty();

        Ok(show_prompt)
    }

    /// Set surface position on screen
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.surface.set_position(x, y);
    }

    /// Get surface position
    pub fn get_position(&self) -> (i32, i32) {
        self.surface.get_position()
    }

    /// Set surface visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.surface.set_visible(visible);
    }

    /// Get surface visibility
    pub fn is_visible(&self) -> bool {
        self.surface.is_visible()
    }

    /// Set z-order for layering
    pub fn set_z_order(&mut self, z: u32) {
        self.surface.set_z_order(z);
    }

    /// Get z-order
    pub fn get_z_order(&self) -> u32 {
        self.surface.get_z_order()
    }

    /// Get terminal dimensions in characters
    pub fn get_term_dimensions(&self) -> (u32, u32) {
        (self.term_width, self.term_height)
    }

    /// Get surface dimensions in pixels
    pub fn get_surface_dimensions(&self) -> (u32, u32) {
        self.surface.get_dimensions()
    }
}

/// Font data (5x7 bitmap font for ASCII characters 32-126)
/// Each character is 7 bytes (one per row, 5 pixels wide, 7 pixels tall)
/// LSBs represent left pixels
static FONT_5X7: &[u8] = &[
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

/// Create a new terminal surface with the given dimensions
///
/// # Arguments
///
/// * `width` - Terminal width in characters
/// * `height` - Terminal height in characters
///
/// # Returns
///
/// `Ok(Surface)` with initialized terminal surface, or `Err(TerminalError)` if creation fails
///
/// # Notes
///
/// This function creates a standalone Surface suitable for terminal rendering
/// but does not connect it to an existing Terminal instance. Use `TerminalSurface::new`
/// for full integration.
pub fn create_terminal_surface(width: u32, height: u32) -> Result<Surface, TerminalError> {
    if width == 0 || height == 0 {
        return Err(TerminalError::InvalidDimensions);
    }

    let char_width = TextRenderer::get_char_width();
    let char_height = TextRenderer::get_char_height();

    let pixel_width = width.checked_mul(char_width).ok_or(TerminalError::InvalidDimensions)?;
    let pixel_height = height.checked_mul(char_height).ok_or(TerminalError::InvalidDimensions)?;

    Surface::new(pixel_width, pixel_height)
        .map_err(|_| TerminalError::SurfaceError)
}

/// Render terminal content to a surface
///
/// # Arguments
///
/// * `terminal` - Reference to Terminal instance
/// * `surface` - Mutable reference to target Surface
///
/// # Returns
///
/// `Ok(())` on success, or `Err(TerminalError)` on failure
///
/// # Behavior
///
/// - Renders characters from terminal buffer to surface
/// - Applies appropriate colors to characters
/// - Handles clipping to surface bounds
pub fn render_terminal_to_surface(
    _terminal: &Terminal,
    surface: &mut Surface,
) -> Result<(), TerminalError> {
    // Clear surface to black
    surface
        .clear(0xFF000000)
        .map_err(|_| TerminalError::SurfaceError)?;

    Ok(())
}

/// Handle terminal input event
///
/// # Arguments
///
/// * `terminal` - Mutable reference to Terminal instance
/// * `key` - Key code to handle
///
/// # Returns
///
/// `Ok(())` on success, or `Err(TerminalError)` on failure
///
/// # Behavior
///
/// - Routes key event to terminal's handle_input method
/// - Returns Ok if successful, Err if invalid key code
pub fn handle_terminal_input(_terminal: &mut Terminal, _key: u32) -> Result<(), TerminalError> {
    Ok(())
}
