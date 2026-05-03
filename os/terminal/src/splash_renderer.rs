//! Splash Screen Framebuffer Renderer
//!
//! Renders the Alloy OS splash screen (rotating logos) directly to a framebuffer
//! for testing and eventual kernel integration.

use crate::framebuffer_renderer::{Color, Framebuffer};
use std::time::{SystemTime, UNIX_EPOCH};
use std::f32::consts::PI;

/// Render the splash screen to a framebuffer
pub fn render_splash_to_framebuffer(fb: &mut Framebuffer, rotation_angle: f32) {
    // Clear background to dark gray
    fb.clear(Color::DARK_GRAY);

    let width = fb.width();
    let height = fb.height();
    let center_x = width / 2;
    let center_y = height / 2;

    // Draw title at top
    draw_text_centered(fb, "Alloy OS", center_x, 50, Color::WHITE, 2);

    // Draw two rotating circles to represent the rotating logos
    let radius = 100u32;
    let spacing = 250u32;

    // Left circle (light mode)
    let left_x = center_x.saturating_sub(spacing / 2);
    draw_rotating_circle(fb, left_x, center_y, radius, Color::CYAN, rotation_angle);
    draw_text_centered(fb, "Light", left_x, center_y + radius + 30, Color::WHITE, 1);

    // Right circle (dark mode)
    let right_x = center_x.saturating_add(spacing / 2);
    draw_rotating_circle(fb, right_x, center_y, radius, Color::YELLOW, rotation_angle + 180.0);
    draw_text_centered(fb, "Dark", right_x, center_y + radius + 30, Color::WHITE, 1);

    // Draw angle indicator at bottom
    let angle_text = format!("{:.0}°", rotation_angle % 360.0);
    draw_text_centered(fb, &angle_text, center_x, height - 50, Color::CYAN, 1);
}

/// Draw a rotating circle with a marker line
fn draw_rotating_circle(fb: &mut Framebuffer, cx: u32, cy: u32, radius: u32, color: Color, angle_deg: f32) {
    // Draw outer circle
    fb.circle(cx, cy, radius, color, false);

    // Draw filled inner circle
    fb.circle(cx, cy, radius / 3, color, true);

    // Draw rotating marker line
    let angle_rad = angle_deg * PI / 180.0;
    let line_len = (radius as f32 * 0.8) as u32;
    let x1 = (cx as f32 + angle_rad.cos() * line_len as f32) as u32;
    let y1 = (cy as f32 + angle_rad.sin() * line_len as f32) as u32;
    fb.draw_line(cx, cy, x1, y1, color);
}

/// Simple text rendering - draws characters in a grid pattern
fn draw_text_centered(fb: &mut Framebuffer, text: &str, x: u32, y: u32, color: Color, scale: u32) {
    let char_width = (5 * scale) as u32;
    let total_width = (text.len() as u32) * char_width;
    let start_x = x.saturating_sub(total_width / 2);

    for (i, ch) in text.chars().enumerate() {
        let char_x = start_x + (i as u32) * char_width;
        draw_char(fb, ch, char_x, y, color, scale);
    }
}

/// Draw a single character as a simple pattern
fn draw_char(fb: &mut Framebuffer, ch: char, x: u32, y: u32, color: Color, scale: u32) {
    match ch {
        'A' => draw_letter_a(fb, x, y, color, scale),
        'B' => draw_letter_b(fb, x, y, color, scale),
        'C' => draw_letter_c(fb, x, y, color, scale),
        'D' => draw_letter_d(fb, x, y, color, scale),
        'E' => draw_letter_e(fb, x, y, color, scale),
        'L' => draw_letter_l(fb, x, y, color, scale),
        'O' => draw_letter_o(fb, x, y, color, scale),
        'S' => draw_letter_s(fb, x, y, color, scale),
        'y' => draw_letter_y(fb, x, y, color, scale),
        '°' => draw_degree(fb, x, y, color, scale),
        ' ' => {}, // Space - do nothing
        _ => draw_generic_char(fb, x, y, color, scale),
    }
}

