//! Graphics module providing display abstraction for Alloy OS kernel.
//!
//! This module defines a platform-independent Display trait that abstracts the
//! underlying graphics hardware. Implementations handle pixel writes, buffer
//! management, and screen resolution queries.
//!
//! # Architecture
//!
//! The graphics module is built with a trait-based design:
//! - **Display trait**: Core abstraction for graphics hardware with associated types
//! - **FramebufferBuffer trait**: Low-level buffer operations and memory management
//! - **Color module**: Color type definitions and conversions
//! - **Framebuffer module**: Hardware framebuffer management
//! - **Text module**: Text rendering utilities
//!
//! # Example
//!
//! ```no_run
//! # use kernel::graphics::Display;
//! // Implementations provide concrete display handlers
//! // let mut display = VgaDisplay::new();
//! // display.clear(0x000000);
//! // display.pixel_put(100, 100, 0xFF0000);  // Red pixel
//! // display.fill_rect(10, 10, 100, 100, 0x00FF00).ok();
//! // display.swap_buffer();
//! ```

pub mod color;
pub mod framebuffer;
pub mod text;
pub mod vesa;

use core::fmt::Debug;

/// Low-level framebuffer buffer operations.
///
/// This trait defines the interface for accessing underlying framebuffer memory
/// and metadata. It is typically implemented by graphics drivers and used internally
/// by the Display trait's helper methods.
///
/// # Safety
///
/// Implementers must ensure that:
/// - The address returned by `address()` points to valid, mapped memory
/// - The size returned by `size()` accurately reflects the allocated buffer
/// - Writes beyond `size()` bytes from the address are prevented by the caller
/// - The pitch value correctly represents scanline stride (including any padding)
///
/// # Example
///
/// ```no_run
/// # use kernel::graphics::FramebufferBuffer;
/// struct VgaBuffer;
///
/// impl FramebufferBuffer for VgaBuffer {
///     fn address(&self) -> *mut u8 {
///         0xB8000 as *mut u8
///     }
///
///     fn pitch(&self) -> u32 {
///         80 * 2  // 80 chars, 2 bytes per char (char + attr)
///     }
///
///     fn size(&self) -> usize {
///         80 * 25 * 2  // 80 x 25 text mode
///     }
/// }
/// ```
pub trait FramebufferBuffer: Debug {
    /// Get the starting memory address of the framebuffer.
    ///
    /// # Returns
    ///
    /// Mutable pointer to the framebuffer memory. The caller is responsible for
    /// ensuring writes stay within buffer bounds as defined by `size()`.
    ///
    /// # Safety
    ///
    /// This pointer is intentionally raw to allow direct hardware memory access.
    /// The caller must ensure that writes are properly bounded and aligned
    /// for the specific display mode.
    fn address(&self) -> *mut u8;

    /// Get the pitch (scanline stride) in bytes.
    ///
    /// The pitch may be larger than width × bytes_per_pixel if the hardware
    /// requires padding for alignment (common in many graphics modes).
    ///
    /// # Returns
    ///
    /// Bytes between the start of consecutive scanlines.
    ///
    /// # Example
    ///
    /// For a 1024×768 display in 32-bit color:
    /// - Minimum pitch: 1024 × 4 = 4096 bytes
    /// - Actual pitch may be 4096, 4224, or higher depending on alignment
    fn pitch(&self) -> u32;

    /// Get the total size of the framebuffer in bytes.
    ///
    /// # Returns
    ///
    /// Total number of bytes that can be safely written to the buffer
    /// starting from `address()`.
    ///
    /// # Notes
    ///
    /// This is typically `pitch() × height`, but may include additional
    /// space for double-buffering or hardware requirements.
    fn size(&self) -> usize;
}

