//! Wayland input event routing
//!
//! Routes kernel input events (keyboard, mouse) to appropriate Wayland clients.
//! Handles coordinate transformation, focus management, and Z-order resolution.

use alloc::vec::Vec;
use core::fmt;

use super::surface::SurfaceId;
use super::focus::{FocusManager, SeatId};
use super::seat::{SeatManager, ButtonState, ModifierState};

/// Input routing error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRoutingError {
    /// No active seat
    NoActiveSeat,
    /// No focused surface
    NoFocusedSurface,
    /// Invalid surface
    InvalidSurface,
    /// Coordinate out of bounds
    OutOfBounds,
}

impl fmt::Display for InputRoutingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputRoutingError::NoActiveSeat => write!(f, "no active seat"),
            InputRoutingError::NoFocusedSurface => write!(f, "no focused surface"),
            InputRoutingError::InvalidSurface => write!(f, "invalid surface"),
            InputRoutingError::OutOfBounds => write!(f, "coordinate out of bounds"),
        }
    }
}

pub type InputRoutingResult<T> = Result<T, InputRoutingError>;

/// Surface position and dimensions for input hit testing
#[derive(Debug, Clone, Copy)]
pub struct SurfaceGeometry {
    /// Unique surface ID
    pub surface_id: SurfaceId,
    /// X coordinate in screen space
    pub x: i32,
    /// Y coordinate in screen space
    pub y: i32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Z-order (higher = on top)
    pub z_order: u32,
    /// Whether surface is visible
    pub visible: bool,
}

impl SurfaceGeometry {
    /// Create new surface geometry
    pub fn new(surface_id: SurfaceId, x: i32, y: i32, width: u32, height: u32, z_order: u32) -> Self {
        Self {
            surface_id,
            x,
            y,
            width,
            height,
            z_order,
            visible: true,
        }
    }

    /// Check if point is within surface bounds
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }

    /// Convert screen coordinates to surface-local coordinates
    pub fn to_local_coordinates(&self, x: i32, y: i32) -> InputRoutingResult<(i32, i32)> {
        if !self.contains_point(x, y) {
            return Err(InputRoutingError::OutOfBounds);
        }

        let local_x = x - self.x;
        let local_y = y - self.y;

        Ok((local_x, local_y))
    }
}

/// Pending input event for batching
#[derive(Debug, Clone, Copy)]
pub enum PendingInputEvent {
    /// Pointer motion: (surface_id, local_x, local_y)
    PointerMotion(SurfaceId, i32, i32),
    /// Pointer button: (surface_id, button, state, local_x, local_y)
    PointerButton(SurfaceId, u32, ButtonState, i32, i32),
    /// Pointer axis: (surface_id, vertical, amount)
    PointerAxis(SurfaceId, bool, i32),
    /// Keyboard key: (surface_id, key_code, pressed)
    KeyboardKey(SurfaceId, u32, bool),
    /// Keyboard modifiers: (surface_id, modifiers)
    KeyboardModifiers(SurfaceId, ModifierState),
}

/// Input event routing and dispatch
pub struct InputRouter {
    /// Focus management
    focus_manager: FocusManager,
    /// Seat management
    seat_manager: SeatManager,
    /// Active seat ID
    active_seat: SeatId,
    /// Pending input events for batching
    pending_events: Vec<PendingInputEvent>,
    /// Current keyboard modifiers
    current_modifiers: ModifierState,
}

impl InputRouter {
    /// Create new input router
    pub fn new() -> Self {
        let mut router = Self {
            focus_manager: FocusManager::new(),
            seat_manager: SeatManager::new(),
            active_seat: SeatId(0),
            pending_events: Vec::new(),
            current_modifiers: ModifierState::default(),
        };

        // Initialize default seat
        let _seat_id = router.seat_manager.create_seat();

        router
    }

    /// Get focus manager
    pub fn focus_manager(&self) -> &FocusManager {
        &self.focus_manager
    }

    /// Get mutable focus manager
    pub fn focus_manager_mut(&mut self) -> &mut FocusManager {
        &mut self.focus_manager
    }

    /// Get seat manager
    pub fn seat_manager(&self) -> &SeatManager {
        &self.seat_manager
    }

    /// Get mutable seat manager
    pub fn seat_manager_mut(&mut self) -> &mut SeatManager {
        &mut self.seat_manager
    }

