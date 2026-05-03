/// Theme system for Alloy OS Terminal

use iced::Color;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub bg: Color,
    pub bg_alt: Color,
    pub border: Color,
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text: Color,
    pub text_muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: Color::from_rgb(25.0 / 255.0, 25.0 / 255.0, 35.0 / 255.0),
            bg_alt: Color::from_rgb(35.0 / 255.0, 35.0 / 255.0, 50.0 / 255.0),
            border: Color::from_rgb(70.0 / 255.0, 70.0 / 255.0, 90.0 / 255.0),
            accent_primary: Color::from_rgb(0.0, 1.0, 1.0),
            accent_secondary: Color::from_rgb(0.0, 1.0, 0.0),
            success: Color::from_rgb(0.0, 1.0, 0.0),
            warning: Color::from_rgb(1.0, 1.0, 0.0),
            error: Color::from_rgb(1.0, 0.0, 0.0),
            text: Color::from_rgb(220.0 / 255.0, 220.0 / 255.0, 230.0 / 255.0),
            text_muted: Color::from_rgb(120.0 / 255.0, 120.0 / 255.0, 140.0 / 255.0),
        }
    }
}
