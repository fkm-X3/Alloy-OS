//! Wayland SHM buffer protocol handlers
//!
//! Handles wl_shm and wl_shm_pool request opcodes from Wayland clients.
//! Manages buffer pool creation and buffer allocation from pools.

use super::client::ClientId;
use super::shm::{ShmFormat, ShmManager};
use super::{WaylandError, WaylandResult};

#[cfg(test)]
use alloc::vec::Vec;

/// Request opcodes for wl_shm interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ShmRequest {
    /// create_pool(fd: i32, size: u32) -> pool_id
    CreatePool = 0,
}

impl TryFrom<u16> for ShmRequest {
    type Error = WaylandError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShmRequest::CreatePool),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

/// Request opcodes for wl_shm_pool interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ShmPoolRequest {
    /// create_buffer(offset: u32, width: u32, height: u32, stride: u32, format: u32) -> buffer_id
    CreateBuffer = 0,
    /// destroy()
    Destroy = 1,
}

impl TryFrom<u16> for ShmPoolRequest {
    type Error = WaylandError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShmPoolRequest::CreateBuffer),
            1 => Ok(ShmPoolRequest::Destroy),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

/// Response from wl_shm handler
#[derive(Debug, Clone)]
pub enum ShmHandlerResponse {
    /// Pool created
    PoolCreated { pool_id: u32 },
}

/// Response from wl_shm_pool handler
#[derive(Debug, Clone)]
pub enum ShmPoolHandlerResponse {
    /// Buffer created
    BufferCreated { buffer_id: u32 },
    /// Pool destroyed
    Destroyed,
}

/// SHM buffer handler for a client
pub struct ShmBufferHandler {
    /// SHM manager for this client
    shm_manager: ShmManager,
}

impl ShmBufferHandler {
    /// Create a new SHM buffer handler
    pub fn new() -> Self {
        Self {
            shm_manager: ShmManager::new(),
        }
    }

    /// Handle wl_shm request
    pub fn handle_shm_request(
        &mut self,
        _client_id: ClientId,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<ShmHandlerResponse> {
        let request = ShmRequest::try_from(opcode)?;

        match request {
            ShmRequest::CreatePool => self.handle_create_pool(payload),
        }
    }

    /// Handle wl_shm_pool request
    pub fn handle_shm_pool_request(
        &mut self,
        _client_id: ClientId,
        pool_id: u32,
        opcode: u16,
        payload: &[u8],
    ) -> WaylandResult<ShmPoolHandlerResponse> {
        let request = ShmPoolRequest::try_from(opcode)?;

        match request {
            ShmPoolRequest::CreateBuffer => self.handle_create_buffer(pool_id, payload),
            ShmPoolRequest::Destroy => self.handle_destroy_pool(pool_id),
        }
    }

    /// Handle wl_shm.create_pool request
    /// Payload: [fd: i32][size: u32][new_id: u32]
    fn handle_create_pool(&mut self, payload: &[u8]) -> WaylandResult<ShmHandlerResponse> {
        if payload.len() < 8 {
            return Err(WaylandError::ProtocolViolation);
        }

        let fd_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let fd = i32::from_le_bytes(fd_bytes);

        let size_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let size = u32::from_le_bytes(size_bytes);

        // Don't care about new_id in payload; we assign our own pool_id
        let pool_id = self.shm_manager.create_pool(fd, size)?;

        Ok(ShmHandlerResponse::PoolCreated { pool_id })
    }

    /// Handle wl_shm_pool.create_buffer request
    /// Payload: [offset: u32][width: i32][height: i32][stride: u32][format: u32][new_id: u32]
    fn handle_create_buffer(
        &mut self,
        pool_id: u32,
        payload: &[u8],
    ) -> WaylandResult<ShmPoolHandlerResponse> {
        if payload.len() < 20 {
            return Err(WaylandError::ProtocolViolation);
        }

        let offset_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let offset = u32::from_le_bytes(offset_bytes);

        let width_bytes = [payload[4], payload[5], payload[6], payload[7]];
        let width = u32::from_le_bytes(width_bytes);

        let height_bytes = [payload[8], payload[9], payload[10], payload[11]];
        let height = u32::from_le_bytes(height_bytes);

        let stride_bytes = [payload[12], payload[13], payload[14], payload[15]];
        let stride = u32::from_le_bytes(stride_bytes);

        let format_bytes = [payload[16], payload[17], payload[18], payload[19]];
        let format_raw = u32::from_le_bytes(format_bytes);

        let format = ShmFormat::try_from(format_raw)?;

        // Create buffer in pool
        let buffer_id = self
            .shm_manager
            .create_buffer(pool_id, offset, width, height, stride, format)?;

        Ok(ShmPoolHandlerResponse::BufferCreated { buffer_id })
    }

    /// Handle wl_shm_pool.destroy request
    fn handle_destroy_pool(&mut self, pool_id: u32) -> WaylandResult<ShmPoolHandlerResponse> {
        self.shm_manager.destroy_pool(pool_id)?;
        Ok(ShmPoolHandlerResponse::Destroyed)
    }

    /// Get reference to SHM manager
    pub fn shm_manager(&self) -> &ShmManager {
        &self.shm_manager
    }

    /// Get mutable reference to SHM manager
    pub fn shm_manager_mut(&mut self) -> &mut ShmManager {
        &mut self.shm_manager
    }
}

impl Default for ShmBufferHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_request_conversion() {
        assert_eq!(ShmRequest::try_from(0).unwrap(), ShmRequest::CreatePool);
        assert!(ShmRequest::try_from(99).is_err());
    }

