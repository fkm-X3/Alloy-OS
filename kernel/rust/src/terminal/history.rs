/// Command history management for terminal
/// 
/// Provides a ring buffer to store and navigate through command history

extern crate alloc;
use alloc::string::String;

const HISTORY_SIZE: usize = 10;

pub struct CommandHistory {
    /// Ring buffer of command strings
    commands: [Option<String>; HISTORY_SIZE],
    /// Next write position in ring buffer
    write_pos: usize,
    /// Current position when browsing history
    browse_pos: Option<usize>,
    /// Total commands stored (up to HISTORY_SIZE)
    count: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        CommandHistory {
            commands: Default::default(),
            write_pos: 0,
            browse_pos: None,
            count: 0,
        }
    }
    
    /// Add a command to history
    /// Skips empty commands and duplicates of the last command
    pub fn add(&mut self, command: &str) {
        let trimmed = command.trim();
        
        // Don't add empty commands
        if trimmed.is_empty() {
            return;
        }
        
        // Don't add if it's the same as the last command
        if self.count > 0 {
            let last_pos = if self.write_pos == 0 {
                HISTORY_SIZE - 1
            } else {
                self.write_pos - 1
            };
            
            if let Some(ref last_cmd) = self.commands[last_pos] {
                if last_cmd == trimmed {
                    return;
                }
            }
        }
        
        // Add command at write position
        self.commands[self.write_pos] = Some(String::from(trimmed));
        self.write_pos = (self.write_pos + 1) % HISTORY_SIZE;
        
        if self.count < HISTORY_SIZE {
            self.count += 1;
        }
        
        // Reset browse position
        self.browse_pos = None;
    }
    
    /// Navigate to previous (older) command in history
    /// Returns the command string if available, None if at the start
    pub fn prev(&mut self) -> Option<&str> {
        if self.count == 0 {
            return None;
        }
        
        match self.browse_pos {
            None => {
                // Start browsing from most recent command
                let pos = if self.write_pos == 0 {
                    self.count - 1
                } else {
                    (self.write_pos - 1 + HISTORY_SIZE) % HISTORY_SIZE
                };
                self.browse_pos = Some(pos);
                self.commands[pos].as_deref()
            }
            Some(pos) => {
                // Move to older command
                let oldest_pos = if self.count < HISTORY_SIZE {
                    0
                } else {
                    self.write_pos
                };
                
                if pos == oldest_pos {
                    // Already at oldest
                    self.commands[pos].as_deref()
                } else {
                    let new_pos = if pos == 0 {
                        HISTORY_SIZE - 1
                    } else {
                        pos - 1
                    };
                    self.browse_pos = Some(new_pos);
                    self.commands[new_pos].as_deref()
                }
            }
        }
    }
    
    /// Navigate to next (newer) command in history
    /// Returns the command string if available, None if at the end (most recent)
    pub fn next(&mut self) -> Option<&str> {
        if self.count == 0 {
            return None;
        }
        
        match self.browse_pos {
            None => None, // Not browsing
            Some(pos) => {
                let newest_pos = if self.write_pos == 0 {
                    self.count - 1
                } else {
                    (self.write_pos - 1 + HISTORY_SIZE) % HISTORY_SIZE
                };
                
                if pos == newest_pos {
                    // At newest, exit browsing mode
                    self.browse_pos = None;
                    None
                } else {
                    let new_pos = (pos + 1) % HISTORY_SIZE;
                    self.browse_pos = Some(new_pos);
                    self.commands[new_pos].as_deref()
                }
            }
        }
    }
    
    /// Get currently selected command (if browsing)
    pub fn current(&self) -> Option<&str> {
        self.browse_pos.and_then(|pos| self.commands[pos].as_deref())
    }
    
    /// Reset browsing state (return to end of history)
    pub fn reset(&mut self) {
        self.browse_pos = None;
    }
    
    /// Check if currently browsing history
    pub fn is_browsing(&self) -> bool {
        self.browse_pos.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_and_navigate() {
        let mut hist = CommandHistory::new();
        hist.add("cmd1");
        hist.add("cmd2");
        hist.add("cmd3");
        
        assert_eq!(hist.prev(), Some("cmd3"));
        assert_eq!(hist.prev(), Some("cmd2"));
        assert_eq!(hist.prev(), Some("cmd1"));
        assert_eq!(hist.prev(), Some("cmd1")); // At oldest
        
        assert_eq!(hist.next(), Some("cmd2"));
        assert_eq!(hist.next(), Some("cmd3"));
        assert_eq!(hist.next(), None); // At newest
    }
    
    #[test]
    fn test_skip_duplicates() {
        let mut hist = CommandHistory::new();
        hist.add("cmd1");
        hist.add("cmd1"); // Should be skipped
        hist.add("cmd2");
        
        assert_eq!(hist.prev(), Some("cmd2"));
        assert_eq!(hist.prev(), Some("cmd1"));
        assert_eq!(hist.prev(), Some("cmd1")); // Only 2 commands
    }
    
    #[test]
    fn test_ring_buffer_overflow() {
        let mut hist = CommandHistory::new();
        for i in 0..15 {
            hist.add(&format!("cmd{}", i));
        }
        
        // Should only have last 10 commands
        assert_eq!(hist.prev(), Some("cmd14"));
        for _ in 0..8 {
            hist.prev();
        }
        assert_eq!(hist.current(), Some("cmd5"));
    }
}
