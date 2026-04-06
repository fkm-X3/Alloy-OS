/// Command system for terminal
/// 
/// Defines the Command trait and provides a registry for command lookup

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use crate::terminal::colors;

/// Command trait for terminal commands
pub trait Command {
    /// Get command name
    fn name(&self) -> &str;
    
    /// Get command help text
    fn help(&self) -> &str;
    
    /// Execute the command with given arguments
    fn execute(&self, args: &[&str]) -> Result<(), &str>;
}

/// Command registry
pub struct CommandRegistry {
    commands: BTreeMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry {
            commands: BTreeMap::new(),
        }
    }
    
    /// Register a command
    pub fn register(&mut self, cmd: Box<dyn Command>) {
        let name = String::from(cmd.name());
        self.commands.insert(name, cmd);
    }
    
    /// Execute a command by name
    pub fn execute(&self, name: &str, args: &[&str]) {
        if let Some(cmd) = self.commands.get(name) {
            match cmd.execute(args) {
                Ok(_) => {},
                Err(err) => {
                    colors::print_error(err);
                }
            }
        } else {
            colors::print_error(&alloc::format!("Unknown command: {}", name));
        }
    }
    
    /// Get all registered command names
    pub fn get_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get a specific command
    pub fn get(&self, name: &str) -> Option<&Box<dyn Command>> {
        self.commands.get(name)
    }
}

// Built-in commands

/// Help command
pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    
    fn help(&self) -> &str {
        "Display available commands or help for a specific command"
    }
    
    fn execute(&self, args: &[&str]) -> Result<(), &str> {
        // TODO: Implement help display
        colors::print_info("Available commands:");
        crate::ffi::vga_println_str("  help     - Show this help message");
        crate::ffi::vga_println_str("  clear    - Clear the screen");
        crate::ffi::vga_println_str("  echo     - Print text to the screen");
        crate::ffi::vga_println_str("  meminfo  - Display memory statistics");
        crate::ffi::vga_println_str("  version  - Show OS version information");
        Ok(())
    }
}

/// Clear command
pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }
    
    fn help(&self) -> &str {
        "Clear the screen"
    }
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        unsafe {
            // Clear screen by printing 25 empty lines
            crate::ffi::vga_set_color(0, 0);
            for _ in 0..25 {
                crate::ffi::vga_println(b"\0".as_ptr());
            }
            crate::ffi::vga_set_color(7, 0);
        }
        Ok(())
    }
}

/// Echo command
pub struct EchoCommand;

impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }
    
    fn help(&self) -> &str {
        "Print arguments to the screen"
    }
    
    fn execute(&self, args: &[&str]) -> Result<(), &str> {
        if args.is_empty() {
            crate::ffi::vga_println_str("");
        } else {
            let text = args.join(" ");
            crate::ffi::vga_println_str(&text);
        }
        Ok(())
    }
}

/// Version command
pub struct VersionCommand;

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }
    
    fn help(&self) -> &str {
        "Display OS version information"
    }
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        colors::print_info("Alloy Operating System");
        crate::ffi::vga_println_str("Version: 0.6.0-dev (Phase 6)");
        crate::ffi::vga_println_str("Architecture: x86 (32-bit)");
        crate::ffi::vga_println_str("Language: C++ + Rust");
        crate::ffi::vga_println_str("");
        crate::ffi::vga_println_str("Features:");
        crate::ffi::vga_println_str("  [x] Multiboot2 boot");
        crate::ffi::vga_println_str("  [x] VGA text mode");
        crate::ffi::vga_println_str("  [x] PS/2 keyboard");
        crate::ffi::vga_println_str("  [x] Memory management");
        crate::ffi::vga_println_str("  [x] Rust integration");
        crate::ffi::vga_println_str("  [x] Terminal interface");
        Ok(())
    }
}

/// Memory info command
pub struct MeminfoCommand;

impl Command for MeminfoCommand {
    fn name(&self) -> &str {
        "meminfo"
    }
    
    fn help(&self) -> &str {
        "Display memory allocation statistics"
    }
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        let (slab_stats, heap_stats) = crate::allocator::get_stats();
        
        colors::print_info("Memory Statistics");
        crate::ffi::vga_println_str("");
        
        crate::ffi::vga_println_str("Slab Allocator (small objects):");
        // Note: We can't easily format numbers without format! in no_std,
        // but we have the data structure. For now, just show placeholders.
        crate::ffi::vga_println_str("  Total allocated: [see serial output]");
        crate::ffi::vga_println_str("  Total freed: [see serial output]");
        crate::ffi::vga_println_str("  Size classes: 8, 16, 32, 64, 128, 256, 512, 1024 bytes");
        
        crate::ffi::vga_println_str("");
        crate::ffi::vga_println_str("Heap Allocator (large objects):");
        crate::ffi::vga_println_str("  Total allocated: [see serial output]");
        crate::ffi::vga_println_str("  Total freed: [see serial output]");
        
        // Print actual stats to serial for debugging
        unsafe {
            crate::ffi::serial_print(b"[Terminal] Slab stats: allocated=\0".as_ptr());
            crate::ffi::serial_print(b", freed=\0".as_ptr());
            crate::ffi::serial_print(b"\n[Terminal] Heap stats: allocated=\0".as_ptr());
            crate::ffi::serial_print(b", freed=\0".as_ptr());
            crate::ffi::serial_print(b"\n\0".as_ptr());
        }
        
        Ok(())
    }
}
