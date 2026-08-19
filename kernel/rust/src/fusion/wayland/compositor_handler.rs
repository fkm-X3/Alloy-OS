//! Wayland compositor (wl_compositor) request handler
//!
//! Handles compositor protocol requests:
//! - create_surface: create a new surface object

use alloc::collections::BTreeMap;

use super::client::ClientId;
use super::damage::DamageRect;
use super::surface::{SurfaceId, SurfaceState};
use super::{WaylandError, WaylandResult};

/// Opcode for wl_compositor requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CompositorRequest {
    /// create_surface(id: u32) -> create surface object
    CreateSurface = 0,
}

impl TryFrom<u16> for CompositorRequest {
    type Error = WaylandError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CompositorRequest::CreateSurface),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

/// Opcode for wl_surface requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SurfaceRequest {
    /// damage(x: i32, y: i32, width: i32, height: i32)
    Damage = 0,
    /// attach(buffer: u32, x: i32, y: i32)
    Attach = 1,
    /// commit()
    Commit = 2,
    /// alloy_set_position(x: i32, y: i32) — Alloy-specific
    AlloySetPosition = 3,
    /// alloy_set_zorder(z_order: u32) — Alloy-specific
    AlloySetZOrder = 4,
    /// destroy()
    Destroy = 5,
}

impl TryFrom<u16> for SurfaceRequest {
    type Error = WaylandError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SurfaceRequest::Damage),
            1 => Ok(SurfaceRequest::Attach),
            2 => Ok(SurfaceRequest::Commit),
            3 => Ok(SurfaceRequest::AlloySetPosition),
            4 => Ok(SurfaceRequest::AlloySetZOrder),
            5 => Ok(SurfaceRequest::Destroy),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

/// Compositor handler state
pub struct CompositorHandler {
    /// Surfaces indexed by SurfaceId
    surfaces: BTreeMap<SurfaceId, SurfaceState>,
    /// Mapping from object_id to SurfaceId for quick lookup
    object_id_map: BTreeMap<u32, SurfaceId>,
    /// Mapping from client_id to their surface IDs for cleanup
    client_surfaces: BTreeMap<ClientId, BTreeMap<SurfaceId, ()>>,
    /// Reverse mapping: SurfaceId -> ClientId for event routing
    surface_to_client: BTreeMap<SurfaceId, ClientId>,
    /// Next surface ID to assign
    next_surface_id: u32,
}

impl CompositorHandler {
    /// Create a new compositor handler
    pub fn new() -> Self {
        Self {
            surfaces: BTreeMap::new(),
            object_id_map: BTreeMap::new(),
            client_surfaces: BTreeMap::new(),
            surface_to_client: BTreeMap::new(),
            next_surface_id: 1,
        }
    }

    /// Handle a wl_compositor request
    pub fn handle_compositor_request(
        &mut self,
        client_id: ClientId,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<CompositorResponse> {
        let request = CompositorRequest::try_from(opcode)?;

        match request {
            CompositorRequest::CreateSurface => self.handle_create_surface(client_id, payload),
        }
    }

    /// Handle wl_surface request
    pub fn handle_surface_request(
        &mut self,
        object_id: u32,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<SurfaceResponse> {
        let request = SurfaceRequest::try_from(opcode)?;

        // Find surface by object ID
        let surface_id = self
            .object_id_map
            .get(&object_id)
            .copied()
            .ok_or(WaylandError::ObjectNotFound)?;

        match request {
            SurfaceRequest::Damage => self.handle_damage(surface_id, payload),
            SurfaceRequest::Attach => self.handle_attach(surface_id, payload),
            SurfaceRequest::Commit => self.handle_commit(surface_id, payload),
            SurfaceRequest::AlloySetPosition => self.handle_set_position(surface_id, payload),
            SurfaceRequest::AlloySetZOrder => self.handle_set_zorder(surface_id, payload),
            SurfaceRequest::Destroy => self.handle_destroy(surface_id),
        }
    }

    /// Handle wl_compositor.create_surface request
    fn handle_create_surface(
        &mut self,
        client_id: ClientId,
        payload: &[u8],
    ) -> WaylandResult<CompositorResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }

        let object_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let object_id = u32::from_le_bytes(object_id_bytes);

        // Create surface
        let surface_id = SurfaceId(self.next_surface_id);
        self.next_surface_id = self.next_surface_id.saturating_add(1);

        let surface = SurfaceState::new(surface_id, object_id, 1);
        self.surfaces.insert(surface_id, surface);
        self.object_id_map.insert(object_id, surface_id);
        self.client_surfaces
            .entry(client_id)
            .or_default()
            .insert(surface_id, ());
        self.surface_to_client.insert(surface_id, client_id);

        crate::println!("[Wayland Compositor] Created surface");

        Ok(CompositorResponse::SurfaceCreated {
            surface_id,
            object_id,
        })
    }

