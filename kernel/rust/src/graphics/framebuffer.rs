//! Framebuffer abstraction for direct hardware memory access.
//!
//! This module provides a safe wrapper around the raw framebuffer memory, handling
//! hardware-specific details like memory addresses, stride calculations, and
//! color format conversions.
//!
//! # Architecture
//!
//! The framebuffer is typically located at a fixed hardware address
//! (e.g., 0xB8000 for VGA text mode, or a higher address for graphics modes).
//! This module abstracts:
//!
//! - Memory address and size validation
//! - Row stride (bytes between consecutive scanlines)
//! - Pixel format and color depth (8, 16, 24, 32-bit)
//! - Bounds checking on all pixel operations
//! - Safe color format conversions
//!
//! # Supported Color Formats
//!
//! - **8-bit (indexed)**: VGA palette mode, 256 colors
//! - **16-bit (RGB565)**: 5 bits red, 6 bits green, 5 bits blue
//! - **24-bit (RGB888)**: 8 bits per channel, 3 bytes per pixel
//! - **32-bit (ARGB8888)**: 8 bits per channel with alpha
//!
//! # Safety Assumptions
//!
//! This module assumes:
//! - The framebuffer address points to mapped, readable/writable memory
//! - The framebuffer size is correctly calculated as stride × height
//! - The kernel manages the lifetime of the framebuffer memory
//! - Single-threaded or externally synchronized access
//!
//! # Examples
//!
//! ```no_run
//! # use kernel::graphics::framebuffer::{FramebufferInfo, Framebuffer};
//! // Create framebuffer info for a 1024×768 VESA display
//! let info = FramebufferInfo::new(
//!     0xFD000000,  // Linear framebuffer address
//!     1024, 768,   // Resolution
//!     1024 * 4,    // Pitch (4 bytes per pixel = 32-bit)
//!     32,          // 32-bit color depth
//!     0xFF0000,    // Red mask
//!     0x00FF00,    // Green mask
//!     0x0000FF,    // Blue mask
//! ).expect("Valid framebuffer info");
//!
//! let mut fb = Framebuffer::new(info).expect("Valid framebuffer");
//! fb.clear(0x000000).ok();  // Clear to black
//! fb.put_pixel(100, 100, 0xFFFFFF).ok();  // Draw white pixel
//! ```

use core::fmt;

/// Error types for framebuffer operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramebufferError {
    /// Resolution or dimensions are invalid (zero or too large)
    InvalidResolution,
    /// Color depth is not supported (must be 8, 16, 24, or 32)
    InvalidColorDepth,
    /// Pixel coordinates are outside framebuffer bounds
    OutOfBounds,
    /// Framebuffer address is invalid or not properly mapped
    InvalidAddress,
    /// Invalid color masks for the given color depth
    InvalidColorMasks,
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FramebufferError::InvalidResolution => write!(f, "Invalid framebuffer resolution"),
            FramebufferError::InvalidColorDepth => write!(f, "Unsupported color depth"),
            FramebufferError::OutOfBounds => write!(f, "Pixel coordinates out of bounds"),
            FramebufferError::InvalidAddress => write!(f, "Invalid framebuffer address"),
            FramebufferError::InvalidColorMasks => write!(f, "Invalid color masks"),
        }
    }
}

/// Framebuffer information and metadata.
///
/// This struct contains the raw framebuffer address and layout information
/// needed to write pixels efficiently, along with comprehensive validation.
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Physical address of the framebuffer in video memory
    pub address: u32,
    /// Width of the display in pixels
    pub width: u32,
    /// Height of the display in pixels
    pub height: u32,
    /// Bytes per scanline (may include padding)
    pub pitch: u32,
    /// Color depth in bits per pixel (8, 16, 24, or 32)
    pub bits_per_pixel: u8,
    /// Bit mask for red channel
    pub red_mask: u32,
    /// Bit mask for green channel
    pub green_mask: u32,
    /// Bit mask for blue channel
    pub blue_mask: u32,
}