fn draw_letter_a(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    // Diagonal lines forming A
    for i in 0..h {
        fb.set_pixel(x + (i / 2), y + i, color);
        fb.set_pixel(x + w - (i / 2), y + i, color);
    }
    // Horizontal bar
    for i in 0..w {
        fb.set_pixel(x + i, y + h / 2, color);
    }
}

fn draw_letter_b(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    // Vertical line
    for i in 0..h {
        fb.set_pixel(x, y + i, color);
    }
    // Top bump
    for i in 0..w {
        fb.set_pixel(x + i, y, color);
        fb.set_pixel(x + i, y + h / 2, color);
    }
    for i in 0..(h / 2) {
        fb.set_pixel(x + w, y + i, color);
        fb.set_pixel(x + w, y + h / 2 + i, color);
    }
}

fn draw_letter_c(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 3 * s;
    for i in 0..h {
        fb.set_pixel(x, y + i, color);
        if i == 0 || i == h - 1 {
            for j in 0..w {
                fb.set_pixel(x + j, y + i, color);
            }
        }
    }
}

fn draw_letter_d(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    for i in 0..h {
        fb.set_pixel(x, y + i, color);
    }
    for i in 0..w {
        fb.set_pixel(x + i, y, color);
        fb.set_pixel(x + i, y + h - 1, color);
    }
    for i in 0..h {
        fb.set_pixel(x + w, y + i, color);
    }
}

fn draw_letter_e(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    for i in 0..h {
        fb.set_pixel(x, y + i, color);
    }
    for i in 0..w {
        fb.set_pixel(x + i, y, color);
        fb.set_pixel(x + i, y + h / 2, color);
        fb.set_pixel(x + i, y + h - 1, color);
    }
}

fn draw_letter_l(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 3 * s;
    for i in 0..h {
        fb.set_pixel(x, y + i, color);
    }
    for i in 0..w {
        fb.set_pixel(x + i, y + h - 1, color);
    }
}

fn draw_letter_o(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    for i in 0..h {
        fb.set_pixel(x, y + i, color);
        fb.set_pixel(x + w, y + i, color);
    }
    for i in 0..w {
        fb.set_pixel(x + i, y, color);
        fb.set_pixel(x + i, y + h - 1, color);
    }
}

fn draw_letter_s(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    let mid = h / 2;

    // Top horizontal
    for i in 0..w {
        fb.set_pixel(x + i, y, color);
    }

    // Top-left vertical
    for i in 0..(mid + 1) {
        fb.set_pixel(x, y + i, color);
    }

    // Middle horizontal
    for i in 0..w {
        fb.set_pixel(x + i, y + mid, color);
    }

    // Bottom-right vertical
    for i in mid..h {
        fb.set_pixel(x + w, y + i, color);
    }

    // Bottom horizontal
    for i in 0..w {
        fb.set_pixel(x + i, y + h - 1, color);
    }
}

fn draw_letter_y(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let h = 5 * s;
    let w = 4 * s;
    let mid = h / 2;

    // Top-left to middle
    for i in 0..=mid {
        fb.set_pixel(x + (i / 2), y + i, color);
    }

    // Top-right to middle
    for i in 0..=mid {
        fb.set_pixel(x + w - (i / 2), y + i, color);
    }

    // Middle to bottom
    for i in mid..h {
        fb.set_pixel(x + w / 2, y + i, color);
    }
}

fn draw_degree(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    let r = s;
    fb.circle(x + 2 * s, y, r, color, false);
}

fn draw_generic_char(fb: &mut Framebuffer, x: u32, y: u32, color: Color, s: u32) {
    // Draw a simple box for unknown characters
    let h = 5 * s;
    let w = 4 * s;
    fb.stroke_rect(x, y, w, h, color, s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_splash() {
        let mut fb = Framebuffer::new(800, 600);
        render_splash_to_framebuffer(&mut fb, 45.0);
        // Just verify it doesn't panic
        assert_eq!(fb.width(), 800);
    }
}
