//! DisplayManager - Core display management and rendering pipeline
//!
//! The DisplayManager service coordinates display operations, queues rendering commands,
//! and manages dirty region tracking for efficient screen updates. It integrates with
//! the graphics module's Display trait to provide a high-level rendering abstraction.
//!
//! # Architecture
//!
//! The manager implements a command-queue pattern for batching rendering operations:
//! - `RenderCommand` enums define discrete rendering operations
//! - Commands are queued for batch processing
//! - Dirty region tracking optimizes updates to changed areas
//! - State management tracks running/stopped status
//!
//! # Usage
//!
//! ```no_run
//! # use kernel::graphics::Display;
//! # use kernel::fusion::DisplayManager;
//! # use kernel::fusion::manager::{RenderCommand, ManagerError};
//! # use kernel::graphics::color::Color;
//! // Create a DisplayManager with a graphics display
//! // let mut display = VgaDisplay::new();
//! // let mut manager = DisplayManager::new(Box::new(display));
//! // manager.start().ok();
//! // manager.queue_render(RenderCommand::ClearScreen(Color::BLACK)).ok();
//! // manager.process_queue().ok();
//! // manager.flush().ok();
//! // manager.stop().ok();
//! ```

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::fmt;

use crate::graphics::Display;
use crate::graphics::color::Color;

/// Rectangular region for dirty tracking and operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// X coordinate of top-left corner
    pub x: u32,
    /// Y coordinate of top-left corner
    pub y: u32,
    /// Width in pixels
    pub w: u32,
    /// Height in pixels
    pub h: u32,
}

impl Rect {
    /// Create a new rectangle
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Rect { x, y, w, h }
    }

    /// Create a rectangle covering the entire screen
    pub const fn full_screen() -> Self {
        Rect {
            x: 0,
            y: 0,
            w: 0xFFFF_FFFF,
            h: 0xFFFF_FFFF,
        }
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    /// Merge this rectangle with another, expanding to cover both
    pub fn merge(&mut self, other: &Rect) {
        let new_x = core::cmp::min(self.x, other.x);
        let new_y = core::cmp::min(self.y, other.y);
        let new_x2 = core::cmp::max(self.x + self.w, other.x + other.w);
        let new_y2 = core::cmp::max(self.y + self.h, other.y + other.h);

        self.x = new_x;
        self.y = new_y;
        self.w = new_x2 - new_x;
        self.h = new_y2 - new_y;
    }
}

/// Rendering commands queued for batch processing
#[derive(Debug, Clone, Copy)]
pub enum RenderCommand {
    /// Clear the entire screen to a color
    ClearScreen(Color),
    /// Draw a single pixel
    DrawPixel { x: u32, y: u32, color: Color },
    /// Draw a filled rectangle
    DrawRect {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        color: Color,
    },
    /// Draw text (static lifetime for text pointer)
    DrawText {
        x: u32,
        y: u32,
        text: &'static str,
        color: Color,
    },
    /// Mark a region as dirty (needing update)
    InvalidateRegion(Rect),
}

impl fmt::Display for RenderCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderCommand::ClearScreen(_) => write!(f, "ClearScreen"),
            RenderCommand::DrawPixel { .. } => write!(f, "DrawPixel"),
            RenderCommand::DrawRect { .. } => write!(f, "DrawRect"),
            RenderCommand::DrawText { .. } => write!(f, "DrawText"),
            RenderCommand::InvalidateRegion(_) => write!(f, "InvalidateRegion"),
        }
    }
}

/// Error types for display manager operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerError {
    /// The render queue is full and cannot accept more commands
    QueueFull,
    /// An error occurred in the underlying Display implementation
    DisplayError,
    /// The manager is not in running state
    NotRunning,
    /// An invalid command was issued or processed
    InvalidCommand,
}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagerError::QueueFull => write!(f, "Render queue is full"),
            ManagerError::DisplayError => write!(f, "Display error occurred"),
            ManagerError::NotRunning => write!(f, "Manager is not running"),
            ManagerError::InvalidCommand => write!(f, "Invalid command"),
        }
    }
}

/// Maximum number of render commands that can be queued
const MAX_QUEUE_SIZE: usize = 256;

