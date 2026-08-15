//! Safe initrd / multiboot-module driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/initrd.rs`. Scans the multiboot2 info
//! structure for module tags (type 3) and records each module's physical
//! range and command line. The kernel crate turns a module whose size is a
//! sector multiple into a [`crate::mem`]-backed ramdisk.
//!
//! The `#[no_mangle]` C-ABI entry points are kept for the ported boot main
//! (`kernel_main`) and the pre-migration kernel-crate call sites; the safe
//! [`Initrd`] methods are what new code uses.

use crate::drivers::serial::Serial;

pub const MAX_INITRD_MODULES: usize = 16;
pub const MULTIBOOT_TAG_TYPE_END: u32 = 0;
pub const MULTIBOOT_TAG_TYPE_MODULE: u32 = 3;

/// One multiboot module discovered in the info structure.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InitrdModule {
    pub start: usize,
    pub end: usize,
    pub size: usize,
    pub cmdline: [u8; 64],
}

impl Default for InitrdModule {
    fn default() -> Self {
        InitrdModule {
            start: 0,
            end: 0,
            size: 0,
            cmdline: [0; 64],
        }
    }
}

#[repr(C)]
struct MultibootTag {
    tag_type: u32,
    size: u32,
}

#[repr(C, packed)]
struct MultibootTagModule {
    tag_type: u32,
    size: u32,
    mod_start: u32,
    mod_end: u32,
    cmdline: [u8; 0],
}

static mut G_MODULES: [InitrdModule; MAX_INITRD_MODULES] = [InitrdModule {
    start: 0,
    end: 0,
    size: 0,
    cmdline: [0; 64],
}; MAX_INITRD_MODULES];
static mut G_MODULE_COUNT: usize = 0;

/// Safe initrd facade.
pub struct Initrd;

impl Initrd {
    /// Scan the multiboot2 info structure at `multiboot_addr` for module tags.
    /// `multiboot_addr == 0` means no bootloader-provided info (e.g. aarch64).
    pub fn init(multiboot_addr: u32) {
        Serial::write_str("[INITRD] Scanning multiboot modules...\n");
        unsafe {
            G_MODULE_COUNT = 0;
        }
        if multiboot_addr == 0 {
            Serial::write_str("[INITRD] No multiboot info\n");
            return;
        }
        let mut tag_addr = (multiboot_addr as usize) + 8;
        loop {
            let tag = unsafe { &*(tag_addr as *const MultibootTag) };
            if tag.tag_type == MULTIBOOT_TAG_TYPE_END {
                break;
            }
            if tag.tag_type == MULTIBOOT_TAG_TYPE_MODULE {
                unsafe {
                    if G_MODULE_COUNT >= MAX_INITRD_MODULES {
                        Serial::write_str("[INITRD] Too many modules\n");
                        break;
                    }
                    let module = &mut *(tag_addr as *mut MultibootTagModule);
                    let start = module.mod_start as usize;
                    let end = module.mod_end as usize;
                    let size = end.wrapping_sub(start);
                    let m = &mut G_MODULES[G_MODULE_COUNT];
                    m.start = start;
                    m.end = end;
                    m.size = size;
                    m.cmdline = [0; 64];
                    let mut i = 0usize;
                    while i < 63 {
                        let c = *module.cmdline.as_ptr().add(i);
                        if c == 0 {
                            break;
                        }
                        m.cmdline[i] = c;
                        i += 1;
                    }
                    m.cmdline[i] = 0;
                    Serial::write_str("[INITRD] Module ");
                    Serial::write_hex(G_MODULE_COUNT as u32);
                    Serial::write_str(": start=0x");
                    Serial::write_hex(start as u32);
                    Serial::write_str(" end=0x");
                    Serial::write_hex(end as u32);
                    Serial::write_str(" size=");
                    Serial::write_hex(size as u32);
                    Serial::write_str(" cmdline=\"");
                    for &c in &m.cmdline {
                        if c == 0 {
                            break;
                        }
                        Serial::write_byte(c);
                    }
                    Serial::write_str("\"\n");
                    G_MODULE_COUNT += 1;
                }
            }
            tag_addr += (tag.size as usize + 7) & !7;
        }
        Serial::write_str("[INITRD] Found ");
        Serial::write_hex(unsafe { G_MODULE_COUNT } as u32);
        Serial::write_str(" module(s)\n");
    }

    /// Number of modules discovered by [`Initrd::init`].
    pub fn module_count() -> usize {
        unsafe { G_MODULE_COUNT }
    }

    /// Copy out module `index`, if present.
    pub fn get_module(index: usize) -> Option<InitrdModule> {
        if index >= unsafe { G_MODULE_COUNT } {
            return None;
        }
        Some(unsafe { G_MODULES[index] })
    }

    /// Whether any module was discovered.
    pub fn has_modules() -> bool {
        unsafe { G_MODULE_COUNT > 0 }
    }
}

/// C-ABI shims kept for the ported boot main and pre-migration call sites.

#[no_mangle]
pub extern "C" fn initrd_init(multiboot_addr: u32) {
    Initrd::init(multiboot_addr);
}

#[no_mangle]
pub extern "C" fn initrd_module_count() -> i32 {
    Initrd::module_count() as i32
}

#[no_mangle]
pub extern "C" fn initrd_module_start_ffi(index: i32) -> usize {
    Initrd::get_module(index as usize).map_or(0, |m| m.start)
}

#[no_mangle]
pub extern "C" fn initrd_module_end_ffi(index: i32) -> usize {
    Initrd::get_module(index as usize).map_or(0, |m| m.end)
}

#[no_mangle]
pub extern "C" fn initrd_module_size_ffi(index: i32) -> usize {
    Initrd::get_module(index as usize).map_or(0, |m| m.size)
}

#[no_mangle]
pub extern "C" fn initrd_module_cmdline_ffi(index: i32, buf: *mut u8, max_len: u32) {
    unsafe {
        if max_len == 0 {
            return;
        }
        *buf = 0;
        if let Some(m) = Initrd::get_module(index as usize) {
            let mut i = 0u32;
            while i + 1 < max_len {
                let c = m.cmdline[i as usize];
                if c == 0 {
                    break;
                }
                *buf.add(i as usize) = c;
                i += 1;
            }
            *buf.add(i as usize) = 0;
        }
    }
}

#[no_mangle]
pub extern "C" fn initrd_has_modules_ffi() -> i32 {
    Initrd::has_modules() as i32
}
