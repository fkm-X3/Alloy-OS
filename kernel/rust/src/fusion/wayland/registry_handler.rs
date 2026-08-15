//! Wayland registry (wl_registry) request handler
//!
//! Handles registry protocol requests:
//! - bind: bind a global object to a client-specified ID

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::client::ClientId;
use super::globals::{GlobalRegistry, InterfaceName};
use super::protocol::{ObjectId, WaylandMessage};
use super::{WaylandError, WaylandResult};

/// Opcode for wl_registry requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RegistryRequest {
    /// bind(name: u32, interface: str, version: u32, id: u32)
    Bind = 0,
}

impl TryFrom<u16> for RegistryRequest {
    type Error = WaylandError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RegistryRequest::Bind),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

/// Opcode for wl_registry events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RegistryEvent {
    /// global(name: u32, interface: str, version: u32)
    Global = 0,
    /// global_remove(name: u32)
    GlobalRemove = 1,
}

/// Registry handler state
pub struct RegistryHandler {
    /// Global registry (shared)
    globals: GlobalRegistry,
    /// Track which clients have bound which globals
    client_bindings: BTreeMap<(ClientId, u32), u32>, // (client, global_name) -> bound_object_id
    /// Per-client registered object IDs (for cleanup)
    client_objects: BTreeMap<ClientId, Vec<u32>>,
}

impl RegistryHandler {
    /// Create a new registry handler
    pub fn new() -> Self {
        Self {
            globals: GlobalRegistry::new(),
            client_bindings: BTreeMap::new(),
            client_objects: BTreeMap::new(),
        }
    }

    /// Handle a wl_registry request
    pub fn handle_request(
        &mut self,
        client_id: ClientId,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<RegistryResponse> {
        let request = RegistryRequest::try_from(opcode)?;

        match request {
            RegistryRequest::Bind => self.handle_bind(client_id, payload),
        }
    }

    /// Handle wl_registry.bind request
    /// Binds a global object to a client-specified object ID
    fn handle_bind(
        &mut self,
        client_id: ClientId,
        payload: &[u8],
    ) -> WaylandResult<RegistryResponse> {
        // bind(name: u32, interface: str, version: u32, id: u32)
        // Payload: [name(4)][interface_str][version(4)][id(4)]
        if payload.len() < 12 {
            return Err(WaylandError::ProtocolViolation);
        }

        let name_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let global_name = u32::from_le_bytes(name_bytes);

        // Find interface string and version in payload
        let interface_start = 4;
        let mut interface_end = interface_start;
        while interface_end < payload.len() && payload[interface_end] != 0 {
            interface_end += 1;
        }

        if interface_end >= payload.len() {
            return Err(WaylandError::ProtocolViolation);
        }

        // Skip null terminator and read version/id
        let remaining = &payload[interface_end + 1..];
        if remaining.len() < 8 {
            return Err(WaylandError::ProtocolViolation);
        }

        let version_bytes = [remaining[0], remaining[1], remaining[2], remaining[3]];
        let requested_version = u32::from_le_bytes(version_bytes);

        let id_bytes = [remaining[4], remaining[5], remaining[6], remaining[7]];
        let object_id = u32::from_le_bytes(id_bytes);

        // Look up global
        let global = self
            .globals
            .get(global_name)
            .ok_or(WaylandError::ObjectNotFound)?;

        // Negotiate version: use minimum of requested and supported
        let bound_version = core::cmp::min(requested_version, global.version());
        let interface = global.interface();

        // Track the binding
        self.client_bindings
            .insert((client_id, global_name), object_id);
        self.client_objects
            .entry(client_id)
            .or_default()
            .push(object_id);

        unsafe {
            crate::println!("[Wayland Registry] Handled bind request");
        }

        Ok(RegistryResponse::Bound {
            global_name,
            object_id,
            interface,
            version: bound_version,
        })
    }

    /// Generate global events for a newly connected registry client
    pub fn get_global_events_for_client(
        &self,
        _client_id: ClientId,
        registry_id: u32,
    ) -> Vec<WaylandMessage> {
        let mut events = Vec::new();

        for (name, global) in self.globals.iter() {
            if let Ok(msg) =
                Self::emit_global(registry_id, *name, global.interface(), global.version())
            {
                events.push(msg);
            }
        }

        events
    }

    /// Emit global events for a newly connected registry
    pub fn get_global_events(&self, registry_id: u32) -> Vec<WaylandMessage> {
        let mut events = Vec::new();

        for (name, global) in self.globals.iter() {
            if let Ok(msg) =
                Self::emit_global(registry_id, *name, global.interface(), global.version())
            {
                events.push(msg);
            }
        }

        events
    }

    /// Emit a global event
    pub fn emit_global(
        registry_id: u32,
        name: u32,
        interface: InterfaceName,
        version: u32,
    ) -> WaylandResult<WaylandMessage> {
        let interface_str: &[u8] = match interface {
            InterfaceName::Compositor => b"wl_compositor\0",
            InterfaceName::Output => b"wl_output\0",
            InterfaceName::XdgShell => b"xdg_wm_base\0",
            InterfaceName::DataDeviceManager => b"wl_data_device_manager\0",
            InterfaceName::Seat => b"wl_seat\0",
            InterfaceName::Shm => b"wl_shm\0",
            InterfaceName::Subcompositor => b"wl_subcompositor\0",
            InterfaceName::LayerShell => b"zwlr_layer_shell_v1\0",
            InterfaceName::XdgOutputManager => b"zxdg_output_manager_v1\0",
        };

        let mut payload = Vec::new();
        payload.extend_from_slice(&name.to_le_bytes());
        payload.extend_from_slice(interface_str);
        payload.extend_from_slice(&version.to_le_bytes());

        Ok(WaylandMessage {
            object_id: ObjectId(registry_id),
            opcode: 0, // global event
            payload,
        })
    }

    /// Emit a global_remove event
    pub fn emit_global_remove(registry_id: u32, name: u32) -> WaylandResult<WaylandMessage> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&name.to_le_bytes());

