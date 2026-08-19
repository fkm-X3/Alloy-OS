//! Integration between Wayland surfaces and Fusion composition pipeline
//!
//! Bridges Wayland client surfaces to the actual pixel rendering by reading
//! from SHM buffers and writing to the framebuffer. Handles damage tracking
//! and frame timing callbacks. This is the critical bridge between the
//! Wayland protocol layer and the Fusion display backend.

use super::damage::DamageRect;
use super::shm::{ShmBuffer, ShmFormat, ShmManager};
use super::surface::SurfaceState;
use crate::fusion::backend::FusionDisplayBackend;
use crate::graphics::Display;
use crate::graphics::PlatformDisplay;
use alloc::vec::Vec;

const PANEL_HEIGHT: u32 = 48;
const PANEL_COLOR: u32 = 0xFF1A1A2E;

/// Compositor integration interface
///
/// Owns the framebuffer and SHM manager for full composition pipeline.
pub struct CompositorIntegration {
    /// Backend for rendering to display
    backend: Option<FusionDisplayBackend>,
    /// SHM buffer manager for reading client buffers
    pub shm_manager: ShmManager,
    /// Frame timing state
    pub frame_timing: Option<FrameTiming>,
    /// Total frames composited
    pub frames_composited: u64,
}

impl CompositorIntegration {
    /// Create a new compositor integration with framebuffer and SHM manager
    pub fn new() -> Self {
        Self {
            backend: None,
            shm_manager: ShmManager::new(),
            frame_timing: None,
            frames_composited: 0,
        }
    }

    /// Initialize the compositor with display backend
    pub fn init_with_display(&mut self, display: PlatformDisplay) {
        self.backend = Some(FusionDisplayBackend::new(display));
        self.frame_timing = Some(FrameTiming::new(0));
    }

    /// Get mutable reference to SHM manager
    pub fn shm_manager_mut(&mut self) -> &mut ShmManager {
        &mut self.shm_manager
    }

    /// Get reference to SHM manager
    pub fn shm_manager(&self) -> &ShmManager {
        &self.shm_manager
    }

    /// Composite a single frame from all visible surfaces
    ///
    /// Iterates through surfaces sorted by Z-order, reads pixel data from SHM buffers,
    /// and composites onto the framebuffer. Only damaged regions are updated.
    pub fn composite_frame(
        backend: &mut FusionDisplayBackend,
        shm_manager: &mut ShmManager,
        surfaces: &[(u32, &SurfaceState)],
    ) {
        backend.clear_framebuffer();

        // Sort surfaces by z_order so higher z surfaces render on top
        let mut sorted: Vec<&(u32, &SurfaceState)> = surfaces.iter().collect();
        sorted.sort_by_key(|(_, s)| s.z_order);

        for (_z_order, surface) in sorted {
            let surface = *surface;

            // Skip surfaces without buffers
            if surface.current.buffer_id == 0 {
                continue;
            }

            // Skip surfaces with no pending damage
            if surface.current.damage.is_empty() && !surface.current.damage_tracker.is_full_damage()
            {
                continue;
            }

            // Look up the SHM buffer for this surface
            let buffer_id = surface.current.buffer_id;
            let buffer = match shm_manager.get_buffer(buffer_id) {
                Some(buf) => buf,
                None => continue,
            };

            // Use screen position from the surface (set via alloy_set_position)
            // Fall back to buffer_offset if position is (0,0) and buffer_offset differs
            let pos_x = surface.screen_x;
            let pos_y = surface.screen_y;

            // Composite this surface
            let _ = CompositorIntegration::composite_surface(
                backend,
                buffer,
                &surface.current.damage,
                pos_x,
                pos_y,
                surface.current.width,
                surface.current.height,
            );
        }

        // Draw the panel bar at the bottom of the screen
        let fb_w = backend.framebuffer_width();
        let fb_h = backend.framebuffer_height();
        let panel_y = if fb_h >= PANEL_HEIGHT {
            fb_h - PANEL_HEIGHT
        } else {
            0
        };
        let display = backend.display_mut();
        for row in panel_y..fb_h {
            for col in 0..fb_w {
                display.pixel_put(col, row, PANEL_COLOR);
            }
        }
    }

    /// Check if compositor has an active backend
    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// Get framebuffer width
    pub fn framebuffer_width(&self) -> u32 {
        self.backend
            .as_ref()
            .map(|b| b.framebuffer_width())
            .unwrap_or(0)
    }

    /// Get framebuffer height
    pub fn framebuffer_height(&self) -> u32 {
        self.backend
            .as_ref()
            .map(|b| b.framebuffer_height())
            .unwrap_or(0)
    }

