use alloc::collections::BTreeMap;
use super::{WaylandError, WaylandResult};
use super::client::ClientId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background = 0,
    Bottom = 1,
    Top = 2,
    Overlay = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    None = 0,
    Top = 1,
    Bottom = 2,
    Left = 4,
    Right = 8,
}

#[derive(Debug, Clone)]
pub struct LayerSurfaceState {
    pub surface_object_id: u32,
    pub layer_surface_object_id: u32,
    pub layer: Layer,
    pub namespace: alloc::string::String,
    pub size: Option<(u32, u32)>,
    pub anchor: u32,
    pub exclusive_zone: i32,
    pub margin: (i32, i32, i32, i32),
}

pub struct LayerShellHandler {
    surfaces: BTreeMap<u32, LayerSurfaceState>,
}

impl LayerShellHandler {
    pub fn new() -> Self {
        Self {
            surfaces: BTreeMap::new(),
        }
    }

    pub fn handle_layer_shell_request(
        &mut self,
        _client_id: ClientId,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<LayerShellResponse> {
        match opcode {
            0 => self.handle_get_layer_surface(payload),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    fn handle_get_layer_surface(&mut self, payload: &[u8]) -> WaylandResult<LayerShellResponse> {
        if payload.len() < 24 {
            return Err(WaylandError::ProtocolViolation);
        }

        let new_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let new_id = u32::from_le_bytes(new_id_bytes);

        let surface_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let surface_object_id = u32::from_le_bytes(surface_bytes);

        let _output_bytes = [payload[8], payload[9], payload[10], payload[11]];
        let _output_id = u32::from_le_bytes(_output_bytes);

        let layer_bytes = [payload[12], payload[13], payload[14], payload[15]];
        let layer_val = u32::from_le_bytes(layer_bytes);

        let layer = match layer_val {
            0 => Layer::Background,
            1 => Layer::Bottom,
            2 => Layer::Top,
            _ => Layer::Overlay,
        };

        let ns_end = payload.iter().position(|&b| b == 0).unwrap_or(payload.len());
        let namespace = alloc::string::String::from_utf8_lossy(&payload[16..ns_end]).into_owned();

        let state = LayerSurfaceState {
            surface_object_id,
            layer_surface_object_id: new_id,
            layer,
            namespace,
            size: None,
            anchor: 0,
            exclusive_zone: 0,
            margin: (0, 0, 0, 0),
        };
        self.surfaces.insert(new_id, state);

        Ok(LayerShellResponse::LayerSurfaceCreated {
            layer_surface_id: new_id,
        })
    }

    pub fn handle_layer_surface_request(
        &mut self,
        object_id: u32,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<LayerSurfaceResponse> {
        match opcode {
            0 => self.handle_layer_surface_destroy(object_id),
            1 => self.handle_set_size(object_id, payload),
            2 => self.handle_set_anchor(object_id, payload),
            3 => self.handle_set_exclusive_zone(object_id, payload),
            4 => self.handle_set_margin(object_id, payload),
            5 => self.handle_set_keyboard_interactivity(object_id, payload),
            6 => self.handle_get_popup(object_id, payload),
            7 => self.handle_ack_configure(object_id, payload),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    fn handle_layer_surface_destroy(&mut self, object_id: u32) -> WaylandResult<LayerSurfaceResponse> {
        self.surfaces.remove(&object_id);
        Ok(LayerSurfaceResponse::Destroyed)
    }

    fn handle_set_size(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        if payload.len() < 8 {
            return Err(WaylandError::ProtocolViolation);
        }
        let w = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let h = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.size = Some((w, h));
        }
        Ok(LayerSurfaceResponse::SizeSet)
    }

    fn handle_set_anchor(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }
        let anchor = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.anchor = anchor;
        }
        Ok(LayerSurfaceResponse::AnchorSet)
    }

    fn handle_set_exclusive_zone(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }
        let zone = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.exclusive_zone = zone;
        }
        Ok(LayerSurfaceResponse::ExclusiveZoneSet)
    }

    fn handle_set_margin(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        if payload.len() < 16 {
            return Err(WaylandError::ProtocolViolation);
        }
        let top = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let right = i32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        let bottom = i32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
        let left = i32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);
        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.margin = (top, right, bottom, left);
        }
        Ok(LayerSurfaceResponse::MarginSet)
    }

    fn handle_set_keyboard_interactivity(&mut self, _object_id: u32, _payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        Ok(LayerSurfaceResponse::KeyboardInteractivitySet)
    }

    fn handle_get_popup(&mut self, _object_id: u32, _payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        Ok(LayerSurfaceResponse::PopupCreated)
    }

    fn handle_ack_configure(&mut self, _object_id: u32, _payload: &[u8]) -> WaylandResult<LayerSurfaceResponse> {
        Ok(LayerSurfaceResponse::ConfigureAcknowledged)
    }

    pub fn remove_surfaces_for_client(&mut self, _client_id: ClientId) {
    }
}

pub enum LayerShellResponse {
    LayerSurfaceCreated { layer_surface_id: u32 },
}

pub enum LayerSurfaceResponse {
    Destroyed,
    SizeSet,
    AnchorSet,
    ExclusiveZoneSet,
    MarginSet,
    KeyboardInteractivitySet,
    PopupCreated,
    ConfigureAcknowledged,
}
