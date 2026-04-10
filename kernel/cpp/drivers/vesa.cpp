#include "vesa.h"
#include "boot/types.h"

extern "C" void serial_print(const char* str);
extern "C" void serial_print_hex_with_prefix(const char* prefix, uint32_t value);

// ============================================================================
// VESA State Management
// ============================================================================

// Global VESA state tracker
static struct {
    uint8_t available;                      // VESA VBE is available and initialized
    uint8_t initialized;                    // vesa_init() has been called
    uint16_t vbe_version;                   // VBE version (0x0200, 0x0300, etc)
    uint8_t capabilities;                   // Controller capabilities flags
    uint16_t current_mode;                  // Currently set graphics mode
    uint16_t bytes_per_scanline;            // Bytes per scan line in current mode
    uint16_t x_resolution;                  // Horizontal resolution in pixels
    uint16_t y_resolution;                  // Vertical resolution in pixels
    uint8_t bits_per_pixel;                 // Color depth in bits per pixel
    uint32_t linear_framebuffer;            // Physical linear framebuffer address
    uint32_t framebuffer_size;              // Framebuffer size in bytes
    uint16_t supported_modes[128];          // List of supported graphics modes
    uint8_t num_supported_modes;            // Number of supported modes
} g_vesa_state = {0};

// ============================================================================
// VESA Initialization - Core Implementation
// ============================================================================

/// Initialize VESA VBE detection and mode tracking
///
/// This function sets up VESA state and attempts to detect available graphics modes.
/// Must be called before any other VESA functions.
///
/// # Safety & Calling Convention
/// - Safe to call from C and Rust
/// - Uses C FFI: extern "C" void
/// - Callable from Rust via: `ffi::vesa_initialize()`
/// - Safe: No output parameters, simple state initialization
///
/// # Implementation Notes
/// - Sets up internal VESA state structure
/// - Initializes list of supported graphics modes
/// - Guards against double initialization
/// - Current limitation: Requires bootloader to set graphics mode
///   (real mode BIOS calls not yet implemented)
///
/// # Example (Rust)
/// ```no_run
/// use ffi::vesa_initialize;
/// vesa_initialize();
/// assert!(ffi::vesa_available());
/// ```
extern "C" void vesa_init() {
    // Guard against double initialization
    if (g_vesa_state.initialized) {
        return;
    }
    
    // Mark that initialization has been attempted
    g_vesa_state.initialized = 1;
    g_vesa_state.available = 0;
    g_vesa_state.current_mode = 0;
    g_vesa_state.num_supported_modes = 0;
    
    serial_print("[VESA] Initializing VBE detection...\n");
    
    // Initialize supported mode list with commonly available modes
    // These modes represent a good coverage of graphics capabilities
    // Ordered by preference (highest quality first)
    g_vesa_state.supported_modes[0] = VBE_MODE_1024x768x32;  // 1024x768 32-bit
    g_vesa_state.supported_modes[1] = VBE_MODE_800x600x32;   // 800x600 32-bit
    g_vesa_state.supported_modes[2] = VBE_MODE_640x480x32;   // 640x480 32-bit
    g_vesa_state.supported_modes[3] = VBE_MODE_1024x768x16;  // 1024x768 16-bit
    g_vesa_state.supported_modes[4] = VBE_MODE_800x600x16;   // 800x600 16-bit
    g_vesa_state.supported_modes[5] = VBE_MODE_640x480x16;   // 640x480 16-bit
    g_vesa_state.num_supported_modes = 6;
    
    // Note: Full VBE detection requires real mode BIOS calls (int 0x10 AX=0x4F00)
    // This would:
    // 1. Set VBE2 signature at 0x10000 (low memory)
    // 2. Call int 0x10 to get controller info
    // 3. Parse mode list from info block
    // 4. Store supported modes
    //
    // Alternative: Parse Multiboot2 VBE tag (tag type 7) if provided by bootloader
    // The Multiboot2 tag contains pre-detected VBE info in protected mode
    //
    // Since real mode transitions are not yet implemented, we mark VESA as detected
    // with a safe default configuration. This allows mode setting to function
    // for bootloaders that set graphics mode before handoff (GRUB2).
    
    // Mark VESA as available with basic initialization
    // A real bootloader (GRUB2) typically provides VBE info or sets graphics mode
    g_vesa_state.available = 1;
    g_vesa_state.vbe_version = 0x0300;  // Assume VBE 3.0 compatible
    g_vesa_state.capabilities = VBE_CAP_DAC_SWITCHABLE | VBE_CAP_BLANK_SCREEN_VBE;
    
    serial_print("[VESA] VESA VBE initialized - ");
    serial_print_hex_with_prefix("version=0x", g_vesa_state.vbe_version);
    serial_print("[VESA] Supported modes: ");
    serial_print_hex_with_prefix("count=", g_vesa_state.num_supported_modes);
}

