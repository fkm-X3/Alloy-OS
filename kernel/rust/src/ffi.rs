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
    pub fn vga_clear();
    pub fn vga_set_cursor(x: u8, y: u8);
    pub fn vga_get_cursor_x() -> u8;
    pub fn vga_get_cursor_y() -> u8;
    pub fn vga_print_hex(value: u32);
    pub fn vga_print_dec(value: u32);
    
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
    
    // Timer functions (from timer.cpp)
    pub fn timer_init_ffi(frequency: u32);
    pub fn timer_get_ticks_ffi() -> u64;
    pub fn timer_get_uptime_ms_ffi() -> u64;
    
    // VESA VBE graphics functions (from vesa.cpp)
    /// Initialize VESA VBE detection and controller check
    pub fn vesa_init();
    
    /// Set a graphics mode
    /// Returns: 0 on success, 1 if not initialized, 2 if mode not supported, 3 if mode setting failed
    pub fn vesa_set_mode(mode: u16) -> u16;
    
    /// Get current framebuffer linear address
    /// Returns: Physical address of linear framebuffer, 0 if not available
    pub fn vesa_get_framebuffer() -> u32;
    
    /// Get current graphics mode resolution
    /// Parameters: width, height - pointers to store resolution
    pub fn vesa_get_resolution(width: *mut u16, height: *mut u16);
    
    /// Get current graphics mode number
    /// Returns: 0 on success, non-zero on failure
    pub fn vesa_get_mode(mode: *mut u16) -> u16;
    
    /// Check if VESA VBE is available
    /// Returns: 1 if available, 0 if not
    pub fn vesa_is_available() -> u8;
    
    /// Get controller capabilities
    /// Returns: Capabilities byte from VBE info block
    pub fn vesa_get_capabilities() -> u8;
    
    /// Get bits per pixel for current mode
    /// Returns: 0, 16, 24, or 32 bits per pixel; 0 if not in graphics mode
    pub fn vesa_get_bits_per_pixel() -> u8;
    
    /// Get bytes per scanline for current mode
    /// Returns: Bytes per scan line, or 0 if not in graphics mode
    pub fn vesa_get_bytes_per_scanline() -> u16;
    
    /// Get total framebuffer size in bytes
    /// Returns: Size in bytes, or 0 if not in graphics mode
    pub fn vesa_get_framebuffer_size() -> u32;
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

/// Get character from keyboard (returns as u8, handles special keys 128-255)
pub fn keyboard_read() -> u8 {
    unsafe {
        let c = keyboard_get_char();
        // Special keys use values 128-255, regular ASCII is 0-127
        // Just cast to u8 to handle full range
        c as u8
    }
}

// Special key codes (match C++ keyboard.h)
pub const SPECIAL_KEY_UP: u8 = 128;
pub const SPECIAL_KEY_DOWN: u8 = 129;
pub const SPECIAL_KEY_LEFT: u8 = 130;
pub const SPECIAL_KEY_RIGHT: u8 = 131;
pub const SPECIAL_KEY_HOME: u8 = 132;
pub const SPECIAL_KEY_END: u8 = 133;
pub const SPECIAL_KEY_DELETE: u8 = 134;
pub const SPECIAL_KEY_PGUP: u8 = 135;
pub const SPECIAL_KEY_PGDN: u8 = 136;

// Page flags for memory mapping
pub const PAGE_PRESENT: u32 = 0x001;
pub const PAGE_WRITE: u32 = 0x002;
pub const PAGE_USER: u32 = 0x004;

// ============================================================================
// VESA VBE Graphics Safe Wrappers
// ============================================================================

/// Initialize VESA VBE graphics mode detection
pub fn vesa_initialize() {
    unsafe {
        vesa_init();
    }
}

/// Set graphics mode with error handling
/// Returns tuple of (success, error_code)
pub fn vesa_set_graphics_mode(mode: u16) -> (bool, u16) {
    unsafe {
        let result = vesa_set_mode(mode);
        (result == 0, result)
    }
}

/// Get framebuffer physical address
/// Returns None if not available
pub fn vesa_framebuffer_addr() -> Option<u32> {
    unsafe {
        let addr = vesa_get_framebuffer();
        if addr != 0 {
            Some(addr)
        } else {
            None
        }
    }
}

/// Get current display resolution in pixels
/// Returns (width, height) or (0, 0) if not available
pub fn vesa_display_resolution() -> (u16, u16) {
    unsafe {
        let mut width: u16 = 0;
        let mut height: u16 = 0;
        vesa_get_resolution(&mut width, &mut height);
        (width, height)
    }
}

/// Get current graphics mode
/// Returns Some(mode) on success, None on failure
pub fn vesa_current_mode() -> Option<u16> {
    unsafe {
        let mut mode: u16 = 0;
        let result = vesa_get_mode(&mut mode);
        if result == 0 {
            Some(mode)
        } else {
            None
        }
    }
}

/// Check if VESA VBE graphics is available
pub fn vesa_available() -> bool {
    unsafe {
        vesa_is_available() != 0
    }
}

/// Get VESA controller capabilities
pub fn vesa_controller_capabilities() -> u8 {
    unsafe {
        vesa_get_capabilities()
    }
}

/// Get current color depth in bits per pixel
pub fn vesa_color_depth() -> u8 {
    unsafe {
        vesa_get_bits_per_pixel()
    }
}

/// Get scanline stride in bytes
pub fn vesa_scanline_bytes() -> u16 {
    unsafe {
        vesa_get_bytes_per_scanline()
    }
}

/// Get total framebuffer size in bytes
pub fn vesa_buffer_size() -> u32 {
    unsafe {
        vesa_get_framebuffer_size()
    }
}
