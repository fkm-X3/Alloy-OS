//! Wayland output interface
//!
//! Implements wl_output for advertising display capabilities to clients.
//! Provides display geometry, resolution, scale, and refresh rate information.

use alloc::collections::BTreeMap;

/// Output subpixel arrangement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubpixelOrder {
    Unknown = 0,
    VerticalRgb = 1,
    VerticalBgr = 2,
    HorizontalRgb = 3,
    HorizontalBgr = 4,
}

/// Output mode (resolution and refresh rate)
#[derive(Debug, Clone, Copy)]
pub struct OutputMode {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Refresh rate in mHz (e.g., 60000 = 60 Hz)
    pub refresh: u32,
    /// Is this the preferred mode
    pub preferred: bool,
    /// Is this the current mode
    pub current: bool,
}

impl OutputMode {
    /// Create a new output mode
    pub fn new(width: u32, height: u32, refresh: u32) -> Self {
        Self {
            width,
            height,
            refresh,
            preferred: false,
            current: false,
        }
    }

    /// Mark as preferred
    pub fn with_preferred(mut self) -> Self {
        self.preferred = true;
        self
    }

    /// Mark as current
    pub fn with_current(mut self) -> Self {
        self.current = true;
        self
    }

    /// Convert to Wayland protocol bitmask
    pub fn to_flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.preferred {
            flags |= 1 << 0;
        }
        if self.current {
            flags |= 1 << 1;
        }
        flags
    }
}

/// Physical output geometry
#[derive(Debug, Clone, Copy)]
pub struct OutputGeometry {
    /// X position in global coordinates (mm)
    pub x: i32,
    /// Y position in global coordinates (mm)
    pub y: i32,
    /// Physical width in millimeters
    pub width_mm: i32,
    /// Physical height in millimeters
    pub height_mm: i32,
    /// Subpixel order
    pub subpixel: SubpixelOrder,
    /// Manufacturer string
    pub make: u32,
    /// Model string
    pub model: u32,
    /// Transform (rotation/flip)
    pub transform: u32,
}

impl OutputGeometry {
    /// Create new output geometry
    pub fn new(width_mm: i32, height_mm: i32) -> Self {
        Self {
            x: 0,
            y: 0,
            width_mm,
            height_mm,
            subpixel: SubpixelOrder::Unknown,
            make: 0,
            model: 0,
            transform: 0, // Normal orientation
        }
    }
}

/// Per-client output binding
pub struct OutputBinding {
    /// Output ID
    output_id: u32,
    /// Display geometry
    geometry: OutputGeometry,
    /// Available modes
    modes: alloc::vec::Vec<OutputMode>,
    /// Current scale factor
    scale: i32,
    /// Whether geometry/modes sent
    initialized: bool,
}

impl OutputBinding {
    /// Create new output binding
    pub fn new(output_id: u32, geometry: OutputGeometry, mut modes: alloc::vec::Vec<OutputMode>) -> Self {
        // Mark first mode as preferred and current
        if !modes.is_empty() {
            modes[0].preferred = true;
            modes[0].current = true;
        }

        Self {
            output_id,
            geometry,
            modes,
            scale: 1,
            initialized: false,
        }
    }

    /// Get output ID
    pub fn output_id(&self) -> u32 {
        self.output_id
    }

    /// Get geometry
    pub fn geometry(&self) -> &OutputGeometry {
        &self.geometry
    }

    /// Get modes
    pub fn modes(&self) -> &[OutputMode] {
        &self.modes
    }

    /// Get scale
    pub fn scale(&self) -> i32 {
        self.scale
    }

