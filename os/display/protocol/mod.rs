use alloc::string::String;
use core::fmt;

/// Protocol version for compatibility negotiation.
pub const PROTOCOL_VERSION_MAJOR: u16 = 0;
pub const PROTOCOL_VERSION_MINOR: u16 = 2;

/// Minimum frame interval accepted by the server.
pub const MIN_FRAME_INTERVAL_MS: u32 = 1;
/// Maximum frame interval accepted by the server.
pub const MAX_FRAME_INTERVAL_MS: u32 = 10000;
/// Maximum supported width/height for a single surface.
pub const MAX_SURFACE_DIMENSION: u32 = 8192;

/// Logical display client identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientId(pub u32);

impl ClientId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Surface identifier assigned by the display server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SurfaceId(pub u32);

impl SurfaceId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Supported pixel formats for surface buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Argb8888,
}

/// Damage rectangle in surface-local coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Requests sent by clients to the display server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayRequest {
    CreateSurface {
        width: u32,
        height: u32,
        format: PixelFormat,
    },
    DestroySurface {
        surface_id: SurfaceId,
    },
    SetSurfacePosition {
        surface_id: SurfaceId,
        x: i32,
        y: i32,
    },
    ResizeSurface {
        surface_id: SurfaceId,
        width: u32,
        height: u32,
    },
    SetSurfaceVisibility {
        surface_id: SurfaceId,
        visible: bool,
    },
    SetSurfaceZOrder {
        surface_id: SurfaceId,
        z_order: u32,
    },
    CommitSurface {
        surface_id: SurfaceId,
        damage: Option<Rect>,
    },
    RequestFocus {
        surface_id: Option<SurfaceId>,
    },
    SetFrameIntervalMs {
        interval_ms: u32,
    },
    /// New: clients announce compositor integration (Wayland/Cosmos)
    AnnounceCompositor {
        name: String,
        version_major: u16,
        version_minor: u16,
    },
    /// New: client sets capability flags indicating integrations supported
    SetClientCapabilities {
        capabilities: u32,
    },
    /// New: userland session announces itself and requests session boundary negotiation
    AnnounceSession {
        session_id: u32,
        session_type: u32,
        capabilities: u32,
    },
    /// New: request to transfer shell ownership to a session
    TransferShell {
        session_id: u32,
    },
    /// New: request to transfer input routing ownership
    TransferInput {
        session_id: u32,
    },
}

/// Response values for successful request execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayResponse {
    Ack,
    SurfaceCreated {
        surface_id: SurfaceId,
    },
    /// Acknowledgement returning the accepted capabilities
    CapabilitiesAck {
        capabilities: u32,
    },
    /// A compositor announce acknowledgement
    CompositorAnnounced {
        name: String,
    },
    /// Session announcement acknowledged with assigned boundary
    SessionAcknowledged {
        session_id: u32,
        boundary: SessionBoundary,
    },
    /// Shell transfer result
    ShellTransferResult {
        success: bool,
        previous_owner: u32,
    },
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Events emitted by the display server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayEvent {
    FocusChanged {
        surface_id: Option<SurfaceId>,
    },
    KeyInput {
        surface_id: Option<SurfaceId>,
        key: u8,
        pressed: bool,
    },
    PointerMotion {
        surface_id: Option<SurfaceId>,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    },
    MouseButton {
        surface_id: Option<SurfaceId>,
        button: MouseButton,
        pressed: bool,
        x: i32,
        y: i32,
    },
    MouseWheel {
        surface_id: Option<SurfaceId>,
        delta: i32,
        x: i32,
        y: i32,
    },
    SurfaceCreated {
        surface_id: SurfaceId,
        owner: ClientId,
    },
    SurfaceDestroyed {
        surface_id: SurfaceId,
    },
    FramePresented {
        frame_id: u64,
    },
    /// New: session boundary change notification
    SessionBoundaryChanged {
        session_id: u32,
        boundary: SessionBoundary,
    },
}

