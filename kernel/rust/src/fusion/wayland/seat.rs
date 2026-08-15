//! Wayland seat and input device interfaces
//!
//! Implements wl_seat, wl_pointer, and wl_keyboard interfaces for input device handling.
//! Manages pointer motion, buttons, scroll events and keyboard key/modifiers events.

use super::focus::SeatId;
use super::surface::SurfaceId;
use alloc::collections::BTreeMap;

/// Wayland seat capabilities
#[derive(Debug, Clone, Copy)]
pub struct SeatCapabilities {
    pub pointer: bool,
    pub keyboard: bool,
    pub touch: bool,
}

impl SeatCapabilities {
    /// Create capabilities with default enabled inputs
    pub fn default_inputs() -> Self {
        Self {
            pointer: true,
            keyboard: true,
            touch: false,
        }
    }

    /// Convert to capability bitmask (Wayland protocol format)
    pub fn to_bitmask(&self) -> u32 {
        let mut mask = 0u32;
        if self.pointer {
            mask |= 1 << 0; // WL_SEAT_CAPABILITY_POINTER
        }
        if self.keyboard {
            mask |= 1 << 1; // WL_SEAT_CAPABILITY_KEYBOARD
        }
        if self.touch {
            mask |= 1 << 2; // WL_SEAT_CAPABILITY_TOUCH
        }
        mask
    }
}

/// Button state (pressed/released)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

/// Keyboard modifier state
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub mod2: bool,
    pub mod3: bool,
    pub mod4: bool,
    pub mod5: bool,
}

impl ModifierState {
    /// Convert to Wayland protocol format (depressed keys bitmask)
    pub fn to_depressed(&self) -> u32 {
        let mut mask = 0u32;
        if self.shift {
            mask |= 1 << 0;
        }
        if self.ctrl {
            mask |= 1 << 2;
        }
        if self.alt {
            mask |= 1 << 3;
        }
        if self.mod2 {
            mask |= 1 << 4;
        }
        if self.mod3 {
            mask |= 1 << 5;
        }
        if self.mod4 {
            mask |= 1 << 6;
        }
        if self.mod5 {
            mask |= 1 << 7;
        }
        mask
    }
}

/// Pointer button codes (Linux input event codes)
pub mod button_codes {
    pub const LEFT: u32 = 0x110;
    pub const RIGHT: u32 = 0x111;
    pub const MIDDLE: u32 = 0x112;
    pub const WHEEL_UP: u32 = 0x04;
    pub const WHEEL_DOWN: u32 = 0x05;
}

/// Per-client pointer state
pub struct PointerState {
    /// Surface currently under pointer
    focused_surface: Option<SurfaceId>,
    /// Last reported position
    last_x: i32,
    pub last_y: i32,
    /// Pressed buttons
    pressed_buttons: BTreeMap<u32, bool>,
    /// Last sent frame time (for event batching)
    last_frame_time: u32,
}

impl PointerState {
    /// Create new pointer state
    pub fn new() -> Self {
        Self {
            focused_surface: None,
            last_x: 0,
            last_y: 0,
            pressed_buttons: BTreeMap::new(),
            last_frame_time: 0,
        }
    }

    /// Get focused surface
    pub fn focused_surface(&self) -> Option<SurfaceId> {
        self.focused_surface
    }

    /// Set pointer focus
    pub fn set_focus(&mut self, surface: Option<SurfaceId>) {
        self.focused_surface = surface;
    }

    /// Get last pointer position
    pub fn last_position(&self) -> (i32, i32) {
        (self.last_x, self.last_y)
    }

    /// Update last pointer position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.last_x = x;
        self.last_y = y;
    }

    /// Check if button is pressed
    pub fn is_button_pressed(&self, button: u32) -> bool {
        self.pressed_buttons.get(&button).copied().unwrap_or(false)
    }

    /// Set button state
    pub fn set_button(&mut self, button: u32, pressed: bool) {
        self.pressed_buttons.insert(button, pressed);
    }

    /// Update frame time
    pub fn set_frame_time(&mut self, time: u32) {
        self.last_frame_time = time;
    }

    /// Get frame time
    pub fn frame_time(&self) -> u32 {
        self.last_frame_time
    }
}

impl Default for PointerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-client keyboard state
pub struct KeyboardState {
    /// Surface with keyboard focus
    focused_surface: Option<SurfaceId>,
    /// Keymap sent to client
    keymap_sent: bool,
    /// Pressed keys
    pressed_keys: BTreeMap<u32, bool>,
    /// Current modifier state
    modifiers: ModifierState,
    /// Last sent frame time
    last_frame_time: u32,
}

impl KeyboardState {
    /// Create new keyboard state
    pub fn new() -> Self {
        Self {
            focused_surface: None,
            keymap_sent: false,
            pressed_keys: BTreeMap::new(),
            modifiers: ModifierState::default(),
            last_frame_time: 0,
        }
    }

    /// Get focused surface
    pub fn focused_surface(&self) -> Option<SurfaceId> {
        self.focused_surface
    }

    /// Set keyboard focus
    pub fn set_focus(&mut self, surface: Option<SurfaceId>) {
        self.focused_surface = surface;
    }

    /// Check if keymap was sent
    pub fn keymap_sent(&self) -> bool {
        self.keymap_sent
    }

    /// Mark keymap as sent
    pub fn set_keymap_sent(&mut self) {
        self.keymap_sent = true;
    }

    /// Check if key is pressed
    pub fn is_key_pressed(&self, key: u32) -> bool {
        self.pressed_keys.get(&key).copied().unwrap_or(false)
    }

    /// Set key state
    pub fn set_key(&mut self, key: u32, pressed: bool) {
        self.pressed_keys.insert(key, pressed);
    }