    #[test]
    fn test_shm_pool_request_conversion() {
        assert_eq!(
            ShmPoolRequest::try_from(0).unwrap(),
            ShmPoolRequest::CreateBuffer
        );
        assert_eq!(
            ShmPoolRequest::try_from(1).unwrap(),
            ShmPoolRequest::Destroy
        );
        assert!(ShmPoolRequest::try_from(99).is_err());
    }

    #[test]
    fn test_shm_handler_create_pool() {
        let mut handler = ShmBufferHandler::new();
        let client_id = ClientId(1);

        let mut payload = Vec::new();
        payload.extend_from_slice(&(-1i32).to_le_bytes()); // fd
        payload.extend_from_slice(&4096u32.to_le_bytes()); // size
        payload.extend_from_slice(&1u32.to_le_bytes()); // new_id (ignored)

        let response = handler.handle_shm_request(client_id, 0, &payload).unwrap();
        match response {
            ShmHandlerResponse::PoolCreated { pool_id } => {
                assert_eq!(pool_id, 1);
            }
        }
    }

    #[test]
    fn test_shm_handler_create_pool_invalid_size() {
        let mut handler = ShmBufferHandler::new();
        let client_id = ClientId(1);

        let mut payload = Vec::new();
        payload.extend_from_slice(&(-1i32).to_le_bytes()); // fd
        payload.extend_from_slice(&0u32.to_le_bytes()); // size = 0 (invalid)

        assert!(handler.handle_shm_request(client_id, 0, &payload).is_err());
    }

    #[test]
    fn test_shm_handler_create_buffer() {
        let mut handler = ShmBufferHandler::new();
        let client_id = ClientId(1);

        // Create pool first
        let mut pool_payload = Vec::new();
        pool_payload.extend_from_slice(&(-1i32).to_le_bytes());
        pool_payload.extend_from_slice(&16384u32.to_le_bytes());
        pool_payload.extend_from_slice(&1u32.to_le_bytes());
        handler
            .handle_shm_request(client_id, 0, &pool_payload)
            .unwrap();

        // Create buffer
        let mut buf_payload = Vec::new();
        buf_payload.extend_from_slice(&0u32.to_le_bytes()); // offset
        buf_payload.extend_from_slice(&64u32.to_le_bytes()); // width
        buf_payload.extend_from_slice(&64u32.to_le_bytes()); // height
        buf_payload.extend_from_slice(&256u32.to_le_bytes()); // stride
        buf_payload.extend_from_slice(&0u32.to_le_bytes()); // format (ARGB8888)
        buf_payload.extend_from_slice(&2u32.to_le_bytes()); // new_id (ignored)

        let response = handler
            .handle_shm_pool_request(client_id, 1, 0, &buf_payload)
            .unwrap();
        match response {
            ShmPoolHandlerResponse::BufferCreated { buffer_id } => {
                assert_eq!(buffer_id, 1);
            }
            _ => panic!("Expected BufferCreated"),
        }
    }

    #[test]
    fn test_shm_handler_destroy_pool() {
        let mut handler = ShmBufferHandler::new();
        let client_id = ClientId(1);

        // Create pool
        let mut pool_payload = Vec::new();
        pool_payload.extend_from_slice(&(-1i32).to_le_bytes());
        pool_payload.extend_from_slice(&4096u32.to_le_bytes());
        pool_payload.extend_from_slice(&1u32.to_le_bytes());
        handler
            .handle_shm_request(client_id, 0, &pool_payload)
            .unwrap();

        // Destroy pool
        let response = handler
            .handle_shm_pool_request(client_id, 1, 1, &[])
            .unwrap();
        match response {
            ShmPoolHandlerResponse::Destroyed => {}
            _ => panic!("Expected Destroyed"),
        }
    }

    #[test]
    fn test_shm_handler_create_buffer_invalid_stride() {
        let mut handler = ShmBufferHandler::new();
        let client_id = ClientId(1);

        // Create pool
        let mut pool_payload = Vec::new();
        pool_payload.extend_from_slice(&(-1i32).to_le_bytes());
        pool_payload.extend_from_slice(&4096u32.to_le_bytes());
        pool_payload.extend_from_slice(&1u32.to_le_bytes());
        handler
            .handle_shm_request(client_id, 0, &pool_payload)
            .unwrap();

        // Try to create buffer with invalid stride (too small for 64px ARGB8888)
        let mut buf_payload = Vec::new();
        buf_payload.extend_from_slice(&0u32.to_le_bytes()); // offset
        buf_payload.extend_from_slice(&64u32.to_le_bytes()); // width
        buf_payload.extend_from_slice(&64u32.to_le_bytes()); // height
        buf_payload.extend_from_slice(&200u32.to_le_bytes()); // stride (too small)
        buf_payload.extend_from_slice(&0u32.to_le_bytes()); // format (ARGB8888)
        buf_payload.extend_from_slice(&2u32.to_le_bytes()); // new_id

        assert!(handler
            .handle_shm_pool_request(client_id, 1, 0, &buf_payload)
            .is_err());
    }
}
