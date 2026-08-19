//! Wayland wire protocol codec and message handling
//!
//! Implements the Wayland message wire format:
//! [object_id: u32 LE][opcode: u16 LE][length: u16 LE][args...]
//!
//! Also handles core wl_display protocol messages and connection state.

use alloc::vec::Vec;

use super::buffer_handler::{ShmBufferHandler, ShmPoolHandlerResponse};
use super::client::ClientId;
use super::compositor_handler::{CompositorHandler, CompositorResponse, SurfaceResponse};
use super::display_handler::{DisplayHandler, DisplayResponse};
use super::registry_handler::{RegistryHandler, RegistryResponse};
use super::{WaylandError, WaylandResult};

/// Wayland message wire header size (in bytes)
const MESSAGE_HEADER_SIZE: usize = 8; // object_id (4) + opcode (2) + length (2)

/// Wayland protocol object IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectId(pub u32);

impl ObjectId {
    /// The display object ID (always 1)
    pub const DISPLAY: ObjectId = ObjectId(1);
}

/// Reserved object IDs for standard interfaces
pub mod object_ids {
    /// wl_display is always ID 1
    pub const DISPLAY: u32 = 1;
    /// Registry typically gets ID 2
    pub const REGISTRY_BASE: u32 = 2;
    /// Compositor typically gets ID 3+
    pub const COMPOSITOR_BASE: u32 = 3;
}

/// Wayland message structure (decoded from wire format)
#[derive(Debug, Clone)]
pub struct WaylandMessage {
    /// Target object ID
    pub object_id: ObjectId,
    /// Operation opcode
    pub opcode: u16,
    /// Raw message payload (everything after header)
    pub payload: Vec<u8>,
}

/// Interface identifier for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceId {
    /// wl_display interface
    Display,
    /// wl_registry interface
    Registry,
    /// wl_compositor interface
    Compositor,
    /// wl_surface interface
    Surface,
    /// wl_callback interface
    Callback,
    /// Unknown interface
    Unknown,
}

impl WaylandMessage {
    /// Encode message to wire format: [object_id][opcode][length][payload]
    pub fn encode(&self) -> WaylandResult<Vec<u8>> {
        // Total length: payload length + header size
        let total_length = self.payload.len() as u16 + MESSAGE_HEADER_SIZE as u16;

        // Maximum message size (reasonable limit)
        if total_length > 4096 {
            return Err(WaylandError::ProtocolViolation);
        }

        let mut buf = Vec::new();

        // object_id (u32 LE)
        buf.extend_from_slice(&self.object_id.0.to_le_bytes());

        // opcode (u16 LE)
        buf.extend_from_slice(&self.opcode.to_le_bytes());

        // length (u16 LE) - includes header
        buf.extend_from_slice(&total_length.to_le_bytes());

        // payload
        buf.extend_from_slice(&self.payload);

        Ok(buf)
    }

    /// Decode message from wire format
    pub fn decode(buf: &[u8]) -> WaylandResult<Option<Self>> {
        // Need at least header
        if buf.len() < MESSAGE_HEADER_SIZE {
            return Ok(None); // Not enough data yet
        }

        // Parse header
        let object_id_le = [buf[0], buf[1], buf[2], buf[3]];
        let object_id = ObjectId(u32::from_le_bytes(object_id_le));

        let opcode_le = [buf[4], buf[5]];
        let opcode = u16::from_le_bytes(opcode_le);

        let length_le = [buf[6], buf[7]];
        let length = u16::from_le_bytes(length_le) as usize;

        // Validate length
        if !(MESSAGE_HEADER_SIZE..=4096).contains(&length) {
            return Err(WaylandError::ProtocolViolation);
        }

        // Check if we have the full message
        if buf.len() < length {
            return Ok(None); // Not enough data yet
        }

        // Extract payload (everything after header)
        let payload = buf[MESSAGE_HEADER_SIZE..length].to_vec();

        Ok(Some(WaylandMessage {
            object_id,
            opcode,
            payload,
        }))
    }
}

/// Core Wayland protocol event types
#[derive(Debug, Clone, Copy)]
pub enum WaylandEvent {
    /// Sync callback completed
    Done(u32),
    /// Registry object created
    RegistryCreated(u32),
}

impl WaylandEvent {
    /// Encode event to message
    pub fn to_message(&self) -> WaylandResult<WaylandMessage> {
        match self {
            WaylandEvent::Done(callback_id) => {
                let mut payload = Vec::new();
                payload.extend_from_slice(&callback_id.to_le_bytes());
                Ok(WaylandMessage {
                    object_id: ObjectId(2), // Callback object
                    opcode: 0,              // done event
                    payload,
                })
            }
            WaylandEvent::RegistryCreated(registry_id) => {
                let mut payload = Vec::new();
                payload.extend_from_slice(&registry_id.to_le_bytes());
                Ok(WaylandMessage {
                    object_id: ObjectId::DISPLAY,
                    opcode: 0, // error/info event
                    payload,
                })
            }
        }
    }
}

/// Determine interface from object ID (simple heuristic for now)
pub fn identify_interface(object_id: u32) -> InterfaceId {
    match object_id {
        1 => InterfaceId::Display,
        2 => InterfaceId::Registry,
        3 => InterfaceId::Compositor,
        4..=100 => InterfaceId::Surface, // Surfaces typically use higher IDs
        _ => InterfaceId::Unknown,
    }
}

