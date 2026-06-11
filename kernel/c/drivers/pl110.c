// PL110 PrimeCell Color LCD Controller driver for QEMU virt aarch64
// Base address: 0x1E200000 on QEMU virt machine

#include "boot/types.h"

#define PL110_BASE 0x1E200000

// Registers (32-bit, MMIO)
#define PL110_LCDTIMING0  0x000
#define PL110_LCDTIMING1  0x004
#define PL110_LCDTIMING2  0x008
#define PL110_LCDTIMING3  0x00C
#define PL110_LCDUPBASE   0x010  // Upper panel frame base
#define PL110_LCDLPBASE   0x014  // Lower panel frame base
#define PL110_LCDCONTROL  0x018
#define PL110_LCDIMSC     0x01C  // Interrupt mask
#define PL110_LCDRIS      0x020  // Raw interrupt status
#define PL110_LCDMIS      0x024  // Masked interrupt status
#define PL110_LCDICR      0x028  // Interrupt clear
#define PL110_LCDENCODE   0x03C  // Palette encode

// LCDCONTROL bits
#define LCDCTL_ENABLE     (1 << 0)
#define LCDCTL_LCDPWR     (1 << 11)
#define LCDCTL_LCDBPP16   (1 << 1)   // 16bpp (RGB565)
#define LCDCTL_LCDBPP24   (1 << 2)   // 24bpp
#define LCDCTL_BGR        (1 << 3)   // BGR order instead of RGB
#define LCDCTL_LCDMONO8   (1 << 4)   // 8-bit mono
#define LCDCTL_TFT        (1 << 5)   // TFT panel
#define LCDCTL_LCDBW      (1 << 6)   // B/W mode
#define LCDCTL_WATERMARK  (1 << 8)   // Watermark level

// PL110 state
static uint32_t framebuffer_phys = 0;
static uint16_t fb_width = 1024;
static uint16_t fb_height = 768;
static uint8_t fb_bpp = 16;
static int pl110_initialized = 0;

static inline void mmio_write32(uintptr_t addr, uint32_t val) {
    *(volatile uint32_t*)addr = val;
}

static inline uint32_t mmio_read32(uintptr_t addr) {
    return *(volatile uint32_t*)addr;
}

void pl110_init(uint32_t fb_addr, uint16_t width, uint16_t height) {
    uintptr_t base = PL110_BASE;

    fb_width = width;
    fb_height = height;
    fb_bpp = 16;
    framebuffer_phys = fb_addr;

    // Disable controller while configuring
    mmio_write32(base + PL110_LCDCONTROL, 0);

    // Configure timing for 1024x768 @ ~60Hz (QEMU virt defaults)
    // These are reasonable default timing values
    // Horizontal: 1024 pixels + HFP(160) + HSW(40) + HBP(160) = 1384
    // Vertical: 768 pixels + VFP(12) + VSW(6) + VBP(24) = 810
    uint32_t ppl = width - 1;       // Pixels per line - 1
    uint32_t hsw = 40;              // Horizontal sync width
    uint32_t hfp = 160;             // Horizontal front porch
    uint32_t hbp = 160;             // Horizontal back porch
    mmio_write32(base + PL110_LCDTIMING0, (hsw << 24) | (ppl << 2));

    uint32_t lpp = height - 1;      // Lines per panel - 1
    uint32_t vsw = 6;               // Vertical sync width
    uint32_t vfp = 12;              // Vertical front porch
    uint32_t vbp = 24;              // Vertical back porch
    mmio_write32(base + PL110_LCDTIMING1, (vsw << 24) | (lpp << 2));

    // Vertical frequency and AC bias
    mmio_write32(base + PL110_LCDTIMING2, (vbp << 8) | vfp);
    // Horizontal frequency
    mmio_write32(base + PL110_LCDTIMING3, (hbp << 8) | hfp);

    // Set framebuffer base address (physical)
    mmio_write32(base + PL110_LCDUPBASE, fb_addr);

    // Configure panel: TFT, 16bpp (RGB565), enable
    mmio_write32(base + PL110_LCDCONTROL, LCDCTL_TFT | LCDCTL_LCDBPP16 | LCDCTL_ENABLE | LCDCTL_LCDPWR);

    // Clear interrupt status
    mmio_write32(base + PL110_LCDICR, 0xFFFFFFFF);

    pl110_initialized = 1;
}

int pl110_is_available() {
    return pl110_initialized;
}

uint32_t pl110_get_framebuffer() {
    if (!pl110_initialized) return 0;
    return framebuffer_phys;
}

void pl110_get_resolution(uint16_t* width, uint16_t* height) {
    if (width)  *width  = fb_width;
    if (height) *height = fb_height;
}

uint8_t pl110_get_bits_per_pixel() {
    return fb_bpp;
}

void pl110_set_pixel(uint16_t x, uint16_t y, uint16_t color) {
    if (!pl110_initialized || x >= fb_width || y >= fb_height) return;
    volatile uint16_t* fb = (volatile uint16_t*)(uintptr_t)framebuffer_phys;
    fb[y * fb_width + x] = color;
}

void pl110_fill_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint16_t color) {
    if (!pl110_initialized) return;
    volatile uint16_t* fb = (volatile uint16_t*)(uintptr_t)framebuffer_phys;
    for (uint32_t row = y; row < y + h && row < fb_height; row++) {
        for (uint32_t col = x; col < x + w && col < fb_width; col++) {
            fb[row * fb_width + col] = color;
        }
    }
}
