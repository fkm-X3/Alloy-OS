/// Application state machine for Alloy OS Terminal with Iced
/// 
/// Manages terminal state, command history, input buffer, and view management

use std::collections::VecDeque;
use crate::ui::view_manager::{ViewManager, ViewType};

#[derive(Debug, Clone)]
pub enum Message {
    SwitchView(ViewType),
    NextView,
    PrevView,
    TerminalInput(String),
    TerminalSubmit,
    Exit,
}

#[derive(Clone, Copy, Debug)]
pub enum InputMode {
    Normal,
    Insert,
}

pub struct App {
    /// Input buffer for current command
    pub terminal_input: String,
    
    /// Command history
    pub history: VecDeque<String>,
    
    /// History position (for navigation)
    pub history_pos: Option<usize>,
    
    /// Output buffer (lines of text)
    pub terminal_output: VecDeque<String>,
    
    /// Current input mode
    pub input_mode: InputMode,
    
    /// Cursor position in input buffer
    pub cursor_pos: usize,
    
    /// View manager for tab navigation
    pub view_manager: ViewManager,
}

impl App {
    /// Maximum lines in output buffer
    pub const MAX_OUTPUT_LINES: usize = 1000;
    
    /// Maximum history entries
    pub const MAX_HISTORY: usize = 500;
    
    pub fn new() -> Self {
        let mut app = App {
            terminal_input: String::new(),
            history: VecDeque::new(),
            history_pos: None,
            terminal_output: VecDeque::new(),
            input_mode: InputMode::Insert,
            cursor_pos: 0,
            view_manager: ViewManager::new(),
        };
        
        app.print_welcome();
        app
    }
    
    
    fn print_welcome(&mut self) {
        self.terminal_output.push_back("╔══════════════════════════════════════════════════════╗".to_string());
        self.terminal_output.push_back("║         Alloy OS Terminal - Iced Edition             ║".to_string());
        self.terminal_output.push_back("║                    Version 0.1.0-dev                  ║".to_string());
        self.terminal_output.push_back("╚══════════════════════════════════════════════════════╝".to_string());
        self.terminal_output.push_back(String::new());
        self.terminal_output.push_back("Type 'help' for available commands, 'exit' to quit".to_string());
        self.terminal_output.push_back(String::new());
    }
    
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SwitchView(view) => {
                self.view_manager.set_view(view);
            }
            Message::NextView => {
                self.view_manager.next_view();
            }
            Message::PrevView => {
                self.view_manager.prev_view();
            }
            Message::TerminalInput(input) => {
                self.terminal_input = input;
            }
            Message::TerminalSubmit => {
                if !self.terminal_input.is_empty() {
                    self.execute_command();
                }
            }
            Message::Exit => {
                // Exit will be handled by the application
            }
        }
    }
    
    fn execute_command(&mut self) {
        let cmd = self.terminal_input.trim().to_string();
        
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
        self.terminal_output.push_back(format!("> {}", cmd));
        
        // Execute command
        let result = self.execute_command_impl(&cmd);
        
        // Print result
        self.terminal_output.push_back(result);
        self.terminal_output.push_back(String::new());
        
        // Trim output if too large
        while self.terminal_output.len() > Self::MAX_OUTPUT_LINES {
            self.terminal_output.pop_front();
        }
        
        // Clear input
        self.terminal_input.clear();
        self.cursor_pos = 0;
    }
    
    fn execute_command_impl(&self, cmd: &str) -> String {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }
        
        match parts[0] {
            "exit" | "quit" => {
                "Exiting terminal...".to_string()
            }
            "help" => self.cmd_help(&parts[1..]),
            "echo" => self.cmd_echo(&parts[1..]),
            "clear" => {
                // Note: Can't clear output in this context, just return empty
                "Use 'clear' to clear the screen".to_string()
            }
            "version" => self.cmd_version(),
            "sysinfo" => self.cmd_sysinfo(),
            "free" => self.cmd_free(),
            "uptime" => self.cmd_uptime(),
            "date" => self.cmd_date(),
            "meminfo" => self.cmd_meminfo(),
            "cpuinfo" => self.cmd_cpuinfo(),
            _ => format!("Unknown command: '{}'. Type 'help' for available commands.", parts[0]),
        }
    }
    
    fn cmd_help(&self, _args: &[&str]) -> String {
        let commands = vec![
            ("help", "Show this help message"),
            ("echo", "Print text"),
            ("version", "Show OS version"),
            ("sysinfo", "Show system information"),
            ("free", "Show memory usage"),
            ("uptime", "Show system uptime"),
            ("date", "Show current date and time"),
            ("meminfo", "Show memory statistics"),
            ("cpuinfo", "Show CPU information"),
        ];
        
        let mut help_text = "Available commands:\n".to_string();
        for (name, help) in commands {
            help_text.push_str(&format!("  {:<8} - {}\n", name, help));
        }
        help_text.push_str("\nType 'help <command>' for more info on a specific command.");
        help_text
    }
    
    fn cmd_echo(&self, args: &[&str]) -> String {
        args.join(" ")
    }
    
    fn cmd_version(&self) -> String {
        "Alloy OS Terminal v0.1.0 (Iced GUI Edition)".to_string()
    }
    
    fn cmd_sysinfo(&self) -> String {
        "System Summary\nAlloy OS v0.1.0\nArchitecture: x86\nUI: Iced GUI".to_string()
    }
    
    fn cmd_free(&self) -> String {
        "Memory Usage: (requires kernel integration)".to_string()
    }
    
    fn cmd_uptime(&self) -> String {
        "System uptime: (requires kernel integration)".to_string()
    }
    
    fn cmd_meminfo(&self) -> String {
        "Memory Statistics: (requires kernel integration)".to_string()
    }
    
    fn cmd_cpuinfo(&self) -> String {
        "CPU Information: (requires kernel integration)".to_string()
    }
    
    fn cmd_date(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                format!("Unix timestamp: {} seconds", secs)
            }
            Err(_) => "Unable to get system time".to_string(),
        }
    }
}