/// Maximum number of dirty regions to track
const MAX_DIRTY_REGIONS: usize = 64;

/// DisplayManager - Core display management service
///
/// Manages rendering operations through a command queue and dirty region tracking.
/// Provides high-level rendering abstraction over the Display trait.
///
/// Uses trait objects to allow any Display implementation to be used. The associated
/// Error type is pinned to ManagerError for consistency, and the Buffer type is
/// abstracted away via trait objects.
pub struct DisplayManager {
    /// Underlying graphics display (trait object allows any Display implementation)
    /// Note: Error type is constrained to ManagerError, Buffer type is abstract
    display: Box<dyn DisplayLike>,
    /// Queue of rendering commands to process
    render_queue: VecDeque<RenderCommand>,
    /// Regions marked as needing update
    dirty_regions: Vec<Rect>,
    /// Manager operational state
    is_running: bool,
}

/// Internal trait wrapper for Display to avoid trait object complications
trait DisplayLike {
    fn pixel_put(&mut self, x: u32, y: u32, color: u32);
    fn clear(&mut self, color: u32);
    fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) -> Result<(), ManagerError>;
    fn swap_buffer(&mut self);
    fn get_resolution(&self) -> (u32, u32);
    fn get_bits_per_pixel(&self) -> u8;
    fn is_dirty(&self) -> bool;
}

/// Wrapper to implement DisplayLike for any Display type
struct DisplayWrapper<D: Display> {
    display: D,
}

impl<D: Display> DisplayLike for DisplayWrapper<D> {
    fn pixel_put(&mut self, x: u32, y: u32, color: u32) {
        self.display.pixel_put(x, y, color);
    }

    fn clear(&mut self, color: u32) {
        self.display.clear(color);
    }

    fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) -> Result<(), ManagerError> {
        self.display
            .fill_rect(x, y, width, height, color)
            .map_err(|_| ManagerError::DisplayError)
    }

    fn swap_buffer(&mut self) {
        self.display.swap_buffer();
    }

    fn get_resolution(&self) -> (u32, u32) {
        self.display.get_resolution()
    }

    fn get_bits_per_pixel(&self) -> u8 {
        self.display.get_bits_per_pixel()
    }

    fn is_dirty(&self) -> bool {
        self.display.is_dirty()
    }
}

