//! Terminal module for Alloy OS
//!
//! Provides a full-featured terminal with command parsing, line editing,
//! history, and built-in commands.

pub mod buffer;
pub mod colors;
pub mod command;
pub mod history;

use crate::ffi;
use buffer::LineBuffer;
use command::CommandRegistry;
use history::CommandHistory;

const PROMPT: &str = "Root:Root/> ";

/// Print a NUL-terminated byte buffer to the VGA console (stops at the first
/// NUL, mirroring the C `vga_print`).
pub fn print_cstr(buf: &[u8]) {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    crate::VgaText::print_bytes(&buf[..end]);
}

/// Print a NUL-terminated byte buffer to the VGA console followed by a
/// newline.
pub fn println_cstr(buf: &[u8]) {
    print_cstr(buf);
    crate::VgaText::putchar(b'\n');
}

pub struct Terminal {
    buffer: LineBuffer,
    commands: Option<CommandRegistry>, // Make optional for lazy init
    commands_initialized: bool,
    history: CommandHistory,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
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
        registry.register(Box::new(SysinfoCommand));
        registry.register(Box::new(UnameCommand));
        registry.register(Box::new(FreeCommand));
        registry.register(Box::new(TicksCommand));
        registry.register(Box::new(MeminfoCommand));
        registry.register(Box::new(CpuInfoCommand));
        registry.register(Box::new(UptimeCommand));
    }

    pub fn show_prompt(&self) {
        colors::print_prompt(PROMPT);
    }

    /// Redraw the current line from cursor position to end
    fn redraw_from_cursor(&self, start_x: u8) {
        let line = self.buffer.get_line();
        let cursor_pos = self.buffer.cursor_pos();

        // Save current cursor position
        let _save_x = crate::VgaText::cursor_x();
        let save_y = crate::VgaText::cursor_y();

        // Move to start position
        crate::VgaText::set_cursor(start_x, save_y);

        // Print from cursor position to end
        for ch in line[cursor_pos..].chars() {
            crate::VgaText::putchar(ch as u8);
        }

        // Clear to end of line (in case line got shorter)
        let _current_x = crate::VgaText::cursor_x();
        while crate::VgaText::cursor_x() < 80 {
            crate::VgaText::putchar(b' ');
        }

        // Restore cursor to correct position
        let _final_x = start_x + (self.buffer.len() - cursor_pos) as u8;
        crate::VgaText::set_cursor(start_x + (cursor_pos as u8), save_y);
    }

    /// Fully redraw the current line
    fn redraw_line(&self, prompt_len: usize) {
        let save_y = crate::VgaText::cursor_y();

        // Move to start of line (after prompt)
        crate::VgaText::set_cursor(prompt_len as u8, save_y);

        // Clear the entire line from prompt onward
        while crate::VgaText::cursor_x() < 80 {
            crate::VgaText::putchar(b' ');
        }

        // Move back to prompt position
        crate::VgaText::set_cursor(prompt_len as u8, save_y);

        // Print the buffer
        let line = self.buffer.get_line();
        for ch in line.chars() {
            crate::VgaText::putchar(ch as u8);
        }

        // Position cursor correctly
        let cursor_pos = self.buffer.cursor_pos();
        crate::VgaText::set_cursor(prompt_len as u8 + cursor_pos as u8, save_y);
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
        const PROMPT_LEN: usize = PROMPT.len();

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
                        let x = crate::VgaText::cursor_x();
                        let y = crate::VgaText::cursor_y();
                        if x > 0 {
                            crate::VgaText::set_cursor(x - 1, y);
                        }
                    }
                    return false;
                }

                // Right arrow - move cursor right
                ffi::SPECIAL_KEY_RIGHT => {
                    if self.buffer.cursor_right() {
                        let x = crate::VgaText::cursor_x();
                        let y = crate::VgaText::cursor_y();
                        crate::VgaText::set_cursor(x + 1, y);
                    }
                    return false;
                }

                // Home - jump to start of line
                ffi::SPECIAL_KEY_HOME => {
                    self.buffer.cursor_home();
                    let y = crate::VgaText::cursor_y();
                    crate::VgaText::set_cursor(PROMPT_LEN as u8, y);
                    return false;
                }

                // End - jump to end of line
                ffi::SPECIAL_KEY_END => {
                    self.buffer.cursor_end();
                    let y = crate::VgaText::cursor_y();
                    let pos = PROMPT_LEN + self.buffer.len();
                    crate::VgaText::set_cursor(pos as u8, y);
                    return false;
                }

                // Delete - remove character at cursor
                ffi::SPECIAL_KEY_DELETE => {
                    if self.buffer.delete() {
                        let cursor_x = crate::VgaText::cursor_x();
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
                    let x = crate::VgaText::cursor_x();
                    let y = crate::VgaText::cursor_y();

                    if x > 0 {
                        // Move cursor back
                        crate::VgaText::set_cursor(x - 1, y);

                        // Redraw from new cursor position
                        self.redraw_from_cursor(x - 1);
                    }
                }
                false
            }
            c if (' '..='~').contains(&c) => {
                // Printable character - insert at cursor
                if self.buffer.insert(c) {
                    let cursor_pos = self.buffer.cursor_pos();
                    let line_len = self.buffer.len();

                    if cursor_pos == line_len {
                        // At end of line - just print character
                        crate::VgaText::putchar(c as u8);
                    } else {
                        // Middle of line - need to redraw from cursor
                        let x = crate::VgaText::cursor_x();
                        crate::VgaText::putchar(c as u8);
                        self.redraw_from_cursor(x + 1);
                        // Position cursor after inserted char
                        let y = crate::VgaText::cursor_y();
                        crate::VgaText::set_cursor(x + 1, y);
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
        let parts: alloc::vec::Vec<&str> = cmd_line.split_whitespace().collect();
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

    /// Get reference to the line buffer for external rendering
    pub fn get_buffer(&self) -> &LineBuffer {
        &self.buffer
    }

    /// Get mutable reference to the line buffer
    pub fn get_buffer_mut(&mut self) -> &mut LineBuffer {
        &mut self.buffer
    }

    pub fn run(&mut self) {
        colors::print_banner();

        crate::VgaText::putchar(b'\n');

        self.show_prompt();

        // Main terminal loop
        loop {
            if ffi::keyboard_has_key() {
                let key = ffi::keyboard_read();
                if key != 0 && self.handle_input(key) {
                    // Show new prompt
                    crate::VgaText::putchar(b'\n');
                    self.show_prompt();
                }
            } else {
                // Halt CPU until next interrupt to save power and prevent busy-waiting
                alloy_kernel_hal::cpu_halt();
            }
        }
    }
}