/// Validation errors for malformed protocol requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    InvalidDimensions,
    InvalidFrameInterval,
    EmptyDamageRect,
    InvalidCapability,
    UnsupportedCompositorName,
    InvalidSessionId,
    SessionDenied,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidDimensions => write!(f, "invalid dimensions"),
            ProtocolError::InvalidFrameInterval => write!(f, "invalid frame interval"),
            ProtocolError::EmptyDamageRect => write!(f, "damage rect must be non-empty"),
            ProtocolError::InvalidCapability => write!(f, "invalid capability flags"),
            ProtocolError::UnsupportedCompositorName => write!(f, "unsupported compositor name"),
            ProtocolError::InvalidSessionId => write!(f, "invalid session ID"),
            ProtocolError::SessionDenied => write!(f, "session request denied"),
        }
    }
}

/// Capability flags for clients (bitmask).
pub type CapabilityFlags = u32;
pub const CAPABILITY_NONE: CapabilityFlags = 0;
pub const CAPABILITY_WAYLAND: CapabilityFlags = 1 << 0;
pub const CAPABILITY_COSMOS: CapabilityFlags = 1 << 1;
pub const CAPABILITY_SESSION: CapabilityFlags = 1 << 2;
pub const CAPABILITY_INPUT: CapabilityFlags = 1 << 3;

/// Session identifier for kernel/userland boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub u32);

impl SessionId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
    /// The kernel-owned session (always 0)
    pub const KERNEL: SessionId = SessionId(0);
}

/// Session boundary marker - distinguishes kernel display primitives
/// from userland session/runtime responsibilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionBoundary {
    /// Which session owns input routing decisions
    pub input_owner: SessionId,
    /// Which session owns shell lifecycle (launcher, panel, window management policy)
    pub shell_owner: SessionId,
    /// Which session owns surface composition ordering
    pub compositor_owner: SessionId,
}

impl Default for SessionBoundary {
    fn default() -> Self {
        Self {
            input_owner: SessionId::KERNEL,
            shell_owner: SessionId::KERNEL,
            compositor_owner: SessionId::KERNEL,
        }
    }
}

/// Session type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    /// Kernel-integrated session (built-in shell/window manager)
    Kernel = 0,
    /// Userland session (external compositor, desktop shell)
    Userland = 1,
}

/// Display server session configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfig {
    /// Session identifier
    pub session_id: SessionId,
    /// Session type
    pub session_type: SessionType,
    /// Capabilities this session advertises
    pub capabilities: CapabilityFlags,
    /// Whether this session should receive input events
    pub receives_input: bool,
    /// Whether this session manages window lifecycle
    pub manages_windows: bool,
    /// Name for debugging/diagnostics
    pub name: alloc::string::String,
}

/// Validate request shape before stateful execution.
pub fn validate_request(request: &DisplayRequest) -> Result<(), ProtocolError> {
    match request {
        DisplayRequest::CreateSurface { width, height, .. }
        | DisplayRequest::ResizeSurface {
            width,
            height,
            ..
        } => {
            if *width == 0
                || *height == 0
                || *width > MAX_SURFACE_DIMENSION
                || *height > MAX_SURFACE_DIMENSION
            {
                return Err(ProtocolError::InvalidDimensions);
            }
        }
        DisplayRequest::CommitSurface {
            damage: Some(damage),
            ..
        } => {
            if damage.is_empty() {
                return Err(ProtocolError::EmptyDamageRect);
            }
        }
        DisplayRequest::SetFrameIntervalMs { interval_ms } => {
            if *interval_ms < MIN_FRAME_INTERVAL_MS || *interval_ms > MAX_FRAME_INTERVAL_MS {
                return Err(ProtocolError::InvalidFrameInterval);
            }
        }
        DisplayRequest::SetClientCapabilities { capabilities } => {
            let allowed = CAPABILITY_WAYLAND | CAPABILITY_COSMOS | CAPABILITY_SESSION | CAPABILITY_INPUT | CAPABILITY_NONE;
            if (*capabilities & !allowed) != 0 {
                return Err(ProtocolError::InvalidCapability);
            }
        }
        DisplayRequest::AnnounceCompositor { name, .. } => {
            if name.is_empty() || name.len() > 64 {
                return Err(ProtocolError::UnsupportedCompositorName);
            }
        }
        _ => {}
    }

    Ok(())
}