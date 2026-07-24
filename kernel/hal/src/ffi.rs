//! Consolidated C FFI declarations
//!
//! This module is the single source of truth for all extern "C" declarations
//! in the Alloy OS kernel. Both the HAL crate and the kernel crate should
//! use these declarations instead of duplicating them.

use core::ffi::c_void;

use crate::CpuContext;

// ============================================================================
// Architecture-independent externs (available on all targets)
// ============================================================================

extern "C" {
    // --- Serial ---
    pub fn serial_print(s: *const u8);
    pub fn serial_print_hex(value: u32);
    pub fn serial_print_hex64(value: u64);

    // --- Virtual memory manager ---
    pub fn vmm_init();
    pub fn vmm_alloc_region(size: usize, flags: u32) -> *mut c_void;
    pub fn vmm_free_region(addr: *mut c_void, size: usize);
    pub fn vmm_map(virt_addr: *mut c_void, phys_addr: *mut c_void, flags: u32) -> bool;
    pub fn vmm_unmap(virt_addr: *mut c_void);
    pub fn vmm_get_allocated_pages() -> u32;
    pub fn vmm_get_heap_start() -> usize;
    pub fn vmm_get_heap_size() -> usize;
    pub fn vmm_get_next_virt_addr() -> usize;

    // --- Physical memory manager ---
    pub fn pmm_init(multiboot_addr: u32);
    pub fn pmm_alloc_frame() -> *mut c_void;
    pub fn pmm_free_frame(addr: *mut c_void);
    pub fn pmm_get_total_frames() -> u32;
    pub fn pmm_get_used_frames() -> u32;
    pub fn pmm_get_total_memory() -> u64;
    pub fn pmm_get_available_memory() -> u64;
    pub fn pmm_refcount_inc(addr: *mut c_void);
    pub fn pmm_refcount_dec(addr: *mut c_void);

    // --- CPU info ---
    pub fn cpu_get_vendor_ffi(vendor: *mut u8);
    pub fn cpu_get_features_ffi() -> u32;
    pub fn cpu_get_model_info_ffi(family: *mut u32, model: *mut u32, stepping: *mut u32);

    // --- System ---
    pub fn get_system_uptime_ms() -> u64;

    // --- Context switching ---
    pub fn context_switch(old_ctx: *mut CpuContext, new_ctx: *mut CpuContext);

    // --- Sockets ---
    pub fn socket(domain: i32, socket_type: i32, protocol: i32) -> i32;
    pub fn bind_socket(fd: i32, addr: *const c_void, addr_len: u32) -> i32;
    pub fn listen_socket(fd: i32, backlog: i32) -> i32;
    pub fn accept_socket(fd: i32) -> i32;
    pub fn connect_socket(fd: i32, addr: *const c_void, addr_len: u32) -> i32;
    pub fn close_socket(fd: i32) -> i32;

    // --- Paging ---
    pub fn paging_init();
    pub fn paging_create_directory_phys() -> usize;
    pub fn paging_switch_to_directory(pd_phys: usize) -> bool;
    pub fn paging_get_kernel_directory_phys() -> usize;
    pub fn paging_get_physical_address(virt: usize) -> usize;
    pub fn paging_destroy_directory(pd_phys: usize);
    pub fn paging_clone_directory(pd_phys: usize) -> usize;
    pub fn paging_fork_directory(pd_phys: usize) -> usize;
    pub fn paging_handle_cow_fault(fault_addr: usize) -> u8;
    pub fn paging_map_page_in_pd(pd_phys: usize, virt_addr: usize, phys_addr: usize, flags: u32) -> bool;
    pub fn paging_temp_map_frame(phys_addr: usize) -> *mut c_void;
    pub fn paging_temp_unmap_frame();
    pub static mut g_current_user_cr3: u64;
    pub static kernel_pml4_phys: u64;

    // --- Timer ---
    pub fn timer_init_ffi(frequency: u32);
    pub fn timer_get_ticks_ffi() -> u64;
    pub fn timer_get_uptime_ms_ffi() -> u64;
    pub fn timer_get_frequency_ffi() -> u32;
}

