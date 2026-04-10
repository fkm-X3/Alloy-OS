//! Application Framework for Fusion
//!
//! Provides a framework for creating and managing launchable applications within Fusion.
//! Applications can manage their own surfaces, handle input, and integrate with the
//! Compositor for rendering.
//!
//! # Architecture
//!
//! Each application:
//! - Implements the ApplicationLifecycle trait
//! - Manages a primary surface for rendering
//! - Handles keyboard input and updates
//! - Can be launched/closed dynamically
//! - Supports multiple instances

pub mod terminal;

pub use terminal::TerminalApp;

use core::fmt;

/// Error types for application operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppError {
    /// Application is not initialized
    NotInitialized,
    /// Compositor operation failed
    CompositorError,
    /// Display operation failed
    DisplayError,
    /// Surface creation failed
    SurfaceError,
    /// Invalid window dimensions
    InvalidDimensions,
    /// Application is already running
    AlreadyRunning,
    /// Application is not running
    NotRunning,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotInitialized => write!(f, "Application not initialized"),
            AppError::CompositorError => write!(f, "Compositor operation failed"),
            AppError::DisplayError => write!(f, "Display operation failed"),
            AppError::SurfaceError => write!(f, "Surface operation failed"),
            AppError::InvalidDimensions => write!(f, "Invalid window dimensions"),
            AppError::AlreadyRunning => write!(f, "Application already running"),
            AppError::NotRunning => write!(f, "Application not running"),
        }
    }
}

/// Application lifecycle trait for Fusion apps
pub trait ApplicationLifecycle {
    /// Launch the application in the compositor
    /// Returns the surface ID if successful
    fn launch(&mut self) -> Result<u32, AppError>;

    /// Close the application
    fn close(&mut self) -> Result<(), AppError>;

    /// Handle keyboard input
    fn handle_input(&mut self, key: u32) -> Result<(), AppError>;

    /// Update application state
    fn update(&mut self) -> Result<(), AppError>;

    /// Check if application is currently active/running
    fn is_active(&self) -> bool;

    /// Get the application name
    fn name(&self) -> &str;
}
