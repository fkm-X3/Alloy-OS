use alloc::collections::BTreeMap;
use core::fmt;

use alloy_os_display::protocol::{PixelFormat, Rect, SurfaceId};
use alloy_os_display::server::DisplayBackend;

use crate::fusion::{Compositor, DisplayAdapter, Surface};
use crate::graphics::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FusionBackendError {
    UnsupportedPixelFormat,
    DuplicateSurfaceId,
    InvalidSurfaceId,
    SurfaceError,
    CompositorError,
    InvalidPixelBuffer,
    DimensionMismatch,
}

impl fmt::Display for FusionBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FusionBackendError::UnsupportedPixelFormat => write!(f, "unsupported pixel format"),
            FusionBackendError::DuplicateSurfaceId => write!(f, "duplicate surface id"),
            FusionBackendError::InvalidSurfaceId => write!(f, "invalid surface id"),
            FusionBackendError::SurfaceError => write!(f, "surface operation failed"),
            FusionBackendError::CompositorError => write!(f, "compositor operation failed"),
            FusionBackendError::InvalidPixelBuffer => write!(f, "invalid pixel buffer"),
            FusionBackendError::DimensionMismatch => write!(f, "surface dimension mismatch"),
        }
    }
}

/// Fusion-backed renderer adapter for the OS display server protocol/runtime.
pub struct FusionDisplayBackend {
    compositor: Compositor,
    surface_map: BTreeMap<SurfaceId, usize>,
    dirty: bool,
}

impl FusionDisplayBackend {
    pub fn new<D: Display + 'static>(display: D) -> Self {
        let adapter = DisplayAdapter::new(display);
        let compositor = Compositor::new(alloc::boxed::Box::new(adapter));
        Self {
            compositor,
            surface_map: BTreeMap::new(),
            dirty: true,
        }
    }

    fn compositor_surface_id(&self, surface_id: SurfaceId) -> Result<usize, FusionBackendError> {
        self.surface_map
            .get(&surface_id)
            .copied()
            .ok_or(FusionBackendError::InvalidSurfaceId)
    }
}

impl DisplayBackend for FusionDisplayBackend {
    type Error = FusionBackendError;

    fn create_surface(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        format: PixelFormat,
    ) -> Result<(), Self::Error> {
        if format != PixelFormat::Argb8888 {
            return Err(FusionBackendError::UnsupportedPixelFormat);
        }
        if self.surface_map.contains_key(&surface_id) {
            return Err(FusionBackendError::DuplicateSurfaceId);
        }

        let surface = Surface::new(width, height).map_err(|_| FusionBackendError::SurfaceError)?;
        let compositor_id = self
            .compositor
            .add_surface(surface)
            .map_err(|_| FusionBackendError::CompositorError)?;
        self.surface_map.insert(surface_id, compositor_id);
        self.dirty = true;
        Ok(())
    }

    fn destroy_surface(&mut self, surface_id: SurfaceId) -> Result<(), Self::Error> {
        let compositor_id = self.compositor_surface_id(surface_id)?;
        self.compositor
            .remove_surface(compositor_id)
            .map_err(|_| FusionBackendError::CompositorError)?;
        self.surface_map.remove(&surface_id);
        self.dirty = true;
        Ok(())
    }

    fn set_surface_position(
        &mut self,
        surface_id: SurfaceId,
        x: i32,
        y: i32,
    ) -> Result<(), Self::Error> {
        let compositor_id = self.compositor_surface_id(surface_id)?;
        let surface = self
            .compositor
            .get_surface_mut(compositor_id)
            .ok_or(FusionBackendError::InvalidSurfaceId)?;
        surface.set_position(x, y);
        self.dirty = true;
        Ok(())
    }

    fn resize_surface(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        let compositor_id = self.compositor_surface_id(surface_id)?;
        let surface = self
            .compositor
            .get_surface_mut(compositor_id)
            .ok_or(FusionBackendError::InvalidSurfaceId)?;
        surface
            .resize(width, height)
            .map_err(|_| FusionBackendError::SurfaceError)?;
        self.dirty = true;
        Ok(())
    }

    fn set_surface_visibility(
        &mut self,
        surface_id: SurfaceId,
        visible: bool,
    ) -> Result<(), Self::Error> {
        let compositor_id = self.compositor_surface_id(surface_id)?;
        let surface = self
            .compositor
            .get_surface_mut(compositor_id)
            .ok_or(FusionBackendError::InvalidSurfaceId)?;
        surface.set_visible(visible);
        self.dirty = true;
        Ok(())
    }

    fn set_surface_z_order(
        &mut self,
        surface_id: SurfaceId,
        z_order: u32,
    ) -> Result<(), Self::Error> {
        let compositor_id = self.compositor_surface_id(surface_id)?;
        self.compositor
            .update_z_order(compositor_id, z_order)
            .map_err(|_| FusionBackendError::CompositorError)?;
        self.dirty = true;
        Ok(())
    }

    fn commit_surface(
        &mut self,
        surface_id: SurfaceId,
        _damage: Option<Rect>,
    ) -> Result<(), Self::Error> {
        let _ = self.compositor_surface_id(surface_id)?;
        self.dirty = true;
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
        let compositor_id = self.compositor_surface_id(surface_id)?;
        let surface = self
            .compositor
            .get_surface_mut(compositor_id)
            .ok_or(FusionBackendError::InvalidSurfaceId)?;

        let (surface_width, surface_height) = surface.get_dimensions();
        if surface_width != width || surface_height != height {
            return Err(FusionBackendError::DimensionMismatch);
        }

        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(FusionBackendError::InvalidPixelBuffer)?;
        if pixels.len() < pixel_count {
            return Err(FusionBackendError::InvalidPixelBuffer);
        }

        let surface_buffer = surface.get_buffer_mut();
        if surface_buffer.len() < pixel_count {
            return Err(FusionBackendError::InvalidPixelBuffer);
        }

        surface_buffer[..pixel_count].copy_from_slice(&pixels[..pixel_count]);
        self.dirty = true;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        if self.dirty {
            self.compositor
                .composite_dirty()
                .map_err(|_| FusionBackendError::CompositorError)?;
            self.dirty = false;
        }
        Ok(())
    }
}
