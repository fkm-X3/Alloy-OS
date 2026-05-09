/// Number formatting utilities for no_std environment
/// 
/// Provides functions to convert numbers to strings without heap allocation

/// Convert a u32 to a decimal string
/// Returns a fixed-size buffer with the number formatted
/// The string is right-aligned with leading spaces
pub fn u32_to_decimal(value: u32) -> [u8; 12] {
    let mut buffer = [b' '; 12]; // Max u32 is 4,294,967,295 (10 digits) + null + padding
    
    if value == 0 {
        buffer[10] = b'0';
        buffer[11] = 0; // Null terminator
        return buffer;
    }
    
    let mut temp = value;
    let mut index = 10; // Start from position 10 (leave room for null at 11)
    
    while temp > 0 && index > 0 {
        buffer[index] = b'0' + (temp % 10) as u8;
        temp /= 10;
        index -= 1;
    }
    
    buffer[11] = 0; // Null terminator
    buffer
}

/// Convert a u32 to a hexadecimal string with '0x' prefix
/// Returns a fixed-size buffer with the hex number formatted
pub fn u32_to_hex(value: u32) -> [u8; 12] {
    let mut buffer = [0u8; 12]; // "0x" + 8 hex digits + null
    buffer[0] = b'0';
    buffer[1] = b'x';
    
    let hex_chars = b"0123456789ABCDEF";
    
    for i in 0..8 {
        let nibble = ((value >> (28 - i * 4)) & 0xF) as usize;
        buffer[2 + i] = hex_chars[nibble];
    }
    
    buffer[10] = 0; // Null terminator
    buffer
}

/// Convert a u64 to a decimal string
/// Returns a fixed-size buffer with the number formatted
pub fn u64_to_decimal(value: u64) -> [u8; 24] {
    let mut buffer = [b' '; 24]; // Max u64 is 18,446,744,073,709,551,615 (20 digits)
    
    if value == 0 {
        buffer[22] = b'0';
        buffer[23] = 0; // Null terminator
        return buffer;
    }
    
    let mut temp = value;
    let mut index = 22; // Start from position 22 (leave room for null at 23)
    
    while temp > 0 && index > 0 {
        buffer[index] = b'0' + (temp % 10) as u8;
        temp /= 10;
        index -= 1;
    }
    
    buffer[23] = 0; // Null terminator
    buffer
}

/// Format bytes as a human-readable size (B, KB, MB, GB)
/// Returns a tuple of (value_string, unit_string)
pub fn format_bytes(bytes: u64) -> ([u8; 12], [u8; 4]) {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    
    let mut unit_buf = [0u8; 4];
    let value_buf: [u8; 12];
    
    if bytes >= GB {
        // Format as GB
        let gb = bytes / GB;
        value_buf = u32_to_decimal(gb as u32);
        unit_buf[0] = b'G';
        unit_buf[1] = b'B';
        unit_buf[2] = 0;
    } else if bytes >= MB {
        // Format as MB
        let mb = bytes / MB;
        value_buf = u32_to_decimal(mb as u32);
        unit_buf[0] = b'M';
        unit_buf[1] = b'B';
        unit_buf[2] = 0;
    } else if bytes >= KB {
        // Format as KB
        let kb = bytes / KB;
        value_buf = u32_to_decimal(kb as u32);
        unit_buf[0] = b'K';
        unit_buf[1] = b'B';
        unit_buf[2] = 0;
    } else {
        // Format as bytes
        value_buf = u32_to_decimal(bytes as u32);
        unit_buf[0] = b'B';
        unit_buf[1] = 0;
    }
    
    (value_buf, unit_buf)
}

/// Helper function to find the first non-space character in a buffer
/// Returns the starting index of the actual content
pub fn trim_leading_spaces(buffer: &[u8]) -> usize {
    for (i, &byte) in buffer.iter().enumerate() {
        if byte != b' ' && byte != 0 {
            return i;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_u32_to_decimal() {
        let buf = u32_to_decimal(12345);
        let start = trim_leading_spaces(&buf);
        assert_eq!(&buf[start..start+5], b"12345");
    }
    
    #[test]
    fn test_u32_to_hex() {
        let buf = u32_to_hex(0xDEADBEEF);
        assert_eq!(&buf[0..10], b"0xDEADBEEF");
    }
    
    #[test]
    fn test_format_bytes() {
        let (val, unit) = format_bytes(2048);
        let start = trim_leading_spaces(&val);
        assert_eq!(&val[start..start+1], b"2");
        assert_eq!(&unit[0..2], b"KB");
    }
}
