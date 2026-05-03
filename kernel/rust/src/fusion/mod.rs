//! Fusion Display System Integration
//!
//! Provides a display backend that bridges kernel graphics and the display server.
//! Manages framebuffer-based rendering for kernel applications.

pub mod backend;
pub mod terminal;

pub use backend::FusionDisplayBackend;
pub use terminal::TerminalSurface;

/// Fusion display system marker
#[derive(Debug, Clone, Copy)]
pub struct Fusion;

impl Fusion {
    /// Create a new Fusion display backend
    pub fn new_backend() -> FusionDisplayBackend {
        FusionDisplayBackend::new()
    }
}