// ============================================================================
// x86-specific externs (VGA, keyboard, mouse, VESA, storage, initrd)
// ============================================================================

#[cfg(feature = "x86_64")]
extern "C" {
    // VGA text mode
    pub fn vga_print(s: *const u8);
    pub fn vga_println(s: *const u8);
    pub fn vga_putchar(c: u8);
    pub fn vga_set_color(fg: u8, bg: u8);
    pub fn vga_clear();
    pub fn vga_set_cursor(x: u8, y: u8);
    pub fn vga_get_cursor_x() -> u8;
    pub fn vga_get_cursor_y() -> u8;
    pub fn vga_print_hex(value: u32);
    pub fn vga_print_dec(value: u32);

    // PS/2 keyboard
    pub fn keyboard_has_data() -> bool;
    pub fn keyboard_get_char() -> i8;

    // PS/2 mouse
    pub fn mouse_has_data() -> bool;
    pub fn mouse_read_event(
        dx: *mut i8, dy: *mut i8, wheel: *mut i8, buttons: *mut u8, flags: *mut u8,
    ) -> bool;
    pub fn mouse_is_initialized() -> bool;
    pub fn mouse_last_init_error() -> u8;

    // VESA framebuffer
    pub fn vesa_cursor_is_available() -> u8;
    pub fn vesa_cursor_enable(enable: u8);
    pub fn vesa_cursor_set_position(x: u16, y: u16);
    pub fn vesa_init();
    pub fn vesa_set_mode(mode: u16) -> u16;
    pub fn vesa_get_framebuffer() -> u64;
    pub fn vesa_get_resolution(width: *mut u16, height: *mut u16);
    pub fn vesa_get_mode(mode: *mut u16) -> u16;
    pub fn vesa_is_available() -> u8;
    pub fn vesa_get_capabilities() -> u8;
    pub fn vesa_get_bits_per_pixel() -> u8;
    pub fn vesa_get_bytes_per_scanline() -> u16;
    pub fn vesa_get_framebuffer_size() -> u64;

    // ATA PIO
    pub fn ata_init() -> i32;
    pub fn ata_drive_present(bus: u8, drive: u8) -> i32;
    pub fn ata_read_sectors(bus: u8, drive: u8, lba: u64, count: u8, buffer: *mut u8) -> i32;
    pub fn ata_write_sectors(bus: u8, drive: u8, lba: u64, count: u8, buffer: *const u8) -> i32;

    // PCI
    pub fn pci_init();
    pub fn pci_device_count() -> i32;

    // AHCI (SATA)
    pub fn ahci_init() -> i32;
    pub fn ahci_drive_count() -> i32;
    pub fn ahci_read_sectors(index: i32, lba: u64, count: u8, buffer: *mut u8) -> i32;
    pub fn ahci_write_sectors(index: i32, lba: u64, count: u8, buffer: *const u8) -> i32;

    // Initrd / ramdisk
    pub fn initrd_init(multiboot_addr: u32);
    pub fn initrd_module_count() -> i32;
    pub fn initrd_module_start_ffi(index: i32) -> usize;
    pub fn initrd_module_end_ffi(index: i32) -> usize;
    pub fn initrd_module_size_ffi(index: i32) -> usize;
    pub fn initrd_module_cmdline_ffi(index: i32, buf: *mut u8, max_len: u32);
    pub fn initrd_has_modules_ffi() -> i32;
}

// ============================================================================
// aarch64-specific externs
// ============================================================================

#[cfg(feature = "aarch64")]
extern "C" {
    // PL110 display controller
    pub fn pl110_init(fb_addr: u64, width: u16, height: u16);
    pub fn pl110_is_available() -> i32;
    pub fn pl110_get_framebuffer() -> u64;
    pub fn pl110_get_resolution(width: *mut u32, height: *mut u32);
    pub fn pl110_get_bits_per_pixel() -> u8;
}
