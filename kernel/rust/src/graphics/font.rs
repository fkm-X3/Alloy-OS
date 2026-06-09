use core::fmt::Debug;

#[allow(clippy::too_many_arguments)]

pub mod bitmap;
pub mod ttf;

pub use bitmap::BitmapFont8x16;

/// Glyph metrics for a rendered character
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub advance_x: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
}

/// Font trait for rendering glyphs to pixel buffers
pub trait Font: Debug {
    fn glyph_metrics(&self, ch: char) -> Option<GlyphMetrics>;
    fn render_glyph(&self, ch: char, pixels: &mut [u32], buf_width: u32, buf_height: u32, x: u32, y: u32, color: u32, bg_color: u32);
    fn char_width(&self) -> u32;
    fn char_height(&self) -> u32;
    fn line_height(&self) -> u32;
}
