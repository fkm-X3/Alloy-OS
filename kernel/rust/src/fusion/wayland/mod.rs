//! Wayland display server implementation for Fusion
//!
//! Provides core Wayland protocol support alongside the existing Fusion display system.
//! Manages client connections via Unix domain sockets and dispatches protocol messages
//! to appropriate handlers.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub mod buffer_handler;
pub mod client;
pub mod compositor_handler;
pub mod compositor_integration;
pub mod damage;
pub mod display_handler;
pub mod focus;
pub mod globals;
pub mod input_routing;
pub mod layer_shell;
pub mod output;
pub mod protocol;
pub mod registry_handler;
pub mod seat;
pub mod shm;
pub mod socket;
pub mod surface;
pub mod xdg_output;
pub mod xdg_shell;

use self::buffer_handler::ShmBufferHandler;
use self::client::{ClientId, ClientState};
use self::compositor_handler::CompositorHandler;
use self::compositor_integration::CompositorIntegration;
use self::display_handler::DisplayHandler;
use self::input_routing::{InputRouter, PendingInputEvent, SurfaceGeometry};
use self::layer_shell::LayerShellHandler;
use self::output::OutputManager;
use self::protocol::{ProtocolHandler, WaylandMessage};
use self::registry_handler::RegistryHandler;
use self::seat::SeatManager;
use self::socket::UnixSocket;
use self::surface::{SurfaceId, SurfaceState};
use self::xdg_output::XdgOutputManagerHandler;
use self::xdg_shell::XdgShellHandler;
use crate::fusion::FusionDisplayBackend;
use crate::graphics::{Display, PlatformDisplay};

/// Wayland server error types
#[derive(Debug, Clone, Copy)]
pub enum WaylandError {
    /// Socket creation failed
    SocketCreationFailed,
    /// Socket bind failed
    SocketBindFailed,
    /// Socket listen failed
    SocketListenFailed,
    /// Accept connection failed
    AcceptFailed,
    /// Invalid file descriptor
    InvalidFd,
    /// Protocol violation
    ProtocolViolation,
    /// Object not found
    ObjectNotFound,
    /// Invalid object ID
    InvalidObjectId,
    /// Memory allocation failed
    AllocationFailed,
    /// Read operation failed
    ReadFailed,
    /// Write operation failed
    WriteFailed,
}

impl core::fmt::Display for WaylandError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WaylandError::SocketCreationFailed => write!(f, "Socket creation failed"),
            WaylandError::SocketBindFailed => write!(f, "Socket bind failed"),
            WaylandError::SocketListenFailed => write!(f, "Socket listen failed"),
            WaylandError::AcceptFailed => write!(f, "Accept connection failed"),
            WaylandError::InvalidFd => write!(f, "Invalid file descriptor"),
            WaylandError::ProtocolViolation => write!(f, "Protocol violation"),
            WaylandError::ObjectNotFound => write!(f, "Object not found"),
            WaylandError::InvalidObjectId => write!(f, "Invalid object ID"),
            WaylandError::AllocationFailed => write!(f, "Memory allocation failed"),
            WaylandError::ReadFailed => write!(f, "Read operation failed"),
            WaylandError::WriteFailed => write!(f, "Write operation failed"),
        }
    }
}

impl WaylandError {
    /// Get error message as a static byte string for serial logging
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            WaylandError::SocketCreationFailed => b"Socket creation failed",
            WaylandError::SocketBindFailed => b"Socket bind failed",
            WaylandError::SocketListenFailed => b"Socket listen failed",
            WaylandError::AcceptFailed => b"Accept connection failed",
            WaylandError::InvalidFd => b"Invalid file descriptor",
            WaylandError::ProtocolViolation => b"Protocol violation",
            WaylandError::ObjectNotFound => b"Object not found",
            WaylandError::InvalidObjectId => b"Invalid object ID",
            WaylandError::AllocationFailed => b"Memory allocation failed",
            WaylandError::ReadFailed => b"Read operation failed",
            WaylandError::WriteFailed => b"Write operation failed",
        }
    }
}

/// Result type for Wayland operations
pub type WaylandResult<T> = Result<T, WaylandError>;

