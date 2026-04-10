// Example usage of rectangle drawing primitives in Alloy OS Fusion UI
// This demonstrates how the rect module integrates with the Fusion framework

// Module hierarchy:
// kernel::fusion::ui::rect
// kernel::fusion::ui (exports rect functions)
// kernel::fusion (re-exports ui functions)

use crate::fusion::ui::{
    draw_filled_rect,
    draw_outline_rect,
    draw_rounded_rect,
    draw_gradient_rect_vertical,
    draw_gradient_rect_horizontal,
    RectError,
};
use crate::fusion::{Surface, SurfaceError};

/// Example: Draw a UI panel background
pub fn draw_panel_background(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), RectError> {
    // Main panel body with gradient background
    draw_gradient_rect_vertical(
        surface,
        x,
        y,
        width,
        height,
        0xFF1a1a2e, // Dark blue
        0xFF16213e, // Slightly darker blue
    )?;

    // Outer border
    draw_outline_rect(surface, x, y, width, height, 0xFF0f3460, 2)?;

    Ok(())
}

/// Example: Draw a rounded button
pub fn draw_button(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    is_hovered: bool,
) -> Result<(), RectError> {
    let color = if is_hovered {
        0xFF00d4ff // Bright cyan when hovered
    } else {
        0xFF00a8cc // Standard cyan
    };

    // Button background with rounded corners
    draw_rounded_rect(surface, x, y, width, height, 8, color)?;

    // Optional: highlight border for depth
    draw_outline_rect(surface, x + 1, y + 1, width - 2, height - 2, 0xFFFFFFFF, 1)?;

    Ok(())
}

/// Example: Draw a window frame
pub fn draw_window_frame(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    title_bar_height: u32,
) -> Result<(), RectError> {
    // Main window background
    draw_filled_rect(surface, x, y, width, height, 0xFF2a2a3e)?;

    // Title bar with gradient
    draw_gradient_rect_horizontal(
        surface,
        x,
        y,
        width,
        title_bar_height,
        0xFF0f3460, // Left: darker
        0xFF00d4ff, // Right: accent color
    )?;

    // Window border
    draw_outline_rect(surface, x, y, width, height, 0xFF0f3460, 1)?;

    Ok(())
}

/// Example: Draw a progress bar
pub fn draw_progress_bar(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    progress: f32, // 0.0 to 1.0
) -> Result<(), RectError> {
    // Background
    draw_filled_rect(surface, x, y, width, height, 0xFF1a1a2e)?;

    // Progress fill
    if progress > 0.0 {
        let filled_width = ((width as f32) * progress) as u32;
        draw_gradient_rect_horizontal(
            surface,
            x,
            y,
            filled_width,
            height,
            0xFF00a8cc, // Left: standard color
            0xFF00ff00, // Right: green
        )?;
    }

    // Border
    draw_outline_rect(surface, x, y, width, height, 0xFF0f3460, 1)?;

    Ok(())
}

/// Example: Draw a checkbox
pub fn draw_checkbox(
    surface: &mut Surface,
    x: u32,
    y: u32,
    size: u32,
    checked: bool,
) -> Result<(), RectError> {
    // Checkbox background
    draw_filled_rect(surface, x, y, size, size, 0xFF2a2a3e)?;

    // Checkbox border
    draw_outline_rect(surface, x, y, size, size, 0xFF0f3460, 2)?;

    if checked {
        // Checkmark fill
        let inner = size / 4;
        draw_filled_rect(
            surface,
            x + inner,
            y + inner,
            size - 2 * inner,
            size - 2 * inner,
            0xFF00d4ff,
        )?;
    }

    Ok(())
}

/// Example: Draw a tooltip/notification box
pub fn draw_tooltip(
    surface: &mut Surface,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), RectError> {
    // Main box with rounded corners
    draw_rounded_rect(surface, x, y, width, height, 4, 0xFF1a1a2e)?;

    // Border
    draw_outline_rect(surface, x, y, width, height, 0xFF00d4ff, 1)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_drawing() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_panel_background(&mut surface, 100, 100, 200, 150).is_ok());
    }

    #[test]
    fn test_button_drawing_normal() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_button(&mut surface, 100, 100, 100, 40, false).is_ok());
    }

    #[test]
    fn test_button_drawing_hovered() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_button(&mut surface, 100, 100, 100, 40, true).is_ok());
    }

    #[test]
    fn test_window_drawing() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_window_frame(&mut surface, 50, 50, 400, 300, 30).is_ok());
    }

    #[test]
    fn test_progress_bar_zero() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_progress_bar(&mut surface, 100, 100, 200, 20, 0.0).is_ok());
    }

    #[test]
    fn test_progress_bar_full() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_progress_bar(&mut surface, 100, 100, 200, 20, 1.0).is_ok());
    }

    #[test]
    fn test_progress_bar_partial() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_progress_bar(&mut surface, 100, 100, 200, 20, 0.5).is_ok());
    }

    #[test]
    fn test_checkbox_unchecked() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_checkbox(&mut surface, 100, 100, 20, false).is_ok());
    }

    #[test]
    fn test_checkbox_checked() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_checkbox(&mut surface, 100, 100, 20, true).is_ok());
    }

    #[test]
    fn test_tooltip_drawing() {
        let mut surface = Surface::new(800, 600).unwrap();
        assert!(draw_tooltip(&mut surface, 200, 200, 150, 50).is_ok());
    }
}
