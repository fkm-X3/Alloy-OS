/// Application state machine for Alloy OS Terminal
/// 
/// Manages terminal state, command history, input buffer, and view management

use crossterm::event::KeyEvent;
use std::collections::VecDeque;
use crate::ui::views::{ViewManager, TerminalView, MonitorView, HelpView, LogsView};

#[derive(Clone, Copy, Debug)]
pub enum InputMode {
    Normal,
    Insert,
}

pub struct App {
    /// Input buffer for current command
    pub input: String,
    
    /// Command history
    pub history: VecDeque<String>,
    
    /// History position (for navigation)
    pub history_pos: Option<usize>,
    
    /// Output buffer (lines of text)
    pub output: VecDeque<String>,
    
    /// Current input mode
    pub input_mode: InputMode,
    
    /// Cursor position in input buffer
    pub cursor_pos: usize,
    
    /// View manager for tab navigation
    pub view_manager: ViewManager,
    
    /// Terminal view
    pub terminal_view: TerminalView,
    
    /// Monitor view
    pub monitor_view: MonitorView,
    
    /// Help view
    pub help_view: HelpView,
    
    /// Logs view
    pub logs_view: LogsView,
}

impl App {
    /// Maximum lines in output buffer
    pub const MAX_OUTPUT_LINES: usize = 1000;
    
    /// Maximum history entries
    pub const MAX_HISTORY: usize = 500;
    
    pub fn new() -> Self {
        let mut app = App {
            input: String::new(),
            history: VecDeque::new(),
            history_pos: None,
            output: VecDeque::new(),
            input_mode: InputMode::Insert,
            cursor_pos: 0,
            view_manager: ViewManager::new(),
            terminal_view: TerminalView::new(),
            monitor_view: MonitorView::new(),
            help_view: HelpView::new(),
            logs_view: LogsView::new(),
        };
        
        app.print_welcome();
        app
    }
    
    fn print_welcome(&mut self) {
        self.output.push_back("╔══════════════════════════════════════════════════════╗".to_string());
        self.output.push_back("║         Alloy OS Terminal - Ratatui Edition           ║".to_string());
        self.output.push_back("║                    Version 0.1.0-dev                  ║".to_string());
        self.output.push_back("╚══════════════════════════════════════════════════════╝".to_string());
        self.output.push_back(String::new());
        self.output.push_back("Type 'help' for available commands, 'exit' to quit".to_string());
        self.output.push_back(String::new());
    }
    
