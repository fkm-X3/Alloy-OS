/// Color support for terminal output
/// 
/// Provides colored output using VGA color codes

use crate::ffi;

// VGA color codes
pub const COLOR_BLACK: u8 = 0;
pub const COLOR_BLUE: u8 = 1;
pub const COLOR_GREEN: u8 = 2;
pub const COLOR_CYAN: u8 = 3;
pub const COLOR_RED: u8 = 4;
pub const COLOR_MAGENTA: u8 = 5;
pub const COLOR_BROWN: u8 = 6;
pub const COLOR_LIGHT_GRAY: u8 = 7;
pub const COLOR_DARK_GRAY: u8 = 8;
pub const COLOR_LIGHT_BLUE: u8 = 9;
pub const COLOR_LIGHT_GREEN: u8 = 10;
pub const COLOR_LIGHT_CYAN: u8 = 11;
pub const COLOR_LIGHT_RED: u8 = 12;
pub const COLOR_LIGHT_MAGENTA: u8 = 13;
pub const COLOR_YELLOW: u8 = 14;
pub const COLOR_WHITE: u8 = 15;

/// Print text in a specific color
pub fn print_colored(text: &str, fg: u8, bg: u8) {
    ffi::set_vga_color(fg, bg);
    ffi::vga_print_str(text);
    ffi::set_vga_color(COLOR_LIGHT_GRAY, COLOR_BLACK); // Reset
}

/// Print error message in red
pub fn print_error(text: &str) {
    ffi::set_vga_color(COLOR_LIGHT_RED, COLOR_BLACK);
    ffi::vga_print_str("Error: ");
    ffi::vga_println_str(text);
    ffi::set_vga_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print success message in green
pub fn print_success(text: &str) {
    ffi::set_vga_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    ffi::vga_println_str(text);
    ffi::set_vga_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print info message in cyan
pub fn print_info(text: &str) {
    ffi::set_vga_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    ffi::vga_println_str(text);
    ffi::set_vga_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print warning message in yellow
pub fn print_warning(text: &str) {
    ffi::set_vga_color(COLOR_YELLOW, COLOR_BLACK);
    ffi::vga_print_str("Warning: ");
    ffi::vga_println_str(text);
    ffi::set_vga_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print prompt in cyan
pub fn print_prompt(prompt: &str) {
    ffi::set_vga_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    ffi::vga_print_str(prompt);
    ffi::set_vga_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print banner with colors
pub fn print_banner() {
    unsafe {
        // Clear screen first
        ffi::vga_set_color(COLOR_BLACK, COLOR_BLACK);
        for _ in 0..25 {
            ffi::vga_println(b"\0".as_ptr());
        }
        
        // Print banner
        ffi::vga_set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
        ffi::vga_println(b"======================================\0".as_ptr());
        ffi::vga_print(b"      \0".as_ptr());
        ffi::vga_set_color(COLOR_WHITE, COLOR_BLACK);
        ffi::vga_print(b"Alloy Operating System\0".as_ptr());
        ffi::vga_set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
        ffi::vga_println(b"\0".as_ptr());
        ffi::vga_println(b"======================================\0".as_ptr());
        
        ffi::vga_set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
        ffi::vga_println(b"\0".as_ptr());
        ffi::vga_print(b"Version: \0".as_ptr());
        ffi::vga_set_color(COLOR_YELLOW, COLOR_BLACK);
        ffi::vga_println(b"0.6.0-dev (Phase 6)\0".as_ptr());
        
        ffi::vga_set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
        ffi::vga_print(b"Type \0".as_ptr());
        ffi::vga_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
        ffi::vga_print(b"'help'\0".as_ptr());
        ffi::vga_set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
        ffi::vga_println(b" for available commands.\0".as_ptr());
        
        ffi::vga_set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
    }
}
