#![allow(dead_code)]

/// Theme system for Alloy OS Terminal
/// 
/// Defines a cohesive modern dark theme with accent colors and style helpers

use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    /// Primary background color
    pub bg: Color,
    /// Alternate background for contrast
    pub bg_alt: Color,
    /// Border and divider color
    pub border: Color,
    /// Primary accent color
    pub accent_primary: Color,
    /// Secondary accent color
    pub accent_secondary: Color,
    /// Success/positive color
    pub success: Color,
    /// Warning color
    pub warning: Color,
    /// Error color
    pub error: Color,
    /// Text color (normal)
    pub text: Color,
    /// Muted text color
    pub text_muted: Color,
}

impl Theme {
    /// Modern dark theme with cyan/green accents
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(25, 25, 35),           // Very dark gray-blue
            bg_alt: Color::Rgb(35, 35, 50),       // Slightly lighter
            border: Color::Rgb(70, 70, 90),       // Medium dark gray
            accent_primary: Color::Cyan,          // Bright cyan
            accent_secondary: Color::Green,       // Bright green
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::Rgb(220, 220, 230),      // Off-white
            text_muted: Color::Rgb(120, 120, 140), // Gray
        }
    }

    /// Get default style with theme background
    pub fn base_style(&self) -> Style {
        Style::default().fg(self.text).bg(self.bg)
    }

    /// Style for title/headers
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent_primary)
            .bg(self.bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border).bg(self.bg)
    }

    /// Style for active/focused elements
    pub fn active_style(&self) -> Style {
        Style::default()
            .fg(self.accent_primary)
            .bg(self.bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for inactive/unfocused elements
    pub fn inactive_style(&self) -> Style {
        Style::default()
            .fg(self.text_muted)
            .bg(self.bg)
    }

    /// Style for selected items
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.accent_primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for success messages
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .bg(self.bg)
    }

    /// Style for warning messages
    pub fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .bg(self.bg)
    }

    /// Style for error messages
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.error)
            .bg(self.bg)
    }

    /// Style for secondary accent elements
    pub fn secondary_style(&self) -> Style {
        Style::default()
            .fg(self.accent_secondary)
            .bg(self.bg)
    }

    /// Style for muted/hint text
    pub fn muted_style(&self) -> Style {
        Style::default()
            .fg(self.text_muted)
            .bg(self.bg)
    }

    /// Style for status bar background
    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .bg(self.bg_alt)
    }

    /// Style for highlighted/active status bar element
    pub fn status_active_style(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.accent_primary)
            .add_modifier(Modifier::BOLD)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
