//! Fusion Display System Integration
//!
//! Provides a display backend that bridges kernel graphics and the display server.
//! Manages framebuffer-based rendering for kernel applications.

pub mod backend;
pub mod terminal;
pub mod framebuffer;

pub use backend::FusionDisplayBackend;
pub use terminal::TerminalSurface;
pub use framebuffer::{FramebufferRenderer, Color};

/// Fusion display system marker
#[derive(Debug, Clone, Copy)]
pub struct Fusion;

impl Fusion {
    /// Create a new Fusion display backend
    pub fn new_backend(display: crate::graphics::vesa::VesaDisplay) -> FusionDisplayBackend {
        FusionDisplayBackend::new(display)
    }

    /// Create a new framebuffer renderer
    pub fn new_renderer(width: u32, height: u32) -> Result<FramebufferRenderer, backend::FusionError> {
        FramebufferRenderer::new(width, height)
    }
}