/// Display trait for graphics hardware abstraction.
///
/// This trait defines the core interface for all graphics operations in the Alloy OS
/// kernel. It provides both required methods for essential operations and optional
/// helper methods with default implementations for common drawing tasks.
///
/// # Design Philosophy
///
/// The Display trait abstracts hardware-specific details while maintaining efficiency.
/// Implementations should:
/// - Minimize copying and memory transfers
/// - Support the native color format of the hardware
/// - Allow both immediate and deferred rendering modes
///
/// # Associated Types
///
/// - **Error**: Error type for rendering operations (must implement Debug)
/// - **Buffer**: The underlying buffer implementation (must implement FramebufferBuffer)
///
/// # Thread Safety
///
/// Display implementations may not be thread-safe. Synchronization must be
/// handled by the caller using mutexes or other synchronization primitives.
///
/// # Performance Notes
///
/// - `pixel_put()` should be used sparingly; batch operations via `fill_rect()` or
///   `draw_line()` are typically more efficient
/// - `clear()` should take advantage of hardware capabilities (e.g., DMA, memset)
/// - `swap_buffer()` may involve expensive operations; avoid calling excessively
/// - Dirty region tracking can reduce bandwidth for buffered displays
///
/// # Safety Considerations
///
/// - Coordinate bounds are not validated by default; implementers must handle
///   out-of-bounds access (panic, clamp, or return error)
/// - Color format depends on bits per pixel; callers must convert colors appropriately
/// - Direct memory access via buffer pointer may bypass synchronization checks
///
/// # Example Implementation
///
/// ```no_run
/// # use kernel::graphics::{Display, FramebufferBuffer};
/// # use core::fmt::Debug;
/// #[derive(Debug)]
/// struct VgaError;
///
/// #[derive(Debug)]
/// struct VgaBuffer {
///     address: *mut u8,
///     pitch: u32,
///     size: usize,
/// }
///
/// impl FramebufferBuffer for VgaBuffer {
///     fn address(&self) -> *mut u8 { self.address }
///     fn pitch(&self) -> u32 { self.pitch }
///     fn size(&self) -> usize { self.size }
/// }
///
/// struct VgaDisplay {
///     buffer: VgaBuffer,
///     width: u32,
///     height: u32,
///     dirty: bool,
/// }
///
/// impl Display for VgaDisplay {
///     type Error = VgaError;
///     type Buffer = VgaBuffer;
///
///     fn pixel_put(&mut self, x: u32, y: u32, color: u32) {
///         if x >= self.width || y >= self.height {
///             return;  // Out of bounds
///         }
///         unsafe {
///             let offset = y as usize * self.buffer.pitch() as usize + x as usize * 4;
///             *(self.buffer.address().add(offset) as *mut u32) = color;
///         }
///         self.dirty = true;
///     }
///
///     fn clear(&mut self, color: u32) {
///         unsafe {
///             // Efficient memset-like operation
///             let buf = self.buffer.address() as *mut u32;
///             let count = self.buffer.size() / 4;
///             for i in 0..count {
///                 *buf.add(i) = color;
///             }
///         }
///         self.dirty = true;
///     }
///
///     fn swap_buffer(&mut self) {
///         self.dirty = false;
///     }
///
///     fn get_resolution(&self) -> (u32, u32) {
///         (self.width, self.height)
///     }
///
///     fn get_bits_per_pixel(&self) -> u8 {
///         32
///     }
///
///     fn get_buffer(&self) -> &Self::Buffer {
///         &self.buffer
///     }
/// }
/// ```
pub trait Display {
    /// Error type for rendering operations.
    ///
    /// Used by helper methods that may fail due to invalid parameters or
    /// hardware limitations. Implementers should define appropriate error variants.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(Debug)]
    /// enum DisplayError {
    ///     OutOfBounds,
    ///     InvalidColor,
    ///     HardwareError,
    /// }
    /// ```
    type Error: Debug;

    /// Framebuffer buffer type used for low-level memory access.
    ///
    /// This associated type allows Display implementations to use different
    /// buffer strategies (e.g., VGA text mode, VESA graphics, software framebuffer).
    type Buffer: FramebufferBuffer;

    // === Required Methods ===

    /// Write a single pixel at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal coordinate (0 = left edge)
    /// * `y` - Vertical coordinate (0 = top edge)
    /// * `color` - Color value (format depends on bits per pixel)
    ///
    /// # Panics
    ///
    /// May panic if coordinates are outside valid screen bounds.
    /// Implementations may also clamp or ignore out-of-bounds writes.
    ///
    /// # Performance
    ///
    /// For drawing multiple pixels, prefer batch operations like `fill_rect()`.
    fn pixel_put(&mut self, x: u32, y: u32, color: u32);

    /// Clear the entire screen to a single color.
    ///
    /// # Arguments
    ///
    /// * `color` - Fill color (format depends on bits per pixel)
    ///
    /// # Implementation Notes
    ///
    /// This should be significantly faster than drawing individual pixels.
    /// Implementations should use hardware capabilities when available (e.g., DMA,
    /// memory fill instructions).
    fn clear(&mut self, color: u32);

    /// Swap or flush the framebuffer to display.
    ///
    /// For double-buffered displays, this swaps the back buffer to front.
    /// For single-buffered displays, this ensures all pending writes are
    /// visible on screen.
    ///
    /// # Implementation Notes
    ///
    /// This operation may be a no-op for hardware with immediate pixel write
    /// capabilities, but implementations must provide it for compatibility.
    /// Some implementations may use this to track dirty regions or trigger
    /// hardware updates.
    fn swap_buffer(&mut self);

