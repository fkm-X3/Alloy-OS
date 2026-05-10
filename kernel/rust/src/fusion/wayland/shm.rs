//! Wayland Shared Memory (SHM) buffer management
//!
//! Implements the wl_shm protocol for shared memory buffers.
//! Clients pass file descriptors to shared memory regions, which are then
//! mapped into kernel virtual address space for the compositor to read pixel data.

use alloc::collections::BTreeMap;

use super::{WaylandError, WaylandResult};

/// Pixel format for SHM buffers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShmFormat {
    /// ARGB 8888 - 32-bit ARGB with 8 bits per channel
    Argb8888 = 0,
    /// XRGB 8888 - 32-bit XRGB with 8 bits per channel (alpha ignored)
    Xrgb8888 = 1,
    /// RGB 565 - 16-bit RGB with 5, 6, 5 bits per channel
    Rgb565 = 4,
}

impl TryFrom<u32> for ShmFormat {
    type Error = WaylandError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShmFormat::Argb8888),
            1 => Ok(ShmFormat::Xrgb8888),
            4 => Ok(ShmFormat::Rgb565),
            _ => Err(WaylandError::ProtocolViolation),
        }
    }
}

impl ShmFormat {
    /// Get bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            ShmFormat::Argb8888 | ShmFormat::Xrgb8888 => 4,
            ShmFormat::Rgb565 => 2,
        }
    }

    /// Validate stride is at least minimum
    pub fn validate_stride(&self, stride: u32, width: u32) -> bool {
        let min_stride = width.saturating_mul(self.bytes_per_pixel() as u32);
        stride >= min_stride
    }
}

/// Shared memory buffer
#[derive(Debug, Clone)]
pub struct ShmBuffer {
    /// Buffer ID (unique per client)
    pub id: u32,
    /// File descriptor for the shared memory region
    pub fd: i32,
    /// Size of the shared memory region in bytes
    pub size: u32,
    /// Buffer width in pixels
    pub width: u32,
    /// Buffer height in pixels
    pub height: u32,
    /// Stride (bytes per row)
    pub stride: u32,
    /// Pixel format
    pub format: ShmFormat,
    /// Offset into the SHM region where pixel data starts
    pub offset: u32,
    /// Kernel virtual address of mapped buffer (if mapped)
    pub kernel_vaddr: Option<u32>,
}

impl ShmBuffer {
    /// Validate buffer parameters
    pub fn validate(&self) -> WaylandResult<()> {
        // Check stride is valid for this format
        if !self.format.validate_stride(self.stride, self.width) {
            return Err(WaylandError::ProtocolViolation);
        }

        // Check buffer size doesn't overflow
        let bytes_per_pixel = self.format.bytes_per_pixel() as u32;
        let required_size = self.offset
            .checked_add(self.height.saturating_mul(self.stride))
            .ok_or(WaylandError::ProtocolViolation)?;

        if required_size > self.size {
            return Err(WaylandError::ProtocolViolation);
        }

        // Check width and height are reasonable
        if self.width == 0 || self.height == 0 || self.width > 4096 || self.height > 4096 {
            return Err(WaylandError::ProtocolViolation);
        }

        Ok(())
    }

    /// Get the total size needed including offset
    pub fn total_size(&self) -> u32 {
        self.offset.saturating_add(self.height.saturating_mul(self.stride))
    }
}

/// Shared memory pool containing multiple buffers
#[derive(Debug, Clone)]
pub struct ShmPool {
    /// Pool ID (unique per client)
    pub id: u32,
    /// File descriptor for the shared memory region
    pub fd: i32,
    /// Size of the shared memory region in bytes
    pub size: u32,
    /// Kernel virtual address of mapped pool (if mapped)
    pub kernel_vaddr: Option<u32>,
    /// Buffers in this pool (indexed by buffer ID)
    pub buffers: BTreeMap<u32, ShmBuffer>,
    /// Next buffer ID to assign
    next_buffer_id: u32,
}

impl ShmPool {
    /// Create a new SHM pool
    pub fn new(id: u32, fd: i32, size: u32) -> Self {
        Self {
            id,
            fd,
            size,
            kernel_vaddr: None,
            buffers: BTreeMap::new(),
            next_buffer_id: 1,
        }
    }

