//! Wayland display (wl_display) request handler
//!
//! Handles core display protocol requests:
//! - sync: create synchronization callback
//! - get_registry: provide registry of available global objects

use alloc::vec::Vec;

use super::{WaylandError, WaylandResult};
use super::client::ClientId;
use super::protocol::{ObjectId, WaylandMessage};

/// Opcode for wl_display requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DisplayRequest {
    /// sync(registry_id: u32) -> create callback object
    Sync = 0,
    /// get_registry(registry_id: u32) -> create registry object
    GetRegistry = 1,
}

impl TryFrom<u16> for DisplayRequest {
    type Error = WaylandError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DisplayRequest::Sync),
            1 => Ok(DisplayRequest::GetRegistry),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

/// Opcode for wl_callback events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CallbackEvent {
    /// done(callback_data: u32)
    Done = 0,
}

/// Display handler state
pub struct DisplayHandler {
    /// Pending callbacks waiting to be resolved
    pending_callbacks: Vec<CallbackInfo>,
}

/// Information about a pending callback
#[derive(Debug, Clone)]
pub struct CallbackInfo {
    /// Client that requested the callback
    pub client_id: ClientId,
    /// Callback object ID
    pub callback_id: u32,
    /// Callback data (serial number)
    pub callback_data: u32,
}

impl DisplayHandler {
    /// Create a new display handler
    pub fn new() -> Self {
        Self {
            pending_callbacks: Vec::new(),
        }
    }

    /// Handle a wl_display request
    pub fn handle_request(
        &mut self,
        client_id: ClientId,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<DisplayResponse> {
        let request = DisplayRequest::try_from(opcode)?;

        match request {
            DisplayRequest::Sync => {
                self.handle_sync(client_id, payload)
            }
            DisplayRequest::GetRegistry => {
                self.handle_get_registry(client_id, payload)
            }
        }
    }

    /// Handle wl_display.sync request
    /// Creates a callback object that will emit a done event
    fn handle_sync(
        &mut self,
        client_id: ClientId,
        payload: &[u8],
    ) -> WaylandResult<DisplayResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }

        // Extract callback object ID
        let callback_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let callback_id = u32::from_le_bytes(callback_id_bytes);

        // Queue callback for done event
        // In a real implementation, this would be tied to frame timing
        let callback_data = 0; // Serial number / timestamp
        self.pending_callbacks.push(CallbackInfo {
            client_id,
            callback_id,
            callback_data,
        });

        unsafe {
            crate::ffi::serial_print(b"[Wayland Display] Handled sync request\n\0".as_ptr());
        }

        Ok(DisplayResponse::SyncAck {
            callback_id,
            callback_data,
        })
    }

    /// Handle wl_display.get_registry request
    /// Creates a registry object and advertises global objects
    fn handle_get_registry(
        &mut self,
        _client_id: ClientId,
        payload: &[u8],
    ) -> WaylandResult<DisplayResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }

        // Extract registry object ID
        let registry_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let registry_id = u32::from_le_bytes(registry_id_bytes);

        unsafe {
            crate::ffi::serial_print(b"[Wayland Display] Handled get_registry request\n\0".as_ptr());
        }

        Ok(DisplayResponse::RegistryCreated { registry_id })
    }

    /// Get next pending callback to process
    pub fn get_pending_callback(&mut self) -> Option<CallbackInfo> {
        if self.pending_callbacks.is_empty() {
            None
        } else {
            Some(self.pending_callbacks.remove(0))
        }
    }

    /// Emit callback done event
    pub fn emit_callback_done(callback_id: u32, callback_data: u32) -> WaylandResult<WaylandMessage> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&callback_data.to_le_bytes());

        Ok(WaylandMessage {
            object_id: ObjectId(callback_id),
            opcode: 0, // done event
            payload,
        })
    }
}

impl Default for DisplayHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from display request handler
#[derive(Debug, Clone)]
pub enum DisplayResponse {
    /// Sync callback created
    SyncAck {
        callback_id: u32,
        callback_data: u32,
    },
    /// Registry created
    RegistryCreated {
        registry_id: u32,
    },
    /// Capabilities acknowledgment
    CapabilitiesAck {
        capabilities: u32,
    },
    /// Compositor announced
    CompositorAnnounced {
        name: u32,
    },
    /// Error response
    Error {
        code: u32,
        message: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_request_conversion() {
        assert_eq!(DisplayRequest::try_from(0).unwrap(), DisplayRequest::Sync);
        assert_eq!(DisplayRequest::try_from(1).unwrap(), DisplayRequest::GetRegistry);
        assert!(DisplayRequest::try_from(99).is_err());
    }

    #[test]
    fn test_display_handler_creation() {
        let handler = DisplayHandler::new();
        assert_eq!(handler.pending_callbacks.len(), 0);
    }

    #[test]
    fn test_sync_request() {
        let mut handler = DisplayHandler::new();
        let client_id = ClientId(1);
        let mut payload = Vec::new();
        payload.extend_from_slice(&42u32.to_le_bytes());

        let response = handler.handle_sync(client_id, &payload).unwrap();
        match response {
            DisplayResponse::SyncAck { callback_id, .. } => {
                assert_eq!(callback_id, 42);
            }
            _ => panic!("Expected SyncAck"),
        }
    }

    #[test]
    fn test_get_registry_request() {
        let mut handler = DisplayHandler::new();
        let client_id = ClientId(1);
        let mut payload = Vec::new();
        payload.extend_from_slice(&2u32.to_le_bytes());

        let response = handler.handle_get_registry(client_id, &payload).unwrap();
        match response {
            DisplayResponse::RegistryCreated { registry_id } => {
                assert_eq!(registry_id, 2);
            }
            _ => panic!("Expected RegistryCreated"),
        }
    }

    #[test]
    fn test_emit_callback_done() {
        let msg = DisplayHandler::emit_callback_done(42, 100).unwrap();
        assert_eq!(msg.object_id, ObjectId(42));
        assert_eq!(msg.opcode, 0);
    }
}
