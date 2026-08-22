//! Wayland surface state management
//!
//! Handles surface creation, damage tracking, buffer attachment, and commit logic.
//! Surfaces represent renderable areas that clients can draw to.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt;

use super::damage::{DamageRect, DamageTracker};

/// Surface ID uniquely identifies a surface within the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u32);

impl fmt::Display for SurfaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Surface({})", self.0)
    }
}

/// Surface pending state (accumulated between commits)
#[derive(Debug, Clone)]
pub struct SurfacePendingState {
    /// Pending damage rectangles
    pub damage: Vec<DamageRect>,
    /// Attached buffer ID (0 = none)
    pub buffer_id: u32,
    /// Buffer offset (x, y)
    pub buffer_offset: (i32, i32),
}

impl SurfacePendingState {
    /// Create new pending state
    pub fn new() -> Self {
        Self {
            damage: Vec::new(),
            buffer_id: 0,
            buffer_offset: (0, 0),
        }
    }

    /// Add damage rectangle
    pub fn add_damage(&mut self, rect: DamageRect) {
        self.damage.push(rect);
    }

    /// Attach buffer
    pub fn attach_buffer(&mut self, buffer_id: u32, x: i32, y: i32) {
        self.buffer_id = buffer_id;
        self.buffer_offset = (x, y);
    }

    /// Clear pending state
    pub fn clear(&mut self) {
        self.damage.clear();
        self.buffer_id = 0;
        self.buffer_offset = (0, 0);
    }
}

impl Default for SurfacePendingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Surface current state (committed state from last commit)
#[derive(Debug, Clone)]
pub struct SurfaceCurrentState {
    /// Current damage rectangles (from last commit)
    pub damage: Vec<DamageRect>,
    /// Damage tracker for efficient merging and optimization
    pub damage_tracker: DamageTracker,
    /// Current buffer ID
    pub buffer_id: u32,
    /// Current buffer offset
    pub buffer_offset: (i32, i32),
    /// Surface width/height (from attached buffer or explicit setting)
    pub width: u32,
    pub height: u32,
}

impl SurfaceCurrentState {
    /// Create new current state
    pub fn new() -> Self {
        Self {
            damage: Vec::new(),
            damage_tracker: DamageTracker::new(0, 0),
            buffer_id: 0,
            buffer_offset: (0, 0),
            width: 0,
            height: 0,
        }
    }

    /// Set surface dimensions and update damage tracker bounds
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.damage_tracker.set_bounds(width as i32, height as i32);
    }
}

impl Default for SurfaceCurrentState {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete surface state
#[derive(Debug, Clone)]
pub struct SurfaceState {
    /// Surface ID
    pub id: SurfaceId,
    /// Wayland object ID
    pub object_id: u32,
    /// Pending state (accumulated changes)
    pub pending: SurfacePendingState,
    /// Current state (committed changes)
    pub current: SurfaceCurrentState,
    /// Protocol version
    pub version: u32,
    /// Parent surface ID (for subsurfaces)
    pub parent_id: Option<SurfaceId>,
    /// Child surface IDs (subsurfaces)
    pub children: Vec<SurfaceId>,
    /// Z-order for compositing (higher = on top)
    pub z_order: u32,
    /// Screen position X
    pub screen_x: i32,
    /// Screen position Y
    pub screen_y: i32,
}

impl SurfaceState {
    /// Create new surface state
    pub fn new(id: SurfaceId, object_id: u32, version: u32) -> Self {
        Self {
            id,
            object_id,
            pending: SurfacePendingState::new(),
            current: SurfaceCurrentState::new(),
            version,
            parent_id: None,
            children: Vec::new(),
            z_order: 0,
            screen_x: 0,
            screen_y: 0,
        }
    }

    /// Commit pending state to current state
    pub fn commit(&mut self) {
        self.current.damage = self.pending.damage.clone();
        self.current.buffer_id = self.pending.buffer_id;
        self.current.buffer_offset = self.pending.buffer_offset;

        // Update damage tracker
        for damage_rect in &self.pending.damage {
            self.current.damage_tracker.add_damage(*damage_rect);
        }
        self.current.damage_tracker.optimize();

        // If buffer changed, update dimensions (placeholder - assumes square buffers for now)
        if self.pending.buffer_id != 0 {
            // Real implementation would query buffer dimensions from buffer manager
            self.current.set_dimensions(512, 512);
            use core::sync::atomic::{AtomicBool, Ordering};
            static DIMS_WARNED: AtomicBool = AtomicBool::new(false);
            if !DIMS_WARNED.swap(true, Ordering::Relaxed) {
                crate::render_trace!(
                    "[T5] {}: commit promoted buffer {} but dims are the PLACEHOLDER 512x512 \
                     (real buffer geometry is never queried — Session 0.2 fix)",
                    self.id,
                    self.current.buffer_id
                );
            }
        }

        self.pending.clear();
    }

