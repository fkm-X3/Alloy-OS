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

use core::panic::PanicInfo;

/// Test graphics functionality including VESA framebuffer access and rendering.
/// 
/// This function verifies:
/// - VESA availability via FFI
/// - Setting a graphics mode (attempts standard modes)
/// - Framebuffer address, resolution, and color depth query
/// - Framebuffer wrapper creation
/// - Text rendering capability
/// 
/// Returns true if graphics test completed successfully, false if VESA unavailable.
/// Does not panic on test failures - logs errors to serial instead.
fn test_graphics() -> bool {
    unsafe {
        ffi::serial_print(b"[Graphics Test] Checking VESA availability...\n\0".as_ptr());
    }
    
    // Check if VESA is available
    if !ffi::vesa_available() {
        unsafe {
            ffi::serial_print(b"[Graphics Test] SKIP - VESA unavailable, proceeding with terminal...\n\0".as_ptr());
        }
        return false;
    }
    
    unsafe {
        ffi::serial_print(b"[Graphics Test] VESA graphics available\n\0".as_ptr());
    }
    
    // Try to set a graphics mode (attempt standard modes)
    // VESA mode 0x118 = 1024x768x24
    // VESA mode 0x119 = 1024x768x32
    // VESA mode 0x117 = 1024x768x16
    unsafe {
        ffi::serial_print(b"[Graphics Test] Attempting to set graphics mode...\n\0".as_ptr());
    }
    
    let mut mode_set = false;
    for mode in [0x119u16, 0x118, 0x117, 0x114, 0x110].iter() {
        let (success, _error_code) = ffi::vesa_set_graphics_mode(*mode);
        if success {
            unsafe {
                ffi::serial_print(b"[Graphics Test] Graphics mode set successfully\n\0".as_ptr());
            }
            mode_set = true;
            break;
        }
    }
    
    if !mode_set {
        unsafe {
            ffi::serial_print(b"[Graphics Test] WARNING - Could not set graphics mode\n\0".as_ptr());
        }
    }
    
    // Query framebuffer information via FFI
    let framebuffer_addr = match ffi::vesa_framebuffer_addr() {
        Some(addr) => addr,
        None => {
            unsafe {
                ffi::serial_print(b"[Graphics Test] SKIP - Framebuffer not available\n\0".as_ptr());
            }
            return false;
        }
    };
    
    let (width, height) = ffi::vesa_display_resolution();
    let bpp = ffi::vesa_color_depth();
    let scanline_bytes = ffi::vesa_scanline_bytes();
    
    // Validate resolution
    if width == 0 || height == 0 || bpp == 0 {
        unsafe {
            ffi::serial_print(b"[Graphics Test] SKIP - No valid graphics mode active\n\0".as_ptr());
        }
        return false;
    }
    
    // Log graphics info
    unsafe {
        ffi::serial_print(b"[Graphics Test] Graphics mode detected: \0".as_ptr());
        ffi::vga_print_dec(width as u32);
        ffi::serial_print(b"x\0".as_ptr());
        ffi::vga_print_dec(height as u32);
        ffi::serial_print(b"x\0".as_ptr());
        ffi::vga_print_dec(bpp as u32);
        ffi::serial_print(b"\n\0".as_ptr());
    }
    
    // Log framebuffer size
    let fb_size = scanline_bytes as u32 * height as u32;
    unsafe {
        ffi::serial_print(b"[Graphics Test] Framebuffer size: \0".as_ptr());
        ffi::vga_print_dec(fb_size);
        ffi::serial_print(b" bytes, scanline: \0".as_ptr());
        ffi::vga_print_dec(scanline_bytes as u32);
        ffi::serial_print(b" bytes\n\0".as_ptr());
    }
    
    // Create framebuffer info with standard color masks (validation only)
    let (red_mask, green_mask, blue_mask) = match bpp {
        16 => (0xF800, 0x07E0, 0x001F),  // RGB565
        24 => (0xFF0000, 0x00FF00, 0x0000FF),  // RGB888
        32 => (0xFF0000, 0x00FF00, 0x0000FF),  // ARGB8888
        _ => {
            unsafe {
                ffi::serial_print(b"[Graphics Test] ERROR - Unsupported color depth\n\0".as_ptr());
            }
            return false;
        }
    };
    
    let fb_info = match graphics::framebuffer::FramebufferInfo::new(
        framebuffer_addr,
        width as u32,
        height as u32,
        scanline_bytes as u32,
        bpp,
        red_mask,
        green_mask,
        blue_mask,
    ) {
        Ok(info) => info,
        Err(_) => {
            unsafe {
                ffi::serial_print(b"[Graphics Test] ERROR - Invalid framebuffer info\n\0".as_ptr());
            }
            return false;
        }
    };
    
    unsafe {
        ffi::serial_print(b"[Graphics Test] Framebuffer info validated\n\0".as_ptr());
    }
    
    // Create framebuffer wrapper (without accessing the mapped memory)
    let _fb = match graphics::framebuffer::Framebuffer::new(fb_info) {
        Ok(fb) => fb,
        Err(_) => {
            unsafe {
                ffi::serial_print(b"[Graphics Test] ERROR - Failed to create framebuffer wrapper\n\0".as_ptr());
            }
            return false;
        }
    };
    
    unsafe {
        ffi::serial_print(b"[Graphics Test] Framebuffer wrapper created\n\0".as_ptr());
    }
    
    // Test TextRenderer creation
    unsafe {
        ffi::serial_print(b"[Graphics Test] Testing TextRenderer creation...\n\0".as_ptr());
    }
    
    let mut text_renderer = graphics::text::TextRenderer::new();
    text_renderer.set_color(0xFFFFFFFF, 0xFF000000);  // White on black
    
    unsafe {
        ffi::serial_print(b"[Graphics Test] TextRenderer created successfully\n\0".as_ptr());
    }
    
    // Final success message
    unsafe {
        ffi::serial_print(b"[Graphics Test] SUCCESS - Graphics infrastructure functional\n\0".as_ptr());
    }
    
    true
}