    /// Handle wl_surface.damage request
    fn handle_damage(
        &mut self,
        surface_id: SurfaceId,
        payload: &[u8],
    ) -> WaylandResult<SurfaceResponse> {
        if payload.len() < 16 {
            return Err(WaylandError::ProtocolViolation);
        }

        let x_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let x = i32::from_le_bytes(x_bytes);

        let y_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let y = i32::from_le_bytes(y_bytes);

        let width_bytes = [payload[8], payload[9], payload[10], payload[11]];
        let width = i32::from_le_bytes(width_bytes);

        let height_bytes = [payload[12], payload[13], payload[14], payload[15]];
        let height = i32::from_le_bytes(height_bytes);

        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            let rect = DamageRect::new(x, y, width, height);
            surface.damage(rect);
        }

        Ok(SurfaceResponse::DamageRecorded)
    }

    /// Handle wl_surface.attach request
    fn handle_attach(
        &mut self,
        surface_id: SurfaceId,
        payload: &[u8],
    ) -> WaylandResult<SurfaceResponse> {
        // attach(buffer: u32, x: i32, y: i32)
        if payload.len() < 12 {
            return Err(WaylandError::ProtocolViolation);
        }

        let buffer_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let buffer_id = u32::from_le_bytes(buffer_bytes);

        let x_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let x = i32::from_le_bytes(x_bytes);

        let y_bytes = [payload[8], payload[9], payload[10], payload[11]];
        let y = i32::from_le_bytes(y_bytes);

        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.attach(buffer_id, x, y);
        }

        Ok(SurfaceResponse::BufferAttached)
    }

    /// Handle wl_surface.commit request
    fn handle_commit(
        &mut self,
        surface_id: SurfaceId,
        _payload: &[u8],
    ) -> WaylandResult<SurfaceResponse> {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.commit();
        }

        Ok(SurfaceResponse::Committed)
    }

    /// Handle wl_surface.destroy request
    fn handle_destroy(&mut self, surface_id: SurfaceId) -> WaylandResult<SurfaceResponse> {
        let object_id = { self.surfaces.get(&surface_id).map(|s| s.object_id) };

        if let Some(oid) = object_id {
            self.object_id_map.remove(&oid);
        }
        self.surfaces.remove(&surface_id);
        self.surface_to_client.remove(&surface_id);

        // Remove from client surface tracking
        for (_client_id, surfaces) in self.client_surfaces.iter_mut() {
            surfaces.remove(&surface_id);
        }

        Ok(SurfaceResponse::Destroyed)
    }

    /// Handle alloy_set_position request
    fn handle_set_position(
        &mut self,
        surface_id: SurfaceId,
        payload: &[u8],
    ) -> WaylandResult<SurfaceResponse> {
        if payload.len() < 8 {
            return Err(WaylandError::ProtocolViolation);
        }

        let x_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let x = i32::from_le_bytes(x_bytes);

        let y_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let y = i32::from_le_bytes(y_bytes);

        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.screen_x = x;
            surface.screen_y = y;
        }

        Ok(SurfaceResponse::PositionSet)
    }

    /// Handle alloy_set_zorder request
    fn handle_set_zorder(
        &mut self,
        surface_id: SurfaceId,
        payload: &[u8],
    ) -> WaylandResult<SurfaceResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }

        let z_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let z_order = u32::from_le_bytes(z_bytes);

        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.z_order = z_order;
        }

        Ok(SurfaceResponse::ZOrderSet)
    }

    /// Remove all surfaces belonging to a client (called on disconnect)
    pub fn clear_surface_for_client(&mut self, client_id: ClientId) {
        if let Some(surface_ids) = self.client_surfaces.remove(&client_id) {
            for surface_id in surface_ids.keys() {
                if let Some(surface) = self.surfaces.remove(surface_id) {
                    self.object_id_map.remove(&surface.object_id);
                }
                self.surface_to_client.remove(surface_id);
            }
        }
    }

    /// Find the client that owns a surface
    pub fn find_client_for_surface(&self, surface_id: SurfaceId) -> Option<ClientId> {
        self.surface_to_client.get(&surface_id).copied()
    }

    /// Get a surface by ID
    pub fn get_surface(&self, surface_id: SurfaceId) -> Option<&SurfaceState> {
        self.surfaces.get(&surface_id)
    }

    /// Get mutable reference to a surface
    pub fn get_surface_mut(&mut self, surface_id: SurfaceId) -> Option<&mut SurfaceState> {
        self.surfaces.get_mut(&surface_id)
    }

    /// Get surface count
    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    /// Iterate over all surfaces
    pub fn iter_surfaces(&self) -> impl Iterator<Item = (&SurfaceId, &SurfaceState)> {
        self.surfaces.iter()
    }
}