    /// Set scale
    pub fn set_scale(&mut self, scale: i32) {
        if scale > 0 && scale <= 4 {
            self.scale = scale;
        }
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Mark as initialized
    pub fn set_initialized(&mut self) {
        self.initialized = true;
    }

    /// Get current mode
    pub fn current_mode(&self) -> Option<&OutputMode> {
        self.modes.iter().find(|m| m.current)
    }
}

/// Global output manager
pub struct OutputManager {
    /// Output bindings per client (client_id -> output_id -> binding)
    bindings: BTreeMap<u32, BTreeMap<u32, OutputBinding>>,
    /// Next output ID
    next_output_id: u32,
}

impl OutputManager {
    /// Create new output manager
    pub fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
            next_output_id: 0,
        }
    }

    /// Bind an output to a client
    pub fn bind_output(
        &mut self,
        client_id: u32,
        geometry: OutputGeometry,
        modes: alloc::vec::Vec<OutputMode>,
    ) -> u32 {
        let output_id = self.next_output_id;
        self.next_output_id = self.next_output_id.saturating_add(1);

        let binding = OutputBinding::new(output_id, geometry, modes);

        self.bindings
            .entry(client_id)
            .or_default()
            .insert(output_id, binding);

        output_id
    }

    /// Get an output binding for a client
    pub fn get_binding_mut(&mut self, client_id: u32, output_id: u32) -> Option<&mut OutputBinding> {
        self.bindings
            .get_mut(&client_id)
            .and_then(|bindings| bindings.get_mut(&output_id))
    }

    /// Get an output binding for a client (immutable)
    pub fn get_binding(&self, client_id: u32, output_id: u32) -> Option<&OutputBinding> {
        self.bindings
            .get(&client_id)
            .and_then(|bindings| bindings.get(&output_id))
    }

    /// Remove a client's output bindings
    pub fn remove_client(&mut self, client_id: u32) {
        self.bindings.remove(&client_id);
    }

    /// Get all bindings for a client
    pub fn get_client_bindings(&self, client_id: u32) -> Option<&BTreeMap<u32, OutputBinding>> {
        self.bindings.get(&client_id)
    }

    /// Get all mutable bindings for a client
    pub fn get_client_bindings_mut(&mut self, client_id: u32) -> Option<&mut BTreeMap<u32, OutputBinding>> {
        self.bindings.get_mut(&client_id)
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create output modes from display dimensions
pub fn create_display_modes(width: u32, height: u32) -> alloc::vec::Vec<OutputMode> {
    alloc::vec![
        OutputMode::new(width, height, 60000).with_preferred().with_current(),
    ]
}

/// Helper to create physical dimensions from pixel dimensions and DPI
/// Typical DPI: 96 (standard), 110 (laptop), 163 (high-DPI)
pub fn calculate_physical_dimensions(width_px: u32, height_px: u32, dpi: u32) -> (i32, i32) {
    if dpi == 0 {
        return (254, 190); // Default ~10" display
    }
    
    // Convert pixels to mm: pixel_count * 25.4 / dpi
    // Approximate: mm ≈ pixels * 254 / dpi
    let width_mm = ((width_px as i32 * 254) / dpi as i32).max(1);
    let height_mm = ((height_px as i32 * 254) / dpi as i32).max(1);
    
    (width_mm, height_mm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode() {
        let mode = OutputMode::new(1024, 768, 60000)
            .with_preferred()
            .with_current();
        
        assert_eq!(mode.width, 1024);
        assert_eq!(mode.height, 768);
        assert_eq!(mode.refresh, 60000);
        assert!(mode.preferred);
        assert!(mode.current);
        
        let flags = mode.to_flags();
        assert_eq!(flags, 0b11);
    }

    #[test]
    fn test_output_geometry() {
        let geom = OutputGeometry::new(300, 225);
        assert_eq!(geom.width_mm, 300);
        assert_eq!(geom.height_mm, 225);
        assert_eq!(geom.x, 0);
        assert_eq!(geom.y, 0);
    }

    #[test]
    fn test_output_binding() {
        let geom = OutputGeometry::new(300, 225);
        let modes = alloc::vec![OutputMode::new(1024, 768, 60000)];
        let binding = OutputBinding::new(0, geom, modes);
        
        assert_eq!(binding.output_id(), 0);
        assert_eq!(binding.scale(), 1);
        assert!(!binding.is_initialized());
        
        let current_mode = binding.current_mode();
        assert!(current_mode.is_some());
    }

    #[test]
    fn test_output_manager() {
        let mut manager = OutputManager::new();
        let geom = OutputGeometry::new(300, 225);
        let modes = alloc::vec![OutputMode::new(1024, 768, 60000)];
        
        let output_id = manager.bind_output(1, geom, modes);
        assert_eq!(output_id, 0);
        
        let binding = manager.get_binding(1, 0);
        assert!(binding.is_some());
        
        manager.remove_client(1);
        let binding = manager.get_binding(1, 0);
        assert!(binding.is_none());
    }

    #[test]
    fn test_physical_dimensions() {
        // 1024x768 at 96 DPI
        let (w, h) = calculate_physical_dimensions(1024, 768, 96);
        assert!(w > 250 && w < 280); // ~268mm
        assert!(h > 190 && h < 210); // ~201mm
    }
}
