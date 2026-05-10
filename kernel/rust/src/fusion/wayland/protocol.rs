//! Wayland wire protocol codec and message handling
//!
//! Implements the Wayland message wire format:
//! [object_id: u32 LE][opcode: u16 LE][length: u16 LE][args...]
//!
//! Also handles core wl_display protocol messages and connection state.

use alloc::vec::Vec;

use super::{WaylandError, WaylandResult};
use super::client::ClientId;

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
        if length < MESSAGE_HEADER_SIZE || length > 4096 {
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
                    opcode: 0, // done event
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

/// Protocol message handler
pub struct ProtocolHandler {
    /// Handler state
    initialized: bool,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Handle an incoming message from a client
    pub fn handle_message(&mut self, client_id: ClientId, message: WaylandMessage) -> WaylandResult<()> {
        match message.object_id {
            ObjectId::DISPLAY => {
                self.handle_display_request(client_id, message)?;
            }
            _ => {
                // Stub handler for other objects
                unsafe {
                    crate::ffi::serial_print(b"[Wayland Protocol] Unhandled object request\n\0".as_ptr());
                }
            }
        }
        Ok(())
    }

    /// Handle wl_display requests
    fn handle_display_request(&mut self, _client_id: ClientId, message: WaylandMessage) -> WaylandResult<()> {
        let opcode = message.opcode;

        match opcode {
            0 => {
                // Sync request: sync(callback_id)
                if message.payload.len() < 4 {
                    return Err(WaylandError::ProtocolViolation);
                }

                let _callback_id_le = [
                    message.payload[0],
                    message.payload[1],
                    message.payload[2],
                    message.payload[3],
                ];

                unsafe {
                    crate::ffi::serial_print(b"[Wayland Protocol] Handled wl_display.sync\n\0".as_ptr());
                }

                // Would send Done event back to client
                // For now, just log the request
            }
            1 => {
                // GetRegistry request: get_registry(registry_id)
                if message.payload.len() < 4 {
                    return Err(WaylandError::ProtocolViolation);
                }

                let _registry_id_le = [
                    message.payload[0],
                    message.payload[1],
                    message.payload[2],
                    message.payload[3],
                ];

                unsafe {
                    crate::ffi::serial_print(b"[Wayland Protocol] Handled wl_display.get_registry\n\0".as_ptr());
                }

                // Would create registry object and send back to client
                // For now, just log the request
            }
            _ => {
                return Err(WaylandError::ProtocolViolation);
            }
        }

        Ok(())
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
}