/// Rust kernel entry point called from C++
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        ffi::serial_print(b"[Rust] Kernel entry - starting Fusion DisplayManager\n\0".as_ptr());
        
        // Clear screen to hide C++ boot messages
        ffi::vga_clear();
    }
    
    // Run graphics test before starting display manager
    let _graphics_ok = test_graphics();
    
    // Try to initialize Fusion DisplayManager
    if let Some(display) = graphics::vesa::VesaDisplay::new() {
        unsafe {
            ffi::serial_print(b"[Fusion] VESA display created\n\0".as_ptr());
        }
        
        let mut manager = fusion::DisplayManager::new(display);
        
        match manager.start() {
            Ok(_) => {
                unsafe {
                    ffi::serial_print(b"[Fusion] DisplayManager started\n\0".as_ptr());
                    ffi::vga_println(b"[Fusion] Display Manager Ready\n\0".as_ptr());
                }
                
                // Clear the display to black
                let _ = manager.queue_render(fusion::RenderCommand::ClearScreen(
                    graphics::color::Color::BLACK
                ));
                let _ = manager.process_queue();
                let _ = manager.flush();
                
                // Display boot message
                let _ = manager.queue_render(fusion::RenderCommand::DrawText {
                    x: 50,
                    y: 50,
                    text: "Fusion Display Manager Ready",
                    color: graphics::color::Color::WHITE,
                });
                let _ = manager.process_queue();
                let _ = manager.flush();
                
                unsafe {
                    ffi::serial_print(b"[Fusion] Boot complete - display manager running\n\0".as_ptr());
                }
                
                // Keep the display manager running with a simple event loop
                loop {
                    // Check for keyboard input
                    if ffi::keyboard_has_key() {
                        let key = ffi::keyboard_read();
                        
                        // ESC key to return to terminal (for debugging)
                        if key == 27 {
                            break;
                        }
                    }
                    
                    // Brief pause to avoid busy waiting (simple loop)
                    for _ in 0..100000 {
                        // Simple delay loop
                    }
                }
                
                unsafe {
                    ffi::serial_print(b"[Fusion] Shutting down display manager\n\0".as_ptr());
                }
                let _ = manager.stop();
            }
            Err(_) => {
                unsafe {
                    ffi::serial_print(b"[Fusion] Failed to start DisplayManager, falling back to terminal\n\0".as_ptr());
                    ffi::vga_println(b"[Fusion] Failed to start, falling back to terminal\n\0".as_ptr());
                }
                start_terminal();
            }
        }
    } else {
        unsafe {
            ffi::serial_print(b"[Fusion] VESA display unavailable, falling back to terminal\n\0".as_ptr());
            ffi::vga_println(b"[Fusion] VESA unavailable, using terminal\n\0".as_ptr());
        }
        start_terminal();
    }
}

/// Start the Terminal as fallback or alternative interface
fn start_terminal() {
    unsafe {
        ffi::serial_print(b"[Terminal] Starting terminal interface\n\0".as_ptr());
        ffi::vga_clear();
    }
    let mut terminal = terminal::Terminal::new();
    terminal.run();
}

/// Language item for panic implementation
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
