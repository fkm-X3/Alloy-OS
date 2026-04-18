use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::fmt;

use crate::protocol::{
    ClientId, DisplayEvent, DisplayRequest, DisplayResponse, MouseButton, PixelFormat, ProtocolError, Rect,
    SurfaceId, validate_request,
};

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
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    pub z_order: u32,
    pub format: PixelFormat,
}

/// Server-side observability counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerDiagnostics {
    pub requests_handled: u64,
    pub events_emitted: u64,
    pub dropped_events: u64,
    pub frames_presented: u64,
    pub backend_errors: u64,
}

/// Protocol-driven display server runtime.
pub struct DisplayServer<B: DisplayBackend> {
    backend: B,
    state: ServerState,
    next_surface_id: u32,
    surfaces: BTreeMap<SurfaceId, SurfaceEntry>,
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
        Self {
            backend,
            state: ServerState::Stopped,
            next_surface_id: 1,
            surfaces: BTreeMap::new(),
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

    pub fn state(&self) -> ServerState {
        self.state
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn surface(&self, surface_id: SurfaceId) -> Option<&SurfaceEntry> {
        self.surfaces.get(&surface_id)
    }

    pub fn focused_surface(&self) -> Option<SurfaceId> {
        self.focused_surface
    }

    pub fn diagnostics(&self) -> ServerDiagnostics {
        self.diagnostics
    }

    pub fn frame_interval_ms(&self) -> u32 {
        self.frame_interval_ms
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
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
            DisplayRequest::CreateSurface {
                width,
                height,
                format,
            } => {
                let surface_id = self
                    .allocate_surface_id()
                    .ok_or(ServerError::SurfaceIdExhausted)?;

                self.backend
                    .create_surface(surface_id, width, height, format)
                    .map_err(|_| self.map_backend_error())?;

                let entry = SurfaceEntry {
                    id: surface_id,
                    owner: client_id,
                    width,
                    height,
                    x: 0,
                    y: 0,
                    visible: true,
                    z_order: 0,
                    format,
                };
                self.surfaces.insert(surface_id, entry);
                self.emit_event(DisplayEvent::SurfaceCreated {
                    surface_id,
                    owner: client_id,
                });
                Ok(DisplayResponse::SurfaceCreated { surface_id })
            }
            DisplayRequest::DestroySurface { surface_id } => {
                self.ensure_owner(client_id, surface_id)?;

                self.backend
                    .destroy_surface(surface_id)
                    .map_err(|_| self.map_backend_error())?;

                self.surfaces.remove(&surface_id);
                if self.focused_surface == Some(surface_id) {
                    self.focused_surface = None;
                    self.emit_event(DisplayEvent::FocusChanged { surface_id: None });
                }
                self.emit_event(DisplayEvent::SurfaceDestroyed { surface_id });
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetSurfacePosition { surface_id, x, y } => {
                self.ensure_owner(client_id, surface_id)?;

                self.backend
                    .set_surface_position(surface_id, x, y)
                    .map_err(|_| self.map_backend_error())?;

                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.x = x;
                    entry.y = y;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::ResizeSurface {
                surface_id,
                width,
                height,
            } => {
                self.ensure_owner(client_id, surface_id)?;

                self.backend
                    .resize_surface(surface_id, width, height)
                    .map_err(|_| self.map_backend_error())?;

                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.width = width;
                    entry.height = height;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetSurfaceVisibility {
                surface_id,
                visible,
            } => {
                self.ensure_owner(client_id, surface_id)?;

                self.backend
                    .set_surface_visibility(surface_id, visible)
                    .map_err(|_| self.map_backend_error())?;

                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.visible = visible;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetSurfaceZOrder {
                surface_id,
                z_order,
            } => {
                self.ensure_owner(client_id, surface_id)?;

                self.backend
                    .set_surface_z_order(surface_id, z_order)
                    .map_err(|_| self.map_backend_error())?;

                if let Some(entry) = self.surfaces.get_mut(&surface_id) {
                    entry.z_order = z_order;
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::CommitSurface { surface_id, damage } => {
                self.ensure_owner(client_id, surface_id)?;

                self.backend
                    .commit_surface(surface_id, damage)
                    .map_err(|_| self.map_backend_error())?;
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::RequestFocus { surface_id } => {
                if let Some(target) = surface_id {
                    self.ensure_owner(client_id, target)?;
                }

                if self.focused_surface != surface_id {
                    self.focused_surface = surface_id;
                    self.emit_event(DisplayEvent::FocusChanged { surface_id });
                }
                Ok(DisplayResponse::Ack)
            }
            DisplayRequest::SetFrameIntervalMs { interval_ms } => {
                self.frame_interval_ms = interval_ms;
                Ok(DisplayResponse::Ack)
            }
        }
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
        self.ensure_owner(client_id, surface_id)?;

        let entry = self
            .surfaces
            .get(&surface_id)
            .ok_or(ServerError::SurfaceNotFound)?;
        if entry.width != width || entry.height != height {
            return Err(ServerError::InvalidRequest(ProtocolError::InvalidDimensions));
        }

        self.backend
            .upload_surface_pixels(surface_id, width, height, pixels, damage)
            .map_err(|_| self.map_backend_error())?;
        self.backend
            .commit_surface(surface_id, damage)
            .map_err(|_| self.map_backend_error())?;

        Ok(())
    }

    pub fn route_key_input(&mut self, key: u8, pressed: bool) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::KeyInput {
            surface_id: self.focused_surface,
            key,
            pressed,
        });
        Ok(())
    }

    pub fn route_pointer_motion(
        &mut self,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    ) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::PointerMotion {
            surface_id: self.focused_surface,
            x,
            y,
            dx,
            dy,
        });
        Ok(())
    }

    pub fn route_mouse_button(
        &mut self,
        button: MouseButton,
        pressed: bool,
        x: i32,
        y: i32,
    ) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::MouseButton {
            surface_id: self.focused_surface,
            button,
            pressed,
            x,
            y,
        });
        Ok(())
    }

    pub fn route_mouse_wheel(&mut self, delta: i32, x: i32, y: i32) -> Result<(), ServerError> {
        self.ensure_running()?;
        self.emit_event(DisplayEvent::MouseWheel {
            surface_id: self.focused_surface,
            delta,
            x,
            y,
        });
        Ok(())
    }

    /// Present a new frame when the configured frame interval elapses.
    pub fn update_frame(&mut self, now_ms: u64) -> Result<bool, ServerError> {
        self.ensure_running()?;

        if now_ms < self.last_present_ms {
            self.last_present_ms = now_ms;
        }

        let elapsed = now_ms.saturating_sub(self.last_present_ms);
        if elapsed < self.frame_interval_ms as u64 {
            return Ok(false);
        }

        self.backend.flush().map_err(|_| self.map_backend_error())?;
        self.last_present_ms = now_ms;
        self.next_frame_id = self.next_frame_id.wrapping_add(1);
        self.diagnostics.frames_presented = self.diagnostics.frames_presented.saturating_add(1);
        self.emit_event(DisplayEvent::FramePresented {
            frame_id: self.next_frame_id,
        });

        Ok(true)
    }

    pub fn poll_event(&mut self) -> Option<DisplayEvent> {
        self.events.pop_front()
    }

    fn ensure_running(&self) -> Result<(), ServerError> {
        if self.state != ServerState::Running {
            return Err(ServerError::NotRunning);
        }
        Ok(())
    }

    fn ensure_owner(&self, client_id: ClientId, surface_id: SurfaceId) -> Result<(), ServerError> {
        let surface = self
            .surfaces
            .get(&surface_id)
            .ok_or(ServerError::SurfaceNotFound)?;
        if surface.owner != client_id {
            return Err(ServerError::PermissionDenied);
        }
        Ok(())
    }

    fn allocate_surface_id(&mut self) -> Option<SurfaceId> {
        let start = self.next_surface_id.max(1);
        let mut candidate = start;

        loop {
            if candidate == 0 {
                candidate = 1;
            }

            let id = SurfaceId(candidate);
            candidate = candidate.wrapping_add(1);

            if !self.surfaces.contains_key(&id) {
                self.next_surface_id = candidate.max(1);
                return Some(id);
            }

            if candidate == start {
                return None;
            }
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

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct MockSurface {
        width: u32,
        height: u32,
    }

    #[derive(Debug, Default)]
    struct MockBackend {
        surfaces: BTreeMap<SurfaceId, MockSurface>,
    }

    impl DisplayBackend for MockBackend {
        type Error = ();

        fn create_surface(
            &mut self,
            surface_id: SurfaceId,
            width: u32,
            height: u32,
            _format: PixelFormat,
        ) -> Result<(), Self::Error> {
            self.surfaces.insert(surface_id, MockSurface { width, height });
            Ok(())
        }

        fn destroy_surface(&mut self, surface_id: SurfaceId) -> Result<(), Self::Error> {
            self.surfaces.remove(&surface_id).map(|_| ()).ok_or(())
        }

        fn set_surface_position(
            &mut self,
            surface_id: SurfaceId,
            _x: i32,
            _y: i32,
        ) -> Result<(), Self::Error> {
            self.surfaces.get(&surface_id).map(|_| ()).ok_or(())
        }

        fn resize_surface(
            &mut self,
            surface_id: SurfaceId,
            width: u32,
            height: u32,
        ) -> Result<(), Self::Error> {
            let surface = self.surfaces.get_mut(&surface_id).ok_or(())?;
            surface.width = width;
            surface.height = height;
            Ok(())
        }

        fn set_surface_visibility(
            &mut self,
            surface_id: SurfaceId,
            _visible: bool,
        ) -> Result<(), Self::Error> {
            self.surfaces.get(&surface_id).map(|_| ()).ok_or(())
        }

        fn set_surface_z_order(
            &mut self,
            surface_id: SurfaceId,
            _z_order: u32,
        ) -> Result<(), Self::Error> {
            self.surfaces.get(&surface_id).map(|_| ()).ok_or(())
        }

        fn commit_surface(
            &mut self,
            surface_id: SurfaceId,
            _damage: Option<Rect>,
        ) -> Result<(), Self::Error> {
            self.surfaces.get(&surface_id).map(|_| ()).ok_or(())
        }

        fn upload_surface_pixels(
            &mut self,
            surface_id: SurfaceId,
            width: u32,
            height: u32,
            pixels: &[u32],
            _damage: Option<Rect>,
        ) -> Result<(), Self::Error> {
            let Some(surface) = self.surfaces.get(&surface_id) else {
                return Err(());
            };
            if surface.width != width || surface.height != height {
                return Err(());
            }
            let required = (width as usize).saturating_mul(height as usize);
            if pixels.len() < required {
                return Err(());
            }
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn mouse_routing_emits_events_with_focused_surface() {
        let mut server = DisplayServer::new(MockBackend::default());
        server.start().expect("server should start");

        let owner = ClientId::new(11);
        let created = server
            .handle_request(
                owner,
                DisplayRequest::CreateSurface {
                    width: 64,
                    height: 64,
                    format: PixelFormat::Argb8888,
                },
            )
            .expect("surface creation should succeed");
        let surface_id = match created {
            DisplayResponse::SurfaceCreated { surface_id } => surface_id,
            _ => panic!("unexpected create response"),
        };
        server
            .handle_request(
                owner,
                DisplayRequest::RequestFocus {
                    surface_id: Some(surface_id),
                },
            )
            .expect("focus request should succeed");

        server
            .route_pointer_motion(100, 80, 5, -3)
            .expect("pointer event should route");
        server
            .route_mouse_button(MouseButton::Left, true, 100, 80)
            .expect("button event should route");
        server
            .route_mouse_wheel(-1, 100, 80)
            .expect("wheel event should route");

        let mut saw_motion = false;
        let mut saw_button = false;
        let mut saw_wheel = false;

        while let Some(event) = server.poll_event() {
            match event {
                DisplayEvent::PointerMotion {
                    surface_id: Some(id),
                    x,
                    y,
                    dx,
                    dy,
                } if id == surface_id => {
                    assert_eq!((x, y, dx, dy), (100, 80, 5, -3));
                    saw_motion = true;
                }
                DisplayEvent::MouseButton {
                    surface_id: Some(id),
                    button,
                    pressed,
                    x,
                    y,
                } if id == surface_id => {
                    assert_eq!(button, MouseButton::Left);
                    assert!(pressed);
                    assert_eq!((x, y), (100, 80));
                    saw_button = true;
                }
                DisplayEvent::MouseWheel {
                    surface_id: Some(id),
                    delta,
                    x,
                    y,
                } if id == surface_id => {
                    assert_eq!(delta, -1);
                    assert_eq!((x, y), (100, 80));
                    saw_wheel = true;
                }
                _ => {}
            }
        }

        assert!(saw_motion, "pointer motion event should be emitted");
        assert!(saw_button, "mouse button event should be emitted");
        assert!(saw_wheel, "mouse wheel event should be emitted");
    }
}
