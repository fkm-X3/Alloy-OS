use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex_with_prefix(prefix: *const ::core::ffi::c_char, value: uint32_t);
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub available: uint8_t,
    pub initialized: uint8_t,
    pub framebuffer_ready: uint8_t,
    pub vbe_version: uint16_t,
    pub capabilities: uint8_t,
    pub current_mode: uint16_t,
    pub bytes_per_scanline: uint16_t,
    pub x_resolution: uint16_t,
    pub y_resolution: uint16_t,
    pub bits_per_pixel: uint8_t,
    pub linear_framebuffer: uint64_t,
    pub framebuffer_size: uint64_t,
    pub supported_modes: [uint16_t; 128],
    pub num_supported_modes: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiboot_tag {
    pub type_0: uint32_t,
    pub size: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct multiboot_tag_framebuffer_common {
    pub type_0: uint32_t,
    pub size: uint32_t,
    pub framebuffer_addr: uint64_t,
    pub framebuffer_pitch: uint32_t,
    pub framebuffer_width: uint32_t,
    pub framebuffer_height: uint32_t,
    pub framebuffer_bpp: uint8_t,
    pub framebuffer_type: uint8_t,
    pub reserved: uint16_t,
}
pub const VBE_MODE_MASK: ::core::ffi::c_int = 0x3fff as ::core::ffi::c_int;
pub const VBE_MODE_640x480x16: ::core::ffi::c_int = 0x111 as ::core::ffi::c_int;
pub const VBE_MODE_800x600x16: ::core::ffi::c_int = 0x114 as ::core::ffi::c_int;
pub const VBE_MODE_1024x768x16: ::core::ffi::c_int = 0x117 as ::core::ffi::c_int;
pub const VBE_MODE_640x480x32: ::core::ffi::c_int = 0x130 as ::core::ffi::c_int;
pub const VBE_MODE_800x600x32: ::core::ffi::c_int = 0x133 as ::core::ffi::c_int;
pub const VBE_MODE_1024x768x32: ::core::ffi::c_int = 0x138 as ::core::ffi::c_int;
pub const VBE_DISPI_IOPORT_INDEX: ::core::ffi::c_int = 0x1ce as ::core::ffi::c_int;
pub const VBE_DISPI_IOPORT_DATA: ::core::ffi::c_int = 0x1cf as ::core::ffi::c_int;
pub const VBE_DISPI_INDEX_CURSOR_X: ::core::ffi::c_int = 0xa as ::core::ffi::c_int;
pub const VBE_DISPI_INDEX_CURSOR_Y: ::core::ffi::c_int = 0xb as ::core::ffi::c_int;
pub const VBE_DISPI_INDEX_CURSOR_ENABLE: ::core::ffi::c_int = 0xc as ::core::ffi::c_int;
pub const VBE_CAP_DAC_SWITCHABLE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VBE_CAP_BLANK_SCREEN_VBE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_END: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_FRAMEBUFFER: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut g_vesa_state: C2RustUnnamed = C2RustUnnamed {
    available: 0 as uint8_t,
    initialized: 0,
    framebuffer_ready: 0,
    vbe_version: 0,
    capabilities: 0,
    current_mode: 0,
    bytes_per_scanline: 0,
    x_resolution: 0,
    y_resolution: 0,
    bits_per_pixel: 0,
    linear_framebuffer: 0,
    framebuffer_size: 0,
    supported_modes: [0; 128],
    num_supported_modes: 0,
};
unsafe extern "C" fn mode_for_dimensions(
    mut width: uint16_t,
    mut height: uint16_t,
    mut bpp: uint8_t,
) -> uint16_t {
    if width as ::core::ffi::c_int == 1024 as ::core::ffi::c_int
        && height as ::core::ffi::c_int == 768 as ::core::ffi::c_int
        && bpp as ::core::ffi::c_int == 16 as ::core::ffi::c_int
    {
        return VBE_MODE_1024x768x16 as uint16_t;
    }
    if width as ::core::ffi::c_int == 800 as ::core::ffi::c_int
        && height as ::core::ffi::c_int == 600 as ::core::ffi::c_int
        && bpp as ::core::ffi::c_int == 16 as ::core::ffi::c_int
    {
        return VBE_MODE_800x600x16 as uint16_t;
    }
    if width as ::core::ffi::c_int == 640 as ::core::ffi::c_int
        && height as ::core::ffi::c_int == 480 as ::core::ffi::c_int
        && bpp as ::core::ffi::c_int == 16 as ::core::ffi::c_int
    {
        return VBE_MODE_640x480x16 as uint16_t;
    }
    if width as ::core::ffi::c_int == 1024 as ::core::ffi::c_int
        && height as ::core::ffi::c_int == 768 as ::core::ffi::c_int
        && bpp as ::core::ffi::c_int == 32 as ::core::ffi::c_int
    {
        return VBE_MODE_1024x768x32 as uint16_t;
    }
    if width as ::core::ffi::c_int == 800 as ::core::ffi::c_int
        && height as ::core::ffi::c_int == 600 as ::core::ffi::c_int
        && bpp as ::core::ffi::c_int == 32 as ::core::ffi::c_int
    {
        return VBE_MODE_800x600x32 as uint16_t;
    }
    if width as ::core::ffi::c_int == 640 as ::core::ffi::c_int
        && height as ::core::ffi::c_int == 480 as ::core::ffi::c_int
        && bpp as ::core::ffi::c_int == 32 as ::core::ffi::c_int
    {
        return VBE_MODE_640x480x32 as uint16_t;
    }
    return 0 as uint16_t;
}
unsafe extern "C" fn load_multiboot_framebuffer(mut multiboot_addr: uint32_t) -> uint8_t {
    if multiboot_addr == 0 as uint32_t {
        return 0 as uint8_t;
    }
    let mut tag: *mut multiboot_tag =
        multiboot_addr.wrapping_add(8 as uint32_t) as *mut multiboot_tag;
    while (*tag).type_0 != MULTIBOOT_TAG_TYPE_END as uint32_t {
        if (*tag).type_0 == MULTIBOOT_TAG_TYPE_FRAMEBUFFER as uint32_t {
            let mut fb: *mut multiboot_tag_framebuffer_common =
                tag as *mut multiboot_tag_framebuffer_common;
            if (*fb).framebuffer_type as ::core::ffi::c_int == MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT {
                serial_print(
                    b"[VESA] Multiboot framebuffer is text mode\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                return 0 as uint8_t;
            }
            if (*fb).framebuffer_addr == 0 as uint64_t
                || (*fb).framebuffer_pitch == 0 as uint32_t
                || (*fb).framebuffer_width == 0 as uint32_t
                || (*fb).framebuffer_height == 0 as uint32_t
                || (*fb).framebuffer_bpp as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || (*fb).framebuffer_width > 0xffff as uint32_t
                || (*fb).framebuffer_height > 0xffff as uint32_t
                || (*fb).framebuffer_pitch > 0xffff as uint32_t
            {
                serial_print(
                    b"[VESA] Invalid multiboot framebuffer metadata\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                return 0 as uint8_t;
            }
            g_vesa_state.linear_framebuffer = (*fb).framebuffer_addr;
            g_vesa_state.bytes_per_scanline = (*fb).framebuffer_pitch as uint16_t;
            g_vesa_state.x_resolution = (*fb).framebuffer_width as uint16_t;
            g_vesa_state.y_resolution = (*fb).framebuffer_height as uint16_t;
            g_vesa_state.bits_per_pixel = (*fb).framebuffer_bpp;
            let mut fb_size: uint64_t = (g_vesa_state.bytes_per_scanline as uint64_t)
                .wrapping_mul(g_vesa_state.y_resolution as uint64_t);
            g_vesa_state.framebuffer_size = fb_size;
            g_vesa_state.current_mode = mode_for_dimensions(
                g_vesa_state.x_resolution,
                g_vesa_state.y_resolution,
                g_vesa_state.bits_per_pixel,
            );
            g_vesa_state.framebuffer_ready = 1 as uint8_t;
            return 1 as uint8_t;
        }
        tag = (tag as *mut uint8_t).offset(
            ((*tag).size.wrapping_add(7 as uint32_t) & !(7 as ::core::ffi::c_int) as uint32_t)
                as isize,
        ) as *mut multiboot_tag;
    }
    return 0 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_init_from_multiboot(mut multiboot_addr: uint32_t) {
    if g_vesa_state.initialized != 0 {
        return;
    }
    g_vesa_state.initialized = 1 as uint8_t;
    g_vesa_state.available = 0 as uint8_t;
    g_vesa_state.framebuffer_ready = 0 as uint8_t;
    g_vesa_state.current_mode = 0 as uint16_t;
    g_vesa_state.num_supported_modes = 0 as uint8_t;
    g_vesa_state.bytes_per_scanline = 0 as uint16_t;
    g_vesa_state.x_resolution = 0 as uint16_t;
    g_vesa_state.y_resolution = 0 as uint16_t;
    g_vesa_state.bits_per_pixel = 0 as uint8_t;
    g_vesa_state.linear_framebuffer = 0 as uint64_t;
    g_vesa_state.framebuffer_size = 0 as uint64_t;
    serial_print(
        b"[VESA] Initializing VBE detection...\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    g_vesa_state.supported_modes[0 as ::core::ffi::c_int as usize] =
        VBE_MODE_1024x768x32 as uint16_t;
    g_vesa_state.supported_modes[1 as ::core::ffi::c_int as usize] =
        VBE_MODE_800x600x32 as uint16_t;
    g_vesa_state.supported_modes[2 as ::core::ffi::c_int as usize] =
        VBE_MODE_640x480x32 as uint16_t;
    g_vesa_state.supported_modes[3 as ::core::ffi::c_int as usize] =
        VBE_MODE_1024x768x16 as uint16_t;
    g_vesa_state.supported_modes[4 as ::core::ffi::c_int as usize] =
        VBE_MODE_800x600x16 as uint16_t;
    g_vesa_state.supported_modes[5 as ::core::ffi::c_int as usize] =
        VBE_MODE_640x480x16 as uint16_t;
    g_vesa_state.num_supported_modes = 6 as uint8_t;
    g_vesa_state.vbe_version = 0x300 as uint16_t;
    g_vesa_state.capabilities = (VBE_CAP_DAC_SWITCHABLE | VBE_CAP_BLANK_SCREEN_VBE) as uint8_t;
    if load_multiboot_framebuffer(multiboot_addr) == 0 {
        serial_print(
            b"[VESA] No valid multiboot framebuffer metadata; graphics unavailable\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return;
    }
    g_vesa_state.available = 1 as uint8_t;
    serial_print(b"[VESA] VESA VBE initialized - \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"version=0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.vbe_version as uint32_t,
    );
    serial_print(b"[VESA] Supported modes: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"count=\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.num_supported_modes as uint32_t,
    );
    serial_print(b"[VESA] Framebuffer: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"addr=0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.linear_framebuffer as uint32_t,
    );
    serial_print_hex_with_prefix(
        b"width=0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.x_resolution as uint32_t,
    );
    serial_print_hex_with_prefix(
        b"height=0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.y_resolution as uint32_t,
    );
    serial_print_hex_with_prefix(
        b"bpp=0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.bits_per_pixel as uint32_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn vesa_init() {
    vesa_init_from_multiboot(0 as uint32_t);
}
#[no_mangle]
pub unsafe extern "C" fn vesa_set_mode(mut mode: uint16_t) -> uint16_t {
    if g_vesa_state.initialized == 0 {
        serial_print(
            b"[VESA] Error: VESA not initialized\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 1 as uint16_t;
    }
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        serial_print(
            b"[VESA] Error: Bootloader framebuffer is unavailable\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return 3 as uint16_t;
    }
    let mut mode_number: uint16_t = (mode as ::core::ffi::c_int & VBE_MODE_MASK) as uint16_t;
    let mut mode_supported: uint8_t = 0 as uint8_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < g_vesa_state.num_supported_modes as ::core::ffi::c_int {
        if g_vesa_state.supported_modes[i as usize] as ::core::ffi::c_int & VBE_MODE_MASK
            == mode_number as ::core::ffi::c_int
        {
            mode_supported = 1 as uint8_t;
            break;
        } else {
            i += 1;
        }
    }
    if mode_supported == 0 {
        serial_print(b"[VESA] Error: Mode \0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex_with_prefix(
            b"0x\0" as *const u8 as *const ::core::ffi::c_char,
            mode_number as uint32_t,
        );
        serial_print(b" not supported\n\0" as *const u8 as *const ::core::ffi::c_char);
        return 2 as uint16_t;
    }
    let mut detected_mode: uint16_t = mode_for_dimensions(
        g_vesa_state.x_resolution,
        g_vesa_state.y_resolution,
        g_vesa_state.bits_per_pixel,
    );
    if detected_mode as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || detected_mode as ::core::ffi::c_int != mode_number as ::core::ffi::c_int
    {
        serial_print(
            b"[VESA] Error: Requested mode does not match active boot framebuffer\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return 3 as uint16_t;
    }
    g_vesa_state.current_mode = mode_number;
    serial_print(b"[VESA] Mode set: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"0x\0" as *const u8 as *const ::core::ffi::c_char,
        mode_number as uint32_t,
    );
    serial_print(b" (\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"width=\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.x_resolution as uint32_t,
    );
    serial_print(b", height=\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.y_resolution as uint32_t,
    );
    serial_print(b", bpp=\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex_with_prefix(
        b"0x\0" as *const u8 as *const ::core::ffi::c_char,
        g_vesa_state.bits_per_pixel as uint32_t,
    );
    serial_print(b")\n\0" as *const u8 as *const ::core::ffi::c_char);
    return 0 as uint16_t;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_is_available() -> uint8_t {
    return g_vesa_state.available;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_capabilities() -> uint8_t {
    if g_vesa_state.available == 0 {
        return 0 as uint8_t;
    }
    return g_vesa_state.capabilities;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_framebuffer() -> uint64_t {
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        return 0 as uint64_t;
    }
    return g_vesa_state.linear_framebuffer;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_resolution(mut width: *mut uint16_t, mut height: *mut uint16_t) {
    if width.is_null() || height.is_null() {
        return;
    }
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        *width = 0 as uint16_t;
        *height = 0 as uint16_t;
        return;
    }
    *width = g_vesa_state.x_resolution;
    *height = g_vesa_state.y_resolution;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_mode(mut mode: *mut uint16_t) -> uint16_t {
    if mode.is_null() {
        return 1 as uint16_t;
    }
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        return 1 as uint16_t;
    }
    *mode = g_vesa_state.current_mode;
    return (if g_vesa_state.current_mode as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as uint16_t;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_bits_per_pixel() -> uint8_t {
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        return 0 as uint8_t;
    }
    return g_vesa_state.bits_per_pixel;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_bytes_per_scanline() -> uint16_t {
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        return 0 as uint16_t;
    }
    return g_vesa_state.bytes_per_scanline;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_get_framebuffer_size() -> uint64_t {
    if g_vesa_state.available == 0 || g_vesa_state.framebuffer_ready == 0 {
        return 0 as uint64_t;
    }
    return g_vesa_state.framebuffer_size;
}
unsafe extern "C" fn vbe_write_register(mut index: uint16_t, mut value: uint16_t) {
    asm!(
        "outw %ax, %dx\n", inlateout("ax") index => _, inlateout("dx")
        VBE_DISPI_IOPORT_INDEX as uint16_t => _, options(preserves_flags, att_syntax)
    );
    asm!(
        "outw %ax, %dx\n", inlateout("dx") VBE_DISPI_IOPORT_DATA as uint16_t => _,
        inlateout("ax") value => _, options(preserves_flags, att_syntax)
    );
}
unsafe extern "C" fn vbe_read_register(mut index: uint16_t) -> uint16_t {
    let mut value: uint16_t = 0;
    asm!(
        "outw %ax, %dx\n", inlateout("ax") index => _, inlateout("dx")
        VBE_DISPI_IOPORT_INDEX as uint16_t => _, options(preserves_flags, att_syntax)
    );
    asm!(
        "inw %dx, %ax\n", lateout("ax") value, inlateout("dx") VBE_DISPI_IOPORT_DATA as
        uint16_t => _, options(preserves_flags, att_syntax)
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_cursor_is_available() -> uint8_t {
    if g_vesa_state.available == 0 {
        return 0 as uint8_t;
    }
    let mut saved: uint16_t = vbe_read_register(VBE_DISPI_INDEX_CURSOR_X as uint16_t);
    vbe_write_register(VBE_DISPI_INDEX_CURSOR_X as uint16_t, 0xaaaa as uint16_t);
    let mut test: uint16_t = vbe_read_register(VBE_DISPI_INDEX_CURSOR_X as uint16_t);
    vbe_write_register(VBE_DISPI_INDEX_CURSOR_X as uint16_t, saved);
    if test as ::core::ffi::c_int == 0xaaaa as ::core::ffi::c_int {
        serial_print(
            b"[VESA] Hardware cursor available\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 1 as uint8_t;
    }
    serial_print(
        b"[VESA] Hardware cursor not available (VBE doesn't support it)\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    return 0 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn vesa_cursor_enable(mut enable: uint8_t) {
    vbe_write_register(
        VBE_DISPI_INDEX_CURSOR_ENABLE as uint16_t,
        (if enable as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint16_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn vesa_cursor_set_position(mut x: uint16_t, mut y: uint16_t) {
    vbe_write_register(VBE_DISPI_INDEX_CURSOR_X as uint16_t, x);
    vbe_write_register(VBE_DISPI_INDEX_CURSOR_Y as uint16_t, y);
}
