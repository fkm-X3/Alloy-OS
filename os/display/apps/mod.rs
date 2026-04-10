use crate::protocol::SurfaceId;

/// Known system app categories for early boot integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Terminal,
}

/// Placement hints used when creating app surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchOptions {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_order: u32,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            z_order: 0,
        }
    }
}

/// Maps an app instance to its primary server surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppSurfaceBinding {
    pub kind: AppKind,
    pub surface_id: SurfaceId,
}

/// Lifecycle contract for server-hosted app integrations.
pub trait ApplicationLifecycle {
    fn kind(&self) -> AppKind;
    fn is_running(&self) -> bool;
    fn primary_surface(&self) -> Option<SurfaceId>;
}
