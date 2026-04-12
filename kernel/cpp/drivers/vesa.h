#ifndef ALLOY_VESA_H
#define ALLOY_VESA_H

#include "boot/types.h"

// VESA VBE 2.0/3.0 Specification structures and definitions

// ============================================================================
// VESA VBE Mode Constants
// ============================================================================

// VBE mode bits
#define VBE_MODE_LINEAR_FRAMEBUFFER 0x4000  // Linear framebuffer mode
#define VBE_MODE_PRESERVE_DISPLAY   0x8000  // Don't clear display
#define VBE_MODE_MASK               0x3FFF  // Mode number mask

// Common VESA graphics modes
#define VBE_MODE_640x480x8          0x101   // 640x480 8-bit color
#define VBE_MODE_800x600x8          0x103   // 800x600 8-bit color
#define VBE_MODE_1024x768x8         0x105   // 1024x768 8-bit color
#define VBE_MODE_1280x1024x8        0x107   // 1280x1024 8-bit color

#define VBE_MODE_640x480x16         0x111   // 640x480 16-bit color
#define VBE_MODE_800x600x16         0x114   // 800x600 16-bit color
#define VBE_MODE_1024x768x16        0x117   // 1024x768 16-bit color
#define VBE_MODE_1280x1024x16       0x11A   // 1280x1024 16-bit color

#define VBE_MODE_640x480x32         0x130   // 640x480 32-bit color
#define VBE_MODE_800x600x32         0x133   // 800x600 32-bit color
#define VBE_MODE_1024x768x32        0x138   // 1024x768 32-bit color

// VBE function numbers
#define VBE_FUNCTION_GET_INFO       0x4F00  // Get VBE controller info
#define VBE_FUNCTION_GET_MODE_INFO  0x4F01  // Get VBE mode info
#define VBE_FUNCTION_SET_MODE       0x4F02  // Set VBE mode
#define VBE_FUNCTION_GET_MODE       0x4F03  // Get current VBE mode
#define VBE_FUNCTION_SAVE_STATE     0x4F04  // Save VBE state
#define VBE_FUNCTION_RESTORE_STATE  0x4F05  // Restore VBE state

// VBE return codes
#define VBE_STATUS_SUCCESS          0x004F  // Function supported and successful
#define VBE_STATUS_UNSUPPORTED      0x014F  // Function not supported
#define VBE_STATUS_INVALID          0x024F  // Invalid in current mode

// ============================================================================
// VBE Info Block (VBE Controller Information)
// ============================================================================

struct vbe_info_block {
    uint8_t signature[4];              // "VBE2" or "VBE3"
    uint16_t version;                  // VBE version (0x0200 for 2.0, 0x0300 for 3.0)
    uint32_t oem_string_ptr;           // Far pointer to OEM string
    uint8_t capabilities;              // Controller capabilities
    uint32_t video_mode_ptr;           // Far pointer to supported modes list
    uint16_t total_memory;             // Total video memory in 64K blocks
    uint16_t oem_software_rev;         // OEM software revision
    uint32_t oem_vendor_name_ptr;      // Far pointer to vendor name
    uint32_t oem_product_name_ptr;     // Far pointer to product name
    uint32_t oem_product_rev_ptr;      // Far pointer to product revision
    
    // VBE 3.0 extensions
    uint8_t reserved[222];             // Reserved for future expansion
    uint8_t oem_data[256];             // OEM-specific data
} __attribute__((packed));

// Capabilities flags
#define VBE_CAP_DAC_SWITCHABLE       0x01  // DAC width is switchable
#define VBE_CAP_NOT_VGA_COMPATIBLE   0x02  // Controller is not VGA compatible
#define VBE_CAP_BLANK_SCREEN_VBE     0x04  // Hardware blank screen via VBE functions
#define VBE_CAP_STEREO_SUPPORT       0x08  // Stereo mode support
#define VBE_CAP_DUAL_DISPLAY         0x10  // Dual display support

