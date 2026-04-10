//! UI Primitives - Basic UI widgets and components
//!
//! Provides fundamental UI building blocks (buttons, text, panels, etc.)
//! for constructing user interface elements.
//!
//! # Modules
//!
//! - `rect` - Rectangle drawing primitives (filled, outline, rounded)
//! - `text` - Text rendering and drawing utilities
//! - `primitives` - High-level UI components (Button, Panel, Label, TextBox)

pub mod rect;
pub mod text;
pub mod primitives;

pub use rect::{
    draw_filled_rect, draw_gradient_rect_horizontal, draw_gradient_rect_vertical,
    draw_outline_rect, draw_rounded_rect, RectError,
};

pub use text::{
    draw_text, draw_text_centered, draw_text_with_bg, measure_text, TextError,
    char_width, char_height, line_height,
};

pub use primitives::{
    Button, Panel, Label, TextBox, PrimitiveError,
};
