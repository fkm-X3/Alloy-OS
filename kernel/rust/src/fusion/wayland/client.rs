//! Per-client state tracking and object registry
//!
//! Manages individual client connection state, including object ID mapping
//! and resource tracking for each connected Wayland client.

use alloc::collections::BTreeMap;
use core::fmt;

/// Unique client identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClientId(pub u32);

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client({})", self.0)
    }
}

/// Wayland object handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectHandle {
    /// Object ID allocated for this client
    id: u32,
}

impl ObjectHandle {
    /// Create a new object handle
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    /// Get the object ID
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// Type of Wayland object
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// wl_display - display server
    Display,
    /// wl_registry - registry of available interfaces
    Registry,
    /// wl_compositor - compositor for surface rendering
    Compositor,
    /// wl_surface - renderable surface
    Surface,
    /// wl_buffer - shared memory or hardware buffer
    Buffer,
    /// wl_callback - synchronization callback
    Callback,
    /// wl_output - display output/monitor
    Output,
    /// wl_seat - input device seat
    Seat,
    /// xdg_wm_base - xdg windowing shell
    XdgWmBase,
    /// xdg_surface - xdg surface
    XdgSurface,
    /// xdg_toplevel - xdg toplevel window
    XdgToplevel,
    /// xdg_popup - xdg popup window
    XdgPopup,
    /// zwlr_layer_shell_v1 - layer shell
    LayerShell,
    /// zwlr_layer_surface_v1 - layer surface
    LayerSurface,
    /// zxdg_output_manager_v1 - xdg output manager
    XdgOutputManager,
    /// zxdg_output_v1 - xdg output
    XdgOutput,
    /// Custom/Unknown type
    Custom,
}

/// Object registry entry
#[derive(Debug, Clone)]
pub struct ObjectEntry {
    /// Object type
    ty: ObjectType,
    /// Version number
    version: u32,
    /// Optional user data pointer (for storing handler state)
    user_data: u32,
}

impl ObjectEntry {
    /// Create a new object entry
    pub fn new(ty: ObjectType, version: u32) -> Self {
        Self {
            ty,
            version,
            user_data: 0,
        }
    }

    /// Set user data
    pub fn set_user_data(&mut self, data: u32) {
        self.user_data = data;
    }

    /// Get user data
    pub fn user_data(&self) -> u32 {
        self.user_data
    }

    /// Get object type
    pub fn object_type(&self) -> ObjectType {
        self.ty
    }

    /// Get version
    pub fn version(&self) -> u32 {
        self.version
    }
}

/// Per-client state tracker
pub struct ClientState {
    /// Object registry: object_id -> object_entry
    objects: BTreeMap<u32, ObjectEntry>,
    /// Next object ID to allocate (clients get IDs > 1)
    next_object_id: u32,
    /// Client protocol version
    protocol_version: u32,
    /// Whether client has completed handshake
    handshake_complete: bool,
}

impl ClientState {
    /// Create a new client state
    pub fn new() -> Self {
        let mut state = Self {
            objects: BTreeMap::new(),
            next_object_id: 2, // 1 is reserved for wl_display
            protocol_version: 1,
            handshake_complete: false,
        };

        // Add the wl_display object (ID 1)
        state.objects.insert(
            1,
            ObjectEntry::new(ObjectType::Display, 1),
        );

        state
    }

    /// Register a new object for this client
    pub fn register_object(&mut self, ty: ObjectType, version: u32) -> ObjectHandle {
        let id = self.next_object_id;
        self.next_object_id = self.next_object_id.saturating_add(1);

        let entry = ObjectEntry::new(ty, version);
        self.objects.insert(id, entry);

        ObjectHandle::new(id)
    }

    /// Get an object from the registry
    pub fn get_object(&self, id: u32) -> Option<&ObjectEntry> {
        self.objects.get(&id)
    }

    /// Get mutable reference to an object
    pub fn get_object_mut(&mut self, id: u32) -> Option<&mut ObjectEntry> {
        self.objects.get_mut(&id)
    }

    /// Remove an object from the registry
    pub fn unregister_object(&mut self, id: u32) -> Option<ObjectEntry> {
        self.objects.remove(&id)
    }

    /// Check if an object exists
    pub fn has_object(&self, id: u32) -> bool {
        self.objects.contains_key(&id)
    }

    /// Get the number of registered objects
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Mark handshake as complete
    pub fn set_handshake_complete(&mut self, complete: bool) {
        self.handshake_complete = complete;
    }

    /// Check if handshake is complete
    pub fn is_handshake_complete(&self) -> bool {
        self.handshake_complete
    }

    /// Set protocol version
    pub fn set_protocol_version(&mut self, version: u32) {
        self.protocol_version = version;
    }

    /// Get protocol version
    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    /// Iterate over all objects
    pub fn iter_objects(&self) -> impl Iterator<Item = (u32, &ObjectEntry)> {
        self.objects.iter().map(|(id, entry)| (*id, entry))
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_ordering() {
        let id1 = ClientId(1);
        let id2 = ClientId(2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_object_registry() {
        let mut state = ClientState::new();
        assert!(state.has_object(1)); // wl_display
        assert_eq!(state.object_count(), 1);

        let handle = state.register_object(ObjectType::Registry, 1);
        assert!(state.has_object(handle.id()));
        assert_eq!(state.object_count(), 2);
    }

    #[test]
    fn test_object_unregister() {
        let mut state = ClientState::new();
        let handle = state.register_object(ObjectType::Surface, 1);
        let id = handle.id();

        assert!(state.has_object(id));
        let removed = state.unregister_object(id);
        assert!(removed.is_some());
        assert!(!state.has_object(id));
    }

    #[test]
    fn test_handshake_state() {
        let mut state = ClientState::new();
        assert!(!state.is_handshake_complete());

        state.set_handshake_complete(true);
        assert!(state.is_handshake_complete());
    }
}
