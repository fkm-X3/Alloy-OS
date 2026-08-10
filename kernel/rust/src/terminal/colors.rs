//! Color support for terminal output
//! 
//! Provides colored output using VGA color codes

use crate::VgaText;

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
    VgaText::set_color(fg, bg);
    VgaText::print(text);
    VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK); // Reset
}

/// Print error message in red
pub fn print_error(text: &str) {
    VgaText::set_color(COLOR_LIGHT_RED, COLOR_BLACK);
    VgaText::print("Error: ");
    VgaText::println(text);
    VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print success message in green
pub fn print_success(text: &str) {
    VgaText::set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    VgaText::println(text);
    VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print info message in cyan
pub fn print_info(text: &str) {
    VgaText::set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    VgaText::println(text);
    VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print warning message in yellow
pub fn print_warning(text: &str) {
    VgaText::set_color(COLOR_YELLOW, COLOR_BLACK);
    VgaText::print("Warning: ");
    VgaText::println(text);
    VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print prompt in cyan
pub fn print_prompt(prompt: &str) {
    VgaText::set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    VgaText::print(prompt);
    VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
}

/// Print banner with colors
pub fn print_banner() {
    unsafe {
        // Print ASCII art banner in cyan using Code Page 437 box drawing characters
        VgaText::set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
        VgaText::println_bytes(b" ");
        // Line 1: ███╗   ██╗██╗     ██╗      ██████╗ ██╗   ██╗    ██╗  ██╗███████╗██████╗ ███╗   ██╗ █████╗ ██╗
        VgaText::println_bytes(b" \xDB\xDB\xDB\xB9   \xDB\xDB\xB9\xDB\xDB\xB9     \xDB\xDB\xB9      \xDB\xDB\xDB\xDB\xDB\xDB\xB9 \xDB\xDB\xB9   \xDB\xDB\xB9    \xDB\xDB\xB9  \xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xB9 \xDB\xDB\xDB\xB9   \xDB\xDB\xB9 \xDB\xDB\xDB\xDB\xDB\xB9 \xDB\xDB\xB9     ");
        // Line 2: ██╔══██╗██║     ██║     ██╔═══██╗╚██╗ ██╔╝    ██║ ██╔╝██╔════╝██╔══██╗████╗  ██║██╔══██╗██║
        VgaText::println_bytes(b"\xDB\xDB\xB2\x94\x94\xDB\xDB\xB9\xDB\xDB\xB9     \xDB\xDB\xB9     \xDB\xDB\xB2\x94\x94\x94\xDB\xDB\xB9\xBA\xDB\xB9 \xDB\xDB\xB2\x94\xB8    \xDB\xDB\xB9 \xDB\xDB\xB2\x94\xB8\xDB\xDB\xB2\x94\x94\x94\x94\x94\x94\xB8\xDB\xDB\xB2\x94\x94\xDB\xDB\xB9\xDB\xDB\xDB\xDB\xB9  \xDB\xDB\xB9\xDB\xDB\xB2\x94\x94\xDB\xDB\xB9\xDB\xDB\xB9     ");
        // Line 3: ███████║██║     ██║     ██║   ██║ ╚████╔╝     █████╔╝ █████╗  ██████╔╝██╔██╗ ██║███████║██║
        VgaText::println_bytes(b"\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9\xDB\xDB\xB9     \xDB\xDB\xB9     \xDB\xDB\xB9   \xDB\xDB\xB9 \xBA\xDB\xDB\xDB\xDB\xB2\x94\xB8     \xDB\xDB\xDB\xDB\xDB\xB2\x94\xB8 \xDB\xDB\xDB\xDB\xDB\xB9  \xDB\xDB\xDB\xDB\xDB\xDB\xB2\x94\xB8\xDB\xDB\xB2\x94\xDB\xDB\xB9 \xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9\xDB\xDB\xB9     ");
        // Line 4: ██╔══██║██║     ██║     ██║   ██║  ╚██╔╝      ██╔═██╗ ██╔══╝  ██╔══██╗██║╚██╗██║██╔══██║██║
        VgaText::println_bytes(b"\xDB\xDB\xB2\x94\x94\xDB\xDB\xB9\xDB\xDB\xB9     \xDB\xDB\xB9     \xDB\xDB\xB9   \xDB\xDB\xB9  \xBA\xDB\xDB\xB2\x94\xB8      \xDB\xDB\xB2\x94\x94\xDB\xDB\xB9 \xDB\xDB\xB2\x94\x94\xB8  \xDB\xDB\xB2\x94\x94\xDB\xDB\xB9\xDB\xDB\xB9\xBA\xDB\xDB\xB9\xDB\xDB\xB9\xDB\xDB\xB2\x94\x94\xDB\xDB\xB9\xDB\xDB\xB9     ");
        // Line 5: ██║  ██║███████╗███████╗╚██████╔╝   ██║       ██║  ██╗███████╗██║  ██║██║ ╚████║██║  ██║███████╗
        VgaText::println_bytes(b"\xDB\xDB\xB9  \xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9\xBA\xDB\xDB\xDB\xDB\xDB\xDB\xB2\x94\xB8   \xDB\xDB\xB9       \xDB\xDB\xB9  \xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9\xDB\xDB\xB9  \xDB\xDB\xB9\xDB\xDB\xB9 \xBA\xDB\xDB\xDB\xDB\xB9\xDB\xDB\xB9  \xDB\xDB\xB9\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xB9");
        // Line 6: ╚═╝  ╚═╝╚══════╝╚══════╝ ╚═════╝    ╚═╝       ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝
        VgaText::println_bytes(b"\xBA\x94\x94\xB8  \xBA\x94\x94\xB8\xBA\x94\x94\x94\x94\x94\x94\x94\x94\xB8\xBA\x94\x94\x94\x94\x94\x94\x94\x94\xB8 \xBA\x94\x94\x94\x94\x94\x94\x94\xB8    \xBA\x94\x94\xB8       \xBA\x94\x94\xB8  \xBA\x94\x94\xB8\xBA\x94\x94\x94\x94\x94\x94\x94\x94\xB8\xBA\x94\x94\xB8  \xBA\x94\x94\xB8\xBA\x94\x94\xB8  \xBA\x94\x94\x94\x94\x94\xB8\xBA\x94\x94\xB8  \xBA\x94\x94\xB8\xBA\x94\x94\x94\x94\x94\x94\x94\x94\xB8");
        VgaText::println_bytes(b" ");
        
        VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
    }
}

/// Print "TeSt 1@3" banner in DOS-style ASCII art
pub fn print_test_banner() {
    unsafe {
        // Print the test banner in yellow/gold (DOS style) with block characters
        VgaText::set_color(COLOR_YELLOW, COLOR_BLACK);
        VgaText::println_bytes(b" ");
        // Create a nice DOS-era style banner with "TeSt 1@3"
        // Line 1: ████████████████████████████████████████████████████████████████
        VgaText::println_bytes(b"\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB");
        // Line 2: ██  TeSt 1@3                                                    ██
        VgaText::println_bytes(b"\xDB\xDB  TeSt 1@3                                                    \xDB\xDB");
        // Line 3: ████████████████████████████████████████████████████████████████
        VgaText::println_bytes(b"\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB\xDB");
        VgaText::println_bytes(b" ");
        
        VgaText::set_color(COLOR_LIGHT_GRAY, COLOR_BLACK);
    }
}
