use super::{Font, GlyphMetrics};

/// TrueType font parsing and rendering (placeholder).
///
/// TTF support requires:
/// - Table directory parsing (cmap, head, hhea, hmtx, glyf, loca)
/// - Glyph outline parsing and rasterization (quadratic Beziers)
/// - Hinting instructions
///
/// This module provides a minimal rasterizer for embedded TTF data
#[derive(Debug)]
pub struct TtfFont {
    data: &'static [u8],
    units_per_em: u16,
    glyph_count: u16,
    pixel_size: u32,
}

impl TtfFont {
    pub fn new(data: &'static [u8], pixel_size: u32) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }
        let sfversion = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if sfversion != 0x00010000 && sfversion != 0x4F54544F {
            return None;
        }
        let _num_tables = u16::from_be_bytes([data[4], data[5]]);
        let units_per_em = Self::read_u16(data, 0x12)?;
        let glyph_count = 0;
        Some(TtfFont { data, units_per_em, glyph_count, pixel_size })
    }

    fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
        if offset + 2 > data.len() { None } else { Some(u16::from_be_bytes([data[offset], data[offset + 1]])) }
    }

    fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
        if offset + 4 > data.len() { None } else { Some(u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])) }
    }
}

impl Font for TtfFont {
    fn glyph_metrics(&self, _ch: char) -> Option<GlyphMetrics> {
        Some(GlyphMetrics { width: self.pixel_size, height: self.pixel_size, advance_x: self.pixel_size, bearing_x: 0, bearing_y: 0 })
    }

    fn render_glyph(&self, _ch: char, _pixels: &mut [u32], _buf_width: u32, _buf_height: u32, _x: u32, _y: u32, _color: u32, _bg_color: u32) {
        // TTF rasterization not yet implemented
    }

    fn char_width(&self) -> u32 { self.pixel_size }
    fn char_height(&self) -> u32 { self.pixel_size }
    fn line_height(&self) -> u32 { self.pixel_size.saturating_add(4) }
}