    /// Create a buffer within this pool
    pub fn create_buffer(
        &mut self,
        offset: u32,
        width: u32,
        height: u32,
        stride: u32,
        format: ShmFormat,
    ) -> WaylandResult<ShmBuffer> {
        let buffer_id = self.next_buffer_id;
        self.next_buffer_id = self.next_buffer_id.saturating_add(1);

        let buffer = ShmBuffer {
            id: buffer_id,
            fd: self.fd,
            size: self.size,
            width,
            height,
            stride,
            format,
            offset,
            kernel_vaddr: self.kernel_vaddr,
        };

        // Validate before adding
        buffer.validate()?;

        self.buffers.insert(buffer_id, buffer.clone());
        Ok(buffer)
    }

    /// Get a buffer by ID
    pub fn get_buffer(&self, buffer_id: u32) -> Option<&ShmBuffer> {
        self.buffers.get(&buffer_id)
    }

    /// Destroy a buffer
    pub fn destroy_buffer(&mut self, buffer_id: u32) -> WaylandResult<()> {
        if self.buffers.remove(&buffer_id).is_some() {
            Ok(())
        } else {
            Err(WaylandError::ObjectNotFound)
        }
    }
}

/// SHM buffer manager per client
pub struct ShmManager {
    /// Pools indexed by ID
    pools: BTreeMap<u32, ShmPool>,
    /// Buffers indexed by ID (for quick lookup across pools)
    buffers: BTreeMap<u32, (u32, u32)>, // (pool_id, buffer_id)
    /// Next pool ID to assign
    next_pool_id: u32,
}

impl ShmManager {
    /// Create a new SHM manager
    pub fn new() -> Self {
        Self {
            pools: BTreeMap::new(),
            buffers: BTreeMap::new(),
            next_pool_id: 1,
        }
    }

    /// Create a new SHM pool
    pub fn create_pool(&mut self, fd: i32, size: u32) -> WaylandResult<u32> {
        let pool_id = self.next_pool_id;
        self.next_pool_id = self.next_pool_id.saturating_add(1);

        // Validate size
        if size == 0 || size > 256 * 1024 * 1024 {
            // Don't allow pools larger than 256MB
            return Err(WaylandError::ProtocolViolation);
        }

        let pool = ShmPool::new(pool_id, fd, size);
        self.pools.insert(pool_id, pool);

        Ok(pool_id)
    }

    /// Create a buffer in a pool
    pub fn create_buffer(
        &mut self,
        pool_id: u32,
        offset: u32,
        width: u32,
        height: u32,
        stride: u32,
        format: ShmFormat,
    ) -> WaylandResult<u32> {
        let pool = self.pools.get_mut(&pool_id)
            .ok_or(WaylandError::ObjectNotFound)?;

        let buffer = pool.create_buffer(offset, width, height, stride, format)?;
        let buffer_id = buffer.id;

        self.buffers.insert(buffer_id, (pool_id, buffer_id));

        Ok(buffer_id)
    }

    /// Get a buffer by ID
    pub fn get_buffer(&self, buffer_id: u32) -> Option<&ShmBuffer> {
        if let Some((pool_id, buffer_id)) = self.buffers.get(&buffer_id) {
            self.pools.get(pool_id).and_then(|p| p.get_buffer(*buffer_id))
        } else {
            None
        }
    }

    /// Destroy a pool
    pub fn destroy_pool(&mut self, pool_id: u32) -> WaylandResult<()> {
        if let Some(pool) = self.pools.remove(&pool_id) {
            // Remove all buffers from this pool
            for buffer_id in pool.buffers.keys() {
                self.buffers.remove(buffer_id);
            }
            Ok(())
        } else {
            Err(WaylandError::ObjectNotFound)
        }
    }

    /// Destroy a buffer
    pub fn destroy_buffer(&mut self, buffer_id: u32) -> WaylandResult<()> {
        if let Some((pool_id, buf_id)) = self.buffers.remove(&buffer_id) {
            if let Some(pool) = self.pools.get_mut(&pool_id) {
                pool.destroy_buffer(buf_id)?;
            }
            Ok(())
        } else {
            Err(WaylandError::ObjectNotFound)
        }
    }