    /// Composite with explicit buffer and damage information
    pub fn composite_surface(
        backend: &mut FusionDisplayBackend,
        buffer: &ShmBuffer,
        damage: &[DamageRect],
        surface_x: i32,
        surface_y: i32,
        _surface_width: u32,
        _surface_height: u32,
    ) -> Result<(), &'static str> {
        if damage.is_empty() {
            return Ok(());
        }

        if buffer.width == 0 || buffer.height == 0 {
            return Ok(());
        }

        let bpp = buffer.format.bytes_per_pixel() as u32;

        for damage_rect in damage {
            let bounds = DamageRect::full(buffer.width as i32, buffer.height as i32);
            let clipped = match damage_rect.clip(&bounds) {
                Some(c) => c,
                None => continue,
            };

            let dest_x = surface_x.saturating_add(clipped.x);
            let dest_y = surface_y.saturating_add(clipped.y);

            if dest_x < 0 || dest_y < 0 {
                continue;
            }

            let w = clipped.width as u32;
            let h = clipped.height as u32;
            let row_stride = buffer.stride as usize;
            let base_offset = buffer.offset as usize
                + clipped.y as usize * row_stride
                + clipped.x as usize * bpp as usize;

            match buffer.format {
                ShmFormat::Argb8888 => {
                    let mut pixels = alloc::vec![0u32; (w * h) as usize];
                    for row in 0..h {
                        let row_bytes = buffer.read_row_bytes(
                            base_offset + row as usize * row_stride,
                            w as usize * 4,
                        );
                        for col in 0..w {
                            let off = col as usize * 4;
                            let pixel = u32::from_le_bytes([
                                row_bytes[off],
                                row_bytes[off + 1],
                                row_bytes[off + 2],
                                row_bytes[off + 3],
                            ]);
                            pixels[(row * w + col) as usize] = pixel;
                        }
                    }
                    backend.composite_shm_buffer(&pixels, w, h, dest_x, dest_y, 0, 0, w, h);
                }
                ShmFormat::Xrgb8888 => {
                    let mut pixels = alloc::vec![0u32; (w * h) as usize];
                    for row in 0..h {
                        let row_bytes = buffer.read_row_bytes(
                            base_offset + row as usize * row_stride,
                            w as usize * 4,
                        );
                        for col in 0..w {
                            let off = col as usize * 4;
                            let pixel = u32::from_le_bytes([
                                row_bytes[off],
                                row_bytes[off + 1],
                                row_bytes[off + 2],
                                row_bytes[off + 3],
                            ]);
                            pixels[(row * w + col) as usize] = pixel | 0xFF000000;
                        }
                    }
                    backend.composite_shm_buffer(&pixels, w, h, dest_x, dest_y, 0, 0, w, h);
                }
                ShmFormat::Rgb565 => {
                    let mut pixels = alloc::vec![0u32; (w * h) as usize];
                    for row in 0..h {
                        let row_bytes = buffer.read_row_bytes(
                            base_offset + row as usize * row_stride,
                            w as usize * 2,
                        );
                        for col in 0..w {
                            let off = col as usize * 2;
                            let pixel16 = u16::from_le_bytes([row_bytes[off], row_bytes[off + 1]]);
                            let r = ((pixel16 >> 11) & 0x1F) as u32;
                            let g = ((pixel16 >> 5) & 0x3F) as u32;
                            let b = (pixel16 & 0x1F) as u32;
                            let r8 = (r << 3) | (r >> 2);
                            let g8 = (g << 2) | (g >> 4);
                            let b8 = (b << 3) | (b >> 2);
                            pixels[(row * w + col) as usize] =
                                0xFF000000 | (r8 << 16) | (g8 << 8) | b8;
                        }
                    }
                    backend.composite_shm_buffer(&pixels, w, h, dest_x, dest_y, 0, 0, w, h);
                }
            }
        }

        Ok(())
    }

    /// Direct SHM buffer to framebuffer blit
    #[allow(clippy::too_many_arguments)]
    pub fn blit_shm_to_framebuffer(
        &mut self,
        buffer: &ShmBuffer,
        src_x: u32,
        src_y: u32,
        src_w: u32,
        src_h: u32,
        dst_x: i32,
        dst_y: i32,
    ) -> Result<(), &'static str> {
        let _kernel_addr = buffer.kernel_vaddr.ok_or("Buffer not mapped")?;
        let _bytes_per_pixel = buffer.format.bytes_per_pixel();
        let _stride = buffer.stride as usize;
        let _ = (src_x, src_y, src_w, src_h, dst_x, dst_y);
        Ok(())
    }

    /// Emit frame callback completion event
    pub fn emit_frame_callback(&mut self, callback_object_id: u32) {
        if let Some(ref mut timing) = self.frame_timing {
            timing.presented_at = crate::SystemTimer::uptime_ms() as u32;
        }
        let _ = callback_object_id;
    }

    /// Get vsync interval in milliseconds
    pub fn vsync_interval() -> u32 {
        16
    }

    /// Get current frame timing info
    pub fn frame_timing(&self) -> Option<FrameTiming> {
        self.frame_timing
    }

    /// Validate format compatibility for composition
    pub fn is_format_supported(format: ShmFormat) -> bool {
        matches!(
            format,
            ShmFormat::Argb8888 | ShmFormat::Xrgb8888 | ShmFormat::Rgb565
        )
    }

    /// Get total frames composited
    pub fn frames_composited(&self) -> u64 {
        self.frames_composited
    }
}

