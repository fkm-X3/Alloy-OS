//! Protocol-driven display server runtime with session boundary support.
//!
//! Manages surface lifecycle, input routing delegation, and session-based
//! boundary negotiation between kernel display primitives and userland
//! desktop session processes.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::fmt;

use crate::protocol::{
     ClientId, DisplayEvent, DisplayRequest, DisplayResponse, MouseButton, PixelFormat, ProtocolError, Rect,
     SessionBoundary, SessionConfig, SessionId, SessionType, SurfaceId, validate_request,
     CAPABILITY_SESSION, CAPABILITY_WAYLAND, CAPABILITY_COSMOS, CAPABILITY_INPUT,
 };

/// Set the session boundary from bootstrap configuration.
pub fn set_session_boundary<B: DisplayBackend>(
    server: &mut DisplayServer<B>,
    boundary: SessionBoundary,
) {
    server.session_boundary = boundary;
}

/// Default display update cadence (about 60fps).
pub const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;
const DEFAULT_EVENT_QUEUE_CAPACITY: usize = 256;

/// High-level server lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Stopped,
    Running,
}

/// Runtime errors from server operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerError {
    NotRunning,
    AlreadyRunning,
    InvalidRequest(ProtocolError),
    SurfaceNotFound,
    PermissionDenied,
    SurfaceIdExhausted,
    BackendError,
    SessionDenied,
    InvalidSession,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::NotRunning => write!(f, "display server not running"),
            ServerError::AlreadyRunning => write!(f, "display server already running"),
            ServerError::InvalidRequest(err) => write!(f, "invalid request: {}", err),
            ServerError::SurfaceNotFound => write!(f, "surface not found"),
            ServerError::PermissionDenied => write!(f, "permission denied"),
            ServerError::SurfaceIdExhausted => write!(f, "surface id space exhausted"),
            ServerError::BackendError => write!(f, "backend operation failed"),
            ServerError::SessionDenied => write!(f, "session request denied"),
            ServerError::InvalidSession => write!(f, "invalid session"),
        }
    }
}

/// Rendering backend abstraction.
///
/// Keeps protocol and server state independent of Fusion internals so the
/// backend can be swapped without touching request/event semantics.
pub trait DisplayBackend {
    type Error: fmt::Debug;

    fn create_surface(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        format: PixelFormat,
    ) -> Result<(), Self::Error>;

    fn destroy_surface(&mut self, surface_id: SurfaceId) -> Result<(), Self::Error>;

    fn set_surface_position(
        &mut self,
        surface_id: SurfaceId,
        x: i32,
        y: i32,
    ) -> Result<(), Self::Error>;

