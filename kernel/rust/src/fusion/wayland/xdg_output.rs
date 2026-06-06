use super::{WaylandError, WaylandResult};
use super::client::ClientId;

pub struct XdgOutputManagerHandler;

impl XdgOutputManagerHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_request(
        &mut self,
        _client_id: ClientId,
        opcode: u16,
        _payload: &[u8],
    ) -> WaylandResult<XdgOutputResponse> {
        match opcode {
            0 => Ok(XdgOutputResponse::GetXdgOutput { xdg_output_id: 0 }),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }

    pub fn handle_xdg_output_request(
        &mut self,
        _client_id: ClientId,
        _object_id: u32,
        _opcode: u16,
        _payload: &[u8],
    ) -> WaylandResult<XdgOutputDetailResponse> {
        Ok(XdgOutputDetailResponse::Done)
    }
}

pub enum XdgOutputResponse {
    GetXdgOutput { xdg_output_id: u32 },
}

pub enum XdgOutputDetailResponse {
    Done,
}
