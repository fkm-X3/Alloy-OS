#![cfg_attr(not(feature = "host"), no_std)]

pub use alloy_os_display::apps::{AppKind, AppSurfaceBinding, ApplicationLifecycle, LaunchOptions};
pub use alloy_os_display::protocol::{ClientId, DisplayEvent, DisplayRequest, DisplayResponse, SurfaceId};
pub use alloy_os_display::server::{DisplayBackend, DisplayServer, ServerError, ServerState};

const MIN_FRAME_INTERVAL_MS: u32 = 8;
const MAX_FRAME_INTERVAL_MS: u32 = 33;
const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CosmosRuntime {
    Cosmos,
    IcedPrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CosmosBootstrapStatus {
    Ready,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CosmosBootProfile {
    pub primary_runtime: CosmosRuntime,
    pub fallback_runtime: CosmosRuntime,
    pub preferred_frame_interval_ms: u32,
}

impl CosmosBootProfile {
    pub const fn cosmos_default() -> Self {
        Self {
            primary_runtime: CosmosRuntime::Cosmos,
            fallback_runtime: CosmosRuntime::IcedPrimary,
            preferred_frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CosmosBootstrapReport {
    pub profile: CosmosBootProfile,
    pub status: CosmosBootstrapStatus,
}

impl CosmosBootstrapReport {
    pub const fn summary_serial_line(self) -> &'static [u8] {
        match self.status {
            CosmosBootstrapStatus::Ready => {
                b"[CosmosDE] Integration profile ready (Cosmos primary, Iced fallback)\n\0"
            }
            CosmosBootstrapStatus::Degraded => {
                b"[CosmosDE] Integration profile degraded; forcing Iced-primary boot\n\0"
            }
        }
    }
}

/// Dependency boundary for the Cosmos DE integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CosmosDe {
    profile: CosmosBootProfile,
}

impl CosmosDe {
    pub const fn new(profile: CosmosBootProfile) -> Self {
        Self { profile }
    }

    pub const fn bootstrap_report(self) -> CosmosBootstrapReport {
        let frame_interval = self.profile.preferred_frame_interval_ms;
        if frame_interval < MIN_FRAME_INTERVAL_MS || frame_interval > MAX_FRAME_INTERVAL_MS {
            CosmosBootstrapReport {
                profile: CosmosBootProfile {
                    primary_runtime: CosmosRuntime::IcedPrimary,
                    fallback_runtime: CosmosRuntime::IcedPrimary,
                    preferred_frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
                },
                status: CosmosBootstrapStatus::Degraded,
            }
        } else {
            CosmosBootstrapReport {
                profile: self.profile,
                status: CosmosBootstrapStatus::Ready,
            }
        }
    }
}

pub fn bootstrap() -> CosmosBootstrapReport {
    CosmosDe::new(CosmosBootProfile::cosmos_default()).bootstrap_report()
}