    fn resize_surface(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error>;

    fn set_surface_visibility(
        &mut self,
        surface_id: SurfaceId,
        visible: bool,
    ) -> Result<(), Self::Error>;

    fn set_surface_z_order(
        &mut self,
        surface_id: SurfaceId,
        z_order: u32,
    ) -> Result<(), Self::Error>;

    fn commit_surface(
        &mut self,
        surface_id: SurfaceId,
        damage: Option<Rect>,
    ) -> Result<(), Self::Error>;

    fn upload_surface_pixels(
        &mut self,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        pixels: &[u32],
        damage: Option<Rect>,
    ) -> Result<(), Self::Error>;

    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Surface metadata tracked by the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceEntry {
    pub id: SurfaceId,
    pub owner: ClientId,
    pub session_id: SessionId,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    pub z_order: u32,
    pub format: PixelFormat,
}

/// Per-session state tracking
#[derive(Debug, Clone)]
struct SessionState {
    config: SessionConfig,
    surfaces: Vec<SurfaceId>,
    focused_surface: Option<SurfaceId>,
}

/// Server-side observability counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerDiagnostics {
    pub requests_handled: u64,
    pub events_emitted: u64,
    pub dropped_events: u64,
    pub frames_presented: u64,
    pub backend_errors: u64,
    pub session_transfers: u64,
}

/// Protocol-driven display server runtime with session support.
pub struct DisplayServer<B: DisplayBackend> {
    backend: B,
    state: ServerState,
    next_surface_id: u32,
    surfaces: BTreeMap<SurfaceId, SurfaceEntry>,
    client_capabilities: BTreeMap<ClientId, u32>,
    client_sessions: BTreeMap<ClientId, SessionId>,
    sessions: BTreeMap<SessionId, SessionState>,
    session_boundary: SessionBoundary,
    next_session_id: u32,
    focused_surface: Option<SurfaceId>,
    events: VecDeque<DisplayEvent>,
    frame_interval_ms: u32,
    last_present_ms: u64,
    next_frame_id: u64,
    max_event_queue: usize,
    diagnostics: ServerDiagnostics,
}

impl<B: DisplayBackend> DisplayServer<B> {
    pub fn new(backend: B) -> Self {
        let mut sessions = BTreeMap::new();
        sessions.insert(
            SessionId::KERNEL,
            SessionState {
                config: SessionConfig {
                    session_id: SessionId::KERNEL,
                    session_type: SessionType::Kernel,
                    capabilities: 0,
                    receives_input: true,
                    manages_windows: true,
                    name: String::from("kernel"),
                },
                surfaces: Vec::new(),
                focused_surface: None,
            },
        );

        Self {
            backend,
            state: ServerState::Stopped,
            next_surface_id: 1,
            surfaces: BTreeMap::new(),
            client_capabilities: BTreeMap::new(),
            client_sessions: BTreeMap::new(),
            sessions,
            session_boundary: SessionBoundary::default(),
            next_session_id: 1,
            focused_surface: None,
            events: VecDeque::new(),
            frame_interval_ms: DEFAULT_FRAME_INTERVAL_MS,
            last_present_ms: 0,
            next_frame_id: 0,
            max_event_queue: DEFAULT_EVENT_QUEUE_CAPACITY,
            diagnostics: ServerDiagnostics::default(),
        }
    }

    pub fn start(&mut self) -> Result<(), ServerError> {
        if self.state == ServerState::Running {
            return Err(ServerError::AlreadyRunning);
        }
        self.state = ServerState::Running;
        self.events.clear();
        self.last_present_ms = 0;
        self.next_frame_id = 0;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), ServerError> {
        if self.state == ServerState::Stopped {
            return Err(ServerError::NotRunning);
        }
        let mut ids = Vec::with_capacity(self.surfaces.len());
        for id in self.surfaces.keys() {
            ids.push(*id);
        }
        for id in ids {
            if self.backend.destroy_surface(id).is_err() {
                self.diagnostics.backend_errors = self.diagnostics.backend_errors.saturating_add(1);
            }
        }
        self.surfaces.clear();
        self.focused_surface = None;
        self.events.clear();
        self.state = ServerState::Stopped;
        Ok(())
    }

    pub fn state(&self) -> ServerState { self.state }
    pub fn surface_count(&self) -> usize { self.surfaces.len() }
    pub fn surface(&self, surface_id: SurfaceId) -> Option<&SurfaceEntry> { self.surfaces.get(&surface_id) }
    pub fn focused_surface(&self) -> Option<SurfaceId> { self.focused_surface }
    pub fn diagnostics(&self) -> ServerDiagnostics { self.diagnostics }
    pub fn frame_interval_ms(&self) -> u32 { self.frame_interval_ms }
    pub fn session_boundary(&self) -> SessionBoundary { self.session_boundary }
    pub fn input_owner(&self) -> SessionId { self.session_boundary.input_owner }
    pub fn shell_owner(&self) -> SessionId { self.session_boundary.shell_owner }
    pub fn compositor_owner(&self) -> SessionId { self.session_boundary.compositor_owner }

    /// Get mutable reference to the backend (used by tests and compositor integration)
    pub fn backend_mut(&mut self) -> &mut B { &mut self.backend }