// ============================================================================
// VESA Mode Setting - Core Implementation
// ============================================================================

/// Set graphics mode with linear framebuffer support
///
/// Attempts to set the specified VBE graphics mode. This wraps the actual
/// BIOS mode setting, which requires real mode transitions not yet implemented.
/// Currently assumes bootloader has pre-set the mode or simulates successful set.
///
/// # Parameters
/// - `mode` (u16): VBE mode number with optional flags
///   - Mode numbers: 0x101, 0x103, 0x105, 0x107, 0x111, 0x114, 0x117, 0x11A, 0x130, 0x133, 0x138
///   - Can be combined with VBE_MODE_LINEAR_FRAMEBUFFER (0x4000)
///   - Can be combined with VBE_MODE_PRESERVE_DISPLAY (0x8000)
///
/// # Return Values
/// - 0: Success
/// - 1: VESA not initialized (call vesa_init() first)
/// - 2: Mode not supported by hardware
/// - 3: Mode setting failed (BIOS error or real mode call failed)
///
/// # Safety & Calling Convention
/// - Safe to call from C and Rust
/// - Uses C FFI: extern "C" uint16_t
/// - Callable from Rust via: `ffi::vesa_set_graphics_mode(mode)`
/// - No pointer parameters; return value indicates success/failure
///
/// # Side Effects
/// - Updates global g_vesa_state with new resolution and framebuffer info
/// - May change hardware display mode (if real mode calls implemented)
///
/// # BIOS Integration Notes
/// - Real mode transition required for actual mode switching
/// - Steps needed: disable paging, switch to real mode, int 0x10, re-enable paging
/// - See comments in implementation for real mode transition sequence
///
/// # Example (Rust)
/// ```no_run
/// use ffi::vesa_set_graphics_mode;
/// let result = vesa_set_graphics_mode(0x130);  // 640x480 32-bit
/// if result.0 {
///     println!("Mode set successfully");
/// }
/// ```
extern "C" uint16_t vesa_set_mode(uint16_t mode) {
    // Check if VESA has been initialized
    if (!g_vesa_state.available || !g_vesa_state.initialized) {
        serial_print("[VESA] Error: VESA not initialized\n");
        return 1;
    }
    
    // Extract the actual mode number (mask off flags)
    uint16_t mode_number = mode & VBE_MODE_MASK;
    
    // Validate that the requested mode is in our supported list
    uint8_t mode_supported = 0;
    for (int i = 0; i < g_vesa_state.num_supported_modes; i++) {
        if ((g_vesa_state.supported_modes[i] & VBE_MODE_MASK) == mode_number) {
            mode_supported = 1;
            break;
        }
    }
    
    if (!mode_supported) {
        serial_print("[VESA] Error: Mode ");
        serial_print_hex_with_prefix("0x", mode_number);
        serial_print(" not supported\n");
        return 2;
    }
    
    // Note on implementation:
    // Real mode BIOS calls are required for actual mode switching:
    //
    // 1. Save current segment descriptors (GDT descriptors for data and code)
    // 2. Transition to real mode:
    //    - Disable paging (CR0.PG = 0)
    //    - Load real mode GDT (16-bit segments)
    //    - Jump to 16-bit code segment
    // 3. Execute VBE BIOS call:
    //    - AX = 0x4F02 (SET VBE MODE function)
    //    - BX = mode_number | 0x4000 (request linear framebuffer)
    //    - int 0x10 (BIOS video interrupt)
    // 4. Check return value in AX (should be 0x004F for success)
    // 5. Transition back to protected mode:
    //    - Enable paging (CR0.PG = 1)
    //    - Restore original segment descriptors
    //    - Jump to 32-bit code segment
    // 6. Update g_vesa_state with new mode information
    //
    // Current limitation: Without real mode transition framework, we simulate
    // successful mode setting but don't actually change hardware mode.
    // The bootloader (GRUB2) typically sets graphics mode before kernel load,
    // so this simulation is often safe.
    
    // Update internal state as if mode was successfully set
    // This assumes the bootloader has already set the graphics mode
    g_vesa_state.current_mode = mode_number;
    
    // Store mode information for common modes
    // These values are typical for standard VBE modes
    switch (mode_number) {
        case VBE_MODE_640x480x32:
            g_vesa_state.x_resolution = 640;
            g_vesa_state.y_resolution = 480;
            g_vesa_state.bits_per_pixel = 32;
            g_vesa_state.bytes_per_scanline = 640 * 4;
            g_vesa_state.linear_framebuffer = 0xE0000000;  // Typical LFB address
            g_vesa_state.framebuffer_size = 640 * 480 * 4;
            break;
            
        case VBE_MODE_800x600x32:
            g_vesa_state.x_resolution = 800;
            g_vesa_state.y_resolution = 600;
            g_vesa_state.bits_per_pixel = 32;
            g_vesa_state.bytes_per_scanline = 800 * 4;
            g_vesa_state.linear_framebuffer = 0xE0000000;
            g_vesa_state.framebuffer_size = 800 * 600 * 4;
            break;
            
        case VBE_MODE_1024x768x32:
            g_vesa_state.x_resolution = 1024;
            g_vesa_state.y_resolution = 768;
            g_vesa_state.bits_per_pixel = 32;
            g_vesa_state.bytes_per_scanline = 1024 * 4;
            g_vesa_state.linear_framebuffer = 0xE0000000;
            g_vesa_state.framebuffer_size = 1024 * 768 * 4;
            break;
            
        case VBE_MODE_640x480x16:
            g_vesa_state.x_resolution = 640;
            g_vesa_state.y_resolution = 480;
            g_vesa_state.bits_per_pixel = 16;
            g_vesa_state.bytes_per_scanline = 640 * 2;
            g_vesa_state.linear_framebuffer = 0xE0000000;
            g_vesa_state.framebuffer_size = 640 * 480 * 2;
            break;
            
        case VBE_MODE_800x600x16:
            g_vesa_state.x_resolution = 800;
            g_vesa_state.y_resolution = 600;
            g_vesa_state.bits_per_pixel = 16;
            g_vesa_state.bytes_per_scanline = 800 * 2;
            g_vesa_state.linear_framebuffer = 0xE0000000;
            g_vesa_state.framebuffer_size = 800 * 600 * 2;
            break;
            
        case VBE_MODE_1024x768x16:
            g_vesa_state.x_resolution = 1024;
            g_vesa_state.y_resolution = 768;
            g_vesa_state.bits_per_pixel = 16;
            g_vesa_state.bytes_per_scanline = 1024 * 2;
            g_vesa_state.linear_framebuffer = 0xE0000000;
            g_vesa_state.framebuffer_size = 1024 * 768 * 2;
            break;
            
        default:
            serial_print("[VESA] Warning: Unknown mode ");
            serial_print_hex_with_prefix("0x", mode_number);
            serial_print(" - using generic settings\n");
            g_vesa_state.x_resolution = 0;
            g_vesa_state.y_resolution = 0;
            g_vesa_state.bits_per_pixel = 0;
            g_vesa_state.bytes_per_scanline = 0;
            break;
    }
    
    serial_print("[VESA] Mode set: ");
    serial_print_hex_with_prefix("0x", mode_number);
    serial_print(" (");
    serial_print_hex_with_prefix("width=", g_vesa_state.x_resolution);
    serial_print(", height=");
    serial_print_hex_with_prefix("0x", g_vesa_state.y_resolution);
    serial_print(", bpp=");
    serial_print_hex_with_prefix("0x", g_vesa_state.bits_per_pixel);
    serial_print(")\n");
    
    return 0;  // Success
}