impl FramebufferInfo {
    /// Create a new FramebufferInfo with validation.
    ///
    /// # Arguments
    ///
    /// * `address` - Physical framebuffer address (must be non-zero and properly mapped)
    /// * `width` - Display width in pixels (must be > 0)
    /// * `height` - Display height in pixels (must be > 0)
    /// * `pitch` - Bytes per scanline (must be >= width × bytes_per_pixel)
    /// * `bits_per_pixel` - Color depth: 8, 16, 24, or 32
    /// * `red_mask` - Bit mask for red channel
    /// * `green_mask` - Bit mask for green channel
    /// * `blue_mask` - Bit mask for blue channel
    ///
    /// # Returns
    ///
    /// `Ok(FramebufferInfo)` if all parameters are valid, or an error otherwise.
    pub fn new(
        address: u32,
        width: u32,
        height: u32,
        pitch: u32,
        bits_per_pixel: u8,
        red_mask: u32,
        green_mask: u32,
        blue_mask: u32,
    ) -> Result<Self, FramebufferError> {
        let info = FramebufferInfo {
            address,
            width,
            height,
            pitch,
            bits_per_pixel,
            red_mask,
            green_mask,
            blue_mask,
        };
        info.validate()?;
        Ok(info)
    }

    /// Validate framebuffer parameters.
    ///
    /// Checks:
    /// - Resolution is non-zero and reasonable
    /// - Color depth is supported (8, 16, 24, 32)
    /// - Pitch is sufficient for the given width and color depth
    /// - Address is non-zero (basic sanity check)
    pub fn validate(&self) -> Result<(), FramebufferError> {
        // Check resolution
        if self.width == 0 || self.height == 0 {
            return Err(FramebufferError::InvalidResolution);
        }
        if self.width > 0x4000 || self.height > 0x4000 {
            // Sanity check: framebuffer should not exceed 16K x 16K
            return Err(FramebufferError::InvalidResolution);
        }

        // Check color depth
        match self.bits_per_pixel {
            8 | 16 | 24 | 32 => {}
            _ => return Err(FramebufferError::InvalidColorDepth),
        }

        // Check pitch is sufficient
        let bytes_per_pixel = self.bits_per_pixel as u32 / 8;
        let min_pitch = self.width.saturating_mul(bytes_per_pixel);
        if self.pitch < min_pitch {
            return Err(FramebufferError::InvalidResolution);
        }

        // Check address (basic sanity: should be non-zero and reasonable kernel space)
        if self.address == 0 {
            return Err(FramebufferError::InvalidAddress);
        }

        // Check color masks are reasonable for the bit depth
        match self.bits_per_pixel {
            8 => {
                // For 8-bit indexed, masks may be 0 or represent VGA palette
                if self.red_mask != 0 && self.red_mask != 0xFF {
                    return Err(FramebufferError::InvalidColorMasks);
                }
            }
            16 => {
                // RGB565: standard masks
                if self.red_mask != 0xF800 && self.red_mask != 0 {
                    return Err(FramebufferError::InvalidColorMasks);
                }
                if self.green_mask != 0x07E0 && self.green_mask != 0 {
                    return Err(FramebufferError::InvalidColorMasks);
                }
                if self.blue_mask != 0x001F && self.blue_mask != 0 {
                    return Err(FramebufferError::InvalidColorMasks);
                }
            }
            24 | 32 => {
                // RGB888 or ARGB8888: more flexible, just check they're not all zero
                if self.red_mask == 0 && self.green_mask == 0 && self.blue_mask == 0 {
                    return Err(FramebufferError::InvalidColorMasks);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get bytes per pixel.
    pub const fn bytes_per_pixel(&self) -> u32 {
        self.bits_per_pixel as u32 / 8
    }

    /// Get total framebuffer size in bytes.
    pub fn size(&self) -> Result<usize, FramebufferError> {
        self.pitch
            .checked_mul(self.height)
            .ok_or(FramebufferError::InvalidResolution)
            .map(|s| s as usize)
    }

    /// Calculate the byte offset for a given pixel coordinate.
    fn pixel_offset(&self, x: u32, y: u32) -> Result<usize, FramebufferError> {
        if x >= self.width || y >= self.height {
            return Err(FramebufferError::OutOfBounds);
        }

        let scanline_offset = (y as usize).saturating_mul(self.pitch as usize);
        let pixel_byte_offset = (x as usize).saturating_mul(self.bytes_per_pixel() as usize);

        // Check for arithmetic overflow
        let offset = scanline_offset
            .checked_add(pixel_byte_offset)
            .ok_or(FramebufferError::OutOfBounds)?;

        let total_size = self.size()?;
        if offset >= total_size {
            return Err(FramebufferError::OutOfBounds);
        }

        Ok(offset)
    }

    /// Get scanline offset for a given Y coordinate.
    fn scanline_offset(&self, y: u32) -> Result<usize, FramebufferError> {
        if y >= self.height {
            return Err(FramebufferError::OutOfBounds);
        }
        Ok((y as usize).saturating_mul(self.pitch as usize))
    }
}

/// Framebuffer wrapper providing safe access to video memory.
///
/// This struct wraps FramebufferInfo and provides safe, bounds-checked
/// operations for pixel and region manipulation. All operations are protected
/// against out-of-bounds access.
///
/// # Safety
///
/// The framebuffer address must point to valid, mapped video memory. The kernel
/// is responsible for:
/// - Ensuring the address points to readable/writable memory
/// - Managing the lifetime of the framebuffer
/// - Synchronizing access if multiple threads access the framebuffer
#[derive(Debug)]
pub struct Framebuffer {
    info: FramebufferInfo,
}

impl Framebuffer {
    /// Create a new framebuffer from validated information.
    ///
    /// # Arguments
    ///
    /// * `info` - FramebufferInfo (must have passed validation)
    ///
    /// # Returns
    ///
    /// `Ok(Framebuffer)` if info is valid, or an error otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::graphics::framebuffer::{FramebufferInfo, Framebuffer};
    /// let info = FramebufferInfo::new(0xFD000000, 1024, 768, 4096, 32, 0xFF0000, 0x00FF00, 0x0000FF)?;
    /// let fb = Framebuffer::new(info)?;
    /// # Ok::<(), _>(())
    /// ```
    pub fn new(info: FramebufferInfo) -> Result<Self, FramebufferError> {
        Ok(Framebuffer { info })
    }

    /// Write a single pixel with bounds checking.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal coordinate (0 = left)
    /// * `y` - Vertical coordinate (0 = top)
    /// * `color` - Color value (format depends on bits_per_pixel)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `OutOfBounds` if coordinates are invalid.
    ///
    /// # Color Format
    ///
    /// Color interpretation depends on bits_per_pixel:
    /// - **8-bit**: Palette index (0-255)
    /// - **16-bit**: RGB565 format (RRRRRGGGGGGBBBBB)
    /// - **24-bit**: RGB888 format (0x00RRGGBB)
    /// - **32-bit**: ARGB8888 format (0xAARRGGBB)
    pub fn put_pixel(&self, x: u32, y: u32, color: u32) -> Result<(), FramebufferError> {
        let offset = self.info.pixel_offset(x, y)?;

        unsafe {
            let base = self.info.address as *mut u8;
            match self.info.bits_per_pixel {
                8 => {
                    let ptr = base.add(offset) as *mut u8;
                    *ptr = (color & 0xFF) as u8;
                }
                16 => {
                    let ptr = base.add(offset) as *mut u16;
                    *ptr = (color & 0xFFFF) as u16;
                }
                24 => {
                    let ptr = base.add(offset);
                    *ptr = (color & 0xFF) as u8;
                    *ptr.add(1) = ((color >> 8) & 0xFF) as u8;
                    *ptr.add(2) = ((color >> 16) & 0xFF) as u8;
                }
                32 => {
                    let ptr = base.add(offset) as *mut u32;
                    *ptr = color;
                }
                _ => return Err(FramebufferError::InvalidColorDepth),
            }
        }

        Ok(())
    }

    /// Clear the entire framebuffer to a single color.
    ///
    /// # Arguments
    ///
    /// * `color` - Fill color (format depends on bits_per_pixel)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if operation cannot be completed.
    pub fn clear(&self, color: u32) -> Result<(), FramebufferError> {
        self.write_rect(0, 0, self.info.width, self.info.height, color)
    }

    /// Fill a rectangular region with a solid color.
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - Rectangle width in pixels
    /// * `height` - Rectangle height in pixels
    /// * `color` - Fill color
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the rectangle is out of bounds.
    ///
    /// # Behavior
    ///
    /// Rectangle coordinates are clipped to framebuffer bounds. If the rectangle
    /// is completely outside the framebuffer, an error is returned.
    pub fn write_rect(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: u32,
    ) -> Result<(), FramebufferError> {
        if x >= self.info.width || y >= self.info.height {
            return Err(FramebufferError::OutOfBounds);
        }

        // Clamp rectangle to framebuffer bounds
        let x_end = core::cmp::min(x + width, self.info.width);
        let y_end = core::cmp::min(y + height, self.info.height);

        for row in y..y_end {
            for col in x..x_end {
                self.put_pixel(col, row, color)?;
            }
        }

        Ok(())
    }

    /// Get the byte offset for the start of a scanline.
    ///
    /// # Arguments
    ///
    /// * `y` - Scanline Y coordinate (0 = top)
    ///
    /// # Returns
    ///
    /// Byte offset from framebuffer base address, or `OutOfBounds` if Y is invalid.
    pub fn get_scanline_offset(&self, y: u32) -> Result<usize, FramebufferError> {
        self.info.scanline_offset(y)
    }

    /// Get a raw mutable pointer to the framebuffer memory.
    ///
    /// # Returns
    ///
    /// Raw mutable pointer to the framebuffer base address.
    ///
    /// # Safety
    ///
    /// This pointer allows direct, unchecked access to video memory.
    /// The caller is responsible for:
    /// - Ensuring writes stay within framebuffer bounds (0..size())
    /// - Respecting the color format and stride
    /// - Managing memory access synchronization
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::graphics::framebuffer::{FramebufferInfo, Framebuffer};
    /// # let info = FramebufferInfo::new(0xFD000000, 1024, 768, 4096, 32, 0xFF0000, 0x00FF00, 0x0000FF)?;
    /// # let fb = Framebuffer::new(info)?;
    /// unsafe {
    ///     let ptr = fb.as_raw_ptr();
    ///     // Direct memory access
    /// }
    /// # Ok::<(), _>(())
    /// ```
    pub fn as_raw_ptr(&self) -> *mut u8 {
        self.info.address as *mut u8
    }

    /// Get framebuffer information.
    pub fn info(&self) -> &FramebufferInfo {
        &self.info
    }

    /// Get framebuffer width in pixels.
    pub fn width(&self) -> u32 {
        self.info.width
    }

    /// Get framebuffer height in pixels.
    pub fn height(&self) -> u32 {
        self.info.height
    }

    /// Get framebuffer pitch (bytes per scanline).
    pub fn pitch(&self) -> u32 {
        self.info.pitch
    }

    /// Get bytes per pixel.
    pub fn bytes_per_pixel(&self) -> u32 {
        self.info.bytes_per_pixel()
    }

    /// Get bits per pixel.
    pub fn bits_per_pixel(&self) -> u8 {
        self.info.bits_per_pixel
    }

    /// Get total framebuffer size in bytes.
    pub fn size(&self) -> Result<usize, FramebufferError> {
        self.info.size()
    }

    /// Convert color from ARGB8888 to the framebuffer's native format.
    ///
    /// # Arguments
    ///
    /// * `argb8888` - Color in ARGB8888 format (0xAARRGGBB)
    ///
    /// # Returns
    ///
    /// Color in the framebuffer's native format.
    pub fn convert_color(&self, argb8888: u32) -> u32 {
        match self.info.bits_per_pixel {
            8 => {
                // Convert to palette index (simplified: grayscale approximation)
                let r = (argb8888 >> 16) & 0xFF;
                let g = (argb8888 >> 8) & 0xFF;
                let b = argb8888 & 0xFF;
                let gray = ((r + g + b) / 3) as u8;
                // Map to VGA palette: 0-3 = dark, 4-11 = mid, 12-15 = bright
                (gray / 16) as u32
            }
            16 => {
                // Convert ARGB8888 to RGB565
                let r = ((argb8888 >> 16) & 0xFF) >> 3;
                let g = ((argb8888 >> 8) & 0xFF) >> 2;
                let b = (argb8888 & 0xFF) >> 3;
                ((r << 11) | (g << 5) | b) & 0xFFFF
            }
            24 => {
                // Convert to RGB888 (drop alpha)
                argb8888 & 0x00FFFFFF
            }
            32 => {
                // Already ARGB8888
                argb8888
            }
            _ => 0,
        }
    }

    /// Get red color component mask.
    pub fn red_mask(&self) -> u32 {
        self.info.red_mask
    }

    /// Get green color component mask.
    pub fn green_mask(&self) -> u32 {
        self.info.green_mask
    }

    /// Get blue color component mask.
    pub fn blue_mask(&self) -> u32 {
        self.info.blue_mask
    }
}