    /// Add damage to pending state
    pub fn damage(&mut self, rect: DamageRect) {
        self.pending.add_damage(rect);
    }

    /// Attach buffer in pending state
    pub fn attach(&mut self, buffer_id: u32, x: i32, y: i32) {
        self.pending.attach_buffer(buffer_id, x, y);
    }

    /// Get current buffer ID
    pub fn get_buffer(&self) -> u32 {
        self.current.buffer_id
    }

    /// Get current damage
    pub fn get_damage(&self) -> &[DamageRect] {
        &self.current.damage
    }

    /// Get damage tracker
    pub fn get_damage_tracker(&self) -> &DamageTracker {
        &self.current.damage_tracker
    }

    /// Get mutable reference to damage tracker
    pub fn get_damage_tracker_mut(&mut self) -> &mut DamageTracker {
        &mut self.current.damage_tracker
    }

    /// Clear damage after compositing
    pub fn clear_damage(&mut self) {
        self.current.damage.clear();
        self.current.damage_tracker.clear();
    }
}

/// Surface manager for a client
pub struct SurfaceManager {
    /// Surfaces indexed by ID
    surfaces: BTreeMap<SurfaceId, SurfaceState>,
    /// Next surface ID to assign
    next_surface_id: u32,
}

impl SurfaceManager {
    /// Create new surface manager
    pub fn new() -> Self {
        Self {
            surfaces: BTreeMap::new(),
            next_surface_id: 1,
        }
    }

    /// Create a new surface
    pub fn create_surface(&mut self, object_id: u32, version: u32) -> SurfaceId {
        let id = SurfaceId(self.next_surface_id);
        self.next_surface_id = self.next_surface_id.saturating_add(1);

        let surface = SurfaceState::new(id, object_id, version);
        self.surfaces.insert(id, surface);

        id
    }

    /// Get a surface by ID
    pub fn get(&self, id: SurfaceId) -> Option<&SurfaceState> {
        self.surfaces.get(&id)
    }

    /// Get mutable reference to a surface
    pub fn get_mut(&mut self, id: SurfaceId) -> Option<&mut SurfaceState> {
        self.surfaces.get_mut(&id)
    }

    /// Delete a surface
    pub fn delete(&mut self, id: SurfaceId) -> Option<SurfaceState> {
        self.surfaces.remove(&id)
    }

    /// Check if surface exists
    pub fn exists(&self, id: SurfaceId) -> bool {
        self.surfaces.contains_key(&id)
    }

    /// Get surface count
    pub fn count(&self) -> usize {
        self.surfaces.len()
    }

    /// Iterate over all surfaces
    pub fn iter(&self) -> impl Iterator<Item = (&SurfaceId, &SurfaceState)> {
        self.surfaces.iter()
    }

    /// Iterate mutably over all surfaces
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&SurfaceId, &mut SurfaceState)> {
        self.surfaces.iter_mut()
    }
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_rect_creation() {
        let rect = DamageRect::new(0, 0, 100, 100);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.width, 100);
    }

    #[test]
    fn test_surface_state_creation() {
        let surface = SurfaceState::new(SurfaceId(1), 3, 1);
        assert_eq!(surface.id, SurfaceId(1));
        assert_eq!(surface.object_id, 3);
        assert_eq!(surface.get_buffer(), 0);
    }

    #[test]
    fn test_surface_damage() {
        let mut surface = SurfaceState::new(SurfaceId(1), 3, 1);
        let rect = DamageRect::new(10, 10, 50, 50);
        surface.damage(rect);
        assert_eq!(surface.pending.damage.len(), 1);
    }

    #[test]
    fn test_surface_attach_and_commit() {
        let mut surface = SurfaceState::new(SurfaceId(1), 3, 1);
        surface.attach(42, 0, 0);
        surface.commit();

        assert_eq!(surface.get_buffer(), 42);
        assert_eq!(surface.pending.buffer_id, 0); // pending cleared after commit
    }

    #[test]
    fn test_surface_manager_creation() {
        let mut manager = SurfaceManager::new();
        let id = manager.create_surface(3, 1);

        assert!(manager.exists(id));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_surface_manager_delete() {
        let mut manager = SurfaceManager::new();
        let id = manager.create_surface(3, 1);

        assert!(manager.exists(id));
        manager.delete(id);
        assert!(!manager.exists(id));
    }
}
