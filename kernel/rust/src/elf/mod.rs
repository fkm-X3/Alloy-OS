// Minimal ELF loader for i386 (32-bit) - supports ET_EXEC with PT_LOAD segments
//
// IMPORTANT: This function may be called with a non-kernel page directory active
// (e.g., during spawn_user_elf). It MUST NOT perform any heap allocations that
// would call vmm_alloc_region, because that function maps pages into the *current*
// page directory and advances the global kernel heap pointer, corrupting VMM state.
// Use fixed-size stack arrays instead of Vec/Box/etc.

use crate::ffi;
use core::ptr;

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Elf32Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u32,
    pub e_phoff: u32,
    pub e_shoff: u32,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Elf32Phdr {
    pub p_type: u32,
    pub p_offset: u32,
    pub p_vaddr: u32,
    pub p_paddr: u32,
    pub p_filesz: u32,
    pub p_memsz: u32,
    pub p_flags: u32,
    pub p_align: u32,
}

const PT_LOAD: u32 = 1;

const MAX_LOADS: usize = 32;

/// Load an ELF image from a bytes slice into memory and return (entry point, phdr_vaddr) on success.
/// phdr_vaddr is the runtime virtual address of the program header table (AT_PHDR). If unknown, returns 0.
///
/// **No heap allocations.** Uses a fixed-size stack array to track load segments.
pub fn load_elf_from_bytes(image: &[u8]) -> Result<(u32,u32), i32> {
    // Basic size checks
    if image.len() < core::mem::size_of::<Elf32Ehdr>() { return Err(-1); }
    let hdr = unsafe { &*(image.as_ptr() as *const Elf32Ehdr) };
    // Check magic 0x7f 'E' 'L' 'F'
    if hdr.e_ident[0] != 0x7f || hdr.e_ident[1] != b'E' || hdr.e_ident[2] != b'L' || hdr.e_ident[3] != b'F' {
        return Err(-1);
    }
    // Only support 32-bit little-endian for now
    if hdr.e_ident[4] != 1 { return Err(-1); } // EI_CLASS = ELFCLASS32

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    // Track loaded PT_LOAD segments on the stack (no heap alloc) to compute phdr runtime address.
    #[derive(Clone, Copy, Default)]
    struct LoadSeg { p_offset: u32, p_vaddr: u32, p_filesz: u32 }
    let mut loads: [LoadSeg; MAX_LOADS] = [LoadSeg::default(); MAX_LOADS];
    let mut load_count: usize = 0;

    for i in 0..phnum {
        let phdr_offset = phoff + i * phentsize;
        if phdr_offset + core::mem::size_of::<Elf32Phdr>() > image.len() { return Err(-1); }
        let ph = unsafe { &*(image.as_ptr().add(phdr_offset) as *const Elf32Phdr) };
        if ph.p_type == PT_LOAD {
            // Map segment to requested virtual address p_vaddr
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr as usize;
            // Align to page boundaries
            let page_size = 4096usize;
            let aligned_start = vaddr & !(page_size - 1);
            let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
            let alloc_size = aligned_end - aligned_start;
            let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;

            // Allocate and map physical frames for each page, then copy file data
            let mut page_addr = aligned_start;
            while page_addr < aligned_start + alloc_size {
                // Allocate physical frame
                let phys = unsafe { ffi::pmm_alloc_frame() };
                if phys.is_null() { return Err(-1); }
                // Map this physical frame at the desired virtual address
                let ok = unsafe { ffi::vmm_map(page_addr as *mut core::ffi::c_void, phys, flags) };
                if !ok { return Err(-1); }
                // If this page overlaps file data, copy the bytes
                let page_offset = page_addr.saturating_sub(vaddr);
                if page_offset < filesz {
                    let copy_from = page_offset;
                    let copy_len = core::cmp::min(page_size, filesz - copy_from);
                    unsafe {
                        let dest = page_addr as *mut u8;
                        let src = image.as_ptr().add(ph.p_offset as usize + copy_from);
                        ptr::copy_nonoverlapping(src, dest, copy_len);
                    }
                }
                page_addr += page_size;
            }

            // Record this load segment (up to MAX_LOADS — enough for any real ELF)
            if load_count < MAX_LOADS {
                loads[load_count] = LoadSeg { p_offset: ph.p_offset, p_vaddr: ph.p_vaddr, p_filesz: ph.p_filesz };
                load_count += 1;
            }
        }
    }

    // Compute runtime phdr address if possible: find load segment that contains file offset e_phoff
    let mut phdr_vaddr: u32 = 0;
    for i in 0..load_count {
        let seg = &loads[i];
        let off = hdr.e_phoff;
        if off >= seg.p_offset && off < seg.p_offset + seg.p_filesz {
            let delta = off - seg.p_offset;
            phdr_vaddr = seg.p_vaddr + delta;
            break;
        }
    }

    Ok((hdr.e_entry, phdr_vaddr))
}

/// Parse minimal ELF header fields useful for execve auxv
pub fn parse_elf_header(image: &[u8]) -> Option<(u32, u16, u16)> {
    if image.len() < core::mem::size_of::<Elf32Ehdr>() { return None; }
    let hdr = unsafe { &*(image.as_ptr() as *const Elf32Ehdr) };
    if hdr.e_ident[0] != 0x7f || hdr.e_ident[1] != b'E' || hdr.e_ident[2] != b'L' || hdr.e_ident[3] != b'F' {
        return None;
    }
    if hdr.e_ident[4] != 1 { return None; } // ELFCLASS32

    Some((hdr.e_entry, hdr.e_phentsize, hdr.e_phnum))
}
