//! Hand-written physical memory manager.
//!
//! Replaces `ported/common/mm_pmm.rs` (and the arch-split `x86_64/mm/pmm.rs`
//! / `aarch64/mm/pmm.rs` dead copies). The bitmap allocator, refcount table
//! and stats behave exactly as the translated C did.
//!
//! The `#[no_mangle] extern "C"` entry points keep the boot main, the
//! surviving ported modules (idt, vesa, ahci, ...) and `raw::ffi` resolving
//! against the same symbols as before.

use core::ffi::c_void;

use crate::drivers::serial::Serial;
use crate::raw::string;

pub const PAGE_SIZE: u32 = 4096;
pub const MAX_PHYSICAL_FRAMES: u32 = 1024 * 1024;

const MULTIBOOT_TAG_TYPE_END: u32 = 0;
const MULTIBOOT_TAG_TYPE_BASIC_MEMINFO: u32 = 4;
const MULTIBOOT_TAG_TYPE_MMAP: u32 = 6;
const MULTIBOOT_MEMORY_AVAILABLE: u32 = 1;

extern "C" {
    static _kernel_start: u32;
    static _kernel_end: u32;
}

/// Allocator bookkeeping, exported under the original C name.
#[repr(C)]
pub struct PhysicalMemoryManager {
    pub bitmap: *mut u32,
    pub total_frames: u32,
    pub used_frames: u32,
    pub total_memory: u64,
    pub available_memory: u64,
}

#[no_mangle]
pub static mut g_pmm: PhysicalMemoryManager = PhysicalMemoryManager {
    bitmap: core::ptr::null_mut(),
    total_frames: 0,
    used_frames: 0,
    total_memory: 0,
    available_memory: 0,
};

/// 32768 words cover 1 MiB frames (MAX_PHYSICAL_FRAMES / 32).
static mut frame_bitmap: [u32; 32768] = [0; 32768];
/// One refcount per frame; only frames below MAX_PHYSICAL_FRAMES use it.
static mut frame_refcounts: [u32; 1048576] = [0; 1048576];

#[inline]
unsafe fn bitmap_addr() -> *mut u32 {
    g_pmm.bitmap
}

#[inline]
fn set_frame(frame: u32) {
    unsafe {
        *bitmap_addr().offset((frame / 32) as isize) |= 1u32 << (frame % 32);
    }
}

#[inline]
fn clear_frame(frame: u32) {
    unsafe {
        *bitmap_addr().offset((frame / 32) as isize) &= !(1u32 << (frame % 32));
    }
}

#[inline]
fn test_frame(frame: u32) -> bool {
    unsafe { *bitmap_addr().offset((frame / 32) as isize) & (1u32 << (frame % 32)) != 0 }
}

/// First free frame, scanning from frame 0 upward, or -1 when full.
fn find_free_frame() -> i32 {
    unsafe {
        for i in 0..(g_pmm.total_frames / 32) {
            let word = *bitmap_addr().offset(i as isize);
            if word != 0xffff_ffff {
                for bit in 0..32 {
                    if word & (1u32 << bit) == 0 {
                        return (i * 32 + bit) as i32;
                    }
                }
            }
        }
    }
    -1
}

#[repr(C)]
struct MultibootTag {
    typ: u32,
    size: u32,
}

#[repr(C)]
struct MultibootTagBasicMeminfo {
    typ: u32,
    size: u32,
    mem_lower: u32,
    mem_upper: u32,
}

#[repr(C)]
struct MultibootMmapEntry {
    addr: u64,
    len: u64,
    typ: u32,
    zero: u32,
}

#[repr(C)]
struct MultibootTagMmap {
    typ: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    entries: [MultibootMmapEntry; 0],
}

