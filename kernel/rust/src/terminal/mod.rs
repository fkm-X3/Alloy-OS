/// Terminal module for Alloy OS
/// 
/// Provides a full-featured terminal with command parsing, line editing,
/// history, and built-in commands.

pub mod buffer;
pub mod command;
pub mod colors;
pub mod history;

use crate::ffi;
use buffer::LineBuffer;
use command::CommandRegistry;
use history::CommandHistory;

pub struct Terminal {
    buffer: LineBuffer,
    commands: Option<CommandRegistry>,  // Make optional for lazy init
    commands_initialized: bool,
    history: CommandHistory,
}

impl Terminal {
    pub fn new() -> Self {
        // Don't create CommandRegistry yet - defer until first use
        Terminal {
            buffer: LineBuffer::new(),
            commands: None,
            commands_initialized: false,
            history: CommandHistory::new(),
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
    
    /// Redraw the current line from cursor position to end
    fn redraw_from_cursor(&self, start_x: u8) {
        unsafe {
            let line = self.buffer.get_line();
            let cursor_pos = self.buffer.cursor_pos();
            
            // Save current cursor position
            let save_x = ffi::vga_get_cursor_x();
            let save_y = ffi::vga_get_cursor_y();
            
            // Move to start position
            ffi::vga_set_cursor(start_x, save_y);
            
            // Print from cursor position to end
            for (i, ch) in line[cursor_pos..].chars().enumerate() {
                ffi::vga_putchar(ch as u8);
            }
            
            // Clear to end of line (in case line got shorter)
            let current_x = ffi::vga_get_cursor_x();
            while ffi::vga_get_cursor_x() < 80 {
                ffi::vga_putchar(b' ');
            }
            
            // Restore cursor to correct position
            let final_x = start_x + (self.buffer.len() - cursor_pos) as u8;
            ffi::vga_set_cursor(start_x + (cursor_pos as u8), save_y);
        }
    }
    
    /// Fully redraw the current line
    fn redraw_line(&self, prompt_len: usize) {
        unsafe {
            let save_y = ffi::vga_get_cursor_y();
            
            // Move to start of line (after prompt)
            ffi::vga_set_cursor(prompt_len as u8, save_y);
            
            // Clear the entire line from prompt onward
            while ffi::vga_get_cursor_x() < 80 {
                ffi::vga_putchar(b' ');
            }
            
            // Move back to prompt position
            ffi::vga_set_cursor(prompt_len as u8, save_y);
            
            // Print the buffer
            let line = self.buffer.get_line();
            for ch in line.chars() {
                ffi::vga_putchar(ch as u8);
            }
            
            // Position cursor correctly
            let cursor_pos = self.buffer.cursor_pos();
            ffi::vga_set_cursor(prompt_len as u8 + cursor_pos as u8, save_y);
        }
    }
    
    /// Load command from history into buffer
    fn load_history_command(&mut self, cmd: &str, prompt_len: usize) {
        self.buffer.clear();
        for ch in cmd.chars() {
            self.buffer.insert(ch);
        }
        self.redraw_line(prompt_len);
    }
    
    pub fn handle_input(&mut self, key: u8) -> bool {
        const PROMPT_LEN: usize = 13; // "Root:Root/> ".len()
        
        // Handle special keys
        if key >= ffi::SPECIAL_KEY_UP {
            match key {
                // Up arrow - previous history
                ffi::SPECIAL_KEY_UP => {
                    if let Some(cmd) = self.history.prev() {
                        let cmd_copy = alloc::string::String::from(cmd);
                        self.load_history_command(&cmd_copy, PROMPT_LEN);
                    }
                    return false;
                }
                
                // Down arrow - next history
                ffi::SPECIAL_KEY_DOWN => {
                    if let Some(cmd) = self.history.next() {
                        let cmd_copy = alloc::string::String::from(cmd);
                        self.load_history_command(&cmd_copy, PROMPT_LEN);
                    } else {
                        // End of history - clear line
                        self.buffer.clear();
                        self.redraw_line(PROMPT_LEN);
                    }
                    return false;
                }
                
                // Left arrow - move cursor left
                ffi::SPECIAL_KEY_LEFT => {
                    if self.buffer.cursor_left() {
                        unsafe {
                            let x = ffi::vga_get_cursor_x();
                            let y = ffi::vga_get_cursor_y();
                            if x > 0 {
                                ffi::vga_set_cursor(x - 1, y);
                            }
                        }
                    }
                    return false;
                }
                
                // Right arrow - move cursor right
                ffi::SPECIAL_KEY_RIGHT => {
                    if self.buffer.cursor_right() {
                        unsafe {
                            let x = ffi::vga_get_cursor_x();
                            let y = ffi::vga_get_cursor_y();
                            ffi::vga_set_cursor(x + 1, y);
                        }
                    }
                    return false;
                }
                
                // Home - jump to start of line
                ffi::SPECIAL_KEY_HOME => {
                    self.buffer.cursor_home();
                    unsafe {
                        let y = ffi::vga_get_cursor_y();
                        ffi::vga_set_cursor(PROMPT_LEN as u8, y);
                    }
                    return false;
                }
                
                // End - jump to end of line
                ffi::SPECIAL_KEY_END => {
                    self.buffer.cursor_end();
                    unsafe {
                        let y = ffi::vga_get_cursor_y();
                        let pos = PROMPT_LEN + self.buffer.len();
                        ffi::vga_set_cursor(pos as u8, y);
                    }
                    return false;
                }
                
                // Delete - remove character at cursor
                ffi::SPECIAL_KEY_DELETE => {
                    if self.buffer.delete() {
                        let cursor_x = unsafe { ffi::vga_get_cursor_x() };
                        self.redraw_from_cursor(cursor_x);
                    }
                    return false;
                }
                
                _ => return false, // Unknown special key
            }
        }
        
        // Handle regular ASCII keys
        match key as char {
            '\n' => {
                // Execute command
                let cmd_line = alloc::string::String::from(self.buffer.get_line());
                
                // Add to history before executing
                self.history.add(&cmd_line);
                
                self.execute_command(&cmd_line);
                self.buffer.clear();
                true // Show new prompt
            }
            '\x08' => {
                // Backspace
                if self.buffer.backspace() {
                    unsafe {
                        let x = ffi::vga_get_cursor_x();
                        let y = ffi::vga_get_cursor_y();
                        
                        if x > 0 {
                            // Move cursor back
                            ffi::vga_set_cursor(x - 1, y);
                            
                            // Redraw from new cursor position
                            self.redraw_from_cursor(x - 1);
                        }
                    }
                }
                false
            }
            c if c >= ' ' && c <= '~' => {
                // Printable character - insert at cursor
                if self.buffer.insert(c) {
                    let cursor_pos = self.buffer.cursor_pos();
                    let line_len = self.buffer.len();
                    
                    if cursor_pos == line_len {
                        // At end of line - just print character
                        unsafe {
                            ffi::vga_putchar(c as u8);
                        }
                    } else {
                        // Middle of line - need to redraw from cursor
                        unsafe {
                            let x = ffi::vga_get_cursor_x();
                            ffi::vga_putchar(c as u8);
                            self.redraw_from_cursor(x + 1);
                            // Position cursor after inserted char
                            let y = ffi::vga_get_cursor_y();
                            ffi::vga_set_cursor(x + 1, y);
                        }
                    }
                }
                false
            }
            _ => false, // Ignore other control characters
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
                    if self.handle_input(key) {
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