    /// Set active seat
    pub fn set_active_seat(&mut self, seat_id: SeatId) {
        self.active_seat = seat_id;
    }

    /// Get active seat
    pub fn active_seat(&self) -> SeatId {
        self.active_seat
    }

    /// Handle pointer motion event
    pub fn handle_pointer_motion(
        &mut self,
        surfaces: &[SurfaceGeometry],
        screen_x: i32,
        screen_y: i32,
    ) -> InputRoutingResult<()> {
        // Find topmost surface under pointer
        let mut focused_surface: Option<SurfaceId> = None;
        let mut max_z = u32::MIN;

        for surface in surfaces {
            if surface.contains_point(screen_x, screen_y) && surface.z_order > max_z {
                max_z = surface.z_order;
                focused_surface = Some(surface.surface_id);
            }
        }

        // Update focus if changed
        if let Some(seat) = self.focus_manager.get_seat(self.active_seat) {
            let old_focus = seat.pointer_focus();
            if old_focus != focused_surface {
                self.focus_manager.get_seat_mut(self.active_seat)
                    .set_pointer_focus(focused_surface);
            }
        }

        // If we have focus, convert to local coords and queue event
        if let Some(surface_id) = focused_surface {
            for surface in surfaces {
                if surface.surface_id == surface_id {
                    if let Ok((local_x, local_y)) =
                        surface.to_local_coordinates(screen_x, screen_y)
                    {
                        self.pending_events.push(PendingInputEvent::PointerMotion(
                            surface_id, local_x, local_y,
                        ));
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle pointer button event
    pub fn handle_pointer_button(
        &mut self,
        surfaces: &[SurfaceGeometry],
        button: u32,
        pressed: bool,
        screen_x: i32,
        screen_y: i32,
    ) -> InputRoutingResult<()> {
        // On button press, set focus to surface under pointer
        if pressed {
            let mut focused_surface: Option<SurfaceId> = None;
            let mut max_z = u32::MIN;

            for surface in surfaces {
                if surface.contains_point(screen_x, screen_y) && surface.z_order > max_z {
                    max_z = surface.z_order;
                    focused_surface = Some(surface.surface_id);
                }
            }

            if let Some(surface_id) = focused_surface {
                self.focus_manager.get_seat_mut(self.active_seat)
                    .set_pointer_focus(Some(surface_id));
            }
        }

        // Send button event to focused surface
        if let Some(seat) = self.focus_manager.get_seat(self.active_seat) {
            if let Some(surface_id) = seat.pointer_focus() {
                for surface in surfaces {
                    if surface.surface_id == surface_id {
                        if let Ok((local_x, local_y)) =
                            surface.to_local_coordinates(screen_x, screen_y)
                        {
                            let state = if pressed {
                                ButtonState::Pressed
                            } else {
                                ButtonState::Released
                            };
                            self.pending_events.push(PendingInputEvent::PointerButton(
                                surface_id, button, state, local_x, local_y,
                            ));
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle pointer axis (scroll) event
    pub fn handle_pointer_axis(
        &mut self,
        _surfaces: &[SurfaceGeometry],
        vertical: bool,
        amount: i32,
    ) -> InputRoutingResult<()> {
        // Send scroll event to pointer-focused surface
        if let Some(seat) = self.focus_manager.get_seat(self.active_seat) {
            if let Some(surface_id) = seat.pointer_focus() {
                self.pending_events.push(PendingInputEvent::PointerAxis(
                    surface_id, vertical, amount,
                ));
            }
        }

        Ok(())
    }

    /// Handle keyboard key event
    pub fn handle_key(
        &mut self,
        key_code: u32,
        pressed: bool,
    ) -> InputRoutingResult<()> {
        // Update keyboard state based on key (simple heuristic for modifiers)
        self.update_modifiers_from_key(key_code, pressed);

        // Send key event to keyboard-focused surface
        if let Some(seat) = self.focus_manager.get_seat(self.active_seat) {
            if let Some(surface_id) = seat.keyboard_focus() {
                self.pending_events.push(PendingInputEvent::KeyboardKey(
                    surface_id, key_code, pressed,
                ));

                // Always send modifiers after key events
                self.pending_events.push(PendingInputEvent::KeyboardModifiers(
                    surface_id,
                    self.current_modifiers,
                ));
            }
        }

        Ok(())
    }

    /// Update modifiers from key code (simple heuristic)
    fn update_modifiers_from_key(&mut self, key_code: u32, pressed: bool) {
        match key_code {
            42 | 54 => self.current_modifiers.shift = pressed, // Shift
            29 | 97 => self.current_modifiers.ctrl = pressed,  // Ctrl
            56 | 100 => self.current_modifiers.alt = pressed,  // Alt
            _ => {}
        }
    }

    /// Get pending events
    pub fn pending_events(&self) -> &[PendingInputEvent] {
        &self.pending_events
    }

    /// Clear pending events
    pub fn clear_pending_events(&mut self) {
        self.pending_events.clear();
    }

    /// Set keyboard focus
    pub fn set_keyboard_focus(&mut self, surface_id: Option<SurfaceId>) {
        self.focus_manager.get_seat_mut(self.active_seat)
            .set_keyboard_focus(surface_id);
    }

    /// Get keyboard focus
    pub fn keyboard_focus(&self) -> Option<SurfaceId> {
        self.focus_manager
            .get_seat(self.active_seat)
            .and_then(|s| s.keyboard_focus())
    }

    /// Get pointer focus
    pub fn pointer_focus(&self) -> Option<SurfaceId> {
        self.focus_manager
            .get_seat(self.active_seat)
            .and_then(|s| s.pointer_focus())
    }

    /// Clear surface from focus (when surface destroyed)
    pub fn clear_surface(&mut self, surface_id: SurfaceId) {
        self.focus_manager.clear_surface(surface_id);
    }
}

impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_geometry_contains() {
        let surface = SurfaceGeometry::new(SurfaceId(1), 10, 20, 100, 100, 1);
        
        assert!(surface.contains_point(50, 70));
        assert!(!surface.contains_point(5, 20));
        assert!(!surface.contains_point(115, 50));
    }

    #[test]
    fn test_surface_geometry_conversion() {
        let surface = SurfaceGeometry::new(SurfaceId(1), 10, 20, 100, 100, 1);
        
        let result = surface.to_local_coordinates(50, 70);
        assert_eq!(result, Ok((40, 50)));
        
        let result = surface.to_local_coordinates(5, 20);
        assert!(result.is_err());
    }

    #[test]
    fn test_input_router_creation() {
        let router = InputRouter::new();
        assert_eq!(router.active_seat(), SeatId(0));
        assert_eq!(router.pending_events().len(), 0);
    }

    #[test]
    fn test_pointer_motion_routing() {
        let mut router = InputRouter::new();
        let surface = SurfaceGeometry::new(SurfaceId(1), 0, 0, 100, 100, 1);
        
        router.handle_pointer_motion(&[surface], 50, 50).ok();
        
        assert_eq!(router.pointer_focus(), Some(SurfaceId(1)));
        assert!(!router.pending_events().is_empty());
    }

    #[test]
    fn test_pointer_button_routing() {
        let mut router = InputRouter::new();
        let surface = SurfaceGeometry::new(SurfaceId(1), 0, 0, 100, 100, 1);
        
        router.handle_pointer_button(&[surface], button_codes::LEFT, true, 50, 50).ok();
        
        assert_eq!(router.pointer_focus(), Some(SurfaceId(1)));
    }

    #[test]
    fn test_keyboard_focus_management() {
        let mut router = InputRouter::new();
        
        assert_eq!(router.keyboard_focus(), None);
        
        router.set_keyboard_focus(Some(SurfaceId(1)));
        assert_eq!(router.keyboard_focus(), Some(SurfaceId(1)));
        
        router.clear_surface(SurfaceId(1));
        assert_eq!(router.keyboard_focus(), None);
    }

    #[test]
    fn test_zorder_resolution() {
        let mut router = InputRouter::new();
        
        let surface1 = SurfaceGeometry::new(SurfaceId(1), 0, 0, 100, 100, 1);
        let surface2 = SurfaceGeometry::new(SurfaceId(2), 0, 0, 100, 100, 2); // Higher z-order
        
        router.handle_pointer_motion(&[surface1, surface2], 50, 50).ok();
        
        // Should focus surface2 (higher z-order)
        assert_eq!(router.pointer_focus(), Some(SurfaceId(2)));
    }
}