// ============================================================================
// Mode Info Block (VBE Mode Information)
// ============================================================================

struct vbe_mode_info_block {
    // Mandatory attributes for VBE 1.0+
    uint16_t mode_attributes;          // Mode attributes
    uint8_t win_a_attributes;          // Window A attributes
    uint8_t win_b_attributes;          // Window B attributes
    uint16_t win_granularity;          // Window granularity in KB
    uint16_t win_size;                 // Window size in KB
    uint16_t win_a_segment;            // Window A start segment
    uint16_t win_b_segment;            // Window B start segment
    uint32_t win_func_ptr;             // Far pointer to window function
    uint16_t bytes_per_scanline;       // Bytes per scan line
    
    // Mandatory attributes for VBE 1.2+
    uint16_t x_resolution;             // Horizontal resolution
    uint16_t y_resolution;             // Vertical resolution
    uint8_t x_char_size;               // Character cell width
    uint8_t y_char_size;               // Character cell height
    uint8_t number_of_planes;          // Number of memory planes
    uint8_t bits_per_pixel;            // Bits per pixel
    uint8_t number_of_banks;           // Number of banks
    uint8_t memory_model;              // Memory model type
    uint8_t bank_size;                 // Bank size in KB
    uint8_t number_of_image_pages;     // Number of image pages
    uint8_t reserved1;                 // Reserved (0x00)
    
    // Direct color fields
    uint8_t red_mask_size;             // Red mask size
    uint8_t red_field_position;        // Red field position
    uint8_t green_mask_size;           // Green mask size
    uint8_t green_field_position;      // Green field position
    uint8_t blue_mask_size;            // Blue mask size
    uint8_t blue_field_position;       // Blue field position
    uint8_t reserved_mask_size;        // Reserved mask size
    uint8_t reserved_field_position;   // Reserved field position
    uint8_t direct_color_mode_info;    // Direct color mode attributes
    
    // Mandatory attributes for VBE 2.0+
    uint32_t linear_framebuffer;       // Physical linear framebuffer address
    uint32_t off_screen_memory_offset; // Offset to off-screen memory
    uint16_t off_screen_memory_size;   // Size of off-screen memory in KB
    
    // VBE 3.0 extensions
    uint16_t linear_bytes_per_scanline;// Bytes per scan line (linear mode)
    uint8_t bank_number_of_image_pages;// Number of image pages in banked mode
    uint8_t linear_number_of_image_pages;// Number of image pages in linear mode
    uint8_t linear_red_mask_size;      // Linear mode red mask size
    uint8_t linear_red_field_position; // Linear mode red field position
    uint8_t linear_green_mask_size;    // Linear mode green mask size
    uint8_t linear_green_field_position;// Linear mode green field position
    uint8_t linear_blue_mask_size;     // Linear mode blue mask size
    uint8_t linear_blue_field_position;// Linear mode blue field position
    uint8_t linear_reserved_mask_size; // Linear mode reserved mask size
    uint8_t linear_reserved_field_position;// Linear mode reserved field position
    uint32_t max_pixel_clock;          // Maximum pixel clock
    
    uint8_t reserved2[189];            // Reserved for future expansion
} __attribute__((packed));

// Mode attributes flags
#define VBE_MODE_SUPPORTED           0x0001  // Mode is supported in hardware
#define VBE_MODE_OPTIONAL_INFO       0x0002  // Optional information available
#define VBE_MODE_BIOS_OUTPUT         0x0004  // BIOS output supported
#define VBE_MODE_COLOR_MODE          0x0008  // Color mode (vs monochrome)
#define VBE_MODE_GRAPHICS_MODE       0x0010  // Graphics mode (vs text mode)
#define VBE_MODE_NOT_VGA_COMPATIBLE  0x0020  // Not VGA compatible
#define VBE_MODE_BANK_SWITCHED       0x0040  // Windowed mode not supported
#define VBE_MODE_LINEAR_FB_AVAILABLE 0x0080  // Linear framebuffer available
#define VBE_MODE_DOUBLE_SCAN         0x0100  // Double scan mode
#define VBE_MODE_INTERLACED          0x0200  // Interlaced mode
#define VBE_MODE_TRIPPLE_BUFFER      0x0400  // Triple buffering
#define VBE_MODE_STEREO              0x0800  // Stereo mode
#define VBE_MODE_DUAL_DISPLAY        0x1000  // Dual display support