    /// Get modifier state
    pub fn modifiers(&self) -> &ModifierState {
        &self.modifiers
    }

    /// Set modifier state
    pub fn set_modifiers(&mut self, modifiers: ModifierState) {
        self.modifiers = modifiers;
    }

    /// Update frame time
    pub fn set_frame_time(&mut self, time: u32) {
        self.last_frame_time = time;
    }

    /// Get frame time
    pub fn frame_time(&self) -> u32 {
        self.last_frame_time
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-client seat bindings
pub struct SeatBinding {
    /// Seat ID
    seat_id: SeatId,
    /// Capabilities advertised to this client
    capabilities: SeatCapabilities,
    /// Pointer state for this binding
    pointer: PointerState,
    /// Keyboard state for this binding
    keyboard: KeyboardState,
}

impl SeatBinding {
    /// Create new seat binding
    pub fn new(seat_id: SeatId, capabilities: SeatCapabilities) -> Self {
        Self {
            seat_id,
            capabilities,
            pointer: PointerState::new(),
            keyboard: KeyboardState::new(),
        }
    }

    /// Get seat ID
    pub fn seat_id(&self) -> SeatId {
        self.seat_id
    }

    /// Get capabilities
    pub fn capabilities(&self) -> &SeatCapabilities {
        &self.capabilities
    }

    /// Get mutable pointer state
    pub fn pointer_mut(&mut self) -> &mut PointerState {
        &mut self.pointer
    }

    /// Get pointer state
    pub fn pointer(&self) -> &PointerState {
        &self.pointer
    }

    /// Get mutable keyboard state
    pub fn keyboard_mut(&mut self) -> &mut KeyboardState {
        &mut self.keyboard
    }

    /// Get keyboard state
    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }
}

/// Global seat manager
pub struct SeatManager {
    /// Seat bindings per client (client_id -> seat_id -> binding)
    bindings: BTreeMap<u32, BTreeMap<u32, SeatBinding>>,
    /// Default seat ID
    default_seat_id: SeatId,
    /// Next seat ID
    next_seat_id: u32,
}

impl SeatManager {
    /// Create new seat manager
    pub fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
            default_seat_id: SeatId(0),
            next_seat_id: 1,
        }
    }

    /// Bind a seat to a client
    pub fn bind_seat(
        &mut self,
        client_id: u32,
        seat_id: SeatId,
        capabilities: SeatCapabilities,
    ) -> u32 {
        let binding = SeatBinding::new(seat_id, capabilities);
        let seat_binding_id = seat_id.0;

        self.bindings
            .entry(client_id)
            .or_default()
            .insert(seat_binding_id, binding);

        seat_binding_id
    }

    /// Get a seat binding for a client
    pub fn get_binding_mut(
        &mut self,
        client_id: u32,
        seat_binding_id: u32,
    ) -> Option<&mut SeatBinding> {
        self.bindings
            .get_mut(&client_id)
            .and_then(|bindings| bindings.get_mut(&seat_binding_id))
    }

    /// Get a seat binding for a client (immutable)
    pub fn get_binding(&self, client_id: u32, seat_binding_id: u32) -> Option<&SeatBinding> {
        self.bindings
            .get(&client_id)
            .and_then(|bindings| bindings.get(&seat_binding_id))
    }

    /// Remove a client's seat bindings
    pub fn remove_client(&mut self, client_id: u32) {
        self.bindings.remove(&client_id);
    }

    /// Create a new seat (returns seat ID)
    pub fn create_seat(&mut self) -> SeatId {
        let id = SeatId(self.next_seat_id);
        self.next_seat_id = self.next_seat_id.saturating_add(1);
        id
    }

    /// Get default seat
    pub fn default_seat_id(&self) -> SeatId {
        self.default_seat_id
    }
}

impl Default for SeatManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seat_capabilities() {
        let caps = SeatCapabilities::default_inputs();
        assert!(caps.pointer);
        assert!(caps.keyboard);
        assert!(!caps.touch);

        let mask = caps.to_bitmask();
        assert_eq!(mask, 0b11);
    }

    #[test]
    fn test_pointer_state() {
        let mut pointer = PointerState::new();
        assert_eq!(pointer.focused_surface(), None);

        let surface = SurfaceId(1);
        pointer.set_focus(Some(surface));
        assert_eq!(pointer.focused_surface(), Some(surface));

        pointer.set_position(100, 200);
        assert_eq!(pointer.last_position(), (100, 200));

        pointer.set_button(button_codes::LEFT, true);
        assert!(pointer.is_button_pressed(button_codes::LEFT));
    }

    #[test]
    fn test_keyboard_state() {
        let mut keyboard = KeyboardState::new();
        assert!(!keyboard.keymap_sent());

        keyboard.set_keymap_sent();
        assert!(keyboard.keymap_sent());

        let surface = SurfaceId(1);
        keyboard.set_focus(Some(surface));
        assert_eq!(keyboard.focused_surface(), Some(surface));

        keyboard.set_key(30, true); // 'a'
        assert!(keyboard.is_key_pressed(30));
    }

    #[test]
    fn test_seat_binding() {
        let caps = SeatCapabilities::default_inputs();
        let binding = SeatBinding::new(SeatId(0), caps);
        assert_eq!(binding.seat_id(), SeatId(0));
        assert!(binding.capabilities().pointer);
    }

    #[test]
    fn test_seat_manager() {
        let mut manager = SeatManager::new();
        let caps = SeatCapabilities::default_inputs();
        let seat_id = SeatId(0);

        manager.bind_seat(1, seat_id, caps);
        let binding = manager.get_binding(1, 0);
        assert!(binding.is_some());

        manager.remove_client(1);
        let binding = manager.get_binding(1, 0);
        assert!(binding.is_none());
    }
}
