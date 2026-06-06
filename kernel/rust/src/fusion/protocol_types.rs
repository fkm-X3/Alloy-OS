use alloc::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientId(pub u32);

impl ClientId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SurfaceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Argb8888,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum DisplayEvent {
    FocusChanged {
        surface_id: Option<SurfaceId>,
    },
    KeyInput {
        key: u8,
        pressed: bool,
    },
    PointerMotion {
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    },
    MouseButton {
        button: MouseButton,
        pressed: bool,
        x: i32,
        y: i32,
    },
    MouseWheel {
        delta: i32,
        x: i32,
        y: i32,
    },
    SurfaceCreated {
        surface_id: SurfaceId,
    },
    SurfaceDestroyed {
        surface_id: SurfaceId,
    },
    FramePresented {
        timestamp_ms: u64,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ServerDiagnostics {
    pub dropped_events: u32,
    pub backend_errors: u32,
}

pub const MAX_EVENTS: usize = 256;

pub struct EventQueue {
    events: VecDeque<DisplayEvent>,
    diagnostics: ServerDiagnostics,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            diagnostics: ServerDiagnostics {
                dropped_events: 0,
                backend_errors: 0,
            },
        }
    }

    pub fn push(&mut self, event: DisplayEvent) {
        if self.events.len() >= MAX_EVENTS {
            self.diagnostics.dropped_events += 1;
            return;
        }
        self.events.push_back(event);
    }

    pub fn pop(&mut self) -> Option<DisplayEvent> {
        self.events.pop_front()
    }

    pub fn diagnostics(&self) -> ServerDiagnostics {
        self.diagnostics
    }

    pub fn record_backend_error(&mut self) {
        self.diagnostics.backend_errors += 1;
    }
}