/// Per-client connection state
pub struct ClientConnection {
    /// Unique client identifier
    id: ClientId,
    /// Socket file descriptor for this client
    fd: u32,
    /// Client-specific state (object registry, etc.)
    state: ClientState,
}

impl ClientConnection {
    /// Create a new client connection
    fn new(id: ClientId, fd: u32) -> Self {
        Self {
            id,
            fd,
            state: ClientState::new(),
        }
    }

    /// Get client ID
    pub fn id(&self) -> ClientId {
        self.id
    }

    /// Get file descriptor
    pub fn fd(&self) -> u32 {
        self.fd
    }

    /// Get client state
    pub fn state(&self) -> &ClientState {
        &self.state
    }

    /// Get mutable client state
    pub fn state_mut(&mut self) -> &mut ClientState {
        &mut self.state
    }
}

/// Wayland event opcodes for keyboard
pub mod keyboard_opcodes {
    pub const KEYMAP: u16 = 0;
    pub const ENTER: u16 = 1;
    pub const LEAVE: u16 = 2;
    pub const KEY: u16 = 3;
    pub const MODIFIERS: u16 = 4;
    pub const REPEAT_INFO: u16 = 5;
}

/// Wayland wl_seat opcodes for client requests
pub mod seat_opcodes {
    pub const GET_POINTER: u16 = 0;
    pub const GET_KEYBOARD: u16 = 1;
}

/// Wayland event opcodes for pointer
pub mod pointer_opcodes {
    pub const ENTER: u16 = 0;
    pub const LEAVE: u16 = 1;
    pub const MOTION: u16 = 2;
    pub const BUTTON: u16 = 4;
    pub const AXIS: u16 = 5;
    pub const FRAME: u16 = 6;
}

/// Per-client protocol object IDs for input
struct ClientInputIds {
    keyboard_id: u32,
    pointer_id: u32,
}

/// Main Wayland server structure
pub struct WaylandServer {
    /// Unix domain socket listener
    socket: Option<UnixSocket>,
    /// Connected clients indexed by ID
    clients: BTreeMap<ClientId, ClientConnection>,
    /// Next client ID to assign
    next_client_id: u32,
    /// Event serial number counter
    next_serial: u32,
    /// Protocol message handler (routes to all sub-handlers)
    protocol_handler: ProtocolHandler,
    /// Display protocol handler (sync, get_registry)
    display_handler: DisplayHandler,
    /// Registry protocol handler
    registry_handler: RegistryHandler,
    /// Compositor protocol handler
    compositor_handler: CompositorHandler,
    /// SHM buffer handler
    shm_buffer_handler: ShmBufferHandler,
    /// Seat manager for input devices
    seat_manager: SeatManager,
    /// Output manager for display info
    output_manager: OutputManager,
    /// Input router for event dispatch
    input_router: InputRouter,
    /// XDG shell handler (lxqt wayland compat)
    xdg_shell_handler: XdgShellHandler,
    /// Layer shell handler (panel/desktop surfaces)
    layer_shell_handler: LayerShellHandler,
    /// XDG output manager handler
    xdg_output_handler: XdgOutputManagerHandler,
    /// Framebuffer reference for compositor integration
    framebuffer: Option<FusionDisplayBackend>,
    /// Per-client keyboard/pointer object IDs
    client_input_ids: BTreeMap<ClientId, ClientInputIds>,
}

impl WaylandServer {
    /// Create a new Wayland server instance
    pub fn new() -> Self {
        Self {
            socket: None,
            clients: BTreeMap::new(),
            next_client_id: 1,
            next_serial: 1,
            protocol_handler: ProtocolHandler::new(),
            display_handler: DisplayHandler::new(),
            registry_handler: RegistryHandler::new(),
            compositor_handler: CompositorHandler::new(),
            shm_buffer_handler: ShmBufferHandler::new(),
            seat_manager: SeatManager::new(),
            output_manager: OutputManager::new(),
            input_router: InputRouter::new(),
            xdg_shell_handler: XdgShellHandler::new(),
            layer_shell_handler: LayerShellHandler::new(),
            xdg_output_handler: XdgOutputManagerHandler::new(),
            framebuffer: None,
            client_input_ids: BTreeMap::new(),
        }
    }