        Ok(WaylandMessage {
            object_id: ObjectId(registry_id),
            opcode: 1, // global_remove event
            payload,
        })
    }

    /// Get the global registry for inspection
    pub fn globals(&self) -> &GlobalRegistry {
        &self.globals
    }

    /// Get global events for a newly connected client's registry
    pub fn handle_get_globals_for_client(&self, client_id: ClientId) -> Vec<WaylandMessage> {
        let registry_id = 2; // Standard registry object ID
        self.get_global_events_for_client(client_id, registry_id)
    }

    /// Remove all client state (called on disconnect)
    pub fn remove_client(&mut self, client_id: ClientId) {
        self.client_bindings.retain(|(cid, _), _| *cid != client_id);
        self.client_objects.remove(&client_id);
    }
}

impl Default for RegistryHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from registry request handler
#[derive(Debug, Clone)]
pub enum RegistryResponse {
    /// Global object bound
    Bound {
        global_name: u32,
        object_id: u32,
        interface: InterfaceName,
        version: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_handler_creation() {
        let handler = RegistryHandler::new();
        assert!(handler.globals().count() > 0);
    }

    #[test]
    fn test_registry_request_conversion() {
        assert_eq!(RegistryRequest::try_from(0).unwrap(), RegistryRequest::Bind);
        assert!(RegistryRequest::try_from(99).is_err());
    }

    #[test]
    fn test_emit_global() {
        let msg = RegistryHandler::emit_global(2, 0, InterfaceName::Compositor, 5).unwrap();
        assert_eq!(msg.object_id, ObjectId(2));
        assert_eq!(msg.opcode, 0);
    }

    #[test]
    fn test_emit_global_remove() {
        let msg = RegistryHandler::emit_global_remove(2, 0).unwrap();
        assert_eq!(msg.object_id, ObjectId(2));
        assert_eq!(msg.opcode, 1);
    }

    #[test]
    fn test_get_global_events() {
        let handler = RegistryHandler::new();
        let events = handler.get_global_events(2);
        assert!(events.len() > 0);
    }

    #[test]
    fn test_remove_client() {
        let mut handler = RegistryHandler::new();
        let client_id = ClientId(1);

        // Simulate a bind creating client objects
        handler
            .client_objects
            .entry(client_id)
            .or_default()
            .push(100);
        handler.client_bindings.insert((client_id, 0), 100);

        handler.remove_client(client_id);
        assert!(handler.client_objects.get(&client_id).is_none());
        assert!(handler
            .client_bindings
            .keys()
            .all(|(cid, _)| *cid != client_id));
    }
}