    fn check_session_permission(&self, client_id: ClientId, surface_id: SurfaceId) -> Result<(), ServerError> {
        let surface = self.surfaces.get(&surface_id).ok_or(ServerError::SurfaceNotFound)?;
        let client_session = self.client_sessions.get(&client_id).copied().unwrap_or(SessionId::KERNEL);
        if surface.owner == client_id { return Ok(()); }
        if surface.session_id == client_session { return Ok(()); }
        if client_session == SessionId::KERNEL { return Ok(()); }
        if self.session_boundary.compositor_owner == client_session { return Ok(()); }
        Err(ServerError::PermissionDenied)
    }

    pub fn handle_request(
        &mut self,
        client_id: ClientId,
        request: DisplayRequest,
    ) -> Result<DisplayResponse, ServerError> {
        self.ensure_running()?;
        validate_request(&request).map_err(ServerError::InvalidRequest)?;
        self.diagnostics.requests_handled = self.diagnostics.requests_handled.saturating_add(1);

        match request {
            DisplayRequest::CreateSurface { width, height, format } => {
                let surface_id = self.allocate_surface_id().ok_or(ServerError::SurfaceIdExhausted)?;
                self.backend.create_surface(surface_id, width, height, format)
                    .map_err(|_| self.map_backend_error())?;
                let session_id = self.client_sessions.get(&client_id).copied().unwrap_or(SessionId::KERNEL);
                let entry = SurfaceEntry {
                    id: surface_id, owner: client_id, session_id, width, height,
                    x: 0, y: 0, visible: true, z_order: 0, format,
                };
                self.surfaces.insert(surface_id, entry);
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.surfaces.push(surface_id);
                }
                self.emit_event(DisplayEvent::SurfaceCreated { surface_id, owner: client_id });
                Ok(DisplayResponse::SurfaceCreated { surface_id })
            }
            DisplayRequest::DestroySurface { surface_id } => {
                self.ensure_owner(client_id, surface_id)?;
                self.backend.destroy_surface(surface_id).map_err(|_| self.map_backend_error())?;
                let entry = self.surfaces.remove(&surface_id);
                if let Some(entry) = entry {
                    if let Some(session) = self.sessions.get_mut(&entry.session_id) {
                        session.surfaces.retain(|&id| id != surface_id);
                    }
                    if self.focused_surface == Some(surface_id) {
                        self.focused_surface = None;
                        self.emit_event(DisplayEvent::FocusChanged { surface_id: None });
                    }
                    self.emit_event(DisplayEvent::SurfaceDestroyed { surface_id });
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetSurfacePosition { surface_id, x, y } => {
                self.check_session_permission(client_id, surface_id)?;
                self.backend.set_surface_position(surface_id, x, y).map_err(|_| self.map_backend_error())?;
                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.x = x; entry.y = y;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::ResizeSurface { surface_id, width, height } => {
                self.check_session_permission(client_id, surface_id)?;
                self.backend.resize_surface(surface_id, width, height).map_err(|_| self.map_backend_error())?;
                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.width = width; entry.height = height;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetSurfaceVisibility { surface_id, visible } => {
                self.check_session_permission(client_id, surface_id)?;
                self.backend.set_surface_visibility(surface_id, visible).map_err(|_| self.map_backend_error())?;
                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.visible = visible;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetSurfaceZOrder { surface_id, z_order } => {
                self.check_session_permission(client_id, surface_id)?;
                self.backend.set_surface_z_order(surface_id, z_order).map_err(|_| self.map_backend_error())?;
                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.z_order = z_order;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::CommitSurface { surface_id, damage } => {
                self.check_session_permission(client_id, surface_id)?;
                self.backend.commit_surface(surface_id, damage).map_err(|_| self.map_backend_error())?;
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::RequestFocus { surface_id } => {
                if let Some(target) = surface_id {
                    let client_session = self.client_sessions.get(&client_id).copied().unwrap_or(SessionId::KERNEL);
                    if client_session != self.session_boundary.input_owner && client_session != SessionId::KERNEL {
                        return Err(ServerError::PermissionDenied);
                    }
                    self.ensure_owner(client_id, target)?;
                }
                if self.focused_surface != surface_id {
                    self.focused_surface = surface_id;
                    if let Some(session) = self.sessions.get_mut(&SessionId::KERNEL) {
                        session.focused_surface = surface_id;
                    }
                    self.emit_event(DisplayEvent::FocusChanged { surface_id });
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetClientCapabilities { capabilities } => {
                self.client_capabilities.insert(client_id, capabilities);
                Ok(DisplayResponse::CapabilitiesAck { capabilities })
            }
            DisplayRequest::AnnounceCompositor { name, .. } => {
                Ok(DisplayResponse::CompositorAnnounced { name })
            }
            DisplayRequest::SetFrameIntervalMs { interval_ms } => {
                self.frame_interval_ms = interval_ms;
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::AnnounceSession { session_id, session_type, capabilities } => {
                self.handle_session_announcement(client_id, session_id, session_type, capabilities)
            }
            DisplayRequest::TransferShell { session_id } => {
                self.handle_shell_transfer(client_id, session_id)
            }
            DisplayRequest::TransferInput { session_id } => {
                self.handle_input_transfer(client_id, session_id)
            }
        }
    }

    fn handle_session_announcement(
        &mut self,
        client_id: ClientId,
        session_id: u32,
        session_type: u32,
        capabilities: u32,
    ) -> Result<DisplayResponse, ServerError> {
        let sid = SessionId(session_id);
        if self.sessions.contains_key(&sid) {
            return Err(ServerError::InvalidSession);
        }
        let stype = match session_type {
            0 => SessionType::Kernel,
            1 => SessionType::Userland,
            _ => return Err(ServerError::InvalidSession),
        };
        let allowed = CAPABILITY_SESSION | CAPABILITY_WAYLAND | CAPABILITY_COSMOS | CAPABILITY_INPUT;
        if (capabilities & !allowed) != 0 {
            return Err(ServerError::SessionDenied);
        }
        let config = SessionConfig {
            session_id: sid,
            session_type: stype,
            capabilities,
            receives_input: (capabilities & CAPABILITY_INPUT) != 0,
            manages_windows: (capabilities & CAPABILITY_SESSION) != 0,
            name: format!("userland-session-{}", session_id),
        };
        self.sessions.insert(sid, SessionState {
            config: config.clone(),
            surfaces: Vec::new(),
            focused_surface: None,
        });
        self.client_sessions.insert(client_id, sid);
        if config.receives_input { self.session_boundary.input_owner = sid; }
        if config.manages_windows { self.session_boundary.shell_owner = sid; }
        let boundary = self.session_boundary;
        Ok(DisplayResponse::SessionAcknowledged { session_id, boundary })
    }

    fn handle_shell_transfer(
        &mut self,
        _client_id: ClientId,
        session_id: u32,
    ) -> Result<DisplayResponse, ServerError> {
        let sid = SessionId(session_id);
        if !self.sessions.contains_key(&sid) {
            return Err(ServerError::InvalidSession);
        }
        let previous = self.session_boundary.shell_owner;
        self.session_boundary.shell_owner = sid;
        self.diagnostics.session_transfers = self.diagnostics.session_transfers.saturating_add(1);
        Ok(DisplayResponse::ShellTransferResult { success: true, previous_owner: previous.0 })
    }

    fn handle_input_transfer(
        &mut self,
        _client_id: ClientId,
        session_id: u32,
    ) -> Result<DisplayResponse, ServerError> {
        let sid = SessionId(session_id);
        if !self.sessions.contains_key(&sid) {
            return Err(ServerError::InvalidSession);
        }
        self.session_boundary.input_owner = sid;
        self.diagnostics.session_transfers = self.diagnostics.session_transfers.saturating_add(1);
        Ok(DisplayResponse::Ack)
    }

    pub fn upload_surface_pixels(
        &mut self,
        client_id: ClientId,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        pixels: &[u32],
        damage: Option<Rect>,
    ) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.check_session_permission(client_id, surface_id)?;
        let entry = self.surfaces.get(&surface_id).ok_or(ServerError::SurfaceNotFound)?;
        if entry.width != width || entry.height != height {
            return Err(ServerError::InvalidRequest(ProtocolError::InvalidDimensions));
        }
        self.backend.upload_surface_pixels(surface_id, width, height, pixels, damage).map_err(|_| self.map_backend_error())?;
        self.backend.commit_surface(surface_id, damage).map_err(|_| self.map_backend_error())?;
        Ok(())
    }

    pub fn route_key_input(&mut self, key: u8, pressed: bool) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::KeyInput { surface_id: self.focused_surface, key, pressed });
        Ok(())
    }

    pub fn route_pointer_motion(&mut self, x: i32, y: i32, dx: i32, dy: i32) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::PointerMotion { surface_id: self.focused_surface, x, y, dx, dy });
        Ok(())
    }

    pub fn route_mouse_button(&mut self, button: MouseButton, pressed: bool, x: i32, y: i32) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::MouseButton { surface_id: self.focused_surface, button, pressed, x, y });
        Ok(())
    }

    pub fn route_mouse_wheel(&mut self, delta: i32, x: i32, y: i32) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::MouseWheel { surface_id: self.focused_surface, delta, x, y });
        Ok(())
    }

