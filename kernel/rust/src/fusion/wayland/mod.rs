//! Wayland display server implementation for Fusion
//!
//! Provides core Wayland protocol support alongside the existing Fusion display system.
//! Manages client connections via Unix domain sockets and dispatches protocol messages
//! to appropriate handlers.

use alloc::collections::BTreeMap;
use core::fmt;

pub mod socket;
pub mod protocol;
pub mod client;
pub mod globals;
pub mod surface;
pub mod display_handler;
pub mod registry_handler;
pub mod compositor_handler;
pub mod shm;
pub mod buffer_handler;
pub mod damage;
pub mod compositor_integration;
pub mod focus;
pub mod seat;
pub mod output;
pub mod input_routing;

use self::socket::UnixSocket;
use self::protocol::{WaylandMessage, ProtocolHandler};
use self::client::{ClientState, ClientId};
use self::display_handler::DisplayHandler;
use self::registry_handler::RegistryHandler;
use self::compositor_handler::CompositorHandler;
use self::buffer_handler::ShmBufferHandler;
use self::seat::SeatManager;
use self::output::OutputManager;
use self::input_routing::InputRouter;

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

impl fmt::Display for WaylandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Main Wayland server structure
pub struct WaylandServer {
    /// Unix domain socket listener
    socket: Option<UnixSocket>,
    /// Connected clients indexed by ID
    clients: BTreeMap<ClientId, ClientConnection>,
    /// Next client ID to assign
    next_client_id: u32,
    /// Protocol message handler
    protocol_handler: ProtocolHandler,
    /// Display protocol handler
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
}

impl WaylandServer {
    /// Create a new Wayland server instance
    pub fn new() -> Self {
        Self {
            socket: None,
            clients: BTreeMap::new(),
            next_client_id: 1,
            protocol_handler: ProtocolHandler::new(),
            display_handler: DisplayHandler::new(),
            registry_handler: RegistryHandler::new(),
            compositor_handler: CompositorHandler::new(),
            shm_buffer_handler: ShmBufferHandler::new(),
            seat_manager: SeatManager::new(),
            output_manager: OutputManager::new(),
            input_router: InputRouter::new(),
        }
    }

    /// Initialize the Wayland server and bind to the standard socket
    pub fn init(&mut self) -> WaylandResult<()> {
        // Create and bind Unix domain socket at standard Wayland path
        let mut socket = UnixSocket::new()?;
        socket.bind("/run/user/1000/wayland-0")?;
        socket.listen()?;

        self.socket = Some(socket);

        unsafe {
            crate::ffi::serial_print(b"[Wayland] Server initialized at /run/user/1000/wayland-0\n\0".as_ptr());
        }

        Ok(())
    }

    /// Accept a new client connection
    pub fn accept_client(&mut self) -> WaylandResult<()> {
        if let Some(ref socket) = self.socket {
            let fd = socket.accept()?;
            let client_id = ClientId(self.next_client_id);
            self.next_client_id += 1;

            let connection = ClientConnection::new(client_id, fd);
            self.clients.insert(client_id, connection);

            unsafe {
                crate::ffi::serial_print(b"[Wayland] Accepted client connection\n\0".as_ptr());
            }

            Ok(())
        } else {
            Err(WaylandError::SocketCreationFailed)
        }
    }

    /// Dispatch a message from a client
    pub fn dispatch_message(&mut self, client_id: ClientId, message: WaylandMessage) -> WaylandResult<()> {
        if let Some(_client) = self.clients.get_mut(&client_id) {
            self.protocol_handler.handle_message(client_id, message)?;
            Ok(())
        } else {
            Err(WaylandError::ObjectNotFound)
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
            // Socket will be closed when connection is dropped
            unsafe {
                crate::ffi::serial_print(b"[Wayland] Client disconnected\n\0".as_ptr());
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
}

impl Default for WaylandServer {
    fn default() -> Self {
        Self::new()
    }
}
