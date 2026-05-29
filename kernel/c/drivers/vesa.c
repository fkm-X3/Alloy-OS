#include "vesa.h"
#include "boot/types.h"
#include "serial.h"
#include "../boot/multiboot2.h"

static struct {
    uint8_t available;
    uint8_t initialized;
    uint8_t framebuffer_ready;
    uint16_t vbe_version;
    uint8_t capabilities;
    uint16_t current_mode;
    uint16_t bytes_per_scanline;
    uint16_t x_resolution;
    uint16_t y_resolution;
    uint8_t bits_per_pixel;
    uint32_t linear_framebuffer;
    uint32_t framebuffer_size;
    uint16_t supported_modes[128];
    uint8_t num_supported_modes;
} g_vesa_state = {0};

static uint16_t mode_for_dimensions(uint16_t width, uint16_t height, uint8_t bpp) {
    if (width == 1024 && height == 768 && bpp == 16) {
        return VBE_MODE_1024x768x16;
    }
    if (width == 800 && height == 600 && bpp == 16) {
        return VBE_MODE_800x600x16;
    }
    if (width == 640 && height == 480 && bpp == 16) {
        return VBE_MODE_640x480x16;
    }
    if (width == 1024 && height == 768 && bpp == 32) {
        return VBE_MODE_1024x768x32;
    }
    if (width == 800 && height == 600 && bpp == 32) {
        return VBE_MODE_800x600x32;
    }
    if (width == 640 && height == 480 && bpp == 32) {
        return VBE_MODE_640x480x32;
    }
    return 0;
}

static uint8_t load_multiboot_framebuffer(uint32_t multiboot_addr) {
    if (multiboot_addr == 0) {
        return 0;
    }

    struct multiboot_tag* tag = (struct multiboot_tag*)(multiboot_addr + 8);
    while (tag->type != MULTIBOOT_TAG_TYPE_END) {
        if (tag->type == MULTIBOOT_TAG_TYPE_FRAMEBUFFER) {
            struct multiboot_tag_framebuffer_common* fb =
                (struct multiboot_tag_framebuffer_common*)tag;

            if (fb->framebuffer_type == MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT) {
                serial_print("[VESA] Multiboot framebuffer is text mode\n");
                return 0;
            }

            if ((fb->framebuffer_addr >> 32) != 0) {
                serial_print("[VESA] Framebuffer address above 4GB is unsupported\n");
                return 0;
            }

            if (fb->framebuffer_addr == 0 ||
                fb->framebuffer_pitch == 0 ||
                fb->framebuffer_width == 0 ||
                fb->framebuffer_height == 0 ||
                fb->framebuffer_bpp == 0 ||
                fb->framebuffer_width > 0xFFFF ||
                fb->framebuffer_height > 0xFFFF ||
                fb->framebuffer_pitch > 0xFFFF) {
                serial_print("[VESA] Invalid multiboot framebuffer metadata\n");
                return 0;
            }

            g_vesa_state.linear_framebuffer = (uint32_t)fb->framebuffer_addr;
            g_vesa_state.bytes_per_scanline = (uint16_t)fb->framebuffer_pitch;
            g_vesa_state.x_resolution = (uint16_t)fb->framebuffer_width;
            g_vesa_state.y_resolution = (uint16_t)fb->framebuffer_height;
            g_vesa_state.bits_per_pixel = fb->framebuffer_bpp;

            uint64_t fb_size = ((uint64_t)g_vesa_state.bytes_per_scanline) *
                               ((uint64_t)g_vesa_state.y_resolution);
            if (fb_size > 0xFFFFFFFFULL) {
                serial_print("[VESA] Framebuffer size overflow\n");
                return 0;
            }
            g_vesa_state.framebuffer_size = (uint32_t)fb_size;
            g_vesa_state.current_mode = mode_for_dimensions(
                g_vesa_state.x_resolution,
                g_vesa_state.y_resolution,
                g_vesa_state.bits_per_pixel
            );
            g_vesa_state.framebuffer_ready = 1;
            return 1;
        }

        tag = (struct multiboot_tag*)((uint8_t*)tag + ((tag->size + 7) & ~7));
    }

    return 0;
}

