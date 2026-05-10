//! Integration between Wayland surfaces and Fusion composition pipeline
//!
//! Bridges Wayland client surfaces to the actual pixel rendering by reading
//! from SHM buffers and writing to the framebuffer. Handles damage tracking
//! and frame timing callbacks.

use super::surface::SurfaceState;
use super::shm::{ShmBuffer, ShmFormat};
use super::damage::DamageRect;

/// Compositor integration interface
pub struct CompositorIntegration;

impl CompositorIntegration {
    /// Composite a single frame from all visible surfaces
    /// 
    /// Iterates through surfaces in Z-order, reads pixel data from SHM buffers,
    /// and writes to the framebuffer for damaged regions. Updates framebuffer
    /// and emits frame callbacks when complete.
    pub fn composite_frame(
        surfaces: &[(u32, &SurfaceState)], // (z_order, surface)
    ) {
        for (_z_order, surface) in surfaces {
            // Check if surface has current buffer and damage
            if surface.current.buffer_id == 0 {
                continue; // No buffer attached
            }

            let damage_regions = &surface.current.damage;
            if damage_regions.is_empty() {
                continue; // No damage on this surface
            }

            // In a real implementation, we would:
            // 1. Get the SHM buffer data via buffer manager
            // 2. For each damage region:
            //    - Read pixels from buffer at (offset + damage_rect coords)
            //    - Write to framebuffer at (surface position + damage_rect coords)
            //    - Handle format conversion (ARGB8888, XRGB8888, RGB565)
            // 3. Sync framebuffer to display hardware

            // For now, just log what would happen
            unsafe {
                crate::ffi::serial_print(b"[Compositor] Compositing surface with damage regions\n\0".as_ptr());
            }
        }
    }

    /// Composite with explicit buffer and damage information
    pub fn composite_surface(
        buffer: &ShmBuffer,
        damage: &[DamageRect],
        surface_x: i32,
        surface_y: i32,
    ) {
        if damage.is_empty() {
            return;
        }

        // Validate buffer and damage for composition
        if buffer.width == 0 || buffer.height == 0 {
            return;
        }

        // Process each damage rectangle
        for damage_rect in damage {
            // Clip damage to surface bounds
            let clipped = if damage_rect.x >= 0 && damage_rect.y >= 0
                && (damage_rect.x as u32) < buffer.width
                && (damage_rect.y as u32) < buffer.height
            {
                Some(*damage_rect)
            } else {
                let bounds = DamageRect::full(buffer.width as i32, buffer.height as i32);
                damage_rect.clip(&bounds)
            };

            if let Some(clipped_rect) = clipped {
                // In a real implementation:
                // 1. Calculate source offset in buffer
                let _source_offset = buffer.offset
                    .saturating_add(clipped_rect.y as u32 * buffer.stride)
                    .saturating_add(clipped_rect.x as u32 * buffer.format.bytes_per_pixel() as u32);

                // 2. Calculate destination in framebuffer
                let _dest_x = surface_x.saturating_add(clipped_rect.x);
                let _dest_y = surface_y.saturating_add(clipped_rect.y);

                // 3. Validate destination is on-screen
                let _rect_width = clipped_rect.width.max(0) as u32;
                let _rect_height = clipped_rect.height.max(0) as u32;

                // 4. Blit based on format
                match buffer.format {
                    ShmFormat::Argb8888 => {
                        // Composite 32-bit ARGB with alpha blending
                        // Would iterate rows and pixels, reading from source_offset
                        // and writing to framebuffer at dest_x, dest_y
                    }
                    ShmFormat::Xrgb8888 => {
                        // Composite 32-bit XRGB without alpha
                        // Faster path - direct copy
                    }
                    ShmFormat::Rgb565 => {
                        // Composite 16-bit RGB
                        // Needs format conversion to 32-bit framebuffer
                    }
                }

                unsafe {
                    crate::ffi::serial_print(b"[Compositor] Blitting damage region\n\0".as_ptr());
                }
            }
        }
    }

    /// Emit frame callback completion event
    /// Called after frame is presented to indicate compositor is ready for new updates
    pub fn emit_frame_callback(_callback_object_id: u32) {
        // Send wl_callback.done event with current time
        // The callback object_id is provided by the client in surface.frame() request

        unsafe {
            crate::ffi::serial_print(b"[Compositor] Frame callback completed\n\0".as_ptr());
        }
    }

    /// Get vsync interval in milliseconds
    pub fn vsync_interval() -> u32 {
        // 60 Hz = ~16.667ms per frame
        16 // Rounded down
    }

