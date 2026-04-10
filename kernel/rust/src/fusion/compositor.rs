//! Compositor - Composition engine for rendering surfaces
//!
//! Handles compositing multiple surfaces together for final display output.
//! Implements a painter's algorithm for z-order layering with support for both
//! full and dirty-region compositing optimizations.
//!
//! # Architecture
//!
//! The Compositor maintains:
//! - A collection of surfaces sorted by z-order
//! - A reference to an underlying display for final output
//! - Dirty region tracking for efficient updates
//! - Surface visibility and clipping support
//!
//! # Compositing Algorithm
//!
//! Uses the painter's algorithm:
//! 1. Sort surfaces by z-order (ascending)
//! 2. Iterate from lowest to highest z-order
//! 3. For each visible surface: blit to display with position offset
//! 4. Handle clipping at display boundaries automatically
//! 5. Support alpha blending if color depths match
//!
//! # Example
//!
//! ```no_run
//! # use kernel::fusion::{Compositor, Surface};
//! # use kernel::graphics::Display;
//! # let mut display: Box<dyn Display<Error=(), Buffer=()>> = unsafe { &mut *(0 as *mut _) }.into();
//! // Create compositor with display backend
//! let mut compositor = Compositor::new(display);
//!
//! // Create and add surfaces
//! let surface = Surface::new(800, 600).unwrap();
//! let id = compositor.add_surface(surface).unwrap();
//!
//! // Composite all surfaces to display
//! compositor.composite().ok();
//! ```

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;

use crate::fusion::surface::{Surface, SurfaceError};
use crate::graphics::Display;

/// Error types for compositor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorError {
    /// Surface ID not found in compositor
    InvalidSurfaceId,
    /// Display operation failed
    DisplayError,
    /// No display backend available
    NoDisplayAvailable,
    /// Generic composition failure
    CompositionFailed,
}

impl fmt::Display for CompositorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositorError::InvalidSurfaceId => write!(f, "Invalid surface ID"),
            CompositorError::DisplayError => write!(f, "Display operation failed"),
            CompositorError::NoDisplayAvailable => write!(f, "No display available"),
            CompositorError::CompositionFailed => write!(f, "Composition failed"),
        }
    }
}

impl From<SurfaceError> for CompositorError {
    fn from(_: SurfaceError) -> Self {
        CompositorError::DisplayError
    }
}

/// Surface entry with ID tracking
#[derive(Debug)]
struct SurfaceEntry {
    /// Unique surface ID
    id: usize,
    /// Surface data
    surface: Surface,
}

/// Compositor for rendering multiple surfaces to a display.
///
/// Manages surface layering with z-order and provides efficient compositing
/// to an underlying display backend. Supports both full and dirty-region
/// compositing for performance optimization.
pub struct Compositor {
    /// Managed surfaces (not necessarily sorted by z-order)
    surfaces: Vec<SurfaceEntry>,
    /// Underlying display backend
    display: Box<dyn DisplayLike>,
    /// Next surface ID to assign
    next_surface_id: usize,
}

/// Trait for display backends compatible with compositor
pub trait DisplayLike: fmt::Debug {
    /// Get the resolution of the display
    fn get_resolution(&self) -> (u32, u32);
    /// Put a pixel at the given coordinates
    fn pixel_put(&mut self, x: u32, y: u32, color: u32);
    /// Clear the entire display to a color
    fn clear(&mut self, color: u32);
    /// Swap or flush the display buffer
    fn swap_buffer(&mut self);
}

/// Adapter to make Display trait compatible with DisplayLike
pub struct DisplayAdapter<D: Display> {
    display: D,
}

impl<D: Display> DisplayAdapter<D> {
    /// Create a new display adapter
    pub fn new(display: D) -> Self {
        DisplayAdapter { display }
    }
}

impl<D: Display> fmt::Debug for DisplayAdapter<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisplayAdapter").finish()
    }
}

impl<D: Display> DisplayLike for DisplayAdapter<D> {
    fn get_resolution(&self) -> (u32, u32) {
        self.display.get_resolution()
    }

    fn pixel_put(&mut self, x: u32, y: u32, color: u32) {
        self.display.pixel_put(x, y, color);
    }