    /// Get the screen resolution in pixels.
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) in pixels.
    fn get_resolution(&self) -> (u32, u32);

    /// Get the color depth of the display.
    ///
    /// # Returns
    ///
    /// Bits per pixel: typically 8, 16, 24, or 32.
    ///
    /// # Color Depth Reference
    ///
    /// - **8 bits**: Indexed color mode (palette mode)
    /// - **16 bits**: RGB565 (5 bits red, 6 bits green, 5 bits blue)
    /// - **24 bits**: RGB888 (3 bytes per pixel, MSB unused)
    /// - **32 bits**: XRGB8888 or ARGB8888 (4 bytes per pixel)
    fn get_bits_per_pixel(&self) -> u8;

    /// Get a reference to the underlying framebuffer buffer.
    ///
    /// # Returns
    ///
    /// Reference to the Display's associated Buffer implementation.
    /// This allows low-level direct access to framebuffer memory when needed.
    ///
    /// # Safety
    ///
    /// Direct access via the buffer's address pointer is unsafe and must be
    /// carefully bounded. Prefer higher-level methods when possible.
    fn get_buffer(&self) -> &Self::Buffer;

    // === Helper Methods (Default Implementations) ===

    /// Fill a rectangular region with a solid color.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of rectangle top-left corner
    /// * `y` - Y coordinate of rectangle top-left corner
    /// * `width` - Rectangle width in pixels
    /// * `height` - Rectangle height in pixels
    /// * `color` - Fill color
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or error if operation cannot be completed.
    ///
    /// # Implementation Notes
    ///
    /// The default implementation uses `pixel_put()` for each pixel, but
    /// implementations are encouraged to override this for better performance
    /// (e.g., using hardware acceleration or efficient memory writes).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::graphics::Display;
    /// # let mut display: &mut dyn Display<Error=(), Buffer=()> = unsafe { &mut *(0 as *mut _) };
    /// // Draw a 100x100 red square at (50, 50)
    /// let _ = display.fill_rect(50, 50, 100, 100, 0xFF0000);
    /// ```
    fn fill_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: u32,
    ) -> Result<(), Self::Error> {
        let (screen_width, screen_height) = self.get_resolution();

        // Clamp rectangle to screen bounds
        let x_end = core::cmp::min(x + width, screen_width);
        let y_end = core::cmp::min(y + height, screen_height);

        if x >= screen_width || y >= screen_height {
            return Ok(());
        }

        for py in y..y_end {
            for px in x..x_end {
                self.pixel_put(px, py, color);
            }
        }

        Ok(())
    }

    /// Draw a line from one point to another using Bresenham's algorithm.
    ///
    /// # Arguments
    ///
    /// * `x1` - Starting X coordinate
    /// * `y1` - Starting Y coordinate
    /// * `x2` - Ending X coordinate
    /// * `y2` - Ending Y coordinate
    /// * `color` - Line color
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or error if operation cannot be completed.
    ///
    /// # Algorithm
    ///
    /// Uses Bresenham's line algorithm for efficient integer-only rasterization.
    /// This provides optimal performance across all line angles.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::graphics::Display;
    /// # let mut display: &mut dyn Display<Error=(), Buffer=()> = unsafe { &mut *(0 as *mut _) };
    /// // Draw a diagonal line from (0,0) to (100,100)
    /// let _ = display.draw_line(0, 0, 100, 100, 0xFFFFFF);
    /// ```
    fn draw_line(
        &mut self,
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        color: u32,
    ) -> Result<(), Self::Error> {
        let (dx, dy) = (
            (x2 as i32 - x1 as i32).abs() as u32,
            (y2 as i32 - y1 as i32).abs() as u32,
        );

        let (sx, sy) = (
            if x1 < x2 { 1i32 } else { -1i32 },
            if y1 < y2 { 1i32 } else { -1i32 },
        );

        let mut err = (dx as i32 - dy as i32) / 2;
        let mut x = x1 as i32;
        let mut y = y1 as i32;

        loop {
            self.pixel_put(x as u32, y as u32, color);

            if x == x2 as i32 && y == y2 as i32 {
                break;
            }

            let e2 = err;
            if e2 > -(dy as i32) {
                err -= dy as i32;
                x += sx;
            }
            if e2 < dx as i32 {
                err += dx as i32;
                y += sy;
            }
        }

        Ok(())
    }

    /// Check if the display buffer needs to be refreshed.
    ///
    /// This is typically used by implementations that support partial updates
    /// or double-buffering to optimize bandwidth usage.
    ///
    /// # Returns
    ///
    /// `true` if the buffer has been modified since the last `swap_buffer()` call,
    /// `false` otherwise.
    ///
    /// # Implementation Notes
    ///
    /// The default implementation returns `true` (conservative: always refresh).
    /// Implementations that track dirty regions should override this method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::graphics::Display;
    /// # let mut display: &mut dyn Display<Error=(), Buffer=()> = unsafe { &mut *(0 as *mut _) };
    /// if display.is_dirty() {
    ///     // Perform expensive update operation
    ///     display.swap_buffer();
    /// }
    /// ```
    fn is_dirty(&self) -> bool {
        true
    }
}
