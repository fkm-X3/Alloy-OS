/// Foreign Function Interface (FFI) to C++ kernel functions
/// 
/// This module provides safe Rust wrappers around C++ functions

use core::ffi::c_void;

// External C++ functions
extern "C" {
    pub fn serial_print(s: *const u8);
    pub fn vga_print(s: *const u8);
    pub fn vga_println(s: *const u8);
    pub fn vga_putchar(c: u8);
    pub fn vga_set_color(fg: u8, bg: u8);
    
    // Memory management functions (from VMM)
    pub fn vmm_alloc_region(size: u32, flags: u32) -> *mut c_void;
    pub fn vmm_free_region(addr: *mut c_void, size: u32);
    pub fn vmm_map(virt_addr: *mut c_void, phys_addr: *mut c_void, flags: u32) -> bool;
    pub fn vmm_unmap(virt_addr: *mut c_void);
    
    // Physical memory management
    pub fn pmm_alloc_frame() -> *mut c_void;
    pub fn pmm_free_frame(addr: *mut c_void);
    
    // Keyboard functions
    pub fn keyboard_has_data() -> bool;
    pub fn keyboard_get_char() -> u8;
}

// Safe wrappers
pub fn print_str(s: &str) {
    // Convert Rust string to null-terminated C string
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0; // Null terminator
    
    unsafe {
        serial_print(buffer.as_ptr());
    }
}

pub fn vga_print_str(s: &str) {
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0;
    
    unsafe {
        vga_print(buffer.as_ptr());
    }
}

pub fn vga_println_str(s: &str) {
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0;
    
    unsafe {
        vga_println(buffer.as_ptr());
    }
}

/// Set VGA color
pub fn set_vga_color(fg: u8, bg: u8) {
    unsafe {
        vga_set_color(fg, bg);
    }
}

/// Print a character to VGA
pub fn put_char(c: char) {
    unsafe {
        vga_putchar(c as u8);
    }
}

/// Check if keyboard has data
pub fn keyboard_has_key() -> bool {
    unsafe {
        keyboard_has_data()
    }
}

/// Get character from keyboard
pub fn keyboard_read() -> u8 {
    unsafe {
        keyboard_get_char()
    }
}

// Page flags for memory mapping
pub const PAGE_PRESENT: u32 = 0x001;
pub const PAGE_WRITE: u32 = 0x002;
pub const PAGE_USER: u32 = 0x004;
