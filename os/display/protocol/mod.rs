use alloc::string::String;
use core::fmt;

/// Protocol version for compatibility negotiation.
pub const PROTOCOL_VERSION_MAJOR: u16 = 0;
pub const PROTOCOL_VERSION_MINOR: u16 = 1;

/// Minimum frame interval accepted by the server.
pub const MIN_FRAME_INTERVAL_MS: u32 = 1;
/// Maximum frame interval accepted by the server.
pub const MAX_FRAME_INTERVAL_MS: u32 = 1000;
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
}

/// Response values for successful request execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayResponse {
    Ack,
    SurfaceCreated {
        surface_id: SurfaceId,
    },
    Error(String),
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
}

/// Validation errors for malformed protocol requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    InvalidDimensions,
    InvalidFrameInterval,
    EmptyDamageRect,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidDimensions => write!(f, "invalid dimensions"),
            ProtocolError::InvalidFrameInterval => write!(f, "invalid frame interval"),
            ProtocolError::EmptyDamageRect => write!(f, "damage rect must be non-empty"),
        }
    }
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
        _ => {}
    }

    Ok(())
}
