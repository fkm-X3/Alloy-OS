//! Foreign Function Interface (FFI) to C kernel functions
//!
//! Raw extern "C" declarations are consolidated in the HAL crate
//! (`alloy_kernel_hal::ffi`). This module re-exports them and adds
//! safe Rust wrappers, constants, and convenience functions.

pub use alloy_kernel_hal::ffi::*;

use core::ffi::c_void;

// === Keyboard / mouse safe facades (x86_64) ===
//
// Implemented by the safe `Keyboard`/`Mouse` drivers in unsafe-core; these
// wrappers keep the old `ffi::keyboard_*`/`ffi::mouse_*` call sites working
// unchanged.

#[cfg(feature = "x86_64")]
pub use alloy_kernel_hal::{
    KeyEvent, Keyboard, Mouse, MouseEvent, SPECIAL_KEY_DELETE, SPECIAL_KEY_DOWN, SPECIAL_KEY_END,
    SPECIAL_KEY_HOME, SPECIAL_KEY_LEFT, SPECIAL_KEY_PGDN, SPECIAL_KEY_PGUP, SPECIAL_KEY_RIGHT,
    SPECIAL_KEY_UP,
};

#[cfg(feature = "x86_64")]
pub use alloy_kernel_hal::{
    MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT, MOUSE_EVENT_FLAG_X_OVERFLOW,
    MOUSE_EVENT_FLAG_Y_OVERFLOW, MOUSE_INIT_ERR_ENABLE_STREAMING,
    MOUSE_INIT_ERR_ENABLE_STREAMING_ACK, MOUSE_INIT_ERR_INPUT_NOT_READY, MOUSE_INIT_ERR_NONE,
    MOUSE_INIT_ERR_OUTPUT_NOT_READY, MOUSE_INIT_ERR_SET_DEFAULTS, MOUSE_INIT_ERR_SET_DEFAULTS_ACK,
};

#[cfg(feature = "x86_64")]
pub fn keyboard_has_key() -> bool {
    Keyboard::has_key()
}

#[cfg(feature = "x86_64")]
pub fn keyboard_read() -> u8 {
    Keyboard::read().unwrap_or(0)
}

#[cfg(feature = "x86_64")]
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

#[cfg(feature = "x86_64")]
pub fn mouse_has_event() -> bool {
    Mouse::has_event()
}

#[cfg(feature = "x86_64")]
pub fn mouse_ready() -> bool {
    Mouse::ready()
}

#[cfg(feature = "x86_64")]
pub fn mouse_init_error_code() -> u8 {
    Mouse::init_error()
}

#[cfg(feature = "x86_64")]
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

#[cfg(feature = "x86_64")]
pub fn mouse_read() -> Option<MouseEvent> {
    Mouse::read()
}

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
//
// Delegated to the safe `Vesa` facade in unsafe-core. The boot main already
// called `Vesa::init(multiboot_addr)` with the real multiboot address, so the
// `Vesa::init(0)` here is an idempotent no-op.

#[cfg(feature = "x86_64")]
pub fn vesa_initialize() {
    alloy_kernel_hal::Vesa::init(0);
}

#[cfg(feature = "x86_64")]
pub fn vesa_set_graphics_mode(mode: u16) -> (bool, u16) {
    match alloy_kernel_hal::Vesa::set_mode(mode) {
        Ok(()) => (true, 0),
        Err(e) => (false, e as u16),
    }
}

#[cfg(feature = "x86_64")]
pub fn vesa_framebuffer_addr() -> Option<u64> {
    let addr = alloy_kernel_hal::Vesa::framebuffer_addr();
    if addr != 0 {
        Some(addr)
    } else {
        None
    }
}

#[cfg(feature = "x86_64")]
pub fn vesa_display_resolution() -> (u16, u16) {
    alloy_kernel_hal::Vesa::resolution()
}

#[cfg(feature = "x86_64")]
pub fn vesa_current_mode() -> Option<u16> {
    alloy_kernel_hal::Vesa::current_mode()
}

#[cfg(feature = "x86_64")]
pub fn vesa_available() -> bool {
    alloy_kernel_hal::Vesa::available()
}

#[cfg(feature = "x86_64")]
pub fn vesa_controller_capabilities() -> u8 {
    alloy_kernel_hal::Vesa::capabilities()
}

#[cfg(feature = "x86_64")]
pub fn vesa_color_depth() -> u8 {
    alloy_kernel_hal::Vesa::bits_per_pixel()
}

#[cfg(feature = "x86_64")]
pub fn vesa_scanline_bytes() -> u16 {
    alloy_kernel_hal::Vesa::bytes_per_scanline()
}