// Memory model types
#define VBE_MEMORY_MODEL_TEXT        0      // Text mode
#define VBE_MEMORY_MODEL_CGA         1      // CGA graphics
#define VBE_MEMORY_MODEL_HERCULES    2      // Hercules graphics
#define VBE_MEMORY_MODEL_PLANAR      3      // Planar mode
#define VBE_MEMORY_MODEL_PACKED_PIXEL 4     // Packed pixel mode
#define VBE_MEMORY_MODEL_NON_CHAIN   5      // Non-chain 4, 256-color mode
#define VBE_MEMORY_MODEL_DIRECT_COLOR 6    // Direct color (RGB)
#define VBE_MEMORY_MODEL_YUV         7      // YUV mode

// ============================================================================
// CRTC Info Block (Timing Information for VBE 3.0)
// ============================================================================

struct vbe_crtc_info_block {
    uint16_t horizontal_total;         // Horizontal total in pixels
    uint16_t horizontal_sync_start;    // Horizontal sync start
    uint16_t horizontal_sync_end;      // Horizontal sync end
    uint16_t vertical_total;           // Vertical total in scan lines
    uint16_t vertical_sync_start;      // Vertical sync start
    uint16_t vertical_sync_end;        // Vertical sync end
    uint8_t flags;                     // Flags
    uint32_t pixel_clock;              // Pixel clock in Hz
    uint16_t refresh_rate;             // Refresh rate in 0.01 Hz units
    uint8_t reserved[40];              // Reserved for future expansion
} __attribute__((packed));

// CRTC flags
#define VBE_CRTC_DOUBLE_SCAN         0x01  // Double scan mode
#define VBE_CRTC_INTERLACED          0x02  // Interlaced mode
#define VBE_CRTC_HSYNC_NEGATIVE      0x04  // Horizontal sync negative polarity
#define VBE_CRTC_VSYNC_NEGATIVE      0x08  // Vertical sync negative polarity

// ============================================================================
// Helper structure for VBE registers
// ============================================================================

struct vbe_registers {
    uint32_t eax;                      // Function number and return status
    uint32_t ebx;                      // Mode number or capabilities
    uint32_t ecx;                      // Reserved
    uint32_t edx;                      // Reserved
    uint32_t esi;                      // Pointer to info block
    uint32_t edi;                      // Pointer to info block
} __attribute__((packed));

// ============================================================================
// VESA VBE C FFI Functions
// ============================================================================