    /// Initialize the Wayland server and bind to the standard socket
    pub fn init(&mut self) -> WaylandResult<()> {
        // Create and bind Unix domain socket at standard Wayland path
        let mut socket = UnixSocket::new()?;
        socket.bind("/tmp/wayland-0")?;
        socket.listen()?;

        self.socket = Some(socket);

        unsafe {
            crate::println!("[Wayland] Server initialized at /tmp/wayland-0");
        }

        Ok(())
    }

    /// Set the framebuffer backend for compositor integration
    pub fn set_framebuffer(&mut self, backend: FusionDisplayBackend) {
        self.framebuffer = Some(backend);
    }

    /// Initialize with framebuffer reference for compositor integration
    pub fn init_with_framebuffer(&mut self, _width: u32, _height: u32) -> WaylandResult<()> {
        let display = PlatformDisplay::new().ok_or(WaylandError::AllocationFailed)?;
        self.framebuffer = Some(FusionDisplayBackend::new(display));
        self.init()
    }

    /// Accept a new client connection
    pub fn accept_client(&mut self) -> WaylandResult<()> {
        if let Some(ref socket) = self.socket {
            let fd = socket.accept()?;
            let client_id = ClientId(self.next_client_id);
            self.next_client_id += 1;

            let connection = ClientConnection::new(client_id, fd);
            self.clients.insert(client_id, connection);

            // Send initial registry globals to new client
            self.send_initial_globals(client_id);

            unsafe {
                crate::println!("[Wayland] Accepted client connection");
            }

            Ok(())
        } else {
            Err(WaylandError::SocketCreationFailed)
        }
    }

    /// Send initial global objects to a newly connected client
    fn send_initial_globals(&mut self, client_id: ClientId) {
        if let Some(connection) = self.clients.get_mut(&client_id) {
            connection
                .state
                .register_object(crate::fusion::wayland::client::ObjectType::Registry, 1);

            let globals = self
                .registry_handler
                .get_global_events_for_client(client_id, 2);
            for msg in globals {
                let _ = self.write_message_to_client(client_id, msg);
            }
        }
    }

    /// Encode and write a Wayland message to a client's socket
    fn write_message_to_client(
        &mut self,
        client_id: ClientId,
        msg: WaylandMessage,
    ) -> WaylandResult<()> {
        let fd = self
            .clients
            .get(&client_id)
            .ok_or(WaylandError::ObjectNotFound)?
            .fd as i32;
        let encoded = msg.encode()?;
        let written = crate::net::socket_write(fd, &encoded);
        if written < 0 {
            return Err(WaylandError::WriteFailed);
        }
        Ok(())
    }

    /// Read and decode a Wayland message from a client's socket.
    /// Blocks until a complete message is available or the client disconnects.
    pub fn read_message_from_client(
        &mut self,
        client_id: ClientId,
    ) -> WaylandResult<Option<WaylandMessage>> {
        let fd = self
            .clients
            .get(&client_id)
            .ok_or(WaylandError::ObjectNotFound)?
            .fd as i32;

        // Read header (8 bytes) — blocks until at least 8 bytes arrive
        let mut header = [0u8; 8];
        let mut off = 0;
        while off < 8 {
            let n = crate::net::socket_read(fd, &mut header[off..]);
            if n < 0 {
                let _ = self.disconnect_client(client_id);
                return Err(WaylandError::ReadFailed);
            }
            off += n as usize;
        }

        let length_le = [header[6], header[7]];
        let total = u16::from_le_bytes(length_le) as usize;

        if total < 8 || total > 4096 {
            let _ = self.disconnect_client(client_id);
            return Err(WaylandError::ProtocolViolation);
        }

        let mut full = alloc::vec![0u8; total];
        full[..8].copy_from_slice(&header);

        // Read remaining payload
        while off < total {
            let n = crate::net::socket_read(fd, &mut full[off..]);
            if n < 0 {
                let _ = self.disconnect_client(client_id);
                return Err(WaylandError::ReadFailed);
            }
            off += n as usize;
        }

        match WaylandMessage::decode(&full) {
            Ok(Some(msg)) => Ok(Some(msg)),
            _ => {
                let _ = self.disconnect_client(client_id);
                Err(WaylandError::ProtocolViolation)
            }
        }
    }