/// Mark the [start, end) frame range free in the bitmap (multiboot mmap path).
///
/// Faithful port of the C's bit-sliced clearing: handles partial words at the
/// start/end and whole words in between.
fn free_frame_range(start_frame: u32, num_frames: u32, max_idx: u32) {
    if num_frames == 0 {
        return;
    }
    let end_frame = start_frame.wrapping_add(num_frames);
    let mut end_idx = (end_frame.wrapping_sub(1)) / 32;
    let mut end_bit = (end_frame.wrapping_sub(1)) & 31;
    let start_idx = start_frame / 32;
    let start_bit = start_frame & 31;

    if end_idx >= max_idx {
        end_idx = max_idx.wrapping_sub(1);
        end_bit = 31;
    }

    unsafe {
        if start_idx >= max_idx {
            Serial::write_str(
                "PMM: WARNING - start_idx out of bounds, skipping entry\n",
            );
            return;
        }

        if start_idx == end_idx {
            let mask: u32 = if end_bit == 31 {
                0xffff_ffffu32 << start_bit
            } else {
                (0xffff_ffffu32 << start_bit) & !(0xffff_ffffu32 << (end_bit + 1))
            };
            *bitmap_addr().offset(start_idx as isize) &= !mask;
        } else {
            *bitmap_addr().offset(start_idx as isize) &= !(0xffff_ffffu32 << start_bit);
            let mut i = start_idx + 1;
            while i < end_idx && i < max_idx {
                *bitmap_addr().offset(i as isize) = 0;
                i += 1;
            }
            if end_idx < max_idx {
                if end_bit == 31 {
                    *bitmap_addr().offset(end_idx as isize) = 0;
                } else {
                    *bitmap_addr().offset(end_idx as isize) &=
                        !(((1u32 << (end_bit + 1)) - 1) as u32);
                }
            }
        }
    }
}