void vesa_init_from_multiboot(uint32_t multiboot_addr) {
    if (g_vesa_state.initialized) {
        return;
    }

    g_vesa_state.initialized = 1;
    g_vesa_state.available = 0;
    g_vesa_state.framebuffer_ready = 0;
    g_vesa_state.current_mode = 0;
    g_vesa_state.num_supported_modes = 0;
    g_vesa_state.bytes_per_scanline = 0;
    g_vesa_state.x_resolution = 0;
    g_vesa_state.y_resolution = 0;
    g_vesa_state.bits_per_pixel = 0;
    g_vesa_state.linear_framebuffer = 0;
    g_vesa_state.framebuffer_size = 0;

    serial_print("[VESA] Initializing VBE detection...\n");

    g_vesa_state.supported_modes[0] = VBE_MODE_1024x768x32;
    g_vesa_state.supported_modes[1] = VBE_MODE_800x600x32;
    g_vesa_state.supported_modes[2] = VBE_MODE_640x480x32;
    g_vesa_state.supported_modes[3] = VBE_MODE_1024x768x16;
    g_vesa_state.supported_modes[4] = VBE_MODE_800x600x16;
    g_vesa_state.supported_modes[5] = VBE_MODE_640x480x16;
    g_vesa_state.num_supported_modes = 6;

    g_vesa_state.vbe_version = 0x0300;
    g_vesa_state.capabilities = VBE_CAP_DAC_SWITCHABLE | VBE_CAP_BLANK_SCREEN_VBE;

    if (!load_multiboot_framebuffer(multiboot_addr)) {
        serial_print("[VESA] No valid multiboot framebuffer metadata; graphics unavailable\n");
        return;
    }

    g_vesa_state.available = 1;

    serial_print("[VESA] VESA VBE initialized - ");
    serial_print_hex_with_prefix("version=0x", g_vesa_state.vbe_version);
    serial_print("[VESA] Supported modes: ");
    serial_print_hex_with_prefix("count=", g_vesa_state.num_supported_modes);
    serial_print("[VESA] Framebuffer: ");
    serial_print_hex_with_prefix("addr=0x", g_vesa_state.linear_framebuffer);
    serial_print_hex_with_prefix("width=0x", g_vesa_state.x_resolution);
    serial_print_hex_with_prefix("height=0x", g_vesa_state.y_resolution);
    serial_print_hex_with_prefix("bpp=0x", g_vesa_state.bits_per_pixel);
}

void vesa_init() {
    vesa_init_from_multiboot(0);
}

uint16_t vesa_set_mode(uint16_t mode) {
    if (!g_vesa_state.initialized) {
        serial_print("[VESA] Error: VESA not initialized\n");
        return 1;
    }

    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        serial_print("[VESA] Error: Bootloader framebuffer is unavailable\n");
        return 3;
    }

    uint16_t mode_number = mode & VBE_MODE_MASK;

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

    uint16_t detected_mode = mode_for_dimensions(
        g_vesa_state.x_resolution,
        g_vesa_state.y_resolution,
        g_vesa_state.bits_per_pixel
    );
    if (detected_mode == 0 || detected_mode != mode_number) {
        serial_print("[VESA] Error: Requested mode does not match active boot framebuffer\n");
        return 3;
    }

    g_vesa_state.current_mode = mode_number;

    serial_print("[VESA] Mode set: ");
    serial_print_hex_with_prefix("0x", mode_number);
    serial_print(" (");
    serial_print_hex_with_prefix("width=", g_vesa_state.x_resolution);
    serial_print(", height=");
    serial_print_hex_with_prefix("0x", g_vesa_state.y_resolution);
    serial_print(", bpp=");
    serial_print_hex_with_prefix("0x", g_vesa_state.bits_per_pixel);
    serial_print(")\n");

    return 0;
}

uint8_t vesa_is_available() {
    return g_vesa_state.available;
}

uint8_t vesa_get_capabilities() {
    if (!g_vesa_state.available) {
        return 0;
    }
    return g_vesa_state.capabilities;
}

uint32_t vesa_get_framebuffer() {
    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        return 0;
    }
    return g_vesa_state.linear_framebuffer;
}

void vesa_get_resolution(uint16_t* width, uint16_t* height) {
    if (!width || !height) {
        return;
    }

    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        *width = 0;
        *height = 0;
        return;
    }

    *width = g_vesa_state.x_resolution;
    *height = g_vesa_state.y_resolution;
}

uint16_t vesa_get_mode(uint16_t* mode) {
    if (!mode) {
        return 1;
    }

    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        return 1;
    }

    *mode = g_vesa_state.current_mode;
    return (g_vesa_state.current_mode == 0) ? 1 : 0;
}

uint8_t vesa_get_bits_per_pixel() {
    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        return 0;
    }
    return g_vesa_state.bits_per_pixel;
}

uint16_t vesa_get_bytes_per_scanline() {
    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        return 0;
    }
    return g_vesa_state.bytes_per_scanline;
}

uint32_t vesa_get_framebuffer_size() {
    if (!g_vesa_state.available || !g_vesa_state.framebuffer_ready) {
        return 0;
    }
    return g_vesa_state.framebuffer_size;
}