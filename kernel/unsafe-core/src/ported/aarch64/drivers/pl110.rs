pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uintptr_t = usize;
pub const PL110_BASE: ::core::ffi::c_int = 0x1e200000 as ::core::ffi::c_int;
pub const PL110_LCDTIMING0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PL110_LCDTIMING1: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PL110_LCDTIMING2: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PL110_LCDTIMING3: ::core::ffi::c_int = 0xc as ::core::ffi::c_int;
pub const PL110_LCDUPBASE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PL110_LCDCONTROL: ::core::ffi::c_int = 0x18 as ::core::ffi::c_int;
pub const PL110_LCDICR: ::core::ffi::c_int = 0x28 as ::core::ffi::c_int;
pub const LCDCTL_ENABLE: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const LCDCTL_LCDPWR: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 11 as ::core::ffi::c_int;
pub const LCDCTL_LCDBPP16: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const LCDCTL_TFT: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
static mut framebuffer_phys: uint32_t = 0 as uint32_t;
static mut fb_width: uint32_t = 1024 as uint32_t;
static mut fb_height: uint32_t = 768 as uint32_t;
static mut fb_bpp: uint8_t = 16 as uint8_t;
static mut pl110_initialized: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mmio_write32(mut addr: uintptr_t, mut val: uint32_t) {
    ::core::ptr::write_volatile((addr as *mut uint32_t), val);
}
#[no_mangle]
pub unsafe extern "C" fn pl110_init(
    mut fb_addr: uint32_t,
    mut width: uint16_t,
    mut height: uint16_t,
) {
    let mut base: uintptr_t = PL110_BASE as uintptr_t;
    fb_width = width as uint32_t;
    fb_height = height as uint32_t;
    fb_bpp = 16 as uint8_t;
    framebuffer_phys = fb_addr;
    mmio_write32(
        base.wrapping_add(PL110_LCDCONTROL as uintptr_t),
        0 as uint32_t,
    );
    let mut ppl: uint32_t = (width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint32_t;
    let mut hsw: uint32_t = 40 as uint32_t;
    let mut hfp: uint32_t = 160 as uint32_t;
    let mut hbp: uint32_t = 160 as uint32_t;
    mmio_write32(
        base.wrapping_add(PL110_LCDTIMING0 as uintptr_t),
        hsw << 24 as ::core::ffi::c_int | ppl << 2 as ::core::ffi::c_int,
    );
    let mut lpp: uint32_t = (height as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint32_t;
    let mut vsw: uint32_t = 6 as uint32_t;
    let mut vfp: uint32_t = 12 as uint32_t;
    let mut vbp: uint32_t = 24 as uint32_t;
    mmio_write32(
        base.wrapping_add(PL110_LCDTIMING1 as uintptr_t),
        vsw << 24 as ::core::ffi::c_int | lpp << 2 as ::core::ffi::c_int,
    );
    mmio_write32(
        base.wrapping_add(PL110_LCDTIMING2 as uintptr_t),
        vbp << 8 as ::core::ffi::c_int | vfp,
    );
    mmio_write32(
        base.wrapping_add(PL110_LCDTIMING3 as uintptr_t),
        hbp << 8 as ::core::ffi::c_int | hfp,
    );
    mmio_write32(base.wrapping_add(PL110_LCDUPBASE as uintptr_t), fb_addr);
    mmio_write32(
        base.wrapping_add(PL110_LCDCONTROL as uintptr_t),
        (LCDCTL_TFT | LCDCTL_LCDBPP16 | LCDCTL_ENABLE | LCDCTL_LCDPWR) as uint32_t,
    );
    mmio_write32(
        base.wrapping_add(PL110_LCDICR as uintptr_t),
        0xffffffff as uint32_t,
    );
    pl110_initialized = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn pl110_is_available() -> ::core::ffi::c_int {
    return pl110_initialized;
}
#[no_mangle]
pub unsafe extern "C" fn pl110_get_framebuffer() -> uint32_t {
    if pl110_initialized == 0 {
        return 0 as uint32_t;
    }
    return framebuffer_phys;
}
#[no_mangle]
pub unsafe extern "C" fn pl110_get_resolution(mut width: *mut uint32_t, mut height: *mut uint32_t) {
    if !width.is_null() {
        *width = fb_width;
    }
    if !height.is_null() {
        *height = fb_height;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pl110_get_bits_per_pixel() -> uint8_t {
    return fb_bpp;
}
#[no_mangle]
pub unsafe extern "C" fn pl110_set_pixel(mut x: uint16_t, mut y: uint16_t, mut color: uint16_t) {
    if pl110_initialized == 0 || x as uint32_t >= fb_width || y as uint32_t >= fb_height {
        return;
    }
    let mut fb: *mut uint16_t = framebuffer_phys as uintptr_t as *mut uint16_t;
    ::core::ptr::write_volatile(
        fb.offset(
            (y as uint32_t)
                .wrapping_mul(fb_width)
                .wrapping_add(x as uint32_t) as isize,
        ),
        color,
    );
}
#[no_mangle]
pub unsafe extern "C" fn pl110_fill_rect(
    mut x: uint16_t,
    mut y: uint16_t,
    mut w: uint16_t,
    mut h: uint16_t,
    mut color: uint16_t,
) {
    if pl110_initialized == 0 {
        return;
    }
    let mut fb: *mut uint16_t = framebuffer_phys as uintptr_t as *mut uint16_t;
    let mut row: uint32_t = y as uint32_t;
    while row < (y as ::core::ffi::c_int + h as ::core::ffi::c_int) as uint32_t && row < fb_height {
        let mut col: uint32_t = x as uint32_t;
        while col < (x as ::core::ffi::c_int + w as ::core::ffi::c_int) as uint32_t
            && col < fb_width
        {
            ::core::ptr::write_volatile(
                fb.offset(row.wrapping_mul(fb_width).wrapping_add(col) as isize),
                color,
            );
            col = col.wrapping_add(1);
        }
        row = row.wrapping_add(1);
    }
}