    pub fn update_frame(&mut self, now_ms: u64) -> Result<bool, ServerError> {
        self.ensure_running()?;
        if now_ms < self.last_present_ms { self.last_present_ms = now_ms; }
        let elapsed = now_ms.saturating_sub(self.last_present_ms);
        if elapsed < self.frame_interval_ms as u64 { return Ok(false); }
        self.backend.flush().map_err(|_| self.map_backend_error())?;
        self.last_present_ms = now_ms;
        self.next_frame_id = self.next_frame_id.wrapping_add(1);
        self.diagnostics.frames_presented = self.diagnostics.frames_presented.saturating_add(1);
        self.emit_event(DisplayEvent::FramePresented { frame_id: self.next_frame_id });
        Ok(true)
    }

    pub fn poll_event(&mut self) -> Option<DisplayEvent> { self.events.pop_front() }

    pub fn session_surface_count(&self, session_id: SessionId) -> usize {
        self.sessions.get(&session_id).map(|s| s.surfaces.len()).unwrap_or(0)
    }

    pub fn get_session_config(&self, session_id: SessionId) -> Option<&SessionConfig> {
        self.sessions.get(&session_id).map(|s| &s.config)
    }

    fn ensure_running(&self) -> Result<(), ServerError> {
        if self.state != ServerState::Running { return Err(ServerError::NotRunning); }
        Ok(())
    }