    /// Poll all connected clients and dispatch their pending messages
    pub fn poll_clients(&mut self) {
        let client_ids: Vec<ClientId> = self.clients.keys().copied().collect();
        for cid in client_ids {
            loop {
                match self.read_message_from_client(cid) {
                    Ok(Some(msg)) => {
                        let _ = self.dispatch_message(cid, msg);
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        }
    }

    /// Dispatch a message from a client
    pub fn dispatch_message(
        &mut self,
        client_id: ClientId,
        message: WaylandMessage,
    ) -> WaylandResult<()> {
        let object_type = match self.clients.get(&client_id) {
            Some(client) => client
                .state()
                .get_object(message.object_id.0)
                .map(|e| e.object_type())
                .unwrap_or(crate::fusion::wayland::client::ObjectType::Custom),
            None => return Err(WaylandError::ObjectNotFound),
        };

        match object_type {
            crate::fusion::wayland::client::ObjectType::Display => {
                let response = self.display_handler.handle_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
                match response {
                    crate::fusion::wayland::display_handler::DisplayResponse::SyncAck {
                        callback_id,
                        callback_data,
                    } => {
                        if let Ok(done_msg) = crate::fusion::wayland::display_handler::DisplayHandler::emit_callback_done(callback_id, callback_data) {
                            let _ = self.write_message_to_client(client_id, done_msg);
                        }
                    }
                    crate::fusion::wayland::display_handler::DisplayResponse::RegistryCreated {
                        registry_id,
                    } => {
                        let globals = self.registry_handler.get_global_events(registry_id);
                        for msg in globals {
                            let _ = self.write_message_to_client(client_id, msg);
                        }
                    }
                    _ => {}
                }
            }
            crate::fusion::wayland::client::ObjectType::Registry => {
                let response = self.registry_handler.handle_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
                match response {
                    crate::fusion::wayland::registry_handler::RegistryResponse::Bound {
                        global_name: _,
                        object_id: _,
                        interface,
                        version: _,
                    } => {
                        let obj_type = match interface {
                            crate::fusion::wayland::globals::InterfaceName::Compositor => {
                                crate::fusion::wayland::client::ObjectType::Compositor
                            }
                            crate::fusion::wayland::globals::InterfaceName::Output => {
                                crate::fusion::wayland::client::ObjectType::Output
                            }
                            crate::fusion::wayland::globals::InterfaceName::Seat => {
                                crate::fusion::wayland::client::ObjectType::Seat
                            }
                            crate::fusion::wayland::globals::InterfaceName::Shm => {
                                crate::fusion::wayland::client::ObjectType::Custom
                            }
                            crate::fusion::wayland::globals::InterfaceName::Subcompositor => {
                                crate::fusion::wayland::client::ObjectType::Custom
                            }
                            crate::fusion::wayland::globals::InterfaceName::DataDeviceManager => {
                                crate::fusion::wayland::client::ObjectType::Custom
                            }
                            crate::fusion::wayland::globals::InterfaceName::XdgShell => {
                                crate::fusion::wayland::client::ObjectType::XdgWmBase
                            }
                            crate::fusion::wayland::globals::InterfaceName::LayerShell => {
                                crate::fusion::wayland::client::ObjectType::LayerShell
                            }
                            crate::fusion::wayland::globals::InterfaceName::XdgOutputManager => {
                                crate::fusion::wayland::client::ObjectType::XdgOutputManager
                            }
                        };
                        if let Some(client) = self.clients.get_mut(&client_id) {
                            client.state_mut().register_object(obj_type, 1);
                        }
                    }
                }
            }
            crate::fusion::wayland::client::ObjectType::Compositor => {
                let _response = self.compositor_handler.handle_compositor_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::Surface => {
                let _ = self.compositor_handler.handle_surface_request(
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::XdgWmBase => {
                let _response = self.xdg_shell_handler.handle_wm_base_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::XdgSurface => {
                let _ = self.xdg_shell_handler.handle_xdg_surface_request(
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::XdgToplevel => {
                let _ = self.xdg_shell_handler.handle_toplevel_request(
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::XdgPopup => {
                let _ = self
                    .xdg_shell_handler
                    .handle_popup_request(message.opcode)?;
            }
            crate::fusion::wayland::client::ObjectType::LayerShell => {
                let _ = self.layer_shell_handler.handle_layer_shell_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::LayerSurface => {
                let _ = self.layer_shell_handler.handle_layer_surface_request(
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::XdgOutputManager => {
                let _ = self.xdg_output_handler.handle_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::XdgOutput => {
                let _ = self.xdg_output_handler.handle_xdg_output_request(
                    client_id,
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                )?;
            }
            crate::fusion::wayland::client::ObjectType::Seat => {
                // Handle wl_seat requests (get_pointer, get_keyboard)
                let payload = &message.payload;
                match message.opcode {
                    seat_opcodes::GET_POINTER => {
                        if payload.len() >= 4 {
                            let new_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
                            let pointer_id = u32::from_le_bytes(new_id_bytes);
                            self.client_input_ids
                                .entry(client_id)
                                .or_insert(ClientInputIds {
                                    keyboard_id: 0,
                                    pointer_id,
                                })
                                .pointer_id = pointer_id;
                        }
                    }
                    seat_opcodes::GET_KEYBOARD => {
                        if payload.len() >= 4 {
                            let new_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
                            let keyboard_id = u32::from_le_bytes(new_id_bytes);
                            self.client_input_ids
                                .entry(client_id)
                                .or_insert(ClientInputIds {
                                    keyboard_id,
                                    pointer_id: 0,
                                })
                                .keyboard_id = keyboard_id;
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                let _ = self.protocol_handler.handle_message(
                    client_id,
                    message,
                    &mut self.display_handler,
                    &mut self.registry_handler,
                    &mut self.compositor_handler,
                    &mut self.shm_buffer_handler,
                )?;
            }
        }
        Ok(())
    }

    /// Process all pending frame callbacks and emit done events
    pub fn process_frame_callbacks(&mut self) {
        while let Some(callback) = self.display_handler.get_pending_callback() {
            if let Ok(msg) = super::wayland::display_handler::DisplayHandler::emit_callback_done(
                callback.callback_id,
                callback.callback_data,
            ) {
                let _ = self.write_message_to_client(callback.client_id, msg);
            }
        }
    }

    /// Composite all surfaces and present to framebuffer
    pub fn composite_frame(&mut self) {
        let surfaces: Vec<(u32, &SurfaceState)> = self
            .compositor_handler
            .iter_surfaces()
            .map(|(_id, surface)| (surface.z_order, surface))
            .collect();

        if let Some(backend) = self.framebuffer.as_mut() {
            let shm_mgr = self.shm_buffer_handler.shm_manager_mut();
            CompositorIntegration::composite_frame(backend, shm_mgr, &surfaces);
            backend.display_mut().swap_buffer();
        }
    }

    /// Get mutable reference to a client connection
    pub fn get_client_mut(&mut self, client_id: ClientId) -> Option<&mut ClientConnection> {
        self.clients.get_mut(&client_id)
    }

    /// Get immutable reference to a client connection
    pub fn get_client(&self, client_id: ClientId) -> Option<&ClientConnection> {
        self.clients.get(&client_id)
    }

    /// Remove a client connection
    pub fn disconnect_client(&mut self, client_id: ClientId) -> WaylandResult<()> {
        if let Some(_connection) = self.clients.remove(&client_id) {
            self.client_input_ids.remove(&client_id);
            self.registry_handler.remove_client(client_id);
            self.seat_manager.remove_client(client_id.0);
            self.output_manager.remove_client(client_id.0);
            self.compositor_handler.clear_surface_for_client(client_id);

            unsafe {
                crate::println!("[Wayland] Client disconnected");
            }
            Ok(())
        } else {
            Err(WaylandError::ObjectNotFound)
        }
    }

    /// Get number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Check if server socket is listening
    pub fn is_listening(&self) -> bool {
        self.socket.is_some()
    }

    /// Check if server socket has pending connections
    pub fn has_pending_connections(&self) -> bool {
        match &self.socket {
            Some(socket) => socket.has_pending_connections(),
            None => false,
        }
    }

    /// Get reference to display handler
    pub fn display_handler(&self) -> &DisplayHandler {
        &self.display_handler
    }

    /// Get mutable reference to display handler
    pub fn display_handler_mut(&mut self) -> &mut DisplayHandler {
        &mut self.display_handler
    }

    /// Get reference to registry handler
    pub fn registry_handler(&self) -> &RegistryHandler {
        &self.registry_handler
    }

    /// Get mutable reference to registry handler
    pub fn registry_handler_mut(&mut self) -> &mut RegistryHandler {
        &mut self.registry_handler
    }

    /// Get reference to compositor handler
    pub fn compositor_handler(&self) -> &CompositorHandler {
        &self.compositor_handler
    }

    /// Get mutable reference to compositor handler
    pub fn compositor_handler_mut(&mut self) -> &mut CompositorHandler {
        &mut self.compositor_handler
    }

    /// Get reference to SHM buffer handler
    pub fn shm_buffer_handler(&self) -> &ShmBufferHandler {
        &self.shm_buffer_handler
    }

    /// Get mutable reference to SHM buffer handler
    pub fn shm_buffer_handler_mut(&mut self) -> &mut ShmBufferHandler {
        &mut self.shm_buffer_handler
    }

    /// Get reference to seat manager
    pub fn seat_manager(&self) -> &SeatManager {
        &self.seat_manager
    }

    /// Get mutable reference to seat manager
    pub fn seat_manager_mut(&mut self) -> &mut SeatManager {
        &mut self.seat_manager
    }

    /// Get reference to output manager
    pub fn output_manager(&self) -> &OutputManager {
        &self.output_manager
    }

    /// Get mutable reference to output manager
    pub fn output_manager_mut(&mut self) -> &mut OutputManager {
        &mut self.output_manager
    }

    /// Get reference to input router
    pub fn input_router(&self) -> &InputRouter {
        &self.input_router
    }

    /// Get mutable reference to input router
    pub fn input_router_mut(&mut self) -> &mut InputRouter {
        &mut self.input_router
    }

    /// Snapshot the current surface geometry for input hit-testing.
    pub fn surface_geometries(&self) -> Vec<SurfaceGeometry> {
        let mut geometries = Vec::new();
        for (surface_id, surface) in self.compositor_handler.iter_surfaces() {
            let width = surface.current.width;
            let height = surface.current.height;
            if width == 0 || height == 0 {
                continue;
            }
            geometries.push(SurfaceGeometry::new(
                *surface_id,
                surface.screen_x,
                surface.screen_y,
                width,
                height,
                surface.z_order,
            ));
        }
        geometries
    }

    /// Get reference to xdg shell handler
    pub fn xdg_shell_handler(&self) -> &XdgShellHandler {
        &self.xdg_shell_handler
    }

    /// Get mutable reference to xdg shell handler
    pub fn xdg_shell_handler_mut(&mut self) -> &mut XdgShellHandler {
        &mut self.xdg_shell_handler
    }

    /// Get reference to layer shell handler
    pub fn layer_shell_handler(&self) -> &LayerShellHandler {
        &self.layer_shell_handler
    }

    /// Get mutable reference to layer shell handler
    pub fn layer_shell_handler_mut(&mut self) -> &mut LayerShellHandler {
        &mut self.layer_shell_handler
    }

    /// Get reference to xdg output handler
    pub fn xdg_output_handler(&self) -> &XdgOutputManagerHandler {
        &self.xdg_output_handler
    }

    /// Get mutable reference to xdg output handler
    pub fn xdg_output_handler_mut(&mut self) -> &mut XdgOutputManagerHandler {
        &mut self.xdg_output_handler
    }

    /// Get the keyboard object ID for a client
    fn get_keyboard_id_for_client(&self, client_id: ClientId) -> Option<u32> {
        self.client_input_ids
            .get(&client_id)
            .map(|ids| ids.keyboard_id)
    }

    /// Get the pointer object ID for a client
    fn get_pointer_id_for_client(&self, client_id: ClientId) -> Option<u32> {
        self.client_input_ids
            .get(&client_id)
            .map(|ids| ids.pointer_id)
    }

    fn pointer_obj_for_client(
        &self,
        client_id: ClientId,
    ) -> crate::fusion::wayland::protocol::ObjectId {
        match self.get_pointer_id_for_client(client_id) {
            Some(id) => crate::fusion::wayland::protocol::ObjectId(id),
            None => crate::fusion::wayland::protocol::ObjectId(6),
        }
    }

    fn keyboard_obj_for_client(
        &self,
        client_id: ClientId,
    ) -> crate::fusion::wayland::protocol::ObjectId {
        match self.get_keyboard_id_for_client(client_id) {
            Some(id) => crate::fusion::wayland::protocol::ObjectId(id),
            None => crate::fusion::wayland::protocol::ObjectId(5),
        }
    }

    /// Build a wl_pointer.enter event
    fn build_pointer_enter(
        &mut self,
        client_id: ClientId,
        surface_id: SurfaceId,
        local_x: i32,
        local_y: i32,
    ) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&surface_id.0.to_le_bytes());
        payload.extend_from_slice(&((local_x as i32) << 8).to_le_bytes());
        payload.extend_from_slice(&((local_y as i32) << 8).to_le_bytes());
        WaylandMessage {
            object_id: self.pointer_obj_for_client(client_id),
            opcode: pointer_opcodes::ENTER,
            payload,
        }
    }

    /// Build a wl_pointer.leave event
    fn build_pointer_leave(
        &mut self,
        client_id: ClientId,
        surface_id: SurfaceId,
    ) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&surface_id.0.to_le_bytes());
        WaylandMessage {
            object_id: self.pointer_obj_for_client(client_id),
            opcode: pointer_opcodes::LEAVE,
            payload,
        }
    }

    /// Build a wl_pointer.motion event
    fn build_pointer_motion(
        &mut self,
        client_id: ClientId,
        local_x: i32,
        local_y: i32,
    ) -> WaylandMessage {
        let mut payload = Vec::new();
        let time: u32 = 0;
        payload.extend_from_slice(&time.to_le_bytes());
        payload.extend_from_slice(&((local_x as i32) << 8).to_le_bytes());
        payload.extend_from_slice(&((local_y as i32) << 8).to_le_bytes());
        WaylandMessage {
            object_id: self.pointer_obj_for_client(client_id),
            opcode: pointer_opcodes::MOTION,
            payload,
        }
    }

    /// Build a wl_pointer.button event
    fn build_pointer_button(
        &mut self,
        client_id: ClientId,
        button: u32,
        state: u32,
        _local_x: i32,
        _local_y: i32,
    ) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        let time: u32 = 0;
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&time.to_le_bytes());
        payload.extend_from_slice(&button.to_le_bytes());
        payload.extend_from_slice(&state.to_le_bytes());
        WaylandMessage {
            object_id: self.pointer_obj_for_client(client_id),
            opcode: pointer_opcodes::BUTTON,
            payload,
        }
    }

    /// Build a wl_pointer.axis event
    fn build_pointer_axis(&mut self, client_id: ClientId, axis: u32, value: i32) -> WaylandMessage {
        let mut payload = Vec::new();
        let time: u32 = 0;
        payload.extend_from_slice(&time.to_le_bytes());
        payload.extend_from_slice(&axis.to_le_bytes());
        payload.extend_from_slice(&((value as i32) << 8).to_le_bytes());
        WaylandMessage {
            object_id: self.pointer_obj_for_client(client_id),
            opcode: pointer_opcodes::AXIS,
            payload,
        }
    }

    /// Build a wl_pointer.frame event
    fn build_pointer_frame(&self, client_id: ClientId) -> WaylandMessage {
        WaylandMessage {
            object_id: self.pointer_obj_for_client(client_id),
            opcode: pointer_opcodes::FRAME,
            payload: Vec::new(),
        }
    }

    /// Build a wl_keyboard.enter event
    fn build_keyboard_enter(
        &mut self,
        client_id: ClientId,
        surface_id: SurfaceId,
    ) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&surface_id.0.to_le_bytes());
        let array_len: u32 = 0;
        payload.extend_from_slice(&array_len.to_le_bytes());
        WaylandMessage {
            object_id: self.keyboard_obj_for_client(client_id),
            opcode: keyboard_opcodes::ENTER,
            payload,
        }
    }

    /// Build a wl_keyboard.leave event
    fn build_keyboard_leave(
        &mut self,
        client_id: ClientId,
        surface_id: SurfaceId,
    ) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&surface_id.0.to_le_bytes());
        WaylandMessage {
            object_id: self.keyboard_obj_for_client(client_id),
            opcode: keyboard_opcodes::LEAVE,
            payload,
        }
    }

    /// Build a wl_keyboard.key event
    fn build_keyboard_key(&mut self, client_id: ClientId, key: u32, state: u32) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        let time: u32 = 0;
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&time.to_le_bytes());
        payload.extend_from_slice(&key.to_le_bytes());
        payload.extend_from_slice(&state.to_le_bytes());
        WaylandMessage {
            object_id: self.keyboard_obj_for_client(client_id),
            opcode: keyboard_opcodes::KEY,
            payload,
        }
    }

    /// Build a wl_keyboard.modifiers event
    fn build_keyboard_modifiers(
        &mut self,
        client_id: ClientId,
        depressed: u32,
        latched: u32,
        locked: u32,
        group: u32,
    ) -> WaylandMessage {
        let serial = self.next_serial;
        self.next_serial += 1;
        let mut payload = Vec::new();
        payload.extend_from_slice(&serial.to_le_bytes());
        payload.extend_from_slice(&depressed.to_le_bytes());
        payload.extend_from_slice(&latched.to_le_bytes());
        payload.extend_from_slice(&locked.to_le_bytes());
        payload.extend_from_slice(&group.to_le_bytes());
        WaylandMessage {
            object_id: self.keyboard_obj_for_client(client_id),
            opcode: keyboard_opcodes::MODIFIERS,
            payload,
        }
    }

    /// Flush pending input events to connected clients
    pub fn flush_input_events(&mut self) {
        let events = self.input_router.pending_events().to_vec();
        self.input_router.clear_pending_events();

        for event in &events {
            match *event {
                PendingInputEvent::PointerMotion(surface_id, local_x, local_y) => {
                    if let Some(client_id) =
                        self.compositor_handler.find_client_for_surface(surface_id)
                    {
                        let msg = self.build_pointer_motion(client_id, local_x, local_y);
                        let _ = self.write_message_to_client(client_id, msg);
                        let frame = self.build_pointer_frame(client_id);
                        let _ = self.write_message_to_client(client_id, frame);
                    }
                }
                PendingInputEvent::PointerButton(surface_id, button, state, local_x, local_y) => {
                    if let Some(client_id) =
                        self.compositor_handler.find_client_for_surface(surface_id)
                    {
                        let s = match state {
                            crate::fusion::wayland::seat::ButtonState::Pressed => 1u32,
                            crate::fusion::wayland::seat::ButtonState::Released => 0u32,
                        };
                        let msg = self.build_pointer_button(client_id, button, s, local_x, local_y);
                        let _ = self.write_message_to_client(client_id, msg);
                        let frame = self.build_pointer_frame(client_id);
                        let _ = self.write_message_to_client(client_id, frame);
                    }
                }
                PendingInputEvent::PointerAxis(surface_id, vertical, amount) => {
                    if let Some(client_id) =
                        self.compositor_handler.find_client_for_surface(surface_id)
                    {
                        let axis = if vertical { 0u32 } else { 1u32 };
                        let msg = self.build_pointer_axis(client_id, axis, amount);
                        let _ = self.write_message_to_client(client_id, msg);
                        let frame = self.build_pointer_frame(client_id);
                        let _ = self.write_message_to_client(client_id, frame);
                    }
                }
                PendingInputEvent::KeyboardKey(surface_id, key, pressed) => {
                    if let Some(client_id) =
                        self.compositor_handler.find_client_for_surface(surface_id)
                    {
                        let state = if pressed { 1u32 } else { 0u32 };
                        let msg = self.build_keyboard_key(client_id, key, state);
                        let _ = self.write_message_to_client(client_id, msg);
                    }
                }
                PendingInputEvent::KeyboardModifiers(surface_id, mods) => {
                    if let Some(client_id) =
                        self.compositor_handler.find_client_for_surface(surface_id)
                    {
                        let depressed = mods.to_depressed();
                        let msg = self.build_keyboard_modifiers(client_id, depressed, 0, 0, 0);
                        let _ = self.write_message_to_client(client_id, msg);
                    }
                }
            }
        }
    }
}

impl Default for WaylandServer {
    fn default() -> Self {
        Self::new()
    }
}