impl Default for CompositorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from compositor request handler
#[derive(Debug, Clone)]
pub enum CompositorResponse {
    /// Surface created
    SurfaceCreated {
        surface_id: SurfaceId,
        object_id: u32,
    },
}

/// Response from surface request handler
#[derive(Debug, Clone)]
pub enum SurfaceResponse {
    /// Damage recorded
    DamageRecorded,
    /// Buffer attached
    BufferAttached,
    /// Surface committed
    Committed,
    /// Surface destroyed
    Destroyed,
    /// Surface position set
    PositionSet,
    /// Surface z-order set
    ZOrderSet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_compositor_handler_creation() {
        let handler = CompositorHandler::new();
        assert_eq!(handler.surface_count(), 0);
    }

    #[test]
    fn test_compositor_request_conversion() {
        assert_eq!(
            CompositorRequest::try_from(0).unwrap(),
            CompositorRequest::CreateSurface
        );
        assert!(CompositorRequest::try_from(99).is_err());
    }

    #[test]
    fn test_surface_request_conversion() {
        assert_eq!(SurfaceRequest::try_from(0).unwrap(), SurfaceRequest::Damage);
        assert_eq!(SurfaceRequest::try_from(1).unwrap(), SurfaceRequest::Attach);
        assert_eq!(SurfaceRequest::try_from(2).unwrap(), SurfaceRequest::Commit);
        assert_eq!(
            SurfaceRequest::try_from(3).unwrap(),
            SurfaceRequest::Destroy
        );
    }

    #[test]
    fn test_create_surface() {
        let mut handler = CompositorHandler::new();
        let client_id = ClientId(1);
        let mut payload = Vec::new();
        payload.extend_from_slice(&3u32.to_le_bytes());

        let response = handler
            .handle_compositor_request(client_id, 0, &payload)
            .unwrap();
        assert_eq!(handler.surface_count(), 1);
        match response {
            CompositorResponse::SurfaceCreated {
                surface_id,
                object_id,
            } => {
                assert_eq!(surface_id.0, 1);
                assert_eq!(object_id, 3);
            }
        }
    }

    #[test]
    fn test_surface_damage() {
        let mut handler = CompositorHandler::new();
        let client_id = ClientId(1);

        let mut create_payload = Vec::new();
        create_payload.extend_from_slice(&3u32.to_le_bytes());
        handler
            .handle_compositor_request(client_id, 0, &create_payload)
            .unwrap();

        let mut damage_payload = Vec::new();
        damage_payload.extend_from_slice(&0i32.to_le_bytes());
        damage_payload.extend_from_slice(&0i32.to_le_bytes());
        damage_payload.extend_from_slice(&100i32.to_le_bytes());
        damage_payload.extend_from_slice(&100i32.to_le_bytes());

        let response = handler
            .handle_surface_request(3, 0, &damage_payload)
            .unwrap();
        match response {
            SurfaceResponse::DamageRecorded => {}
            _ => panic!("Expected DamageRecorded"),
        }
    }

    #[test]
    fn test_surface_commit() {
        let mut handler = CompositorHandler::new();
        let client_id = ClientId(1);

        let mut create_payload = Vec::new();
        create_payload.extend_from_slice(&3u32.to_le_bytes());
        handler
            .handle_compositor_request(client_id, 0, &create_payload)
            .unwrap();

        let response = handler.handle_surface_request(3, 2, &[]).unwrap();
        match response {
            SurfaceResponse::Committed => {}
            _ => panic!("Expected Committed"),
        }
    }

    #[test]
    fn test_clear_client_surfaces() {
        let mut handler = CompositorHandler::new();
        let client_id = ClientId(1);

        let mut payload = Vec::new();
        payload.extend_from_slice(&3u32.to_le_bytes());
        handler
            .handle_compositor_request(client_id, 0, &payload)
            .unwrap();

        assert_eq!(handler.surface_count(), 1);
        handler.clear_surface_for_client(client_id);
        assert_eq!(handler.surface_count(), 0);
    }
}
