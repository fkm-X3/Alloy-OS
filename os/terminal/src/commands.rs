/// Commands module for Alloy OS Terminal
/// 
/// Contains implementations of all terminal commands,
/// ported from kernel/rust/src/terminal/command.rs

/// System information structure
pub struct SystemInfo {
    pub os_name: &'static str,
    pub os_version: &'static str,
    pub os_arch: &'static str,
    pub os_language: &'static str,
    pub os_uname: &'static str,
}

impl Default for SystemInfo {
    fn default() -> Self {
        SystemInfo {
            os_name: "Alloy Operating System",
            os_version: "0.7.0-dev (Phase 7)",
            os_arch: "x86 (32-bit)",
            os_language: "C++ + Rust",
            os_uname: "AlloyOS",
        }
    }
}

impl SystemInfo {
    pub fn features() -> Vec<&'static str> {
        vec![
            "[x] Multiboot2 boot",
            "[x] VGA text mode",
            "[x] PS/2 keyboard",
            "[x] Memory management",
            "[x] Rust integration",
            "[x] Terminal interface",
            "[x] Diagnostic commands",
            "[x] Ratatui UI",
        ]
    }
}
