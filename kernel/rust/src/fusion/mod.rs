//! Fusion Display Manager Module
//!
//! Provides the core display management system for Alloy OS, integrating with the graphics module
//! to offer a unified display abstraction layer.
//!
//! The Fusion module is organized as follows:
//! - `manager`: DisplayManager struct for managing display state and operations
//! - `surface`: Surface abstraction for drawable surfaces
//! - `compositor`: Composition engine for rendering multiple surfaces
//! - `terminal`: Terminal Surface wrapper for rendering Terminal in Fusion
//! - `ui`: UI primitives and widgets (buttons, text, etc.)
//! - `apps`: Application framework for launchable Fusion applications

pub mod manager;
pub mod surface;
pub mod compositor;
pub mod backend;
pub mod terminal;
pub mod ui;
pub mod apps;

// Re-export main types for convenient access
pub use manager::{DisplayManager, RenderCommand, ManagerError, Rect};
pub use surface::{Surface, SurfaceError};
pub use compositor::{Compositor, CompositorError, DisplayLike, DisplayAdapter};
pub use backend::{FusionBackendError, FusionDisplayBackend};
pub use terminal::{TerminalSurface, TerminalError};
pub use apps::{AppError, ApplicationLifecycle, TerminalApp};