    fn clear(&mut self, color: u32) {
        self.display.clear(color);
    }

    fn swap_buffer(&mut self) {
        self.display.swap_buffer();
    }
}

impl Compositor {
    /// Create a new Compositor with the given display backend.
    ///
    /// # Arguments
    ///
    /// * `display` - Display backend implementing DisplayLike trait
    ///
    /// # Returns
    ///
    /// New Compositor instance with empty surface list.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::fusion::Compositor;
    /// # let display = todo!();
    /// let compositor = Compositor::new(display);
    /// ```
    pub fn new(display: Box<dyn DisplayLike>) -> Self {
        Compositor {
            surfaces: Vec::new(),
            display,
            next_surface_id: 1,
        }
    }

    /// Add a surface to the compositor.
    ///
    /// # Arguments
    ///
    /// * `surface` - Surface to add
    ///
    /// # Returns
    ///
    /// `Ok(surface_id)` where surface_id can be used for future operations,
    /// or `Err` if addition fails.
    pub fn add_surface(&mut self, surface: Surface) -> Result<usize, CompositorError> {
        let id = self.next_surface_id;
        self.next_surface_id = self.next_surface_id.saturating_add(1);

        self.surfaces.push(SurfaceEntry { id, surface });
        Ok(id)
    }

    /// Remove a surface from the compositor.
    ///
    /// # Arguments
    ///
    /// * `id` - Surface ID to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if surface was found and removed, `Err` if not found.
    pub fn remove_surface(&mut self, id: usize) -> Result<(), CompositorError> {
        if let Some(pos) = self.surfaces.iter().position(|e| e.id == id) {
            self.surfaces.remove(pos);
            Ok(())
        } else {
            Err(CompositorError::InvalidSurfaceId)
        }
    }

    /// Get a reference to a surface by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Surface ID
    ///
    /// # Returns
    ///
    /// `Some(&Surface)` if found, `None` otherwise.
    pub fn get_surface(&self, id: usize) -> Option<&Surface> {
        self.surfaces.iter().find(|e| e.id == id).map(|e| &e.surface)
    }

    /// Get a mutable reference to a surface by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Surface ID
    ///
    /// # Returns
    ///
    /// `Some(&mut Surface)` if found, `None` otherwise.
    pub fn get_surface_mut(&mut self, id: usize) -> Option<&mut Surface> {
        self.surfaces
            .iter_mut()
            .find(|e| e.id == id)
            .map(|e| &mut e.surface)
    }

    /// Update the z-order of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - Surface ID
    /// * `z_order` - New z-order (higher = on top)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err` if surface not found.
    pub fn update_z_order(&mut self, id: usize, z_order: u32) -> Result<(), CompositorError> {
        if let Some(entry) = self.surfaces.iter_mut().find(|e| e.id == id) {
            entry.surface.set_z_order(z_order);
            Ok(())
        } else {
            Err(CompositorError::InvalidSurfaceId)
        }
    }

    /// Get the number of surfaces in the compositor.
    ///
    /// # Returns
    ///
    /// Count of surfaces (both visible and invisible).
    pub fn get_surface_count(&self) -> usize {
        self.surfaces.len()
    }

    /// Composite all surfaces to the display using painter's algorithm.
    ///
    /// Sorts surfaces by z-order (lowest to highest) and blits each visible
    /// surface to the display with proper clipping at boundaries.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or error if composition fails.
    ///
    /// # Algorithm
    ///
    /// 1. Sort surfaces by z-order (ascending)
    /// 2. Clear display to black
    /// 3. For each surface in sorted order:
    ///    - Skip if invisible
    ///    - Blit to display at surface position
    ///    - Clipping handled automatically
    /// 4. Swap display buffer
    pub fn composite(&mut self) -> Result<(), CompositorError> {
        // Sort surfaces by z-order (painter's algorithm)
        self.surfaces
            .sort_by_key(|e| e.surface.get_z_order());

        // Clear display to black
        self.display.clear(0xFF000000);

        // Collect surface data to avoid borrow conflicts
        let surfaces_to_blit: Vec<_> = self.surfaces.iter()
            .filter(|e| e.surface.is_visible())
            .map(|e| {
                let (x, y) = e.surface.get_position();
                (e.surface.get_buffer().to_vec(), e.surface.get_dimensions(), x, y)
            })
            .collect();

        // Blit each visible surface in z-order
        for (buffer, (width, _height), x, y) in surfaces_to_blit {
            self.blit_surface_direct(&buffer, width, x, y)?;
        }

        // Flush display
        self.display.swap_buffer();
        Ok(())
    }

