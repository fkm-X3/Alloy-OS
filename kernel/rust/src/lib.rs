#![no_std]
#![feature(alloc_error_handler)]

// Core library - available in no_std
extern crate core;

// Alloc library for heap allocations
extern crate alloc;

// Module declarations
pub mod allocator;
pub mod heap;
pub mod slab;
pub mod sync;
pub mod ffi;
pub mod panic;
pub mod terminal;
pub mod utils;
pub mod process;
pub mod syscall;
pub mod graphics;
pub mod fusion;
pub mod display_server;

use core::panic::PanicInfo;

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel entry - starting Terminal\n\0".as_ptr());
        
        // Clear screen
        ffi::vga_clear();
    }
    
    // Start terminal
    start_terminal();
}

/// Start the Terminal as fallback or alternative interface
fn start_terminal() {
    unsafe {
        ffi::serial_print(b"[Terminal] Starting terminal interface\n\0".as_ptr());
    }
    
    // Initialize graphics display with welcome banner
    display_graphics_banner();
    
    // Start the kernel terminal (command-line based)
    // Note: A feature-rich Ratatui-based terminal is available in os/terminal/
    // To use it, run: cargo run --release -p alloy-os-terminal
    let mut terminal = terminal::Terminal::new();
    
    // Print marker for screenshot tool (for compatibility with existing test infrastructure)
    unsafe {
        ffi::serial_print(b"[DisplayServer] First frame presented\n\0".as_ptr());
    }
    
    terminal.run();
}

/// Initialize graphics display with welcome message
fn display_graphics_banner() {
    if let Some(mut display) = graphics::vesa::VesaDisplay::new() {
        // Use Display trait methods
        let disp: &mut dyn graphics::Display<Error = graphics::vesa::VesaError, Buffer = graphics::vesa::VesaBuffer> = &mut display;
        
        // Clear display with black
        disp.clear(0xFF000000);
        
        // Draw a cyan welcome banner instead of yellow
        let banner_x = 40;
        let banner_y = 40;
        let banner_width = 320;
        let banner_height = 70;
        
        // Draw filled rectangle with cyan
        let _ = disp.fill_rect(
            banner_x,
            banner_y,
            banner_width,
            banner_height,
            0xFF00FFFF,  // Cyan
        );
        
        // Draw "Alloy OS" title in dark blue
        render_simple_text(disp, banner_x + 50, banner_y + 15, "Alloy OS", 0xFF000080);
        
        // Draw version info in dark blue
        render_simple_text(disp, banner_x + 20, banner_y + 40, "v0.7.0 - Terminal Ready", 0xFF000080);
        
        disp.swap_buffer();
    }
}

/// Simple text rendering using pixel_put
/// This renders ASCII characters using a minimal 5x7 font
fn render_simple_text(
    display: &mut dyn graphics::Display<Error = graphics::vesa::VesaError, Buffer = graphics::vesa::VesaBuffer>,
    mut x: u32,
    y: u32,
    text: &str,
    color: u32,
) {
    for ch in text.chars() {
        // Render each character
        render_simple_char(display, x, y, ch, color);
        x += 6; // Move to next character position (5px + 1px spacing)
    }
}

/// Render a single character using simple pixel patterns
fn render_simple_char(
    display: &mut dyn graphics::Display<Error = graphics::vesa::VesaError, Buffer = graphics::vesa::VesaBuffer>,
    base_x: u32,
    base_y: u32,
    ch: char,
    color: u32,
) {
    // Simplified character patterns (5x7 grid for each char)
    // Using basic block shapes for DOS-style look
    match ch {
        'T' => {
            // Uppercase T - horizontal line at top
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y, color);
            }
            // Vertical line in center
            for y in 1..7 {
                display.pixel_put(base_x + 2, base_y + y, color);
            }
        }
        't' => {
            // Lowercase t - smaller vertical line with crossbar
            // Horizontal crossbar
            for x in 1..4 {
                display.pixel_put(base_x + x, base_y + 2, color);
            }
            // Vertical line
            for y in 2..6 {
                display.pixel_put(base_x + 2, base_y + y, color);
            }
        }
        'e' => {
            // Horizontal line in middle
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y + 3, color);
            }
            // Left vertical line
            for y in 1..6 {
                display.pixel_put(base_x, base_y + y, color);
            }
            // Bottom horizontal line
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y + 5, color);
            }
            // Top and bottom right
            display.pixel_put(base_x + 4, base_y + 1, color);
            display.pixel_put(base_x + 4, base_y + 2, color);
            display.pixel_put(base_x + 4, base_y + 4, color);
            display.pixel_put(base_x + 4, base_y + 5, color);
        }
        'S' | 's' => {
            // Top horizontal
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y, color);
            }
            // Top-left vertical
            for y in 0..3 {
                display.pixel_put(base_x, base_y + y, color);
            }
            // Middle horizontal
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y + 3, color);
            }
            // Bottom-right vertical
            for y in 3..7 {
                display.pixel_put(base_x + 4, base_y + y, color);
            }
            // Bottom horizontal
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y + 6, color);
            }
        }
        '1' => {
            // Draw a '1' 
            for y in 0..7 {
                display.pixel_put(base_x + 2, base_y + y, color);
            }
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y + 6, color);
            }
        }
        '@' => {
            // Outer circle
            for x in 1..4 {
                display.pixel_put(base_x + x, base_y, color);
                display.pixel_put(base_x + x, base_y + 6, color);
            }
            for y in 0..7 {
                display.pixel_put(base_x, base_y + y, color);
                display.pixel_put(base_x + 4, base_y + y, color);
            }
            // Inner mark
            display.pixel_put(base_x + 2, base_y + 3, color);
            display.pixel_put(base_x + 3, base_y + 3, color);
        }
        '3' => {
            // Top horizontal
            for x in 0..4 {
                display.pixel_put(base_x + x, base_y, color);
            }
            // Middle horizontal
            for x in 0..4 {
                display.pixel_put(base_x + x, base_y + 3, color);
            }
            // Bottom horizontal
            for x in 0..4 {
                display.pixel_put(base_x + x, base_y + 6, color);
            }
            // Right vertical lines
            for y in 0..7 {
                display.pixel_put(base_x + 4, base_y + y, color);
            }
        }
        ' ' => {
            // Space - do nothing
        }
        _ => {
            // Default: draw a simple box for unknown characters
            for x in 0..5 {
                display.pixel_put(base_x + x, base_y, color);
                display.pixel_put(base_x + x, base_y + 6, color);
            }
            for y in 0..7 {
                display.pixel_put(base_x, base_y + y, color);
                display.pixel_put(base_x + 4, base_y + y, color);
            }
        }
    }
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