/// Protocol message handler (routing)
///
/// Dispatches incoming Wayland messages to the appropriate sub-handler
/// based on the object ID and interface. This is the central routing point
/// for all client protocol traffic.
pub struct ProtocolHandler {
    initialized: bool,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Handle an incoming message from a client, routing to the appropriate handler.
    ///
    /// Dispatches based on object_id to the relevant protocol handler:
    /// - Object 1: wl_display (core display protocol)
    /// - Object 2: wl_registry (global registry)
    /// - Object 3: wl_compositor (surface creation)
    /// - 4..=100: wl_surface (surface operations)
    /// - Higher IDs: delegated to registry handler lookup
    pub fn handle_message(
        &mut self,
        client_id: ClientId,
        message: WaylandMessage,
        display_handler: &mut DisplayHandler,
        registry_handler: &mut RegistryHandler,
        compositor_handler: &mut CompositorHandler,
        buffer_handler: &mut ShmBufferHandler,
    ) -> WaylandResult<()> {
        match message.object_id {
            ObjectId::DISPLAY => {
                let response =
                    display_handler.handle_request(client_id, message.opcode, &message.payload)?;
                self.handle_display_response(response);
            }
            ObjectId(2) => {
                // wl_registry - bind requests create client-side objects
                let response =
                    registry_handler.handle_request(client_id, message.opcode, &message.payload)?;
                self.handle_registry_response(response);
            }
            ObjectId(3) => {
                // wl_compositor
                let response = compositor_handler.handle_compositor_request(
                    client_id,
                    message.opcode,
                    &message.payload,
                )?;
                self.handle_compositor_response(response);
            }
            ObjectId(4..=100) => {
                // Could be wl_surface or wl_shm_pool
                // Try compositor surface handler first, then shm pool handler
                let surface_result = compositor_handler.handle_surface_request(
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                );
                match surface_result {
                    Ok(SurfaceResponse::DamageRecorded)
                    | Ok(SurfaceResponse::BufferAttached)
                    | Ok(SurfaceResponse::Committed)
                    | Ok(SurfaceResponse::Destroyed)
                    | Ok(SurfaceResponse::PositionSet)
                    | Ok(SurfaceResponse::ZOrderSet) => {
                        self.handle_surface_response(message.object_id.0, surface_result.unwrap());
                    }
                    Err(WaylandError::ProtocolViolation) | Err(WaylandError::ObjectNotFound) => {
                        // Not a surface request, try shm pool handler
                        let pool_result = buffer_handler.handle_shm_pool_request(
                            client_id,
                            message.object_id.0,
                            message.opcode,
                            &message.payload,
                        );
                        match pool_result {
                            Ok(ShmPoolHandlerResponse::BufferCreated { buffer_id: _ })
                            | Ok(ShmPoolHandlerResponse::Destroyed) => {
                                self.handle_shm_pool_response(
                                    message.object_id.0,
                                    pool_result.unwrap(),
                                );
                            }
                            Err(e) => {
                                crate::println!("[Wayland Protocol] Unhandled object request");
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            _ => {
                // Extended object IDs - try buffer handler for shared memory objects
                crate::println!("[Wayland Protocol] Extended object routing");
                let _ = buffer_handler.handle_shm_pool_request(
                    client_id,
                    message.object_id.0,
                    message.opcode,
                    &message.payload,
                );
            }
        }
        Ok(())
    }

    /// Handle display protocol responses (send events back to client)
    fn handle_display_response(&mut self, response: DisplayResponse) {
        match response {
            DisplayResponse::SyncAck {
                callback_id,
                callback_data,
            } => {
                let _ = (callback_id, callback_data);
            }
            DisplayResponse::RegistryCreated { registry_id } => {
                let _ = registry_id;
            }
            DisplayResponse::CapabilitiesAck { capabilities } => {
                let _ = capabilities;
            }
            DisplayResponse::CompositorAnnounced { name } => {
                let _ = name;
            }
            DisplayResponse::Error { code, message } => {
                let _ = (code, message);
            }
        }
    }

    /// Handle registry responses
    fn handle_registry_response(&mut self, response: RegistryResponse) {
        match response {
            RegistryResponse::Bound {
                global_name,
                object_id,
                interface,
                version,
            } => {
                let _ = (global_name, object_id, interface, version);
            }
        }
    }

    /// Handle compositor responses
    fn handle_compositor_response(&mut self, response: CompositorResponse) {
        match response {
            CompositorResponse::SurfaceCreated {
                surface_id,
                object_id,
            } => {
                let _ = (surface_id, object_id);
            }
        }
    }

    /// Handle surface responses
    fn handle_surface_response(&mut self, _object_id: u32, _response: SurfaceResponse) {}

    /// Handle SHM pool responses
    fn handle_shm_pool_response(&mut self, _pool_id: u32, _response: ShmPoolHandlerResponse) {}

    /// Initialize the protocol handler (called when a client connects)
    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    /// Check if handler is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_encode_decode() {
        let msg = WaylandMessage {
            object_id: ObjectId::DISPLAY,
            opcode: 0,
            payload: alloc::vec![1, 2, 3, 4],
        };

        let encoded = msg.encode().unwrap();
        let decoded = WaylandMessage::decode(&encoded).unwrap().unwrap();

        assert_eq!(decoded.object_id, ObjectId::DISPLAY);
        assert_eq!(decoded.opcode, 0);
        assert_eq!(decoded.payload, alloc::vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_message_header_size() {
        assert_eq!(size_of::<u32>(), 4);
        assert_eq!(size_of::<u16>(), 2);
        assert_eq!(MESSAGE_HEADER_SIZE, 8);
    }

    #[test]
    fn test_identify_interface() {
        assert_eq!(identify_interface(1), InterfaceId::Display);
        assert_eq!(identify_interface(2), InterfaceId::Registry);
        assert_eq!(identify_interface(3), InterfaceId::Compositor);
        assert_eq!(identify_interface(10), InterfaceId::Surface);
        assert_eq!(identify_interface(999), InterfaceId::Unknown);
    }
}
