#![cfg_attr(not(feature = "host"), no_std)]

extern crate alloc;

// Cosmos DE Integration Layer
//
// This module provides the bridge between Alloy's kernel display server and
// COSMIC-aligned desktop environment components. It handles session bootstrap,
// runtime selection, and incremental integration of upstream COSMIC components
// via compatibility adapters.
//
// ## Architecture
//
// The Cosmos DE layer operates in two modes:
// - **Cosmos mode**: In-kernel desktop shell with Wayland compositor bridge
// - **IcedPrimary mode**: Fallback software-rendered UI for compatibility
//
// Session boundaries are enforced: the kernel manages surface lifecycle and
// frame composition, while userland sessions (including COSMIC components)
// handle shell lifecycle, app orchestration, and input policy.

pub use alloy_os_display::apps::{AppKind, AppSurfaceBinding, ApplicationLifecycle, LaunchOptions};
pub use alloy_os_display::protocol::{
    ClientId, DisplayEvent, DisplayRequest, DisplayResponse, SessionBoundary, SessionConfig,
    SessionId, SessionType, SurfaceId, CAPABILITY_SESSION, CAPABILITY_WAYLAND,
};
pub use alloy_os_display::server::{DisplayBackend, DisplayServer, ServerError, ServerState};

const MIN_FRAME_INTERVAL_MS: u32 = 8;
const MAX_FRAME_INTERVAL_MS: u32 = 33;
const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;

/// Runtime mode selection for the desktop environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CosmosRuntime {
    /// Full Cosmos DE with Wayland compositor and userland session
    Cosmos,
    /// Iced-primary fallback (software-rendered, fully in-kernel)
    IcedPrimary,
}

/// Bootstrap status indicating integration readiness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CosmosBootstrapStatus {
    /// COSMIC integration ready, session boundary established
    Ready,
    /// Degraded mode - some components unavailable
    Degraded,
    /// Fallback to IcedPrimary required
    FallbackRequired,
}

/// Boot profile defining the primary and fallback runtimes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosBootProfile {
    pub primary_runtime: CosmosRuntime,
    pub fallback_runtime: CosmosRuntime,
    pub preferred_frame_interval_ms: u32,
    /// Enable Wayland compositor integration
    pub enable_wayland: bool,
    /// Session boundary configuration for userland desktop
    pub session_config: Option<SessionConfig>,
}

impl CosmosBootProfile {
    pub fn cosmos_default() -> Self {
        Self {
            primary_runtime: CosmosRuntime::Cosmos,
            fallback_runtime: CosmosRuntime::IcedPrimary,
            preferred_frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
            enable_wayland: true,
            session_config: None,
        }
    }

    /// Create a boot profile with userland session support
    pub fn with_session(
        primary: CosmosRuntime,
        session_id: u32,
        capabilities: u32,
    ) -> Self {
        Self {
            primary_runtime: primary,
            fallback_runtime: CosmosRuntime::IcedPrimary,
            preferred_frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
            enable_wayland: true,
            session_config: Some(SessionConfig {
                session_id: SessionId(session_id),
                session_type: SessionType::Userland,
                capabilities,
                receives_input: true,
                manages_windows: true,
                name: alloc::string::String::from("cosmic-desktop"),
            }),
        }
    }
}

/// Bootstrap report with session boundary information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosBootstrapReport {
    pub profile: CosmosBootProfile,
    pub status: CosmosBootstrapStatus,
    /// The session boundary negotiated during bootstrap
    pub boundary: SessionBoundary,
}

impl CosmosBootstrapReport {
    pub fn summary_serial_line(&self) -> &'static [u8] {
        match self.status {
            CosmosBootstrapStatus::Ready => {
                b"[CosmosDE] Integration profile ready - session boundary established\n\0"
            }
            CosmosBootstrapStatus::Degraded => {
                b"[CosmosDE] Integration profile degraded - forcing Iced-primary boot\n\0"
            }
            CosmosBootstrapStatus::FallbackRequired => {
                b"[CosmosDE] Fallback required - Cosmos mode unavailable\n\0"
            }
        }
    }
}

