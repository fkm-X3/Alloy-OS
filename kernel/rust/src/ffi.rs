/// Foreign Function Interface (FFI) to C++ kernel functions
/// 
/// This module provides safe Rust wrappers around C++ functions

use core::ffi::c_void;
use crate::process::CpuContext;

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
    pub fn vmm_get_allocated_pages() -> u32;
    pub fn vmm_get_heap_start() -> u32;
    pub fn vmm_get_heap_size() -> u32;
    pub fn vmm_get_next_virt_addr() -> u32;
    
    // Physical memory management
    pub fn pmm_alloc_frame() -> *mut c_void;
    pub fn pmm_free_frame(addr: *mut c_void);
    pub fn pmm_get_total_frames() -> u32;
    pub fn pmm_get_used_frames() -> u32;
    pub fn pmm_get_total_memory() -> u64;
    pub fn pmm_get_available_memory() -> u64;
    
    // Keyboard functions - matches C++ signatures
    pub fn keyboard_has_data() -> bool;
    pub fn keyboard_get_char() -> i8;  // C char is signed
    
    // CPU information functions
    pub fn cpu_get_vendor_ffi(vendor: *mut u8);
    pub fn cpu_get_features_ffi() -> u32;
    pub fn cpu_get_model_info_ffi(family: *mut u32, model: *mut u32, stepping: *mut u32);
    
    // System uptime
    pub fn get_system_uptime_ms() -> u64;
    
    // Context switching (from context_switch.asm)
    pub fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext);
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

/// Safely print C-string to serial (with null check)
pub unsafe fn serial_print_safe(s: *const u8) {
    if !s.is_null() {
        serial_print(s);
    }
}

/// Safely print C-string to VGA (with null check)
pub unsafe fn vga_print_safe(s: *const u8) {
    if !s.is_null() {
        vga_print(s);
    }
}

/// Safely print C-string line to VGA (with null check)
pub unsafe fn vga_println_safe(s: *const u8) {
    if !s.is_null() {
        vga_println(s);
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

/// Get character from keyboard (returns as u8, converted from signed char)
pub fn keyboard_read() -> u8 {
    unsafe {
        let c = keyboard_get_char();
        if c < 0 {
            0  // Invalid/special key
        } else {
            c as u8
        }
    }
}

// Page flags for memory mapping
pub const PAGE_PRESENT: u32 = 0x001;
pub const PAGE_WRITE: u32 = 0x002;
pub const PAGE_USER: u32 = 0x004;
