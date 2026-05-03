//! Fusion Display Backend - Implements DisplayBackend for framebuffer rendering
//!
//! Bridges kernel framebuffer with the display server protocol, allowing
//! applications to render through the composited display pipeline.

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloy_os_display::server::DisplayBackend;
use alloy_os_display::protocol::{SurfaceId, PixelFormat, Rect};
use crate::graphics::vesa::VesaDisplay;

/// Error type for Fusion display operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FusionError {
    /// Surface not found
    SurfaceNotFound,
    /// Invalid surface dimensions
    InvalidDimensions,
    /// Memory allocation failed
    AllocationFailed,
    /// Invalid pixel data
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
}

/// Fusion Display Backend
/// 
/// Manages framebuffer surfaces for the display server. Each surface represents
/// a renderable area that can be positioned, resized, and composited onto the
/// main framebuffer.
#[derive(Debug)]
pub struct FusionDisplayBackend {
    surfaces: BTreeMap<u32, SurfaceData>,
    next_surface_id: u32,
    display: Option<()>, // VesaDisplay reference (stored as unit for now due to lifetimes)
}

impl FusionDisplayBackend {
    /// Create a new Fusion display backend
    pub fn new(_display: VesaDisplay) -> Self {
        FusionDisplayBackend {
            surfaces: BTreeMap::new(),
            next_surface_id: 1,
            display: Some(()), // Note: VesaDisplay is owned, we're just noting it exists
        }
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
    pub fn get_surface_mut(&mut self, id: u32) -> Option<&mut SurfaceData> {
        self.surfaces.get_mut(&id)
    }

    /// Get a reference to a surface
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

        Ok(())
    }

    /// Set surface visibility
    pub fn set_visibility(&mut self, id: u32, visible: bool) -> Result<(), FusionError> {
        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;
        surface.visible = visible;
        Ok(())
    }

    /// Set surface z-order (draw order)
    pub fn set_z_order(&mut self, id: u32, z_order: u32) -> Result<(), FusionError> {
        let surface = self.surfaces.get_mut(&id).ok_or(FusionError::SurfaceNotFound)?;
        surface.z_order = z_order;
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

        // Validate dimensions match
        if surface.width != width || surface.height != height {
            return Err(FusionError::InvalidDimensions);
        }

        let expected_len = (width as usize) * (height as usize);
        if pixels.len() != expected_len {
            return Err(FusionError::InvalidPixelData);
        }

        // Copy pixel data
        surface.pixels.copy_from_slice(pixels);
        Ok(())
    }

    /// Get all surface IDs sorted by z-order
    pub fn surfaces_by_z_order(&self) -> Vec<u32> {
        let mut ids: Vec<_> = self.surfaces.iter()
            .filter(|(_, s)| s.visible)
            .collect();
        
        ids.sort_by_key(|(_, s)| s.z_order);
        ids.iter().map(|(id, _)| **id).collect()
    }
}

// Implement DisplayBackend trait for integration with display server
impl DisplayBackend for FusionDisplayBackend {
    type Error = FusionError;

    fn create_surface(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        _format: PixelFormat,
    ) -> Result<(), Self::Error> {
        if self.surfaces.contains_key(&surface_id.0) {
            return Err(FusionError::SurfaceNotFound);
        }
        
        let surface = SurfaceData::new(width, height)?;
        self.surfaces.insert(surface_id.0, surface);
        Ok(())
    }

    fn destroy_surface(&mut self, surface_id: SurfaceId) -> Result<(), Self::Error> {
        self.surfaces.remove(&surface_id.0).ok_or(FusionError::SurfaceNotFound)?;
        Ok(())
    }

    fn set_surface_position(
        &mut self,
        surface_id: SurfaceId,
        x: i32,
        y: i32,
    ) -> Result<(), Self::Error> {
        let surface = self.surfaces.get_mut(&surface_id.0).ok_or(FusionError::SurfaceNotFound)?;
        surface.x = x;
        surface.y = y;
        Ok(())
    }

    fn resize_surface(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        if width == 0 || height == 0 {
            return Err(FusionError::InvalidDimensions);
        }

        let surface = self.surfaces.get_mut(&surface_id.0).ok_or(FusionError::SurfaceNotFound)?;
        let new_pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(FusionError::InvalidDimensions)?;

        surface.width = width;
        surface.height = height;
        surface.pixels.clear();
        surface.pixels.resize(new_pixel_count, 0u32);

        Ok(())
    }

    fn set_surface_visibility(
        &mut self,
        surface_id: SurfaceId,
        visible: bool,
    ) -> Result<(), Self::Error> {
        let surface = self.surfaces.get_mut(&surface_id.0).ok_or(FusionError::SurfaceNotFound)?;
        surface.visible = visible;
        Ok(())
    }

    fn set_surface_z_order(
        &mut self,
        surface_id: SurfaceId,
        z_order: u32,
    ) -> Result<(), Self::Error> {
        let surface = self.surfaces.get_mut(&surface_id.0).ok_or(FusionError::SurfaceNotFound)?;
        surface.z_order = z_order;
        Ok(())
    }

    fn commit_surface(
        &mut self,
        _surface_id: SurfaceId,
        _damage: Option<Rect>,
    ) -> Result<(), Self::Error> {
        // No-op for now - just accept commit requests
        Ok(())
    }

    fn upload_surface_pixels(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        pixels: &[u32],
        _damage: Option<Rect>,
    ) -> Result<(), Self::Error> {
        let surface = self.surfaces.get_mut(&surface_id.0).ok_or(FusionError::SurfaceNotFound)?;

        // Validate dimensions match
        if surface.width != width || surface.height != height {
            return Err(FusionError::InvalidDimensions);
        }

        let expected_len = (width as usize) * (height as usize);
        if pixels.len() != expected_len {
            return Err(FusionError::InvalidPixelData);
        }

        // Copy pixel data
        surface.pixels.copy_from_slice(pixels);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // No-op for now - compositing would happen here
        Ok(())
    }
}
