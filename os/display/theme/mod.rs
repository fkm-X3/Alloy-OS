pub mod effects;
pub mod fonts;

pub const ALLOY_BG_PRIMARY: u32 = 0xFF0D1117;
pub const ALLOY_BG_SECONDARY: u32 = 0xFF161B22;
pub const ALLOY_BG_TERTIARY: u32 = 0xFF1C2333;
pub const ALLOY_BG_HOVER: u32 = 0xFF21262D;
pub const ALLOY_BG_ACTIVE: u32 = 0xFF283044;

pub const ALLOY_ACCENT_PRIMARY: u32 = 0xFF58A6FF;
pub const ALLOY_ACCENT_SECONDARY: u32 = 0xFF3FB950;
pub const ALLOY_ACCENT_WARNING: u32 = 0xFFD29922;
pub const ALLOY_ACCENT_DANGER: u32 = 0xFFF85149;

pub const ALLOY_TEXT_PRIMARY: u32 = 0xFFE6EDF3;
pub const ALLOY_TEXT_SECONDARY: u32 = 0xFF8B949E;
pub const ALLOY_TEXT_MUTED: u32 = 0xFF484F58;

pub const ALLOY_BORDER: u32 = 0xFF30363D;
pub const ALLOY_BORDER_ACTIVE: u32 = 0xFF58A6FF;

pub const ALLOY_WINDOW_FRAME_ACTIVE: u32 = 0xFF1A2332;
pub const ALLOY_WINDOW_FRAME_INACTIVE: u32 = 0xFF1C1C22;
pub const ALLOY_WINDOW_BODY: u32 = 0xFF0D1117;

pub const ALLOY_PANEL_TOP_LINE: u32 = 0xFF30363D;
pub const ALLOY_PANEL_TOP_LINE_ACTIVE: u32 = 0xFF58A6FF;

pub const ALLOY_GRADIENT_START: u32 = 0xFF0D1117;
pub const ALLOY_GRADIENT_MID: u32 = 0xFF111827;
pub const ALLOY_GRADIENT_END: u32 = 0xFF161B22;

pub const WINDOW_CORNER_RADIUS: u32 = 6;
pub const PANEL_CORNER_RADIUS: u32 = 0;
pub const LAUNCHER_CORNER_RADIUS: u32 = 8;
pub const BORDER_THICKNESS: u32 = 2;
pub const TITLE_BAR_HEIGHT: u32 = 22;
pub const PANEL_HEIGHT: u32 = 36;
pub const BUTTON_SIZE: u32 = 8;
pub const BUTTON_GAP: u32 = 5;
pub const SIMULATED_BLUR_RADIUS: u32 = 4;

pub const CLOCK_24H: bool = true;
pub const BACKGROUND_PATTERN_ENABLED: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherMode {
    List,
    Grid,
}

pub const LAUNCHER_DEFAULT_MODE: LauncherMode = LauncherMode::List;
pub const LAUNCHER_WIDTH: u32 = 360;
pub const LAUNCHER_HEIGHT_LIST: u32 = 280;
pub const LAUNCHER_HEIGHT_GRID: u32 = 240;
pub const LAUNCHER_SEARCH_HEIGHT: u32 = 32;
pub const LAUNCHER_FOOTER_HEIGHT: u32 = 24;
pub const LAUNCHER_ITEM_HEIGHT_LIST: u32 = 36;
pub const LAUNCHER_GRID_TILE_SIZE: u32 = 80;

fn extract_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    (r, g, b)
}

fn blend_color(base: u32, overlay: u32, alpha: u8) -> u32 {
    let (br, bg, bb) = extract_rgb(base);
    let (or, og, ob) = extract_rgb(overlay);
    let a = alpha as u32;
    let inv_a = 255 - a as u32;
    let r = ((br as u32 * inv_a + or as u32 * a) / 255) as u8;
    let g = ((bg as u32 * inv_a + og as u32 * a) / 255) as u8;
    let b = ((bb as u32 * inv_a + ob as u32 * a) / 255) as u8;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