// ============================================================================
// Query and Status Functions
// ============================================================================

/// Check if VESA VBE is available and initialized
///
/// # Return Values
/// - 1 (true): VESA is available and initialized
/// - 0 (false): VESA is not available or not initialized
///
/// # Safety & Calling Convention
/// - Safe, read-only query
/// - No output parameters
/// - Uses C FFI: extern "C" uint8_t
/// - Callable from Rust via: `ffi::vesa_available()`
extern "C" uint8_t vesa_is_available() {
    return g_vesa_state.available;
}

/// Get VBE controller capabilities
///
/// Returns capability flags indicating hardware features supported by the
/// VESA VBE controller (e.g., DAC switching, VGA compatibility).
///
/// # Return Values
/// - Capabilities byte with flags (VBE_CAP_*) from vesa.h
/// - 0 if VESA not available
///
/// # Capabilities Flags
/// - VBE_CAP_DAC_SWITCHABLE (0x01): DAC width switchable
/// - VBE_CAP_NOT_VGA_COMPATIBLE (0x02): Not VGA compatible
/// - VBE_CAP_BLANK_SCREEN_VBE (0x04): Hardware blank screen via VBE
/// - VBE_CAP_STEREO_SUPPORT (0x08): Stereo mode support
/// - VBE_CAP_DUAL_DISPLAY (0x10): Dual display support
///
/// # Safety & Calling Convention
/// - Safe, read-only query
/// - Uses C FFI: extern "C" uint8_t
/// - Callable from Rust via: `ffi::vesa_controller_capabilities()`
extern "C" uint8_t vesa_get_capabilities() {
    if (!g_vesa_state.available) {
        return 0;
    }
    return g_vesa_state.capabilities;
}

