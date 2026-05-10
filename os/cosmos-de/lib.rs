#![cfg_attr(not(feature = "host"), no_std)]

pub use alloy_os_display::apps::{AppKind, AppSurfaceBinding, ApplicationLifecycle, LaunchOptions};
pub use alloy_os_display::protocol::{ClientId, DisplayEvent, DisplayRequest, DisplayResponse, SurfaceId};
pub use alloy_os_display::server::{DisplayBackend, DisplayServer, ServerError, ServerState};

/// Dependency boundary for the Cosmos DE integration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CosmosDe;

impl CosmosDe {
    pub const fn new() -> Self {
        Self
    }
}

pub fn bootstrap() {
    let _ = CosmosDe::new();
}