    /// Get a pool by ID
    pub fn get_pool(&self, pool_id: u32) -> Option<&ShmPool> {
        self.pools.get(&pool_id)
    }

    /// Get mutable reference to a pool
    pub fn get_pool_mut(&mut self, pool_id: u32) -> Option<&mut ShmPool> {
        self.pools.get_mut(&pool_id)
    }
}

impl Default for ShmManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_format_bytes_per_pixel() {
        assert_eq!(ShmFormat::Argb8888.bytes_per_pixel(), 4);
        assert_eq!(ShmFormat::Xrgb8888.bytes_per_pixel(), 4);
        assert_eq!(ShmFormat::Rgb565.bytes_per_pixel(), 2);
    }

    #[test]
    fn test_shm_format_stride_validation() {
        // 800px wide ARGB8888 needs at least 3200 bytes stride
        assert!(ShmFormat::Argb8888.validate_stride(3200, 800));
        assert!(!ShmFormat::Argb8888.validate_stride(3199, 800));

        // 800px wide RGB565 needs at least 1600 bytes stride
        assert!(ShmFormat::Rgb565.validate_stride(1600, 800));
        assert!(!ShmFormat::Rgb565.validate_stride(1599, 800));
    }

    #[test]
    fn test_shm_buffer_validation() {
        let buffer = ShmBuffer {
            id: 1,
            fd: -1,
            size: 4096,
            width: 64,
            height: 64,
            stride: 256, // 64 * 4 = 256
            format: ShmFormat::Argb8888,
            offset: 0,
            kernel_vaddr: None,
        };

        assert!(buffer.validate().is_ok());
    }

    #[test]
    fn test_shm_buffer_validation_stride_too_small() {
        let buffer = ShmBuffer {
            id: 1,
            fd: -1,
            size: 4096,
            width: 64,
            height: 64,
            stride: 200, // Too small for 64px wide ARGB8888
            format: ShmFormat::Argb8888,
            offset: 0,
            kernel_vaddr: None,
        };

        assert!(buffer.validate().is_err());
    }

    #[test]
    fn test_shm_buffer_validation_exceeds_pool() {
        let buffer = ShmBuffer {
            id: 1,
            fd: -1,
            size: 1000, // Pool is only 1000 bytes
            width: 64,
            height: 64,
            stride: 256,
            format: ShmFormat::Argb8888,
            offset: 0,
            kernel_vaddr: None,
        };

        assert!(buffer.validate().is_err()); // 64 * 256 = 16384 > 1000
    }

    #[test]
    fn test_shm_pool_creation() {
        let pool = ShmPool::new(1, -1, 4096);
        assert_eq!(pool.id, 1);
        assert_eq!(pool.size, 4096);
        assert_eq!(pool.buffers.len(), 0);
    }

    #[test]
    fn test_shm_pool_create_buffer() {
        let mut pool = ShmPool::new(1, -1, 16384);
        let buffer = pool.create_buffer(0, 64, 64, 256, ShmFormat::Argb8888).unwrap();
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
        assert_eq!(pool.buffers.len(), 1);
    }

    #[test]
    fn test_shm_manager_pool_lifecycle() {
        let mut mgr = ShmManager::new();
        let pool_id = mgr.create_pool(-1, 4096).unwrap();
        assert_eq!(pool_id, 1);
        assert!(mgr.get_pool(pool_id).is_some());
        assert!(mgr.destroy_pool(pool_id).is_ok());
        assert!(mgr.get_pool(pool_id).is_none());
    }

    #[test]
    fn test_shm_manager_buffer_lifecycle() {
        let mut mgr = ShmManager::new();
        let pool_id = mgr.create_pool(-1, 16384).unwrap();
        let buffer_id = mgr.create_buffer(pool_id, 0, 64, 64, 256, ShmFormat::Argb8888).unwrap();
        assert!(mgr.get_buffer(buffer_id).is_some());
        assert!(mgr.destroy_buffer(buffer_id).is_ok());
        assert!(mgr.get_buffer(buffer_id).is_none());
    }
}