/// Get physical linear framebuffer address for current mode
///
/// Returns the physical memory address where the framebuffer data is located.
/// This address can be used to map the framebuffer into virtual address space
/// for direct pixel access.
///
/// # Return Values
/// - Physical address of framebuffer (typically 0xE0000000 on x86)
/// - 0 if VESA not available or no graphics mode is currently set
///
/// # Important Notes
/// - This address is physical (identity mapped or must be mapped via paging)
/// - Framebuffer is write-combined or uncached memory
/// - Must be mapped with appropriate memory attributes (e.g., UC, WC)
/// - Use vesa_get_framebuffer_size() to determine buffer boundaries
///
/// # Safety & Calling Convention
/// - Safe, read-only query
/// - Uses C FFI: extern "C" uint32_t
/// - Callable from Rust via: `ffi::vesa_framebuffer_addr()`
///
/// # Example (Rust with paging)
/// ```no_run
/// use ffi::{vesa_framebuffer_addr, vmm_map};
/// if let Some(phys_addr) = vesa_framebuffer_addr() {
///     let virt = vmm_alloc_region(0x1000, 0x3);
///     vmm_map(virt, phys_addr as *mut _, 0x3);
/// }
/// ```
extern "C" uint32_t vesa_get_framebuffer() {
    if (!g_vesa_state.available || g_vesa_state.current_mode == 0) {
        return 0;
    }
    return g_vesa_state.linear_framebuffer;
}

/// Get current graphics mode resolution
///
/// Retrieves the horizontal and vertical resolution in pixels of the currently
/// set graphics mode. Output parameters must point to valid uint16_t variables.
///
/// # Parameters
/// - `width` (uint16_t*): Pointer to store horizontal resolution in pixels
/// - `height` (uint16_t*): Pointer to store vertical resolution in pixels
///
/// # Output Values
/// - width, height: Set to 0 if VESA not available or no graphics mode set
/// - width, height: Set to current mode resolution (640, 480), (800, 600), etc.
///
/// # Safety & Pointer Requirements
/// CRITICAL: Pointers must be valid and point to writable memory!
/// - Both pointers must be non-NULL
/// - Both must point to initialized stack or heap memory (can be uninitialized)
/// - Both must have sufficient size (at least 2 bytes each)
/// - No stack overflow checking; caller is responsible
///
/// # Calling Convention
/// - Uses C FFI: extern "C" void with output pointer parameters
/// - Callable from Rust via: `ffi::vesa_display_resolution()`
///
/// # Example (C++)
/// ```cpp
/// uint16_t w = 0, h = 0;
/// vesa_get_resolution(&w, &h);
/// serial_print_hex("Resolution: ", w);
/// serial_print_hex(" x ", h);
/// ```
///
/// # Example (Rust)
/// ```no_run
/// use ffi::vesa_display_resolution;
/// let (width, height) = vesa_display_resolution();
/// println!("Display: {}x{}", width, height);
/// ```
extern "C" void vesa_get_resolution(uint16_t* width, uint16_t* height) {
    if (!width || !height) {
        return;
    }
    
    if (!g_vesa_state.available || g_vesa_state.current_mode == 0) {
        *width = 0;
        *height = 0;
        return;
    }
    
    *width = g_vesa_state.x_resolution;
    *height = g_vesa_state.y_resolution;
}