/// FFI Interface for VESA VBE Graphics Mode Support
///
/// This module provides C Foreign Function Interface (FFI) functions for
/// accessing VESA VBE graphics capabilities from both C++ and Rust code.
/// All functions use C linkage (extern "C") for language interoperability.
///
/// # Architecture Overview
///
/// The VESA driver implements a 9-function FFI interface for graphics mode
/// management:
///
/// 1. **Initialization**: vesa_init()
/// 2. **Mode Control**: vesa_set_mode()
/// 3. **Status Queries**: vesa_is_available(), vesa_get_mode()
/// 4. **Capability Queries**: vesa_get_capabilities()
/// 5. **Framebuffer Access**: vesa_get_framebuffer(), vesa_get_bits_per_pixel(),
///    vesa_get_bytes_per_scanline(), vesa_get_framebuffer_size()
/// 6. **Resolution Queries**: vesa_get_resolution()
///
/// # Memory Safety
///
/// Functions with pointer parameters (vesa_get_resolution, vesa_get_mode)
/// require caller to ensure pointer validity:
/// - Non-NULL pointer check is performed internally
/// - Caller responsible for sufficient buffer space
/// - No bounds checking on written data
/// - Uninitialized pointers are safe to pass (they will be initialized)
///
/// # Rust Integration
///
/// Rust code accesses VESA via FFI module:
/// - Raw C FFI declarations in kernel/rust/src/ffi.rs
/// - Safe wrappers: vesa_initialize(), vesa_set_graphics_mode(), etc.
/// - Types: u16, u32, u8 for return values; mutable pointers for output
///
/// # Example (Rust Usage)
/// ```no_run
/// use ffi;
///
/// // Initialize graphics
/// ffi::vesa_initialize();
/// assert!(ffi::vesa_available());
///
/// // Set mode
/// let (ok, code) = ffi::vesa_set_graphics_mode(0x130);  // 640x480 32-bit
/// if ok {
///     let (width, height) = ffi::vesa_display_resolution();
///     println!("Display: {}x{}", width, height);
///
///     if let Some(addr) = ffi::vesa_framebuffer_addr() {
///         println!("Framebuffer at: 0x{:x}", addr);
///     }
/// }
/// ```
///
/// # Limitations & Implementation Notes
///
/// - Real mode BIOS calls (int 0x10) not yet implemented
/// - Mode switching assumes bootloader pre-configures graphics
/// - Linear framebuffer address is typical (0xE0000000) but may vary
/// - Some mode info computed from known standard VBE modes
/// - No hardware abstraction for different video card types yet
///
/// # BIOS Integration (Future Work)
///
/// For actual mode setting without bootloader assistance:
/// 1. Prepare VBE info block at low memory address
/// 2. Disable paging and switch to real mode
/// 3. Execute: AX = 0x4F02, BX = mode | 0x4000, int 0x10
/// 4. Check return status (AX should be 0x004F)
/// 5. Re-enable paging and protected mode
/// 6. Update internal state from returned mode info
///
extern "C" {
    // Initialize VESA VBE detection and controller check
    void vesa_init();
    
    // Initialize VESA using Multiboot2 framebuffer metadata (recommended boot path)
    void vesa_init_from_multiboot(uint32_t multiboot_addr);
    
    // Set a graphics mode
    // Parameters: mode - VBE mode number (with optional VBE_MODE_LINEAR_FRAMEBUFFER flag)
    // Returns: 0 on success, 1 if not initialized, 2 if mode not supported, 3 if mode setting failed
    uint16_t vesa_set_mode(uint16_t mode);
    
    // Get current framebuffer linear address
    // Returns: Physical address of linear framebuffer, 0 if not available
    uint32_t vesa_get_framebuffer();
    
    // Get current graphics mode resolution
    // Parameters: width, height - pointers to store resolution
    void vesa_get_resolution(uint16_t* width, uint16_t* height);
    
    // Get current mode info
    // Parameters: mode - pointer to store current mode number
    // Returns: 0 on success, non-zero on failure
    uint16_t vesa_get_mode(uint16_t* mode);
    
    // Check if VESA VBE is available
    // Returns: 1 if available, 0 if not
    uint8_t vesa_is_available();
    
    // Get controller capabilities
    // Returns: Capabilities byte from VBE info block
    uint8_t vesa_get_capabilities();
    
    // Get bits per pixel for current mode
    // Returns: 0, 16, 24, or 32 bits per pixel; 0 if not in graphics mode
    uint8_t vesa_get_bits_per_pixel();
    
    // Get bytes per scanline for current mode
    // Returns: Bytes per scan line, or 0 if not in graphics mode
    uint16_t vesa_get_bytes_per_scanline();
    
    // Get total framebuffer size in bytes
    // Returns: Size in bytes, or 0 if not in graphics mode
    uint32_t vesa_get_framebuffer_size();
}

#endif // ALLOY_VESA_H
