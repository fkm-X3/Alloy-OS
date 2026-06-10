//! Foreign Function Interface (FFI) to C kernel functions
//!
//! Raw extern "C" declarations are consolidated in the HAL crate
//! (`alloy_kernel_hal::ffi`). This module re-exports them and adds
//! safe Rust wrappers, constants, and convenience functions.

pub use alloy_kernel_hal::ffi::*;

use core::ffi::c_void;

// === Safe wrappers ===

pub fn print_str(s: &str) {
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0;
    unsafe {
        serial_print(buffer.as_ptr());
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vga_print_str(s: &str) {
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0;
    unsafe { vga_print(buffer.as_ptr()) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vga_println_str(s: &str) {
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0;
    unsafe { vga_println(buffer.as_ptr()) }
}

/// # Safety
/// `s` must be a valid null-terminated C string pointer or null.
pub unsafe fn serial_print_safe(s: *const u8) {
    if !s.is_null() {
        serial_print(s);
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub unsafe fn vga_print_safe(s: *const u8) {
    if !s.is_null() {
        vga_print(s);
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub unsafe fn vga_println_safe(s: *const u8) {
    if !s.is_null() {
        vga_println(s);
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn set_vga_color(fg: u8, bg: u8) {
    unsafe { vga_set_color(fg, bg) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn put_char(c: char) {
    unsafe { vga_putchar(c as u8) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn keyboard_has_key() -> bool {
    unsafe { keyboard_has_data() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn keyboard_read() -> u8 {
    unsafe { keyboard_get_char() as u8 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn keyboard_read_blocking() -> u8 {
    loop {
        if keyboard_has_key() {
            return keyboard_read();
        }
        crate::process::scheduler::Scheduler::block_current_on(
            &crate::process::scheduler::KEYBOARD_WAIT,
        );
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub dx: i8,
    pub dy: i8,
    pub wheel: i8,
    pub buttons: u8,
    pub flags: u8,
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn mouse_has_event() -> bool {
    unsafe { mouse_has_data() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn mouse_ready() -> bool {
    unsafe { mouse_is_initialized() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn mouse_init_error_code() -> u8 {
    unsafe { mouse_last_init_error() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn mouse_read_blocking() -> MouseEvent {
    loop {
        if let Some(event) = mouse_read() {
            return event;
        }
        crate::process::scheduler::Scheduler::block_current_on(
            &crate::process::scheduler::MOUSE_WAIT,
        );
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn mouse_read() -> Option<MouseEvent> {
    let mut dx: i8 = 0;
    let mut dy: i8 = 0;
    let mut wheel: i8 = 0;
    let mut buttons: u8 = 0;
    let mut flags: u8 = 0;

    let has_event = unsafe {
        mouse_read_event(
            &mut dx as *mut i8,
            &mut dy as *mut i8,
            &mut wheel as *mut i8,
            &mut buttons as *mut u8,
            &mut flags as *mut u8,
        )
    };

    if !has_event {
        return None;
    }

    Some(MouseEvent { dx, dy, wheel, buttons, flags })
}

// Special key codes (match C++ keyboard.h)
pub const SPECIAL_KEY_UP: u8 = 128;
pub const SPECIAL_KEY_DOWN: u8 = 129;
pub const SPECIAL_KEY_LEFT: u8 = 130;
pub const SPECIAL_KEY_RIGHT: u8 = 131;
pub const SPECIAL_KEY_HOME: u8 = 132;
pub const SPECIAL_KEY_END: u8 = 133;
pub const SPECIAL_KEY_DELETE: u8 = 134;
pub const SPECIAL_KEY_PGUP: u8 = 135;
pub const SPECIAL_KEY_PGDN: u8 = 136;

pub const MOUSE_BUTTON_LEFT: u8 = 0x01;
pub const MOUSE_BUTTON_RIGHT: u8 = 0x02;
pub const MOUSE_BUTTON_MIDDLE: u8 = 0x04;

pub const MOUSE_EVENT_FLAG_X_OVERFLOW: u8 = 0x01;
pub const MOUSE_EVENT_FLAG_Y_OVERFLOW: u8 = 0x02;

/// Socket convenience wrappers
pub fn socket_create(domain: i32, socket_type: i32, protocol: i32) -> i32 {
    unsafe { socket(domain, socket_type, protocol) }
}

/// # Safety
/// `addr` must be a valid pointer to a sockaddr structure of at least `addr_len` bytes.
pub unsafe fn socket_bind(fd: i32, addr: *const c_void, addr_len: u32) -> i32 {
    bind_socket(fd, addr, addr_len)
}

pub fn socket_listen(fd: i32, backlog: i32) -> i32 {
    unsafe { listen_socket(fd, backlog) }
}

pub fn socket_accept(fd: i32) -> i32 {
    unsafe { accept_socket(fd) }
}

/// # Safety
/// `addr` must be a valid pointer to a sockaddr structure of at least `addr_len` bytes.
pub unsafe fn socket_connect(fd: i32, addr: *const c_void, addr_len: u32) -> i32 {
    connect_socket(fd, addr, addr_len)
}

pub fn socket_close(fd: i32) -> i32 {
    unsafe { close_socket(fd) }
}

// Page flags for memory mapping
pub const PAGE_PRESENT: u32 = 0x001;
pub const PAGE_WRITE: u32 = 0x002;
pub const PAGE_USER: u32 = 0x004;

// ============================================================================
// VESA VBE Graphics Safe Wrappers (x86 only)
// ============================================================================

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_initialize() {
    unsafe { vesa_init() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_set_graphics_mode(mode: u16) -> (bool, u16) {
    unsafe {
        let result = vesa_set_mode(mode);
        (result == 0, result)
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_framebuffer_addr() -> Option<u32> {
    unsafe {
        let addr = vesa_get_framebuffer();
        if addr != 0 { Some(addr) } else { None }
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_display_resolution() -> (u16, u16) {
    unsafe {
        let mut width: u16 = 0;
        let mut height: u16 = 0;
        vesa_get_resolution(&mut width, &mut height);
        (width, height)
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_current_mode() -> Option<u16> {
    unsafe {
        let mut mode: u16 = 0;
        let result = vesa_get_mode(&mut mode);
        if result == 0 { Some(mode) } else { None }
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_available() -> bool {
    unsafe { vesa_is_available() != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_controller_capabilities() -> u8 {
    unsafe { vesa_get_capabilities() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_color_depth() -> u8 {
    unsafe { vesa_get_bits_per_pixel() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_scanline_bytes() -> u16 {
    unsafe { vesa_get_bytes_per_scanline() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_buffer_size() -> u32 {
    unsafe { vesa_get_framebuffer_size() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_hardware_cursor_available() -> bool {
    unsafe { vesa_cursor_is_available() != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_hardware_cursor_set_enabled(enabled: bool) {
    unsafe { vesa_cursor_enable(enabled as u8) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn vesa_hardware_cursor_set_position(x: u16, y: u16) {
    unsafe { vesa_cursor_set_position(x, y) }
}

// ============================================================================
// ATA PIO Driver Safe Wrappers (x86 only)
// ============================================================================

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub struct AtaDriveInfo {
    pub present: bool,
    pub is_lba48: bool,
    pub num_sectors: u64,
    pub model: [u8; 41],
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
impl AtaDriveInfo {
    pub fn probe(bus: u8, drive: u8) -> Self {
        let present = unsafe { ata_drive_present(bus, drive) != 0 };
        if !present {
            return AtaDriveInfo { present: false, is_lba48: false, num_sectors: 0, model: [0u8; 41] };
        }
        AtaDriveInfo { present: true, is_lba48: true, num_sectors: 0, model: [0u8; 41] }
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ata_initialize() -> bool {
    unsafe { ata_init() != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ata_drive_exists(bus: u8, drive: u8) -> bool {
    unsafe { ata_drive_present(bus, drive) != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ata_read(bus: u8, drive: u8, lba: u64, count: u8, buf: &mut [u8]) -> bool {
    unsafe { ata_read_sectors(bus, drive, lba, count, buf.as_mut_ptr()) != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ata_write(bus: u8, drive: u8, lba: u64, count: u8, buf: &[u8]) -> bool {
    unsafe { ata_write_sectors(bus, drive, lba, count, buf.as_ptr()) != 0 }
}

// ============================================================================
// AHCI Driver Safe Wrappers (x86 only)
// ============================================================================

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub struct AhciDriveInfo {
    pub present: bool,
    pub port_num: u8,
    pub num_sectors: u64,
    pub model: [u8; 41],
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
impl AhciDriveInfo {
    #[allow(unused_variables)]
    pub fn probe(index: i32) -> Self {
        AhciDriveInfo { present: true, port_num: 0, num_sectors: 0, model: [0u8; 41] }
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ahci_initialize() -> bool {
    unsafe { ahci_init() != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ahci_drive_count_ffi() -> i32 {
    unsafe { ahci_drive_count() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ahci_read(drive: i32, lba: u64, count: u8, buf: &mut [u8]) -> bool {
    unsafe { ahci_read_sectors(drive, lba, count, buf.as_mut_ptr()) != 0 }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn ahci_write(drive: i32, lba: u64, count: u8, buf: &[u8]) -> bool {
    unsafe { ahci_write_sectors(drive, lba, count, buf.as_ptr()) != 0 }
}

// ============================================================================
// Initrd / Ramdisk Safe Wrappers (x86 only)
// ============================================================================

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_initialize(multiboot_addr: u32) {
    unsafe { initrd_init(multiboot_addr) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_module_count_ffi() -> i32 {
    unsafe { initrd_module_count() }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_module_start(index: i32) -> u32 {
    unsafe { initrd_module_start_ffi(index) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_module_end(index: i32) -> u32 {
    unsafe { initrd_module_end_ffi(index) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_module_size(index: i32) -> u32 {
    unsafe { initrd_module_size_ffi(index) }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_module_cmdline(index: i32) -> [u8; 64] {
    let mut buf = [0u8; 64];
    unsafe { initrd_module_cmdline_ffi(index, buf.as_mut_ptr(), 64); }
    buf
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub fn initrd_has_modules() -> bool {
    unsafe { initrd_has_modules_ffi() != 0 }
}