/// The Cosmos DE integration boundary - manages the transition between
/// kernel display primitives and userland desktop session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosDe {
    profile: CosmosBootProfile,
}

impl CosmosDe {
    pub fn new(profile: CosmosBootProfile) -> Self {
        Self { profile }
    }

    /// Perform bootstrap validation and establish session boundary
    pub fn bootstrap_report(self) -> CosmosBootstrapReport {
        let frame_interval = self.profile.preferred_frame_interval_ms;

        if frame_interval < MIN_FRAME_INTERVAL_MS || frame_interval > MAX_FRAME_INTERVAL_MS {
            CosmosBootstrapReport {
                profile: CosmosBootProfile {
                    primary_runtime: CosmosRuntime::IcedPrimary,
                    fallback_runtime: CosmosRuntime::IcedPrimary,
                    preferred_frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
                    enable_wayland: false,
                    session_config: None,
                },
                status: CosmosBootstrapStatus::Degraded,
                boundary: SessionBoundary {
                    input_owner: SessionId::KERNEL,
                    shell_owner: SessionId::KERNEL,
                    compositor_owner: SessionId::KERNEL,
                },
            }
        } else if self.profile.enable_wayland {
            let boundary = if let Some(ref session_config) = self.profile.session_config {
                SessionBoundary {
                    input_owner: session_config.session_id,
                    shell_owner: session_config.session_id,
                    compositor_owner: SessionId::KERNEL,
                }
            } else {
                SessionBoundary {
                    input_owner: SessionId::KERNEL,
                    shell_owner: SessionId::KERNEL,
                    compositor_owner: SessionId::KERNEL,
                }
            };

            CosmosBootstrapReport {
                profile: self.profile,
                status: CosmosBootstrapStatus::Ready,
                boundary,
            }
        } else {
            CosmosBootstrapReport {
                profile: self.profile,
                status: CosmosBootstrapStatus::Ready,
                boundary: SessionBoundary {
                    input_owner: SessionId::KERNEL,
                    shell_owner: SessionId::KERNEL,
                    compositor_owner: SessionId::KERNEL,
                },
            }
        }
    }
}

/// Userland session bootstrap information
#[derive(Debug, Clone)]
pub struct UserlandSessionBootstrap {
    /// Display server communication endpoint
    pub display_endpoint: &'static str,
    /// Session identifier
    pub session_id: SessionId,
    /// Session capabilities
    pub capabilities: u32,
    /// Initial session boundary
    pub boundary: SessionBoundary,
    /// Framebuffer dimensions
    pub display_width: u32,
    pub display_height: u32,
    /// Frame interval in milliseconds
    pub frame_interval_ms: u32,
}

impl UserlandSessionBootstrap {
    /// Create bootstrap info for a new userland session
    pub fn new(
        session_id: SessionId,
        boundary: SessionBoundary,
        display_width: u32,
        display_height: u32,
    ) -> Self {
        Self {
            display_endpoint: "/run/user/1000/wayland-0",
            session_id,
            capabilities: CAPABILITY_SESSION | CAPABILITY_WAYLAND,
            boundary,
            display_width,
            display_height,
            frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
        }
    }
}

/// Initialize the COSMOS DE integration with userland session support.
pub fn bootstrap() -> CosmosBootstrapReport {
    CosmosDe::new(CosmosBootProfile::cosmos_default()).bootstrap_report()
}

/// Initialize a userland session bootstrap for COSMIC desktop components.
pub fn userland_bootstrap(
    session_id: u32,
    display_width: u32,
    display_height: u32,
) -> UserlandSessionBootstrap {
    let profile = CosmosBootProfile::with_session(
        CosmosRuntime::Cosmos,
        session_id,
        CAPABILITY_SESSION | CAPABILITY_WAYLAND,
    );
    let report = CosmosDe::new(profile).bootstrap_report();

    UserlandSessionBootstrap::new(
        SessionId(session_id),
        report.boundary,
        display_width,
        display_height,
    )
}