/// `pmm_init(multiboot_addr)`: initialize the bitmap allocator.
///
/// On x86_64 `multiboot_addr` is the multiboot2 info structure; the memory map
/// tags drive which frames become available. On aarch64 (address 0) the 128 MiB
/// QEMU `virt` RAM is used with the PL110/userland reservations.
#[no_mangle]
pub unsafe extern "C" fn pmm_init(multiboot_addr: u32) {
    Serial::write_str("PMM: Initializing physical memory manager...\n");

    g_pmm.bitmap = &raw mut frame_bitmap as *mut u32;
    g_pmm.total_frames = 0;
    g_pmm.used_frames = 0;
    g_pmm.total_memory = 0;
    g_pmm.available_memory = 0;

    // Everything reserved by default; free what the memory map/layout says.
    for w in frame_bitmap.iter_mut() {
        *w = 0xffff_ffff;
    }
    for r in frame_refcounts.iter_mut() {
        *r = 0;
    }

    if multiboot_addr == 0 {
        // aarch64: no multiboot, use the fixed QEMU virt layout.
        Serial::write_str("PMM: No multiboot info (aarch64), using default memory layout\n");
        g_pmm.total_memory = 128 * 1024 * 1024;
        g_pmm.available_memory = 128 * 1024 * 1024;

        let ram_start_frame = (kernel_end() + PAGE_SIZE - 1) / PAGE_SIZE;
        // 2.2.2: reserve the top 3 MiB for the PL110 framebuffer.
        let ram_end_frame = 0x47d0_0000 / PAGE_SIZE;
        g_pmm.total_frames = ram_end_frame;
        let mut f = ram_start_frame;
        while f < ram_end_frame {
            clear_frame(f);
            f += 1;
        }
    } else {
        // x86_64: walk the multiboot2 info tags.
        let mut tag_ptr = (multiboot_addr as usize + 8) as *const MultibootTag;
        loop {
            let tag = &*tag_ptr;
            if tag.typ == MULTIBOOT_TAG_TYPE_END {
                break;
            }
            match tag.typ {
                MULTIBOOT_TAG_TYPE_BASIC_MEMINFO => {
                    let meminfo = &*(tag_ptr as *const MultibootTagBasicMeminfo);
                    Serial::write_str("PMM: Basic memory info:\n");
                    Serial::write_str("  Lower memory: ");
                    Serial::write_hex(meminfo.mem_lower);
                    Serial::write_str(" KB\n");
                    Serial::write_str("  Upper memory: ");
                    Serial::write_hex(meminfo.mem_upper);
                    Serial::write_str(" KB\n");
                }
                MULTIBOOT_TAG_TYPE_MMAP => {
                    let mmap = &*(tag_ptr as *const MultibootTagMmap);
                    Serial::write_str("PMM: Memory map:\n");
                    Serial::write_str("  entry_size=");
                    Serial::write_hex(mmap.entry_size);
                    Serial::write_str(", tag->size=");
                    Serial::write_hex(tag.size);
                    Serial::write_str("\n");

                    let tag_end = (tag_ptr as *const u8).add(tag.size as usize);
                    let mut entry_ptr = &raw const mmap.entries as *const MultibootMmapEntry as *const u8;
                    let entry_size = mmap.entry_size as usize;
                    let max_idx = (frame_bitmap.len()) as u32;

                    while (entry_ptr as usize) + core::mem::size_of::<MultibootMmapEntry>()
                        <= tag_end as usize
                    {
                        let entry = &*(entry_ptr as *const MultibootMmapEntry);
                        Serial::write_str("  Region: addr=0x");
                        Serial::write_hex(entry.addr as u32);
                        Serial::write_str(", len=0x");
                        Serial::write_hex(entry.len as u32);
                        Serial::write_str(", type=");
                        Serial::write_hex(entry.typ);
                        Serial::write_str("\n");

                        g_pmm.total_memory = g_pmm.total_memory.wrapping_add(entry.len);
                        if entry.typ == MULTIBOOT_MEMORY_AVAILABLE {
                            g_pmm.available_memory = g_pmm.available_memory.wrapping_add(entry.len);

                            let mut base = entry.addr;
                            let mut length = entry.len;
                            if base % PAGE_SIZE as u64 != 0 {
                                let offset = PAGE_SIZE as u64 - base % PAGE_SIZE as u64;
                                base += offset;
                                length = if length > offset { length - offset } else { 0 };
                            }
                            length = (length / PAGE_SIZE as u64) * PAGE_SIZE as u64;

                            let start_frame = (base / PAGE_SIZE as u64) as u32;
                            let num_frames = (length / PAGE_SIZE as u64) as u32;
                            let end_frame = start_frame.wrapping_add(num_frames);
                            free_frame_range(start_frame, num_frames, max_idx);
                            if end_frame > g_pmm.total_frames {
                                g_pmm.total_frames = end_frame;
                            }
                        }
                        entry_ptr = entry_ptr.add(entry_size);
                    }
                    Serial::write_str("PMM: Memory map entries done\n");
                }
                _ => {}
            }
            // Tags are 8-byte aligned.
            let advance = ((tag.size + 7) & !7) as usize;
            tag_ptr = (tag_ptr as *const u8).add(advance) as *const MultibootTag;
        }
    }

    // Reserve the first 1 MiB (frame 0..255) regardless of the map.
    let mut f = 0u32;
    while f < 256 {
        set_frame(f);
        g_pmm.used_frames += 1;
        f += 1;
    }

    // Reserve the kernel's own frames.
    let kernel_start = &raw const _kernel_start as usize as u32;
    let kernel_end = &raw const _kernel_end as usize as u32;
    Serial::write_str("  Kernel region start: 0x");
    Serial::write_hex(kernel_start);
    Serial::write_str("\n");
    Serial::write_str("  Kernel region end: 0x");
    Serial::write_hex(kernel_end);
    Serial::write_str("\n");

    let kernel_start_frame = kernel_start / PAGE_SIZE;
    let kernel_end_frame = (kernel_end + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut f = kernel_start_frame;
    while f < kernel_end_frame {
        if !test_frame(f) {
            set_frame(f);
            g_pmm.used_frames += 1;
        }
        f += 1;
    }

    // 2.2.3: reserve the aarch64 identity-mapped userland region. Kept out of
    // the bitmap so the heap can never hand out a frame the running user
    // program occupies (the MMU is disabled; userland is at fixed phys).
    if multiboot_addr == 0 {
        let base = 0x47a0_0000 / PAGE_SIZE;
        let end = 0x47c0_0000 / PAGE_SIZE;
        let mut f = base;
        while f < end {
            if !test_frame(f) {
                set_frame(f);
                g_pmm.used_frames += 1;
            }
            f += 1;
        }
    }

    Serial::write_str("PMM: Initialization complete\n");
    Serial::write_str("  Total memory: ");
    Serial::write_hex((g_pmm.total_memory / 1024 / 1024) as u32);
    Serial::write_str(" MB\n");
    Serial::write_str("  Available memory: ");
    Serial::write_hex((g_pmm.available_memory / 1024 / 1024) as u32);
    Serial::write_str(" MB\n");
    Serial::write_str("  Total frames: ");
    Serial::write_hex(g_pmm.total_frames);
    Serial::write_str("\n");
    Serial::write_str("  Used frames: ");
    Serial::write_hex(g_pmm.used_frames);
    Serial::write_str("\n");
}

#[inline]
fn kernel_end() -> u32 {
    &raw const _kernel_end as usize as u32
}

/// `pmm_alloc_frame()`: allocate one 4 KiB physical frame.
#[no_mangle]
pub unsafe extern "C" fn pmm_alloc_frame() -> *mut c_void {
    let frame = find_free_frame();
    if frame < 0 {
        Serial::write_str("PMM: ERROR - Out of memory!\n");
        return core::ptr::null_mut();
    }
    set_frame(frame as u32);
    g_pmm.used_frames += 1;
    if (frame as u32) < MAX_PHYSICAL_FRAMES {
        frame_refcounts[frame as usize] = 1;
    }
    (frame as usize * PAGE_SIZE as usize) as *mut c_void
}

/// `pmm_free_frame(addr)`: return a frame to the bitmap.
#[no_mangle]
pub unsafe extern "C" fn pmm_free_frame(addr: *mut c_void) {
    let frame = (addr as u32) / PAGE_SIZE;
    if frame >= g_pmm.total_frames {
        Serial::write_str("PMM: ERROR - Invalid frame address\n");
        return;
    }
    if !test_frame(frame) {
        Serial::write_str("PMM: WARNING - Double free detected\n");
        return;
    }
    clear_frame(frame);
    g_pmm.used_frames -= 1;
}

/// `pmm_get_total_memory()`: total physical memory in bytes.
#[no_mangle]
pub unsafe extern "C" fn pmm_get_total_memory() -> u64 {
    g_pmm.total_memory
}

/// `pmm_get_available_memory()`: free physical memory in bytes.
#[no_mangle]
pub unsafe extern "C" fn pmm_get_available_memory() -> u64 {
    g_pmm.available_memory
}

/// `pmm_get_total_frames()`: number of managed physical frames.
#[no_mangle]
pub unsafe extern "C" fn pmm_get_total_frames() -> u32 {
    g_pmm.total_frames
}

/// `pmm_get_used_frames()`: number of allocated physical frames.
#[no_mangle]
pub unsafe extern "C" fn pmm_get_used_frames() -> u32 {
    g_pmm.used_frames
}

/// `pmm_refcount_inc(addr)`: increment a frame's COW refcount.
#[no_mangle]
pub unsafe extern "C" fn pmm_refcount_inc(addr: *mut c_void) {
    let frame = (addr as u32) / PAGE_SIZE;
    if (frame as u32) < MAX_PHYSICAL_FRAMES {
        frame_refcounts[frame as usize] = frame_refcounts[frame as usize].wrapping_add(1);
    }
}

/// `pmm_refcount_dec(addr)`: decrement a frame's refcount, freeing at zero.
#[no_mangle]
pub unsafe extern "C" fn pmm_refcount_dec(addr: *mut c_void) {
    let frame = (addr as u32) / PAGE_SIZE;
    if (frame as u32) < MAX_PHYSICAL_FRAMES {
        if frame_refcounts[frame as usize] > 0 {
            frame_refcounts[frame as usize] -= 1;
        }
        if frame_refcounts[frame as usize] == 0 {
            pmm_free_frame(addr);
        }
    }
}

/// `pmm_refcount_get(addr)`: current COW refcount of a frame.
#[no_mangle]
pub unsafe extern "C" fn pmm_refcount_get(addr: *mut c_void) -> u32 {
    let frame = (addr as u32) / PAGE_SIZE;
    if (frame as u32) < MAX_PHYSICAL_FRAMES {
        frame_refcounts[frame as usize]
    } else {
        0
    }
}

// ----------------------------------------------------------------------------
// Internal helpers used by the rest of the safe mem layer.
// ----------------------------------------------------------------------------

/// Zero `n` bytes at `dst` (wraps the raw builtin).
pub(crate) unsafe fn zero(dst: *mut u8, n: usize) {
    string::memset(dst as *mut c_void, 0, n);
}

/// Copy `n` bytes from `src` to `dst` (wraps the raw builtin).
pub(crate) unsafe fn copy_bytes(dst: *mut u8, src: *const u8, n: usize) {
    string::memcpy(dst as *mut c_void, src as *const c_void, n);
}