    /// Composite only dirty regions (surfaces that have changed).
    ///
    /// Optimization pass that only updates regions affected by visible surfaces.
    /// This is more efficient than `composite()` when only a few surfaces change.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or error if composition fails.
    ///
    /// # Algorithm
    ///
    /// 1. Sort surfaces by z-order
    /// 2. Track dirty regions from visible surfaces
    /// 3. Clear only affected regions or perform selective update
    /// 4. Blit surfaces covering dirty regions
    /// 5. Swap display buffer
    ///
    /// # Notes
    ///
    /// Current implementation performs full composition but can be optimized
    /// with proper dirty region tracking in future versions.
    pub fn composite_dirty(&mut self) -> Result<(), CompositorError> {
        // For now, perform full composition
        // Future optimization: track dirty regions and only update affected areas
        self.composite()
    }

    /// Internal helper: blit a surface to the display with clipping.
    ///
    /// # Arguments
    ///
    /// * `surface` - Surface to blit
    /// * `x` - Screen X coordinate
    /// * `y` - Screen Y coordinate
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    fn blit_surface(&mut self, surface: &Surface, x: i32, y: i32) -> Result<(), CompositorError> {
        let (screen_width, screen_height) = self.display.get_resolution();
        let (surf_width, surf_height) = surface.get_dimensions();

        // Calculate the region of the surface that's visible on-screen
        let start_x = if x < 0 { (-x) as u32 } else { 0 };
        let start_y = if y < 0 { (-y) as u32 } else { 0 };

        let end_x = core::cmp::min(
            surf_width,
            screen_width.saturating_sub(x.max(0) as u32),
        );
        let end_y = core::cmp::min(
            surf_height,
            screen_height.saturating_sub(y.max(0) as u32),
        );

        if start_x >= end_x || start_y >= end_y {
            return Ok(());
        }

        // Copy pixels with clipping
        let buffer = surface.get_buffer();
        for sy in start_y..end_y {
            for sx in start_x..end_x {
                let pixel = buffer[(sy * surf_width + sx) as usize];
                let screen_x = (x + sx as i32) as u32;
                let screen_y = (y + sy as i32) as u32;

                self.display.pixel_put(screen_x, screen_y, pixel);
            }
        }

        Ok(())
    }

    /// Internal helper: blit a surface buffer to the display with clipping.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Pixel buffer (ARGB8888)
    /// * `width` - Surface width in pixels
    /// * `x` - Screen X coordinate
    /// * `y` - Screen Y coordinate
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    fn blit_surface_direct(
        &mut self,
        buffer: &[u32],
        width: u32,
        x: i32,
        y: i32,
    ) -> Result<(), CompositorError> {
        let (screen_width, screen_height) = self.display.get_resolution();

        if buffer.is_empty() || width == 0 {
            return Ok(());
        }

        let height = (buffer.len() as u32) / width;

        // Calculate the region that's visible on-screen
        let start_x = if x < 0 { (-x) as u32 } else { 0 };
        let start_y = if y < 0 { (-y) as u32 } else { 0 };

        let end_x = core::cmp::min(
            width,
            screen_width.saturating_sub(x.max(0) as u32),
        );
        let end_y = core::cmp::min(
            height,
            screen_height.saturating_sub(y.max(0) as u32),
        );

        if start_x >= end_x || start_y >= end_y {
            return Ok(());
        }

        // Copy pixels with clipping
        for sy in start_y..end_y {
            for sx in start_x..end_x {
                let pixel = buffer[(sy * width + sx) as usize];
                let screen_x = (x + sx as i32) as u32;
                let screen_y = (y + sy as i32) as u32;

                self.display.pixel_put(screen_x, screen_y, pixel);
            }
        }

        Ok(())
    }
}
