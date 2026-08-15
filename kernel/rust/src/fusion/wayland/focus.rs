//! Wayland focus management
//!
//! Tracks pointer and keyboard focus, managing which surfaces receive input events.
//! Ensures only one surface has focus at a time and handles focus transitions.

use super::surface::SurfaceId;
use alloc::collections::BTreeMap;

/// Wayland seat identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeatId(pub u32);

/// Focus state tracking for a Wayland seat
pub struct FocusState {
    /// Current surface with pointer focus
    pointer_focus: Option<SurfaceId>,
    /// Current surface with keyboard focus
    keyboard_focus: Option<SurfaceId>,
    /// Surfaces that have received enter/focus events
    active_surfaces: BTreeMap<SurfaceId, u32>, // surface_id -> ref count
}

impl FocusState {
    /// Create a new focus state
    pub fn new() -> Self {
        Self {
            pointer_focus: None,
            keyboard_focus: None,
            active_surfaces: BTreeMap::new(),
        }
    }

    /// Get current pointer focus
    pub fn pointer_focus(&self) -> Option<SurfaceId> {
        self.pointer_focus
    }

    /// Get current keyboard focus
    pub fn keyboard_focus(&self) -> Option<SurfaceId> {
        self.keyboard_focus
    }

    /// Set pointer focus
    pub fn set_pointer_focus(&mut self, surface: Option<SurfaceId>) {
        if self.pointer_focus != surface {
            // Dereference old surface
            if let Some(old) = self.pointer_focus {
                if let Some(count) = self.active_surfaces.get_mut(&old) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.active_surfaces.remove(&old);
                    }
                }
            }

            // Reference new surface
            if let Some(new) = surface {
                *self.active_surfaces.entry(new).or_insert(0) += 1;
            }

            self.pointer_focus = surface;
        }
    }

    /// Set keyboard focus
    pub fn set_keyboard_focus(&mut self, surface: Option<SurfaceId>) {
        if self.keyboard_focus != surface {
            // Dereference old surface
            if let Some(old) = self.keyboard_focus {
                if let Some(count) = self.active_surfaces.get_mut(&old) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.active_surfaces.remove(&old);
                    }
                }
            }

            // Reference new surface
            if let Some(new) = surface {
                *self.active_surfaces.entry(new).or_insert(0) += 1;
            }

            self.keyboard_focus = surface;
        }
    }

    /// Clear all focus from a surface (when surface is destroyed)
    pub fn clear_surface(&mut self, surface: SurfaceId) {
        if self.pointer_focus == Some(surface) {
            self.set_pointer_focus(None);
        }
        if self.keyboard_focus == Some(surface) {
            self.set_keyboard_focus(None);
        }
    }

    /// Check if a surface has any focus
    pub fn is_active(&self, surface: SurfaceId) -> bool {
        self.active_surfaces.contains_key(&surface)
    }

    /// Get all active surfaces
    pub fn active_surfaces(&self) -> impl Iterator<Item = &SurfaceId> {
        self.active_surfaces.keys()
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-seat focus manager
pub struct FocusManager {
    /// Focus state per seat
    seats: BTreeMap<SeatId, FocusState>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            seats: BTreeMap::new(),
        }
    }

    /// Get or create a seat's focus state
    pub fn get_seat_mut(&mut self, seat_id: SeatId) -> &mut FocusState {
        self.seats.entry(seat_id).or_default()
    }

    /// Get a seat's focus state
    pub fn get_seat(&self, seat_id: SeatId) -> Option<&FocusState> {
        self.seats.get(&seat_id)
    }

    /// Clear all focus from a surface across all seats
    pub fn clear_surface(&mut self, surface: SurfaceId) {
        for focus_state in self.seats.values_mut() {
            focus_state.clear_surface(surface);
        }
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_state_creation() {
        let state = FocusState::new();
        assert_eq!(state.pointer_focus(), None);
        assert_eq!(state.keyboard_focus(), None);
    }

    #[test]
    fn test_set_pointer_focus() {
        let mut state = FocusState::new();
        let surface = SurfaceId(1);

        state.set_pointer_focus(Some(surface));
        assert_eq!(state.pointer_focus(), Some(surface));
        assert!(state.is_active(surface));
    }

    #[test]
    fn test_focus_transition() {
        let mut state = FocusState::new();
        let surface1 = SurfaceId(1);
        let surface2 = SurfaceId(2);

        state.set_pointer_focus(Some(surface1));
        assert!(state.is_active(surface1));

        state.set_pointer_focus(Some(surface2));
        assert!(!state.is_active(surface1));
        assert!(state.is_active(surface2));
    }

    #[test]
    fn test_clear_surface() {
        let mut state = FocusState::new();
        let surface = SurfaceId(1);

        state.set_pointer_focus(Some(surface));
        state.set_keyboard_focus(Some(surface));

        state.clear_surface(surface);
        assert_eq!(state.pointer_focus(), None);
        assert_eq!(state.keyboard_focus(), None);
        assert!(!state.is_active(surface));
    }

    #[test]
    fn test_focus_manager() {
        let mut manager = FocusManager::new();
        let seat = SeatId(0);
        let surface = SurfaceId(1);

        let seat_state = manager.get_seat_mut(seat);
        seat_state.set_pointer_focus(Some(surface));

        assert_eq!(
            manager.get_seat(seat).unwrap().pointer_focus(),
            Some(surface)
        );

        manager.clear_surface(surface);
        assert_eq!(manager.get_seat(seat).unwrap().pointer_focus(), None);
    }
}