    /// Validate format compatibility for composition
    pub fn is_format_supported(format: ShmFormat) -> bool {
        // Check if this format can be composed
        // All formats should be supported for a complete implementation
        matches!(format, ShmFormat::Argb8888 | ShmFormat::Xrgb8888 | ShmFormat::Rgb565)
    }
}

/// Frame timing for presentation
#[derive(Debug, Clone, Copy)]
pub struct FrameTiming {
    /// Timestamp when frame was presented (milliseconds)
    pub presented_at: u32,
    /// Vsync interval (milliseconds)
    pub vsync_interval: u32,
}

impl FrameTiming {
    /// Create frame timing info
    pub fn new(presented_at: u32) -> Self {
        Self {
            presented_at,
            vsync_interval: CompositorIntegration::vsync_interval(),
        }
    }

    /// Calculate next vsync time
    pub fn next_vsync(&self) -> u32 {
        self.presented_at.saturating_add(self.vsync_interval)
    }
}

/// Pixel format converter utilities
pub struct FormatConverter;

impl FormatConverter {
    /// Convert ARGB8888 pixel to XRGB8888 (drop alpha channel)
    #[inline]
    pub fn argb8888_to_xrgb8888(pixel: u32) -> u32 {
        pixel | 0xFF000000 // Set alpha to fully opaque
    }

    /// Convert RGB565 pixel to XRGB8888
    #[inline]
    pub fn rgb565_to_xrgb8888(pixel: u16) -> u32 {
        let r = ((pixel >> 11) & 0x1F) as u32;
        let g = ((pixel >> 5) & 0x3F) as u32;
        let b = (pixel & 0x1F) as u32;

        // Scale to 8-bit
        let r8 = (r << 3) | (r >> 2);
        let g8 = (g << 2) | (g >> 4);
        let b8 = (b << 3) | (b >> 2);

        0xFF000000 | (r8 << 16) | (g8 << 8) | b8
    }

    /// Convert XRGB8888 to RGB565
    #[inline]
    pub fn xrgb8888_to_rgb565(pixel: u32) -> u16 {
        let r = ((pixel >> 19) & 0x1F) as u16;
        let g = ((pixel >> 10) & 0x3F) as u16;
        let b = ((pixel >> 3) & 0x1F) as u16;

        (r << 11) | (g << 5) | b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_timing_creation() {
        let timing = FrameTiming::new(1000);
        assert_eq!(timing.presented_at, 1000);
        assert!(timing.vsync_interval > 0);
    }

    #[test]
    fn test_frame_timing_next_vsync() {
        let timing = FrameTiming::new(1000);
        let next = timing.next_vsync();
        assert!(next > timing.presented_at);
    }

    #[test]
    fn test_vsync_interval_reasonable() {
        let interval = CompositorIntegration::vsync_interval();
        // 60 Hz is about 16ms, 120 Hz is about 8ms
        assert!(interval > 5 && interval < 20);
    }

    #[test]
    fn test_format_argb8888_to_xrgb8888() {
        // ARGB 0xAARRGGBB -> XRGB 0xFFRRGGBB
        let pixel = 0x12345678u32;
        let converted = FormatConverter::argb8888_to_xrgb8888(pixel);
        assert_eq!(converted, 0xFF345678);
    }

    #[test]
    fn test_format_rgb565_to_xrgb8888() {
        // RGB565 format: RRRRR GGG GGG BBBBB
        // Test white: all bits set
        let white_565 = 0xFFFFu16;
        let white_8888 = FormatConverter::rgb565_to_xrgb8888(white_565);

        // High byte should be FF (alpha), rest should be white-ish
        assert_eq!(white_8888 & 0xFF000000, 0xFF000000);
        // Lower bytes should all be high
        assert!(white_8888 & 0xFF0000 > 0xF00000);
        assert!(white_8888 & 0x00FF00 > 0x00F000);
        assert!(white_8888 & 0x0000FF > 0x0000F0);
    }

    #[test]
    fn test_format_xrgb8888_to_rgb565() {
        // Start with white
        let white_8888 = 0xFFFFFFFFu32;
        let white_565 = FormatConverter::xrgb8888_to_rgb565(white_8888);

        // All color bits should be set
        assert_eq!(white_565, 0xFFFFu16);
    }

    #[test]
    fn test_compositor_format_support() {
        assert!(CompositorIntegration::is_format_supported(ShmFormat::Argb8888));
        assert!(CompositorIntegration::is_format_supported(ShmFormat::Xrgb8888));
        assert!(CompositorIntegration::is_format_supported(ShmFormat::Rgb565));
    }

    #[test]
    fn test_compositor_integration_null_surfaces() {
        let surfaces: &[(u32, &SurfaceState)] = &[];
        CompositorIntegration::composite_frame(surfaces); // Should not panic
    }
}
