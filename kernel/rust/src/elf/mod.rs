// Minimal ELF loader for i386 (32-bit) - supports ET_EXEC with PT_LOAD segments

use crate::ffi;
use core::ptr;

#[repr(C)]
struct Elf32Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf32Phdr {
    p_type: u32,
    p_offset: u32,
    p_vaddr: u32,
    p_paddr: u32,
    p_filesz: u32,
    p_memsz: u32,
    p_flags: u32,
    p_align: u32,
}

const PT_LOAD: u32 = 1;

/// Load an ELF image from a bytes slice into memory and return (entry point, phdr_vaddr) on success.
/// phdr_vaddr is the runtime virtual address of the program header table (AT_PHDR). If unknown, returns 0.
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

    // Track loaded PT_LOAD segments to help compute phdr runtime address
    #[derive(Clone, Copy)]
    struct LoadSeg { p_offset: u32, p_vaddr: u32, p_filesz: u32 };
    let mut loads: alloc::vec::Vec<LoadSeg> = alloc::vec::Vec::new();

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
            let aligned_end = ((vaddr + memsz + page_size - 1) / page_size) * page_size;
            let alloc_size = aligned_end - aligned_start;
            let flags = (ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER) as u32;

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
                        let dest = (page_addr + 0) as *mut u8;
                        let src = image.as_ptr().add(ph.p_offset as usize + copy_from);
                        ptr::copy_nonoverlapping(src, dest, copy_len);
                    }
                }
                page_addr += page_size;
            }

            // Record this load segment
            loads.push(LoadSeg { p_offset: ph.p_offset, p_vaddr: ph.p_vaddr, p_filesz: ph.p_filesz });
        }
    }

    // Compute runtime phdr address if possible: find load segment that contains file offset e_phoff
    let mut phdr_vaddr: u32 = 0;
    for seg in loads.iter() {
        let off = hdr.e_phoff as u32;
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