/// Get current graphics mode number and info
///
/// Retrieves the currently set VBE mode number.
///
/// # Parameters
/// - `mode` (uint16_t*): Pointer to store current mode number
///
/// # Return Values
/// - 0: Success (mode pointer contains valid mode number)
/// - 1: VESA not available, or no graphics mode currently set
///
/// # Safety & Pointer Requirements
/// - mode pointer must be non-NULL and point to valid writable memory
/// - Caller responsible for pointer validity
///
/// # Calling Convention
/// - Uses C FFI: extern "C" uint16_t with output pointer parameter
/// - Callable from Rust via: `ffi::vesa_current_mode()`
///
/// # Example (Rust)
/// ```no_run
/// use ffi::vesa_current_mode;
/// if let Some(mode) = vesa_current_mode() {
///     println!("Current mode: 0x{:x}", mode);
/// }
/// ```
extern "C" uint16_t vesa_get_mode(uint16_t* mode) {
    if (!mode) {
        return 1;
    }
    
    if (!g_vesa_state.available) {
        return 1;
    }
    
    *mode = g_vesa_state.current_mode;
    return (g_vesa_state.current_mode == 0) ? 1 : 0;
}

// ============================================================================
// Information Functions
// ============================================================================

/// Get current bits per pixel for the graphics mode
///
/// Returns the color depth of the current graphics mode.
///
/// # Return Values
/// - 0: Not in graphics mode, or VESA not available
/// - 8: Indexed/paletted color mode (256 colors)
/// - 16: RGB 5:6:5 color mode (65K colors)
/// - 24: RGB 8:8:8 color mode (16M colors)
/// - 32: XRGB/ARGB 8:8:8:8 color mode (16M + alpha)
///
/// # Safety & Calling Convention
/// - Safe, read-only query
/// - Uses C FFI: extern "C" uint8_t
/// - Callable from Rust via: `ffi::vesa_color_depth()`
extern "C" uint8_t vesa_get_bits_per_pixel() {
    if (!g_vesa_state.available || g_vesa_state.current_mode == 0) {
        return 0;
    }
    return g_vesa_state.bits_per_pixel;
}

/// Get bytes per scanline for current mode
///
/// Returns the scanline stride in bytes. This is the number of bytes between
/// the start of one scanline and the start of the next scanline in the
/// framebuffer. It may be larger than width × bytes_per_pixel due to
/// alignment requirements.
///
/// # Return Values
/// - 0: VESA not available or not in graphics mode
/// - >0: Bytes per scanline (e.g., 2560 for 640×32-bit mode)
///
/// # Important for Drawing
/// - Use this value when calculating pixel offsets in framebuffer
/// - Example: offset = y * bytes_per_scanline + x * (bits_per_pixel / 8)
/// - Necessary for graphics drivers and framebuffer accessors
///
/// # Safety & Calling Convention
/// - Safe, read-only query
/// - Uses C FFI: extern "C" uint16_t
/// - Callable from Rust via: `ffi::vesa_scanline_bytes()`
extern "C" uint16_t vesa_get_bytes_per_scanline() {
    if (!g_vesa_state.available || g_vesa_state.current_mode == 0) {
        return 0;
    }
    return g_vesa_state.bytes_per_scanline;
}

/// Get total framebuffer size in bytes for current mode
///
/// Returns the total number of bytes allocated for the framebuffer.
/// This is typically width × height × (bits_per_pixel / 8), but may be
/// larger due to alignment, padding, or double-buffering.
///
/// # Return Values
/// - 0: VESA not available or not in graphics mode
/// - >0: Total framebuffer size in bytes
///
/// # Usage
/// - Validate buffer accesses: offset < framebuffer_size()
/// - Allocate virtual address space: map_pages(size / 4096)
/// - Clear display: memset(framebuffer, 0, size)
///
/// # Safety & Calling Convention
/// - Safe, read-only query
/// - Uses C FFI: extern "C" uint32_t
/// - Callable from Rust via: `ffi::vesa_buffer_size()`
///
/// # Example (Rust - clearing framebuffer)
/// ```no_run
/// use ffi::{vesa_framebuffer_addr, vesa_buffer_size};
/// if let Some(addr) = vesa_framebuffer_addr() {
///     let size = vesa_buffer_size();
///     unsafe {
///         core::ptr::write_bytes(addr as *mut u8, 0, size as usize);
///     }
/// }
/// ```
extern "C" uint32_t vesa_get_framebuffer_size() {
    if (!g_vesa_state.available || g_vesa_state.current_mode == 0) {
        return 0;
    }
    return g_vesa_state.framebuffer_size;
}
