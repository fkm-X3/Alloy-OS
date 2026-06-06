//! Global object registry for Wayland interfaces
//!
//! Maintains the set of available global objects (compositor, output, shell, etc.)
//! that clients can bind to. Each global is advertised with a name and version.

use alloc::collections::BTreeMap;
use core::fmt;

/// Wayland interface name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InterfaceName {
    /// wl_compositor - surface rendering
    Compositor,
    /// wl_output - display/monitor
    Output,
    /// xdg_shell - application window management (ext)
    XdgShell,
    /// wl_data_device_manager - clipboard and DnD
    DataDeviceManager,
    /// wl_seat - input device (keyboard, mouse, touch)
    Seat,
    /// wl_shm - shared memory buffers
    Shm,
    /// wl_subcompositor - subsurface support
    Subcompositor,
    /// zwlr_layer_shell_v1 - layer shell for panels/desktop
    LayerShell,
    /// zxdg_output_manager_v1 - output properties
    XdgOutputManager,
}

impl fmt::Display for InterfaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterfaceName::Compositor => write!(f, "wl_compositor"),
            InterfaceName::Output => write!(f, "wl_output"),
            InterfaceName::XdgShell => write!(f, "xdg_shell"),
            InterfaceName::DataDeviceManager => write!(f, "wl_data_device_manager"),
            InterfaceName::Seat => write!(f, "wl_seat"),
            InterfaceName::Shm => write!(f, "wl_shm"),
            InterfaceName::Subcompositor => write!(f, "wl_subcompositor"),
            InterfaceName::LayerShell => write!(f, "zwlr_layer_shell_v1"),
            InterfaceName::XdgOutputManager => write!(f, "zxdg_output_manager_v1"),
        }
    }
}

/// Global object registry entry
#[derive(Debug, Clone)]
pub struct GlobalEntry {
    /// Global name (incrementing index)
    name: u32,
    /// Interface name
    interface: InterfaceName,
    /// Supported version
    version: u32,
}

impl GlobalEntry {
    /// Create a new global entry
    pub fn new(name: u32, interface: InterfaceName, version: u32) -> Self {
        Self {
            name,
            interface,
            version,
        }
    }

    /// Get the name
    pub fn name(&self) -> u32 {
        self.name
    }

    /// Get the interface name
    pub fn interface(&self) -> InterfaceName {
        self.interface
    }

    /// Get the version
    pub fn version(&self) -> u32 {
        self.version
    }
}

/// Wayland global object registry
pub struct GlobalRegistry {
    /// Global objects indexed by name (incrementing)
    globals: BTreeMap<u32, GlobalEntry>,
    /// Next global name to assign
    next_name: u32,
}

impl GlobalRegistry {
    /// Create a new global registry with standard globals
    pub fn new() -> Self {
        let mut registry = Self {
            globals: BTreeMap::new(),
            next_name: 0,
        };

        // Advertise standard globals in order
        registry.add_global(InterfaceName::Compositor, 5);
        registry.add_global(InterfaceName::Output, 4);
        registry.add_global(InterfaceName::Seat, 7);
        registry.add_global(InterfaceName::Shm, 2);
        registry.add_global(InterfaceName::Subcompositor, 1);
        registry.add_global(InterfaceName::DataDeviceManager, 3);
        registry.add_global(InterfaceName::XdgShell, 6);
        registry.add_global(InterfaceName::LayerShell, 4);
        registry.add_global(InterfaceName::XdgOutputManager, 3);

        registry
    }

    /// Register a new global interface
    fn add_global(&mut self, interface: InterfaceName, version: u32) {
        let name = self.next_name;
        self.next_name += 1;
        let entry = GlobalEntry::new(name, interface, version);
        self.globals.insert(name, entry);
    }

    /// Get a global by name
    pub fn get(&self, name: u32) -> Option<&GlobalEntry> {
        self.globals.get(&name)
    }

    /// Check if a global exists
    pub fn exists(&self, name: u32) -> bool {
        self.globals.contains_key(&name)
    }

    /// Iterate over all globals
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &GlobalEntry)> {
        self.globals.iter()
    }

    /// Get the count of registered globals
    pub fn count(&self) -> usize {
        self.globals.len()
    }
}

impl Default for GlobalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_registry_creation() {
        let registry = GlobalRegistry::new();
        assert!(registry.count() > 0);
    }

    #[test]
    fn test_global_exists() {
        let registry = GlobalRegistry::new();
        assert!(registry.exists(0)); // compositor
        assert!(!registry.exists(999)); // non-existent
    }

    #[test]
    fn test_global_get() {
        let registry = GlobalRegistry::new();
        let global = registry.get(0);
        assert!(global.is_some());
        assert_eq!(global.unwrap().interface(), InterfaceName::Compositor);
    }

    #[test]
    fn test_global_iteration() {
        let registry = GlobalRegistry::new();
        let count = registry.iter().count();
        assert_eq!(count, registry.count());
    }
}
