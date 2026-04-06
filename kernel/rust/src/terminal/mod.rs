/// Terminal module for Alloy OS
/// 
/// Provides a full-featured terminal with command parsing, line editing,
/// history, and built-in commands.

pub mod buffer;
pub mod command;
pub mod colors;

use crate::ffi;
use buffer::LineBuffer;
use command::CommandRegistry;

pub struct Terminal {
    buffer: LineBuffer,
    commands: Option<CommandRegistry>,  // Make optional for lazy init
    commands_initialized: bool,
}

impl Terminal {
    pub fn new() -> Self {
        // Don't create CommandRegistry yet - defer until first use
        Terminal {
            buffer: LineBuffer::new(),
            commands: None,
            commands_initialized: false,
        }
    }
    
    fn ensure_commands_initialized(&mut self) {
        if !self.commands_initialized {
            let mut registry = CommandRegistry::new();
            self.register_builtin_commands(&mut registry);
            self.commands = Some(registry);
            self.commands_initialized = true;
        }
    }
    
    fn register_builtin_commands(&self, registry: &mut CommandRegistry) {
        use alloc::boxed::Box;
        use command::*;
        
        registry.register(Box::new(HelpCommand));
        registry.register(Box::new(ClearCommand));
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(VersionCommand));
        registry.register(Box::new(MeminfoCommand));
        registry.register(Box::new(CpuInfoCommand));
        registry.register(Box::new(UptimeCommand));
    }
    
    pub fn show_prompt(&self) {
        colors::print_prompt("Root:Root/> ");
    }
    
    pub fn handle_input(&mut self, c: char) -> bool {
        match c {
            '\n' => {
                // Execute command - make a copy to avoid borrow issues
                let cmd_line = alloc::string::String::from(self.buffer.get_line());
                self.execute_command(&cmd_line);
                self.buffer.clear();
                true // Show new prompt
            }
            '\x08' => {
                // Backspace
                if self.buffer.backspace() {
                    unsafe {
                        // Move cursor back, print space, move back again
                        ffi::vga_putchar(b'\x08');
                        ffi::vga_putchar(b' ');
                        ffi::vga_putchar(b'\x08');
                    }
                }
                false
            }
            _ => {
                // Add character to buffer and echo
                if self.buffer.insert(c) {
                    unsafe {
                        ffi::vga_putchar(c as u8);
                    }
                }
                false
            }
        }
    }
    
    fn execute_command(&mut self, cmd_line: &str) {
        if cmd_line.trim().is_empty() {
            return;
        }
        
        // Ensure commands are initialized before executing
        self.ensure_commands_initialized();
        
        // Parse command and arguments
        let parts: alloc::vec::Vec<&str> = cmd_line.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        
        let cmd_name = parts[0];
        let args = &parts[1..];
        
        // Execute command
        if let Some(ref commands) = self.commands {
            commands.execute(cmd_name, args);
        }
    }
    
    pub fn run(&mut self) {
        colors::print_banner();
        
        unsafe {
            ffi::vga_println(b"\n\0".as_ptr());
        }
        
        self.show_prompt();
        
        // Main terminal loop
        loop {
            if ffi::keyboard_has_key() {
                let key = ffi::keyboard_read();
                if key != 0 {
                    if self.handle_input(key as char) {
                        // Show new prompt
                        ffi::put_char('\n');
                        self.show_prompt();
                    }
                }
            } else {
                // Halt CPU until next interrupt to save power and prevent busy-waiting
                unsafe {
                    core::arch::asm!("hlt");
                }
            }
        }
    }
}
