//! Fusion Display Backend - framebuffer compositing for LXQt/Fusion
//!
//! Manages surfaces, z-order compositing, and VESA framebuffer output.

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::graphics::vesa::VesaDisplay;
use crate::graphics::Display;

const COMPOSITOR_CLEAR_COLOR: u32 = 0x0011141C;

/// Error type for Fusion display operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FusionError {
    SurfaceNotFound,
    InvalidDimensions,
    AllocationFailed,
    InvalidPixelData,
}

impl core::fmt::Display for FusionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FusionError::SurfaceNotFound => write!(f, "Surface not found"),
            FusionError::InvalidDimensions => write!(f, "Invalid surface dimensions"),
            FusionError::AllocationFailed => write!(f, "Memory allocation failed"),
            FusionError::InvalidPixelData => write!(f, "Invalid pixel data"),
        }
    }
}

/// Surface metadata and pixel buffer
#[derive(Debug, Clone)]
struct SurfaceData {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    visible: bool,
    z_order: u32,
    pixels: Vec<u32>,
}

impl SurfaceData {
    fn new(width: u32, height: u32) -> Result<Self, FusionError> {
        if width == 0 || height == 0 {
            return Err(FusionError::InvalidDimensions);
        }

        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(FusionError::InvalidDimensions)?;

        Ok(SurfaceData {
            width,
            height,
            x: 0,
            y: 0,
            visible: true,
            z_order: 0,
            pixels: alloc::vec![0u32; pixel_count],
        })
    }

    #[allow(dead_code)]
    fn pixel_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
        } else {
            None
        }
    }
}

/// Fusion Display Backend
pub struct FusionDisplayBackend {
    surfaces: BTreeMap<u32, SurfaceData>,
    next_surface_id: u32,
    display: VesaDisplay,
    framebuffer_width: u32,
    framebuffer_height: u32,
    dirty: bool,
}

impl core::fmt::Debug for FusionDisplayBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FusionDisplayBackend")
            .field("surfaces", &self.surfaces.len())
            .field("framebuffer_width", &self.framebuffer_width)
            .field("framebuffer_height", &self.framebuffer_height)
            .finish()
    }
}

impl FusionDisplayBackend {
    /// Create a new Fusion display backend
    pub fn new(mut display: VesaDisplay) -> Self {
        let (width, height) = display.get_resolution();
        display.clear(COMPOSITOR_CLEAR_COLOR);
        display.swap_buffer();

        FusionDisplayBackend {
            surfaces: BTreeMap::new(),
            next_surface_id: 1,
            display,
            framebuffer_width: width,
            framebuffer_height: height,
            dirty: false,
        }
    }

    /// Get framebuffer width
    pub fn framebuffer_width(&self) -> u32 {
        self.framebuffer_width
    }

    /// Get framebuffer height
    pub fn framebuffer_height(&self) -> u32 {
        self.framebuffer_height
    }

    /// Clear the entire framebuffer
    pub fn clear_framebuffer(&mut self) {
        self.display.clear(COMPOSITOR_CLEAR_COLOR);
        self.dirty = true;
    }

    /// Create a new framebuffer surface
    pub fn create_surface(&mut self, width: u32, height: u32) -> Result<u32, FusionError> {
        let id = self.next_surface_id;
        self.next_surface_id = self.next_surface_id.wrapping_add(1);

        let surface = SurfaceData::new(width, height)?;
        self.surfaces.insert(id, surface);

        Ok(id)
    }

    /// Get a mutable reference to a surface for rendering
    #[allow(private_interfaces)]
    pub fn get_surface_mut(&mut self, id: u32) -> Option<&mut SurfaceData> {
        self.surfaces.get_mut(&id)
    }

    /// Get a reference to a surface
    #[allow(private_interfaces)]
    pub fn get_surface(&self, id: u32) -> Option<&SurfaceData> {
        self.surfaces.get(&id)
    }

    /// Destroy a surface
    pub fn destroy_surface(&mut self, id: u32) -> Result<(), FusionError> {
        self.surfaces.remove(&id).ok_or(FusionError::SurfaceNotFound)?;
        Ok(())
    }

    /// Set surface position
    pub fn set_position(&mut self, id: u32, x: i32, y: i32) -> Result<(), FusionError> {
        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;
        surface.x = x;
        surface.y = y;
        self.dirty = true;
        Ok(())
    }

    /// Resize a surface
    pub fn resize(&mut self, id: u32, width: u32, height: u32) -> Result<(), FusionError> {
        if width == 0 || height == 0 {
            return Err(FusionError::InvalidDimensions);
        }

        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;
        let new_pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(FusionError::InvalidDimensions)?;

        surface.width = width;
        surface.height = height;
        surface.pixels.clear();
        surface.pixels.resize(new_pixel_count, 0u32);
        self.dirty = true;

        Ok(())
    }