/// Create a userland session bootstrap from an existing bootstrap report.
pub fn session_bootstrap_from_report(
    report: &CosmosBootstrapReport,
    display_width: u32,
    display_height: u32,
) -> UserlandSessionBootstrap {
    UserlandSessionBootstrap {
        display_endpoint: "/run/user/1000/wayland-0",
        session_id: report.boundary.shell_owner,
        capabilities: CAPABILITY_SESSION | CAPABILITY_WAYLAND,
        boundary: report.boundary,
        display_width,
        display_height,
        frame_interval_ms: report.profile.preferred_frame_interval_ms,
    }
}

// ============================================================================
// COSMIC Component Adapters (Compatibility Layer)
// ============================================================================

/// Adapter trait for wrapping COSMIC components to work with Alloy's display system.
pub trait CosmicComponentAdapter {
    /// Initialize the component with the display server
    fn init(&mut self, display: &mut DisplayServer<impl DisplayBackend>) -> Result<(), &'static str>;

    /// Notify the component of a display event
    fn handle_event(&mut self, event: DisplayEvent) -> bool;

    /// Render the component's current state
    fn render(&mut self) -> bool;

    /// Get the component's name for debugging
    fn name(&self) -> &'static str;
}

/// Shell component adapter - bridges between COSMIC shell protocol and
/// Alloy's window management system.
pub struct ShellAdapter {
    initialized: bool,
    active_surface: Option<SurfaceId>,
}

impl ShellAdapter {
    pub fn new() -> Self {
        Self {
            initialized: false,
            active_surface: None,
        }
    }
}

impl CosmicComponentAdapter for ShellAdapter {
    fn init(&mut self, _display: &mut DisplayServer<impl DisplayBackend>) -> Result<(), &'static str> {
        self.initialized = true;
        Ok(())
    }

    fn handle_event(&mut self, event: DisplayEvent) -> bool {
        match event {
            DisplayEvent::FocusChanged { surface_id } => {
                self.active_surface = surface_id;
                true
            }
            DisplayEvent::SurfaceCreated { .. } | DisplayEvent::SurfaceDestroyed { .. } => true,
            _ => false,
        }
    }

    fn render(&mut self) -> bool {
        self.initialized
    }

    fn name(&self) -> &'static str {
        "COSMIC Shell Adapter"
    }
}

/// Window management adapter - bridges COSMIC tiling/stacking protocol
/// to Alloy's window manager.
pub struct WindowManagerAdapter {
    initialized: bool,
}

impl WindowManagerAdapter {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl CosmicComponentAdapter for WindowManagerAdapter {
    fn init(&mut self, _display: &mut DisplayServer<impl DisplayBackend>) -> Result<(), &'static str> {
        self.initialized = true;
        Ok(())
    }

    fn handle_event(&mut self, event: DisplayEvent) -> bool {
        match event {
            DisplayEvent::SurfaceCreated { .. }
            | DisplayEvent::SurfaceDestroyed { .. }
            | DisplayEvent::FocusChanged { .. } => true,
            _ => false,
        }
    }

    fn render(&mut self) -> bool {
        self.initialized
    }

    fn name(&self) -> &'static str {
        "COSMIC Window Manager Adapter"
    }
}

/// Launcher adapter - bridges COSMIC application launching protocol
/// to Alloy's app runtime system.
pub struct LauncherAdapter {
    initialized: bool,
}

impl LauncherAdapter {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl CosmicComponentAdapter for LauncherAdapter {
    fn init(&mut self, _display: &mut DisplayServer<impl DisplayBackend>) -> Result<(), &'static str> {
        self.initialized = true;
        Ok(())
    }

    fn handle_event(&mut self, _event: DisplayEvent) -> bool {
        false
    }

    fn render(&mut self) -> bool {
        self.initialized
    }

    fn name(&self) -> &'static str {
        "COSMIC Launcher Adapter"
    }
}