    fn ensure_owner(&self, client_id: ClientId, surface_id: SurfaceId) -> Result<(), ServerError> {
        let surface = self.surfaces.get(&surface_id).ok_or(ServerError::SurfaceNotFound)?;
        if surface.owner != client_id {
            let client_session = self.client_sessions.get(&client_id).copied().unwrap_or(SessionId::KERNEL);
            if surface.session_id != client_session && client_session != SessionId::KERNEL {
                return Err(ServerError::PermissionDenied);
            }
        }
        Ok(())
    }

    fn allocate_surface_id(&mut self) -> Option<SurfaceId> {
        let start = self.next_surface_id.max(1);
        let mut candidate = start;
        loop {
            if candidate == 0 { candidate = 1; }
            let id = SurfaceId(candidate);
            candidate = candidate.wrapping_add(1);
            if !self.surfaces.contains_key(&id) {
                self.next_surface_id = candidate.max(1);
                return Some(id);
            }
            if candidate == start { return None; }
        }
    }

    fn map_backend_error(&mut self) -> ServerError {
        self.diagnostics.backend_errors = self.diagnostics.backend_errors.saturating_add(1);
        ServerError::BackendError
    }

    fn emit_event(&mut self, event: DisplayEvent) {
        if self.events.len() >= self.max_event_queue {
            self.diagnostics.dropped_events = self.diagnostics.dropped_events.saturating_add(1);
            return;
        }
        self.events.push_back(event);
        self.diagnostics.events_emitted = self.diagnostics.events_emitted.saturating_add(1);
    }
}