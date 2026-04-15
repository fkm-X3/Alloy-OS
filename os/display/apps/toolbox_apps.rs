use alloc::vec::Vec;
use core::fmt;

use crate::apps::desktop_shell::ShellApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolboxAppError {
    InvalidDimensions,
    InvalidBuffer,
}

impl fmt::Display for ToolboxAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolboxAppError::InvalidDimensions => write!(f, "invalid app dimensions"),
            ToolboxAppError::InvalidBuffer => write!(f, "invalid app buffer"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToolboxAppState {
    pub app: ShellApp,
    pub accent_step: u8,
}

impl ToolboxAppState {
    pub const fn new(app: ShellApp) -> Self {
        Self {
            app,
            accent_step: 0,
        }
    }

    pub fn handle_input(&mut self, key: u8) -> bool {
        match key {
            b' ' | b'\n' => {
                self.accent_step = self.accent_step.wrapping_add(1);
                true
            }
            _ => false,
        }
    }
}

pub fn render_toolbox_app(
    app: ShellApp,
    state: &ToolboxAppState,
    width: u32,
    height: u32,
) -> Result<Vec<u32>, ToolboxAppError> {
    if width == 0 || height == 0 {
        return Err(ToolboxAppError::InvalidDimensions);
    }

    let pixel_count = width
        .checked_mul(height)
        .ok_or(ToolboxAppError::InvalidDimensions)? as usize;

    let mut pixels = alloc::vec![base_color(app); pixel_count];
    let header_height = 24u32.min(height);
    fill_rect(
        &mut pixels,
        width,
        0,
        0,
        width,
        header_height,
        header_color(app, state.accent_step),
    );

    let inset = 10u32.min(width / 4).min(height / 4);
    let body_h = height.saturating_sub(header_height + inset);
    fill_rect(
        &mut pixels,
        width,
        inset,
        header_height.saturating_add(inset / 2),
        width.saturating_sub(inset.saturating_mul(2)).max(1),
        body_h.max(1),
        0xFF171A20,
    );

    match app {
        ShellApp::Settings => {
            draw_cross(&mut pixels, width, width / 2, height / 2, 14, 0xFF95B8FF);
        }
        ShellApp::FileExplorer => {
            fill_rect(
                &mut pixels,
                width,
                inset.saturating_add(8),
                header_height.saturating_add(inset),
                width.saturating_sub(inset.saturating_mul(2)).saturating_sub(16).max(1),
                10.min(height.saturating_sub(header_height.saturating_add(inset))),
                0xFFE0B46D,
            );
            fill_rect(
                &mut pixels,
                width,
                inset.saturating_add(8),
                header_height.saturating_add(inset).saturating_add(14),
                width.saturating_sub(inset.saturating_mul(2)).saturating_sub(16).max(1),
                body_h.saturating_sub(20).max(1),
                0xFF314156,
            );
        }
        ShellApp::TextEditor => {
            let left = inset.saturating_add(12);
            let mut y = header_height.saturating_add(inset).saturating_add(8);
            for _ in 0..4 {
                fill_rect(
                    &mut pixels,
                    width,
                    left,
                    y,
                    width
                        .saturating_sub(left)
                        .saturating_sub(inset.saturating_add(12))
                        .max(1),
                    2,
                    0xFFBFC7D5,
                );
                y = y.saturating_add(10);
            }
        }
        ShellApp::Calculator => {
            let key_w = (width.saturating_sub(inset.saturating_mul(2)).saturating_sub(12) / 4).max(8);
            let key_h = (height.saturating_sub(header_height).saturating_sub(inset).saturating_sub(12) / 3).max(8);
            for row in 0..3u32 {
                for col in 0..4u32 {
                    let x = inset + 4 + col.saturating_mul(key_w + 2);
                    let y = header_height + inset + 4 + row.saturating_mul(key_h + 2);
                    fill_rect(&mut pixels, width, x, y, key_w, key_h, 0xFF364458);
                }
            }
        }
        ShellApp::Terminal => return Err(ToolboxAppError::InvalidBuffer),
    }

    Ok(pixels)
}

fn base_color(app: ShellApp) -> u32 {
    match app {
        ShellApp::Settings => 0xFF1B1F2A,
        ShellApp::FileExplorer => 0xFF1A212C,
        ShellApp::TextEditor => 0xFF1E1E24,
        ShellApp::Calculator => 0xFF1B1D22,
        ShellApp::Terminal => 0xFF111111,
    }
}

fn header_color(app: ShellApp, accent_step: u8) -> u32 {
    let tweak = (accent_step as u32) & 0x0F;
    match app {
        ShellApp::Settings => 0xFF4C6FA7 + tweak,
        ShellApp::FileExplorer => 0xFF8C6A3B + tweak,
        ShellApp::TextEditor => 0xFF5C5F79 + tweak,
        ShellApp::Calculator => 0xFF4A7055 + tweak,
        ShellApp::Terminal => 0xFF333333,
    }
}

fn draw_cross(pixels: &mut [u32], stride: u32, cx: u32, cy: u32, arm: u32, color: u32) {
    fill_rect(
        pixels,
        stride,
        cx.saturating_sub(1),
        cy.saturating_sub(arm),
        3,
        arm.saturating_mul(2).saturating_add(1),
        color,
    );
    fill_rect(
        pixels,
        stride,
        cx.saturating_sub(arm),
        cy.saturating_sub(1),
        arm.saturating_mul(2).saturating_add(1),
        3,
        color,
    );
}

fn fill_rect(
    pixels: &mut [u32],
    stride: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
) {
    if stride == 0 || width == 0 || height == 0 {
        return;
    }
    let max_height = (pixels.len() as u32) / stride;
    let end_x = x.saturating_add(width).min(stride);
    let end_y = y.saturating_add(height).min(max_height);
    for row in y..end_y {
        let row_offset = (row * stride) as usize;
        for col in x..end_x {
            pixels[row_offset + col as usize] = color;
        }
    }
}