    /// Set surface visibility
    pub fn set_visibility(&mut self, id: u32, visible: bool) -> Result<(), FusionError> {
        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;
        surface.visible = visible;
        self.dirty = true;
        Ok(())
    }

    /// Set surface z-order (draw order)
    pub fn set_z_order(&mut self, id: u32, z_order: u32) -> Result<(), FusionError> {
        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;
        surface.z_order = z_order;
        self.dirty = true;
        Ok(())
    }

    /// Update surface pixel data
    pub fn upload_pixels(
        &mut self,
        id: u32,
        width: u32,
        height: u32,
        pixels: &[u32],
    ) -> Result<(), FusionError> {
        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;

        if surface.width != width || surface.height != height {
            return Err(FusionError::InvalidDimensions);
        }

        let expected_len = (width as usize) * (height as usize);
        if pixels.len() != expected_len {
            return Err(FusionError::InvalidPixelData);
        }

        surface.pixels.copy_from_slice(pixels);
        self.dirty = true;
        Ok(())
    }

    /// Get all visible surface IDs sorted by z-order for composition
    #[allow(private_interfaces)]
    pub fn surfaces_by_z_order(&self) -> Vec<(u32, &SurfaceData)> {
        let mut surfaces: Vec<_> = self
            .surfaces
            .iter()
            .filter(|(_, s)| s.visible)
            .map(|(id, s)| (*id, s))
            .collect();

        surfaces.sort_by_key(|(_, s)| s.z_order);
        surfaces
    }

    /// Composite a single SHM buffer onto the framebuffer at given position
    /// Used by the Wayland compositor integration layer
    #[allow(clippy::too_many_arguments)]
    pub fn composite_shm_buffer(
        &mut self,
        buffer: &[u32],
        buffer_width: u32,
        buffer_height: u32,
        dst_x: i32,
        dst_y: i32,
        src_x: u32,
        src_y: u32,
        src_w: u32,
        src_h: u32,
    ) {
        for row in 0..src_h {
            let fb_y = dst_y + row as i32;
            if fb_y < 0 || fb_y >= self.framebuffer_height as i32 {
                continue;
            }

            let src_row = src_y + row;
            if src_row >= buffer_height {
                break;
            }

            for col in 0..src_w {
                let fb_x = dst_x + col as i32;
                if fb_x < 0 || fb_x >= self.framebuffer_width as i32 {
                    continue;
                }

                let src_col = src_x + col;
                if src_col >= buffer_width {
                    break;
                }

                let src_idx = (src_row * buffer_width + src_col) as usize;
                if src_idx >= buffer.len() {
                    continue;
                }

                let pixel = buffer[src_idx];
                if pixel == 0 {
                    continue; // Skip fully transparent pixels
                }

                let fb_idx = (fb_y as u32 * self.framebuffer_width + fb_x as u32) as usize;
                let fb_size = (self.framebuffer_width * self.framebuffer_height) as usize;

                if fb_idx < fb_size {
                    self.display.pixel_put(fb_x as u32, fb_y as u32, pixel);
                }
            }
        }
    }

    /// Get mutable access to the underlying display
    pub fn display_mut(&mut self) -> &mut VesaDisplay {
        &mut self.display
    }

    /// Composite all visible surfaces onto the framebuffer and swap
    pub fn flush(&mut self) -> Result<(), FusionError> {
        if !self.dirty {
            return Ok(());
        }

        let sorted: Vec<(u32, SurfaceData)> = {
            let mut tmp: Vec<_> = self
                .surfaces
                .iter()
                .filter(|(_, s)| s.visible && s.x >= 0 && s.y >= 0)
                .map(|(id, s)| (*id, s.clone()))
                .collect();
            tmp.sort_by_key(|(_, s)| s.z_order);
            tmp
        };

        self.display.clear(COMPOSITOR_CLEAR_COLOR);

        for (_id, surface) in &sorted {
            for row in 0..surface.height {
                let fb_y = surface.y + row as i32;
                if fb_y >= self.framebuffer_height as i32 {
                    break;
                }
                for col in 0..surface.width {
                    let fb_x = surface.x + col as i32;
                    if fb_x >= self.framebuffer_width as i32 {
                        continue;
                    }
                    let pixel_idx = (row * surface.width + col) as usize;
                    if pixel_idx < surface.pixels.len() {
                        let pixel = surface.pixels[pixel_idx];
                        if pixel != 0 {
                            self.display.pixel_put(
                                fb_x as u32,
                                fb_y as u32,
                                pixel,
                            );
                        }
                    }
                }
            }
        }

        self.display.swap_buffer();
        self.dirty = false;
        Ok(())
    }
}