#[cfg(feature = "x86_64")]
pub fn vesa_buffer_size() -> u64 {
    alloy_kernel_hal::Vesa::framebuffer_size()
}

#[cfg(feature = "x86_64")]
pub fn vesa_hardware_cursor_available() -> bool {
    alloy_kernel_hal::Vesa::cursor_available()
}

#[cfg(feature = "x86_64")]
pub fn vesa_hardware_cursor_set_enabled(enabled: bool) {
    alloy_kernel_hal::Vesa::cursor_enable(enabled);
}

#[cfg(feature = "x86_64")]
pub fn vesa_hardware_cursor_set_position(x: u16, y: u16) {
    alloy_kernel_hal::Vesa::cursor_set_position(x, y);
}

// ============================================================================
// ATA PIO Driver Safe Wrappers (x86 only)
// ============================================================================
//
// Delegated to the safe `Ata` facade in unsafe-core.

#[cfg(feature = "x86_64")]
pub use alloy_kernel_hal::AtaDriveInfo;

#[cfg(feature = "x86_64")]
pub fn ata_initialize() -> bool {
    alloy_kernel_hal::Ata::init()
}

#[cfg(feature = "x86_64")]
pub fn ata_drive_exists(bus: u8, drive: u8) -> bool {
    alloy_kernel_hal::Ata::drive_present(bus, drive)
}

#[cfg(feature = "x86_64")]
pub fn ata_read(bus: u8, drive: u8, lba: u64, count: u8, buf: &mut [u8]) -> bool {
    alloy_kernel_hal::Ata::read_sectors(bus, drive, lba, count, buf)
}

#[cfg(feature = "x86_64")]
pub fn ata_write(bus: u8, drive: u8, lba: u64, count: u8, buf: &[u8]) -> bool {
    alloy_kernel_hal::Ata::write_sectors(bus, drive, lba, count, buf)
}

// ============================================================================
// AHCI Driver Safe Wrappers (x86 only)
// ============================================================================
//
// Delegated to the safe `Ahci` facade in unsafe-core.

#[cfg(feature = "x86_64")]
pub use alloy_kernel_hal::AhciDriveInfo;

#[cfg(feature = "x86_64")]
pub fn ahci_initialize() -> bool {
    alloy_kernel_hal::Ahci::init()
}

#[cfg(feature = "x86_64")]
pub fn ahci_drive_count_ffi() -> i32 {
    alloy_kernel_hal::Ahci::drive_count() as i32
}

#[cfg(feature = "x86_64")]
pub fn ahci_read(drive: i32, lba: u64, count: u8, buf: &mut [u8]) -> bool {
    alloy_kernel_hal::Ahci::read_sectors(drive as usize, lba, count, buf)
}

#[cfg(feature = "x86_64")]
pub fn ahci_write(drive: i32, lba: u64, count: u8, buf: &[u8]) -> bool {
    alloy_kernel_hal::Ahci::write_sectors(drive as usize, lba, count, buf)
}

// ============================================================================
// Initrd / Ramdisk Safe Wrappers (x86 only)
// ============================================================================
//
// Delegated to the safe `Initrd` facade in unsafe-core. `initrd_initialize`
// is normally superseded by the boot main's `Initrd::init(multiboot_addr)`;
// it exists so callers without bootloader info can still drive the scan.

#[cfg(feature = "x86_64")]
pub fn initrd_initialize(multiboot_addr: u32) {
    alloy_kernel_hal::Initrd::init(multiboot_addr);
}

#[cfg(feature = "x86_64")]
pub fn initrd_module_count_ffi() -> i32 {
    alloy_kernel_hal::Initrd::module_count() as i32
}

#[cfg(feature = "x86_64")]
pub fn initrd_module_start(index: i32) -> usize {
    alloy_kernel_hal::Initrd::get_module(index as usize).map_or(0, |m| m.start)
}

#[cfg(feature = "x86_64")]
pub fn initrd_module_end(index: i32) -> usize {
    alloy_kernel_hal::Initrd::get_module(index as usize).map_or(0, |m| m.end)
}

#[cfg(feature = "x86_64")]
pub fn initrd_module_size(index: i32) -> usize {
    alloy_kernel_hal::Initrd::get_module(index as usize).map_or(0, |m| m.size)
}

#[cfg(feature = "x86_64")]
pub fn initrd_module_cmdline(index: i32) -> [u8; 64] {
    alloy_kernel_hal::Initrd::get_module(index as usize).map_or([0u8; 64], |m| m.cmdline)
}

#[cfg(feature = "x86_64")]
pub fn initrd_has_modules() -> bool {
    alloy_kernel_hal::Initrd::has_modules()
}