impl DisplayManager {
    /// Create a new DisplayManager with a Display implementation
    ///
    /// # Arguments
    ///
    /// * `display` - A Display implementation to manage
    ///
    /// # Returns
    ///
    /// A new DisplayManager instance in stopped state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kernel::fusion::DisplayManager;
    /// // let vga_display = VgaDisplay::new();
    /// // let mut manager = DisplayManager::new(vga_display);
    /// ```
    pub fn new<D: Display + 'static>(display: D) -> Self {
        DisplayManager {
            display: Box::new(DisplayWrapper { display }),
            render_queue: VecDeque::with_capacity(MAX_QUEUE_SIZE),
            dirty_regions: Vec::with_capacity(MAX_DIRTY_REGIONS),
            is_running: false,
        }
    }

    /// Queue a rendering command for batch processing
    ///
    /// # Arguments
    ///
    /// * `cmd` - The RenderCommand to queue
    ///
    /// # Returns
    ///
    /// `Ok(())` if the command was queued, or `QueueFull` if the queue is at capacity.
    pub fn queue_render(&mut self, cmd: RenderCommand) -> Result<(), ManagerError> {
        if self.render_queue.len() >= MAX_QUEUE_SIZE {
            return Err(ManagerError::QueueFull);
        }

        self.render_queue.push_back(cmd);
        Ok(())
    }

    /// Process all queued rendering commands
    ///
    /// Executes each command in the queue in order. Invalid or erroneous commands
    /// may be skipped. Call `flush()` after processing to make changes visible.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all commands were processed, or an error if processing failed.
    pub fn process_queue(&mut self) -> Result<(), ManagerError> {
        if !self.is_running {
            return Err(ManagerError::NotRunning);
        }

        while let Some(cmd) = self.render_queue.pop_front() {
            self.execute_command(cmd)?;
        }

        Ok(())
    }

    /// Execute a single render command
    fn execute_command(&mut self, cmd: RenderCommand) -> Result<(), ManagerError> {
        match cmd {
            RenderCommand::ClearScreen(color) => {
                self.display.clear(color.to_argb8888());
                self.mark_dirty(Rect::full_screen());
                Ok(())
            }
            RenderCommand::DrawPixel { x, y, color } => {
                self.display.pixel_put(x, y, color.to_argb8888());
                self.mark_dirty(Rect::new(x, y, 1, 1));
                Ok(())
            }
            RenderCommand::DrawRect { x, y, w, h, color } => {
                self.display
                    .fill_rect(x, y, w, h, color.to_argb8888())?;
                self.mark_dirty(Rect::new(x, y, w, h));
                Ok(())
            }
            RenderCommand::DrawText { x, y, text, .. } => {
                // Text rendering would require a text renderer implementation
                // For now, acknowledge the command without error
                // Estimate text bounds: each character is roughly 8x16 pixels
                self.mark_dirty(Rect::new(x, y, text.len() as u32 * 8, 16));
                Ok(())
            }
            RenderCommand::InvalidateRegion(rect) => {
                self.mark_dirty(rect);
                Ok(())
            }
        }
    }

    /// Mark a rectangular region as needing update
    ///
    /// # Arguments
    ///
    /// * `rect` - The region to mark dirty
    ///
    /// # Notes
    ///
    /// Dirty regions are merged when possible to optimize batch updates.
    /// If the number of tracked regions exceeds MAX_DIRTY_REGIONS, they may be
    /// consolidated into a single full-screen region.
    pub fn mark_dirty(&mut self, rect: Rect) {
        if self.dirty_regions.len() >= MAX_DIRTY_REGIONS {
            // Too many regions; consolidate to full screen
            self.dirty_regions.clear();
            self.dirty_regions.push(Rect::full_screen());
            return;
        }

        // Try to merge with existing regions if they intersect
        for region in &mut self.dirty_regions {
            if region.intersects(&rect) {
                region.merge(&rect);
                return;
            }
        }

        // No intersection found, add as new region
        self.dirty_regions.push(rect);
    }

    /// Flush dirty regions to the display
    ///
    /// Updates only the regions marked as dirty since the last flush.
    /// Clears the dirty region list after flushing.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the flush was successful, or `NotRunning` if the manager
    /// is not in running state.
    pub fn flush(&mut self) -> Result<(), ManagerError> {
        if !self.is_running {
            return Err(ManagerError::NotRunning);
        }

        // Swap buffer to make all pending changes visible
        self.display.swap_buffer();

        // Clear dirty regions after flushing
        self.dirty_regions.clear();

        Ok(())
    }

    /// Get the number of currently queued render commands
    pub fn queue_size(&self) -> usize {
        self.render_queue.len()
    }

    /// Get the number of tracked dirty regions
    pub fn dirty_region_count(&self) -> usize {
        self.dirty_regions.len()
    }

    /// Check if the manager is currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Start the display manager
    ///
    /// Transitions the manager to running state, allowing command queuing
    /// and processing.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successfully started, or an error if startup failed.
    pub fn start(&mut self) -> Result<(), ManagerError> {
        if self.is_running {
            return Ok(());
        }

        self.is_running = true;
        self.render_queue.clear();
        self.dirty_regions.clear();

        Ok(())
    }

    /// Stop the display manager
    ///
    /// Transitions the manager to stopped state. Queued commands are not processed
    /// while stopped. The display state is preserved.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successfully stopped, or `NotRunning` if already stopped.
    pub fn stop(&mut self) -> Result<(), ManagerError> {
        if !self.is_running {
            return Err(ManagerError::NotRunning);
        }

        self.is_running = false;
        self.render_queue.clear();

        Ok(())
    }

    /// Get display resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.display.get_resolution()
    }

    /// Get display color depth in bits per pixel
    pub fn color_depth(&self) -> u8 {
        self.display.get_bits_per_pixel()
    }

    /// Check if the display buffer needs refreshing
    pub fn is_display_dirty(&self) -> bool {
        self.display.is_dirty() || !self.dirty_regions.is_empty()
    }
}
