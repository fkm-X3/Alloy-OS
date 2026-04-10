//! Surface - Drawable surface abstraction
//!
//! Provides an abstraction for drawable surfaces that can be rendered to and manipulated.
//! Surfaces support off-screen drawing, positioning, visibility, and z-ordering for
//! compositing with other surfaces.
//!
//! # Architecture
//!
//! Each Surface maintains:
//! - An off-screen pixel buffer (ARGB8888 format)
//! - Position and visibility state
//! - Z-order for layering
//!
//! Surfaces can be rendered to independently and then composited onto the display
//! or other surfaces with efficient blitting and clipping.

use alloc::vec::Vec;
use core::fmt::Debug;
use crate::graphics::Display;

/// Error types for surface operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceError {
    /// Pixel operation out of bounds
    OutOfBounds,
    /// Buffer allocation failed
    AllocationFailed,
    /// Display operation failed
    DisplayError,
    /// Width or height is invalid (zero or too large)
    InvalidDimensions,
}

impl core::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SurfaceError::OutOfBounds => write!(f, "Pixel operation out of bounds"),
            SurfaceError::AllocationFailed => write!(f, "Buffer allocation failed"),
            SurfaceError::DisplayError => write!(f, "Display operation failed"),
            SurfaceError::InvalidDimensions => write!(f, "Invalid surface dimensions"),
        }
    }
}

/// Drawable surface abstraction for off-screen rendering.
///
/// Surfaces provide an abstraction for drawing operations independent of the display.
/// They maintain their own pixel buffer and can be positioned, made visible/invisible,
/// and composed with other surfaces.
///
/// # Memory Layout
///
/// The surface buffer stores pixels in ARGB8888 format:
/// - Each pixel is 4 bytes: [Alpha | Red | Green | Blue]
/// - Pixels are stored in row-major order (left-to-right, top-to-bottom)
/// - Total memory = width × height × 4 bytes
///
/// # Example
///
/// ```no_run
/// # use kernel::fusion::surface::{Surface, SurfaceError};
/// let mut surface = Surface::new(800, 600)?;
/// surface.clear(0xFF000000)?;  // Clear to black
/// surface.put_pixel(100, 100, 0xFFFF0000)?;  // Draw red pixel
/// surface.fill_rect(50, 50, 200, 200, 0xFF00FF00)?;  // Draw green rectangle
/// # Ok::<(), SurfaceError>(())
/// ```
#[derive(Debug)]
pub struct Surface {
    /// Off-screen pixel buffer (ARGB8888, row-major)
    buffer: Vec<u32>,
    /// Surface width in pixels
    width: u32,
    /// Surface height in pixels
    height: u32,
    /// Screen position X (can be negative)
    x: i32,
    /// Screen position Y (can be negative)
    y: i32,
    /// Visibility flag
    visible: bool,
    /// Z-order for layering (higher = on top)
    z_order: u32,
}

impl Surface {
    /// Create a new Surface with the given dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Surface width in pixels (must be > 0)
    /// * `height` - Surface height in pixels (must be > 0)
    ///
    /// # Returns
    ///
    /// `Ok(Surface)` with initialized black buffer, or `Err` if allocation fails.
    ///
    /// # Properties
    ///
    /// New surfaces are:
    /// - Filled with black (0xFF000000)
    /// - Positioned at (0, 0)
    /// - Visible by default
    /// - Z-order 0
    pub fn new(width: u32, height: u32) -> Result<Self, SurfaceError> {
        // Validate dimensions
        if width == 0 || height == 0 {
            return Err(SurfaceError::InvalidDimensions);
        }

        // Check for overflow in size calculation
        let total_pixels = width
            .checked_mul(height)
            .ok_or(SurfaceError::InvalidDimensions)?;

        // Allocate buffer, initialized to black (0xFF000000 = opaque black)
        let buffer = alloc::vec![0xFF000000; total_pixels as usize];

        if buffer.len() != total_pixels as usize {
            return Err(SurfaceError::AllocationFailed);
        }

        Ok(Surface {
            buffer,
            width,
            height,
            x: 0,
            y: 0,
            visible: true,
            z_order: 0,
        })
    }

