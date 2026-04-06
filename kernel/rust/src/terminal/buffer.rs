/// Line buffer with editing support
/// 
/// Provides a fixed-size buffer for line input with cursor tracking

const BUFFER_SIZE: usize = 256;

pub struct LineBuffer {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
    length: usize,
}

impl LineBuffer {
    pub fn new() -> Self {
        LineBuffer {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
            length: 0,
        }
    }
    
    /// Insert character at cursor position
    pub fn insert(&mut self, c: char) -> bool {
        if self.length >= BUFFER_SIZE - 1 {
            return false; // Buffer full
        }
        
        let c_byte = c as u8;
        
        // Shift characters right from cursor
        if self.cursor < self.length {
            for i in (self.cursor..self.length).rev() {
                self.buffer[i + 1] = self.buffer[i];
            }
        }
        
        self.buffer[self.cursor] = c_byte;
        self.cursor += 1;
        self.length += 1;
        
        true
    }
    
    /// Remove character before cursor (backspace)
    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false; // Nothing to delete
        }
        
        // Shift characters left from cursor
        for i in self.cursor..self.length {
            self.buffer[i - 1] = self.buffer[i];
        }
        
        self.cursor -= 1;
        self.length -= 1;
        self.buffer[self.length] = 0;
        
        true
    }
    
    /// Remove character at cursor (delete)
    pub fn delete(&mut self) -> bool {
        if self.cursor >= self.length {
            return false; // Nothing to delete
        }
        
        // Shift characters left from cursor+1
        for i in (self.cursor + 1)..self.length {
            self.buffer[i - 1] = self.buffer[i];
        }
        
        self.length -= 1;
        self.buffer[self.length] = 0;
        
        true
    }
    
    /// Move cursor left
    pub fn cursor_left(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }
    
    /// Move cursor right
    pub fn cursor_right(&mut self) -> bool {
        if self.cursor < self.length {
            self.cursor += 1;
            true
        } else {
            false
        }
    }
    
    /// Move cursor to start of line
    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }
    
    /// Move cursor to end of line
    pub fn cursor_end(&mut self) {
        self.cursor = self.length;
    }
    
    /// Get current line as string slice
    pub fn get_line(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.length]).unwrap_or("")
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.fill(0);
        self.cursor = 0;
        self.length = 0;
    }
    
    /// Get cursor position
    pub fn cursor_pos(&self) -> usize {
        self.cursor
    }
    
    /// Get buffer length
    pub fn len(&self) -> usize {
        self.length
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_insert() {
        let mut buf = LineBuffer::new();
        assert!(buf.insert('a'));
        assert!(buf.insert('b'));
        assert_eq!(buf.get_line(), "ab");
        assert_eq!(buf.len(), 2);
    }
    
    #[test]
    fn test_backspace() {
        let mut buf = LineBuffer::new();
        buf.insert('a');
        buf.insert('b');
        assert!(buf.backspace());
        assert_eq!(buf.get_line(), "a");
        assert!(buf.backspace());
        assert_eq!(buf.get_line(), "");
        assert!(!buf.backspace()); // Empty buffer
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut buf = LineBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        
        assert!(buf.cursor_left());
        assert_eq!(buf.cursor_pos(), 2);
        assert!(buf.cursor_left());
        assert_eq!(buf.cursor_pos(), 1);
        
        assert!(buf.cursor_right());
        assert_eq!(buf.cursor_pos(), 2);
    }
}
