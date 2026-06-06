use alloc::collections::BTreeMap;
use super::{WaylandError, WaylandResult};
use super::client::ClientId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XdgSurfaceRole {
    None,
    Toplevel,
    Popup,
}

#[derive(Debug, Clone)]
pub struct XdgSurfaceState {
    pub surface_object_id: u32,
    pub xdg_surface_object_id: u32,
    pub role: XdgSurfaceRole,
    pub title: Option<alloc::string::String>,
    pub app_id: Option<alloc::string::String>,
    pub window_geometry: Option<(i32, i32, u32, u32)>,
    pub configured: bool,
}

pub struct XdgShellHandler {
    surfaces: BTreeMap<u32, XdgSurfaceState>,
    #[allow(dead_code)]
    next_xdg_surface_id: u32,
}

impl XdgShellHandler {
    pub fn new() -> Self {
        Self {
            surfaces: BTreeMap::new(),
            next_xdg_surface_id: 100,
        }
    }

    pub fn handle_wm_base_request(
        &mut self,
        _client_id: ClientId,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<XdgWmBaseResponse> {
        match opcode {
            0 => self.handle_create_xdg_surface(payload),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    fn handle_create_xdg_surface(&mut self, payload: &[u8]) -> WaylandResult<XdgWmBaseResponse> {
        if payload.len() < 8 {
            return Err(WaylandError::ProtocolViolation);
        }
        let id_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let surface_id = u32::from_le_bytes(id_bytes);
        let new_id_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let xdg_surface_id = u32::from_le_bytes(new_id_bytes);

        let state = XdgSurfaceState {
            surface_object_id: surface_id,
            xdg_surface_object_id: xdg_surface_id,
            role: XdgSurfaceRole::None,
            title: None,
            app_id: None,
            window_geometry: None,
            configured: false,
        };
        self.surfaces.insert(xdg_surface_id, state);

        Ok(XdgWmBaseResponse::XdgSurfaceCreated { xdg_surface_id })
    }

    pub fn handle_xdg_surface_request(
        &mut self,
        object_id: u32,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<XdgSurfaceResponse> {
        match opcode {
            0 => self.handle_destroy(object_id),
            1 => self.handle_get_toplevel(object_id, payload),
            2 => self.handle_get_popup(object_id, payload),
            3 => self.handle_set_window_geometry(object_id, payload),
            4 => self.handle_ack_configure(object_id, payload),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    fn handle_destroy(&mut self, object_id: u32) -> WaylandResult<XdgSurfaceResponse> {
        self.surfaces.remove(&object_id);
        Ok(XdgSurfaceResponse::Destroyed)
    }

    fn handle_get_toplevel(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<XdgSurfaceResponse> {
        if payload.len() < 4 {
            return Err(WaylandError::ProtocolViolation);
        }
        let new_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let toplevel_id = u32::from_le_bytes(new_id_bytes);

        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.role = XdgSurfaceRole::Toplevel;
        }

        Ok(XdgSurfaceResponse::ToplevelCreated { toplevel_id })
    }

    fn handle_get_popup(&mut self, object_id: u32, _payload: &[u8]) -> WaylandResult<XdgSurfaceResponse> {
        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.role = XdgSurfaceRole::Popup;
        }
        Ok(XdgSurfaceResponse::PopupCreated)
    }

    fn handle_set_window_geometry(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<XdgSurfaceResponse> {
        if payload.len() < 16 {
            return Err(WaylandError::ProtocolViolation);
        }
        let x = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let y = i32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        let w = u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
        let h = u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);

        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.window_geometry = Some((x, y, w, h));
        }
        Ok(XdgSurfaceResponse::GeometrySet)
    }

    fn handle_ack_configure(&mut self, object_id: u32, _payload: &[u8]) -> WaylandResult<XdgSurfaceResponse> {
        if let Some(surface) = self.surfaces.get_mut(&object_id) {
            surface.configured = true;
        }
        Ok(XdgSurfaceResponse::ConfigureAcknowledged)
    }

    pub fn handle_toplevel_request(
        &mut self,
        object_id: u32,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<ToplevelResponse> {
        match opcode {
            0 => self.handle_toplevel_destroy(object_id),
            1 => self.handle_toplevel_set_title(object_id, payload),
            2 => self.handle_toplevel_set_app_id(object_id, payload),
            3 => Ok(ToplevelResponse::MoveRequest),
            4 => Ok(ToplevelResponse::ResizeRequest),
            5 => Ok(ToplevelResponse::SetMaximized),
            6 => Ok(ToplevelResponse::SetFullscreen),
            7 => Ok(ToplevelResponse::SetMinimized),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    fn handle_toplevel_destroy(&mut self, _object_id: u32) -> WaylandResult<ToplevelResponse> {
        Ok(ToplevelResponse::Destroyed)
    }

    fn handle_toplevel_set_title(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<ToplevelResponse> {
        let title = alloc::string::String::from_utf8_lossy(payload).into_owned();
        for (_id, surface) in self.surfaces.iter_mut() {
            if surface.xdg_surface_object_id == object_id || true {
                surface.title = Some(title.clone());
                break;
            }
        }
        Ok(ToplevelResponse::TitleSet)
    }

    fn handle_toplevel_set_app_id(&mut self, object_id: u32, payload: &[u8]) -> WaylandResult<ToplevelResponse> {
        let app_id = alloc::string::String::from_utf8_lossy(payload).into_owned();
        for (_id, surface) in self.surfaces.iter_mut() {
            if surface.xdg_surface_object_id == object_id || true {
                surface.app_id = Some(app_id.clone());
                break;
            }
        }
        Ok(ToplevelResponse::AppIdSet)
    }

    pub fn handle_popup_request(&mut self, opcode: u16) -> WaylandResult<PopupResponse> {
        match opcode {
            0 => Ok(PopupResponse::Destroyed),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    pub fn remove_surfaces_for_client(&mut self, _client_id: ClientId) {
    }
}

pub enum XdgWmBaseResponse {
    XdgSurfaceCreated { xdg_surface_id: u32 },
}

pub enum XdgSurfaceResponse {
    Destroyed,
    ToplevelCreated { toplevel_id: u32 },
    PopupCreated,
    GeometrySet,
    ConfigureAcknowledged,
}

pub enum ToplevelResponse {
    Destroyed,
    TitleSet,
    AppIdSet,
    MoveRequest,
    ResizeRequest,
    SetMaximized,
    SetFullscreen,
    SetMinimized,
}

pub enum PopupResponse {
    Destroyed,
}