    /// Write a single pixel at the given surface coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal coordinate within surface (0 = left edge)
    /// * `y` - Vertical coordinate within surface (0 = top edge)
    /// * `color` - Color value in ARGB8888 format (0xAARRGGBB)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `OutOfBounds` if coordinates are outside surface.
    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) -> Result<(), SurfaceError> {
        if x >= self.width || y >= self.height {
            return Err(SurfaceError::OutOfBounds);
        }

        let index = (y * self.width + x) as usize;
        if index >= self.buffer.len() {
            return Err(SurfaceError::OutOfBounds);
        }

        self.buffer[index] = color;
        Ok(())
    }

    /// Clear the entire surface to a single color.
    ///
    /// # Arguments
    ///
    /// * `color` - Fill color in ARGB8888 format (0xAARRGGBB)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    pub fn clear(&mut self, color: u32) -> Result<(), SurfaceError> {
        for pixel in self.buffer.iter_mut() {
            *pixel = color;
        }
        Ok(())
    }

    /// Fill a rectangular region within the surface with a solid color.
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate (relative to surface origin)
    /// * `y` - Top-left Y coordinate (relative to surface origin)
    /// * `w` - Rectangle width in pixels
    /// * `h` - Rectangle height in pixels
    /// * `color` - Fill color in ARGB8888 format (0xAARRGGBB)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `OutOfBounds` if rectangle is completely outside surface.
    ///
    /// # Clipping
    ///
    /// The rectangle is automatically clipped to surface bounds. Partial overlaps
    /// are handled gracefully.
    pub fn fill_rect(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        color: u32,
    ) -> Result<(), SurfaceError> {
        if x >= self.width || y >= self.height {
            return Err(SurfaceError::OutOfBounds);
        }

        // Clamp rectangle to surface bounds
        let x_end = core::cmp::min(x + w, self.width);
        let y_end = core::cmp::min(y + h, self.height);

        for row in y..y_end {
            for col in x..x_end {
                let index = (row * self.width + col) as usize;
                if index < self.buffer.len() {
                    self.buffer[index] = color;
                }
            }
        }

        Ok(())
    }

    /// Set the surface position on the screen.
    ///
    /// # Arguments
    ///
    /// * `x` - Screen X coordinate (can be negative)
    /// * `y` - Screen Y coordinate (can be negative)
    ///
    /// # Notes
    ///
    /// Surfaces can be partially or completely off-screen. Clipping is performed
    /// during blitting to the display.
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Get the surface position.
    ///
    /// # Returns
    ///
    /// Tuple of (x, y) screen coordinates.
    pub fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// Set the surface visibility.
    ///
    /// # Arguments
    ///
    /// * `visible` - true to show, false to hide
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if the surface is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set the z-order (stacking order) for layering.
    ///
    /// # Arguments
    ///
    /// * `z` - Higher values are drawn on top of lower values
    pub fn set_z_order(&mut self, z: u32) {
        self.z_order = z;
    }

    /// Get the z-order.
    pub fn get_z_order(&self) -> u32 {
        self.z_order
    }

    /// Get the surface dimensions.
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) in pixels.
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get a reference to the pixel buffer.
    ///
    /// # Returns
    ///
    /// Slice of ARGB8888 pixels in row-major order.
    pub fn get_buffer(&self) -> &[u32] {
        &self.buffer
    }

    /// Get a mutable reference to the pixel buffer.
    ///
    /// # Returns
    ///
    /// Mutable slice of ARGB8888 pixels in row-major order.
    pub fn get_buffer_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    /// Blit (copy) this surface to a display with automatic clipping.
    ///
    /// # Arguments
    ///
    /// * `display` - Target display (must implement Display with Error = SurfaceError)
    /// * `x` - Screen X coordinate where surface is drawn
    /// * `y` - Screen Y coordinate where surface is drawn
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `DisplayError` if display operation fails.
    ///
    /// # Behavior
    ///
    /// - Pixels from this surface are copied to the display at the given screen coordinates
    /// - Out-of-bounds pixels are clipped automatically
    /// - Partially off-screen surfaces are handled gracefully
    /// - The entire surface is copied; use `blit_rect` for partial blits
    ///
    /// # Performance
    ///
    /// This is an efficient pixel-by-pixel copy with clipping. For large surfaces
    /// or many surfaces, consider batching operations.
    pub fn blit_to_display<B>(
        &self,
        display: &mut dyn Display<Error = SurfaceError, Buffer = B>,
        x: i32,
        y: i32,
    ) -> Result<(), SurfaceError>
    where
        B: crate::graphics::FramebufferBuffer,
    {
        let (screen_width, screen_height) = display.get_resolution();

        // Calculate the region of the surface that's visible on-screen
        let start_x = if x < 0 { (-x) as u32 } else { 0 };
        let start_y = if y < 0 { (-y) as u32 } else { 0 };

        let end_x = core::cmp::min(self.width, screen_width.saturating_sub(x.max(0) as u32));
        let end_y = core::cmp::min(self.height, screen_height.saturating_sub(y.max(0) as u32));

        if start_x >= end_x || start_y >= end_y {
            return Ok(());
        }

        // Copy pixels with clipping
        for sy in start_y..end_y {
            for sx in start_x..end_x {
                let pixel = self.buffer[(sy * self.width + sx) as usize];
                let screen_x = (x + sx as i32) as u32;
                let screen_y = (y + sy as i32) as u32;

                display.pixel_put(screen_x, screen_y, pixel);
            }
        }

        Ok(())
    }

    /// Blit a rectangular region of this surface to a display.
    ///
    /// # Arguments
    ///
    /// * `display` - Target display
    /// * `src_x` - Source X coordinate within surface
    /// * `src_y` - Source Y coordinate within surface
    /// * `src_w` - Source width in pixels
    /// * `src_h` - Source height in pixels
    /// * `dst_x` - Destination screen X coordinate
    /// * `dst_y` - Destination screen Y coordinate
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or error if the source region is invalid.
    ///
    /// # Clipping
    ///
    /// Both source and destination are clipped appropriately to their bounds.
    pub fn blit_rect_to_display<B>(
        &self,
        display: &mut dyn Display<Error = SurfaceError, Buffer = B>,
        src_x: u32,
        src_y: u32,
        src_w: u32,
        src_h: u32,
        dst_x: i32,
        dst_y: i32,
    ) -> Result<(), SurfaceError>
    where
        B: crate::graphics::FramebufferBuffer,
    {
        // Validate source region
        if src_x >= self.width || src_y >= self.height {
            return Err(SurfaceError::OutOfBounds);
        }

        // Clamp source region to surface bounds
        let src_x_end = core::cmp::min(src_x + src_w, self.width);
        let src_y_end = core::cmp::min(src_y + src_h, self.height);

        let (screen_width, screen_height) = display.get_resolution();

        // Copy pixels with clipping
        for sy in src_y..src_y_end {
            for sx in src_x..src_x_end {
                let rel_x = (sx - src_x) as i32;
                let rel_y = (sy - src_y) as i32;
                let screen_x = dst_x + rel_x;
                let screen_y = dst_y + rel_y;

                if screen_x >= 0 && screen_y >= 0
                    && (screen_x as u32) < screen_width
                    && (screen_y as u32) < screen_height
                {
                    let pixel = self.buffer[(sy * self.width + sx) as usize];
                    display.pixel_put(screen_x as u32, screen_y as u32, pixel);
                }
            }
        }

        Ok(())
    }

    /// Resize the surface to new dimensions.
    ///
    /// # Arguments
    ///
    /// * `new_width` - New width in pixels
    /// * `new_height` - New height in pixels
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or error if allocation fails.
    ///
    /// # Behavior
    ///
    /// - New buffer is filled with black (0xFF000000)
    /// - Old content is discarded
    /// - Position and z-order are preserved
    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<(), SurfaceError> {
        if new_width == 0 || new_height == 0 {
            return Err(SurfaceError::InvalidDimensions);
        }

        let total_pixels = new_width
            .checked_mul(new_height)
            .ok_or(SurfaceError::InvalidDimensions)?;

        let new_buffer = alloc::vec![0xFF000000; total_pixels as usize];

        if new_buffer.len() != total_pixels as usize {
            return Err(SurfaceError::AllocationFailed);
        }

        self.buffer = new_buffer;
        self.width = new_width;
        self.height = new_height;

        Ok(())
    }
}