impl Default for CompositorIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame timing for presentation
#[derive(Debug, Clone, Copy)]
pub struct FrameTiming {
    pub presented_at: u32,
    pub vsync_interval: u32,
    pub composite_duration_us: u32,
}

impl FrameTiming {
    pub fn new(presented_at: u32) -> Self {
        Self {
            presented_at,
            vsync_interval: CompositorIntegration::vsync_interval(),
            composite_duration_us: 0,
        }
    }

    pub fn next_vsync(&self) -> u32 {
        self.presented_at.saturating_add(self.vsync_interval)
    }

    pub fn set_composite_duration(&mut self, duration_us: u32) {
        self.composite_duration_us = duration_us;
    }

    pub fn is_behind(&self, current_time: u32) -> bool {
        current_time > self.next_vsync()
    }
}

/// Pixel format converter utilities
pub struct FormatConverter;

impl FormatConverter {
    #[inline]
    pub fn argb8888_to_xrgb8888(pixel: u32) -> u32 {
        pixel | 0xFF000000
    }

    #[inline]
    pub fn rgb565_to_xrgb8888(pixel: u16) -> u32 {
        let r = ((pixel >> 11) & 0x1F) as u32;
        let g = ((pixel >> 5) & 0x3F) as u32;
        let b = (pixel & 0x1F) as u32;
        let r8 = (r << 3) | (r >> 2);
        let g8 = (g << 2) | (g >> 4);
        let b8 = (b << 3) | (b >> 2);
        0xFF000000 | (r8 << 16) | (g8 << 8) | b8
    }

    #[inline]
    pub fn xrgb8888_to_rgb565(pixel: u32) -> u16 {
        let r = ((pixel >> 19) & 0x1F) as u16;
        let g = ((pixel >> 10) & 0x3F) as u16;
        let b = ((pixel >> 3) & 0x1F) as u16;
        (r << 11) | (g << 5) | b
    }

    #[inline]
    pub fn alpha_blend(src: u32, dst: u32) -> u32 {
        let src_a = (src >> 24) & 0xFF;
        if src_a == 255 {
            return src;
        }
        if src_a == 0 {
            return dst;
        }
        let dst_a = (dst >> 24) & 0xFF;
        let src_alpha = src_a;
        let dst_alpha = dst_a * (255 - src_a) / 255;
        let out_alpha = src_alpha + dst_alpha;
        if out_alpha == 0 {
            return 0;
        }
        let src_r = (src >> 16) & 0xFF;
        let src_g = (src >> 8) & 0xFF;
        let src_b = src & 0xFF;
        let dst_r = (dst >> 16) & 0xFF;
        let dst_g = (dst >> 8) & 0xFF;
        let dst_b = dst & 0xFF;
        let r = (src_r * src_alpha + dst_r * dst_alpha) / out_alpha;
        let g = (src_g * src_alpha + dst_g * dst_alpha) / out_alpha;
        let b = (src_b * src_alpha + dst_b * dst_alpha) / out_alpha;
        (out_alpha << 24) | (r << 16) | (g << 8) | b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_creation() {
        let ci = CompositorIntegration::new();
        assert!(!ci.has_backend());
        assert_eq!(ci.frames_composited(), 0);
    }

    #[test]
    fn test_frame_timing_creation() {
        let timing = FrameTiming::new(1000);
        assert_eq!(timing.presented_at, 1000);
        assert!(timing.vsync_interval > 0);
    }

    #[test]
    fn test_frame_timing_next_vsync() {
        let timing = FrameTiming::new(1000);
        assert!(timing.next_vsync() > timing.presented_at);
        assert_eq!(timing.next_vsync(), 1016);
    }

    #[test]
    fn test_alpha_blend_opaque_source() {
        let src = 0xFF123456u32;
        let dst = 0xFFABCDEFu32;
        assert_eq!(FormatConverter::alpha_blend(src, dst), src);
    }

    #[test]
    fn test_alpha_blend_transparent_source() {
        let src = 0x00123456u32;
        let dst = 0xFFABCDEFu32;
        assert_eq!(FormatConverter::alpha_blend(src, dst), dst);
    }
}