    /// Handle keyboard input
    pub async fn handle_input(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match key.code {
            KeyCode::Enter => {
                self.execute_command().await;
            }
            KeyCode::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match ch {
                        'u' => self.input.clear(),  // Clear line (Ctrl+U)
                        'w' => self.clear_word(),   // Clear word (Ctrl+W)
                        _ => {}
                    }
                } else {
                    self.input.insert(self.cursor_pos, ch);
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
            }
            KeyCode::Up => {
                self.history_prev();
            }
            KeyCode::Down => {
                self.history_next();
            }
            _ => {}
        }
    }
    
    /// Clear a word from cursor position
    fn clear_word(&mut self) {
        let end = self.cursor_pos;
        let start = self.input[..end]
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);
        
        self.input.drain(start..end);
        self.cursor_pos = start;
    }
    
    /// Navigate to previous history entry
    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        
        let new_pos = match self.history_pos {
            None => self.history.len() - 1,
            Some(pos) => {
                if pos > 0 {
                    pos - 1
                } else {
                    return;
                }
            }
        };
        
        self.history_pos = Some(new_pos);
        if let Some(cmd) = self.history.get(new_pos) {
            self.input = cmd.clone();
            self.cursor_pos = self.input.len();
        }
    }
    
    /// Navigate to next history entry
    fn history_next(&mut self) {
        match self.history_pos {
            None => {}
            Some(pos) => {
                if pos < self.history.len() - 1 {
                    let new_pos = pos + 1;
                    self.history_pos = Some(new_pos);
                    if let Some(cmd) = self.history.get(new_pos) {
                        self.input = cmd.clone();
                        self.cursor_pos = self.input.len();
                    }
                } else {
                    self.history_pos = None;
                    self.input.clear();
                    self.cursor_pos = 0;
                }
            }
        }
    }
    
    /// Execute the current input buffer as a command
    async fn execute_command(&mut self) {
        let cmd = self.input.trim().to_string();
        
        if cmd.is_empty() {
            return;
        }
        
        // Add to history
        self.history.push_back(cmd.clone());
        if self.history.len() > Self::MAX_HISTORY {
            self.history.pop_front();
        }
        self.history_pos = None;
        
        // Print command to output
        self.output.push_back(format!("> {}", cmd));
        
        // Execute command
        let result = self.execute_command_impl(&cmd).await;
        
        // Print result
        self.output.push_back(result);
        self.output.push_back(String::new());
        
        // Trim output if too large
        while self.output.len() > Self::MAX_OUTPUT_LINES {
            self.output.pop_front();
        }
        
        // Clear input
        self.input.clear();
        self.cursor_pos = 0;
    }
    
    /// Execute command implementation (delegates to commands module)
    async fn execute_command_impl(&mut self, cmd: &str) -> String {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }
        
        match parts[0] {
            "exit" | "quit" => {
                // Exit will be handled by main loop
                "Exiting terminal...".to_string()
            }
            "help" => self.cmd_help(&parts[1..]),
            "echo" => self.cmd_echo(&parts[1..]),
            "clear" => {
                self.output.clear();
                String::new()
            }
            "version" => self.cmd_version(),
            "sysinfo" => self.cmd_sysinfo(),
            "free" => self.cmd_free(),
            "uptime" => self.cmd_uptime(),
            "meminfo" => self.cmd_meminfo(),
            "cpuinfo" => self.cmd_cpuinfo(),
            "ticks" => self.cmd_ticks(),
            "uname" => self.cmd_uname(),
            _ => format!("Unknown command: {}", parts[0]),
        }
    }
    
    /// Command: help
    fn cmd_help(&self, args: &[&str]) -> String {
        let commands = vec![
            ("help", "Show this help message"),
            ("echo", "Print text"),
            ("clear", "Clear screen"),
            ("version", "Show OS version"),
            ("sysinfo", "Show system information"),
            ("free", "Show memory usage"),
            ("uptime", "Show system uptime"),
            ("meminfo", "Show memory info"),
            ("cpuinfo", "Show CPU info"),
            ("ticks", "Show system ticks"),
            ("uname", "Show OS name"),
        ];
        
        if let Some(cmd) = args.first() {
            for (name, help) in commands {
                if name == *cmd {
                    return format!("{}: {}", name, help);
                }
            }
            return format!("Unknown command: {}", cmd);
        }
        
        let mut help_text = "Available commands:\n".to_string();
        for (name, help) in commands {
            help_text.push_str(&format!("  {:8} - {}\n", name, help));
        }
        help_text
    }
    
    /// Command: echo
    fn cmd_echo(&self, args: &[&str]) -> String {
        args.join(" ")
    }
    
    /// Command: version
    fn cmd_version(&self) -> String {
        let mut output = String::from("Alloy Operating System\n");
        output.push_str("Version: 0.7.0-dev (Phase 7)\n");
        output.push_str("Architecture: x86 (32-bit)\n");
        output.push_str("Language: C++ + Rust\n");
        output.push_str("UI: Ratatui (TUI Edition)\n\n");
        output.push_str("Features:\n");
        output.push_str("  [x] Multiboot2 boot\n");
        output.push_str("  [x] VGA text mode\n");
        output.push_str("  [x] PS/2 keyboard\n");
        output.push_str("  [x] Memory management\n");
        output.push_str("  [x] Rust integration\n");
        output.push_str("  [x] Terminal interface\n");
        output.push_str("  [x] Diagnostic commands");
        output
    }
    
    /// Command: sysinfo
    fn cmd_sysinfo(&self) -> String {
        let mut output = String::from("System Summary\n\n");
        output.push_str("Alloy Operating System\n");
        output.push_str("Version: 0.7.0-dev (Phase 7)\n");
        output.push_str("Architecture: x86 (32-bit)\n");
        output.push_str("\n[Note: Real-time data requires kernel integration]\n");
        output.push_str("CPU Vendor: (requires kernel FFI)\n");
        output.push_str("Memory Total: (requires kernel FFI)\n");
        output.push_str("Memory Used:  (requires kernel FFI)\n");
        output.push_str("Memory Free:  (requires kernel FFI)\n");
        output.push_str("Uptime: (requires kernel FFI)");
        output
    }
    
    /// Command: free
    fn cmd_free(&self) -> String {
        let mut output = String::from("Memory Usage (requires kernel FFI)\n\n");
        output.push_str("Physical:\n");
        output.push_str("  Total: (requires kernel)\n");
        output.push_str("  Used:  (requires kernel)\n");
        output.push_str("  Free:  (requires kernel)\n\n");
        output.push_str("Virtual Heap:\n");
        output.push_str("  Mapped bytes: (requires kernel)\n");
        output.push_str("  Alloc pages:  (requires kernel)");
        output
    }
    
    /// Command: uptime
    fn cmd_uptime(&self) -> String {
        "System uptime (requires kernel FFI)".to_string()
    }
    
    /// Command: meminfo
    fn cmd_meminfo(&self) -> String {
        let mut output = String::from("Memory Statistics (requires kernel FFI)\n\n");
        output.push_str("Physical Memory Manager:\n");
        output.push_str("  Total memory:     (requires kernel)\n");
        output.push_str("  Available memory: (requires kernel)\n");
        output.push_str("  Total frames:     (requires kernel)\n");
        output.push_str("  Used frames:      (requires kernel)\n");
        output.push_str("  Free frames:      (requires kernel)");
        output
    }
    
    /// Command: cpuinfo
    fn cmd_cpuinfo(&self) -> String {
        let mut output = String::from("CPU Information (requires kernel FFI)\n\n");
        output.push_str("Vendor:   (requires kernel)\n");
        output.push_str("Family:   (requires kernel)\n");
        output.push_str("Model:    (requires kernel)\n");
        output.push_str("Stepping: (requires kernel)\n\n");
        output.push_str("Features:\n");
        output.push_str("  [?] FPU   - Floating Point Unit\n");
        output.push_str("  [?] TSC   - Time Stamp Counter\n");
        output.push_str("  [?] PAE   - Physical Address Extension\n");
        output.push_str("  [?] APIC  - Advanced Programmable Interrupt Controller\n");
        output.push_str("  [?] MMX   - MMX Instructions\n");
        output.push_str("  [?] SSE   - Streaming SIMD Extensions\n");
        output.push_str("  [?] SSE2  - Streaming SIMD Extensions 2");
        output
    }
    
    /// Command: ticks
    fn cmd_ticks(&self) -> String {
        let mut output = String::from("Timer Statistics (requires kernel FFI)\n\n");
        output.push_str("Tick count:     (requires kernel)\n");
        output.push_str("Uptime (ms):    (requires kernel)\n");
        output.push_str("Frequency (Hz): (requires kernel)\n");
        output.push_str("Avg ms/tick:    (requires kernel)");
        output
    }
    
    /// Command: uname
    fn cmd_uname(&self) -> String {
        "AlloyOS".to_string()
    }
    
    /// Update app state (called every frame)
    pub async fn update(&mut self) {
        // Placeholder for async operations
    }
}
