// Minimal ELF loader - supports ET_EXEC with PT_LOAD segments (ELF32 and ELF64)
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
    pub(crate) e_ident: [u8; 16],
    pub(crate) e_type: u16,
    pub(crate) e_machine: u16,
    pub(crate) e_version: u32,
    pub(crate) e_entry: u32,
    pub(crate) e_phoff: u32,
    pub(crate) e_shoff: u32,
    pub(crate) e_flags: u32,
    pub(crate) e_ehsize: u16,
    pub(crate) e_phentsize: u16,
    pub(crate) e_phnum: u16,
    pub(crate) e_shentsize: u16,
    pub(crate) e_shnum: u16,
    pub(crate) e_shstrndx: u16,
}

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Elf32Phdr {
    pub(crate) p_type: u32,
    pub(crate) p_offset: u32,
    pub(crate) p_vaddr: u32,
    pub(crate) p_paddr: u32,
    pub(crate) p_filesz: u32,
    pub(crate) p_memsz: u32,
    pub(crate) p_flags: u32,
    pub(crate) p_align: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Elf64Ehdr {
    pub(crate) e_ident: [u8; 16],
    pub(crate) e_type: u16,
    pub(crate) e_machine: u16,
    pub(crate) e_version: u32,
    pub(crate) e_entry: u64,
    pub(crate) e_phoff: u64,
    pub(crate) e_shoff: u64,
    pub(crate) e_flags: u32,
    pub(crate) e_ehsize: u16,
    pub(crate) e_phentsize: u16,
    pub(crate) e_phnum: u16,
    pub(crate) e_shentsize: u16,
    pub(crate) e_shnum: u16,
    pub(crate) e_shstrndx: u16,
}

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Elf64Phdr {
    pub(crate) p_type: u32,
    pub(crate) p_flags: u32,
    pub(crate) p_offset: u64,
    pub(crate) p_vaddr: u64,
    pub(crate) p_paddr: u64,
    pub(crate) p_filesz: u64,
    pub(crate) p_memsz: u64,
    pub(crate) p_align: u64,
}

const PT_LOAD: u32 = 1;
const PT_TLS: u32 = 7;

const MAX_LOADS: usize = 32;

/// Common abstraction over ELF32/64 header fields
enum ElfClass {
    Bits32,
    Bits64,
}

/// Load an ELF image from a bytes slice into memory and return (entry point, phdr_vaddr) on success.
/// phdr_vaddr is the runtime virtual address of the program header table (AT_PHDR). If unknown, returns 0.
///
/// **No heap allocations.** Uses a fixed-size stack array to track load segments.
pub fn load_elf_from_bytes(image: &[u8]) -> Result<(u64,u64), i32> {
    // Check magic
    if image.len() < 16 { return Err(-1); }
    if image[0] != 0x7f || image[1] != b'E' || image[2] != b'L' || image[3] != b'F' {
        return Err(-1);
    }

    match image[4] {
        1 => load_elf32(image),
        2 => load_elf64(image),
        _ => Err(-1),
    }
}

fn load_elf32(image: &[u8]) -> Result<(u64,u64), i32> {
    if image.len() < core::mem::size_of::<Elf32Ehdr>() { return Err(-1); }
    let hdr = unsafe { &*(image.as_ptr() as *const Elf32Ehdr) };

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    #[derive(Clone, Copy, Default)]
    struct LoadSeg { p_offset: u32, p_vaddr: u32, p_filesz: u32 }
    let mut loads: [LoadSeg; MAX_LOADS] = [LoadSeg::default(); MAX_LOADS];
    let mut load_count: usize = 0;

    for i in 0..phnum {
        let phdr_offset = phoff + i * phentsize;
        if phdr_offset + core::mem::size_of::<Elf32Phdr>() > image.len() { return Err(-1); }
        let ph = unsafe { &*(image.as_ptr().add(phdr_offset) as *const Elf32Phdr) };
        if ph.p_type == PT_LOAD {
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr as usize;
            let page_size = 4096usize;
            let aligned_start = vaddr & !(page_size - 1);
            let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
            let alloc_size = aligned_end - aligned_start;
            let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;

            let mut page_addr = aligned_start;
            while page_addr < aligned_start + alloc_size {
                let phys = unsafe { ffi::pmm_alloc_frame() };
                if phys.is_null() { return Err(-1); }
                let ok = unsafe { ffi::vmm_map(page_addr as *mut core::ffi::c_void, phys, flags) };
                if !ok { return Err(-1); }
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

            if load_count < MAX_LOADS {
                loads[load_count] = LoadSeg { p_offset: ph.p_offset, p_vaddr: ph.p_vaddr, p_filesz: ph.p_filesz };
                load_count += 1;
            }
        }
    }

    let mut phdr_vaddr: u64 = 0;
    for i in 0..load_count {
        let seg = &loads[i];
        let off = hdr.e_phoff;
        if off >= seg.p_offset && off < seg.p_offset + seg.p_filesz {
            let delta = off - seg.p_offset;
            phdr_vaddr = (seg.p_vaddr + delta) as u64;
            break;
        }
    }

    Ok((hdr.e_entry as u64, phdr_vaddr))
}

fn load_elf64(image: &[u8]) -> Result<(u64,u64), i32> {
    if image.len() < core::mem::size_of::<Elf64Ehdr>() as usize { return Err(-1); }
    let hdr = unsafe { &*(image.as_ptr() as *const Elf64Ehdr) };

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    #[derive(Clone, Copy, Default)]
    struct LoadSeg64 { p_offset: u64, p_vaddr: u64, p_filesz: u64 }
    let mut loads: [LoadSeg64; MAX_LOADS] = [LoadSeg64::default(); MAX_LOADS];
    let mut load_count: usize = 0;

    for i in 0..phnum {
        let phdr_offset = phoff + i * phentsize;
        if phdr_offset + core::mem::size_of::<Elf64Phdr>() > image.len() { return Err(-1); }
        let ph = unsafe { &*(image.as_ptr().add(phdr_offset) as *const Elf64Phdr) };
        if ph.p_type == PT_LOAD {
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr as usize;
            let page_size = 4096usize;
            let aligned_start = vaddr & !(page_size - 1);
            let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
            let alloc_size = aligned_end - aligned_start;
            let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;

            let mut page_addr = aligned_start;
            while page_addr < aligned_start + alloc_size {
                let phys = unsafe { ffi::pmm_alloc_frame() };
                if phys.is_null() { return Err(-1); }
                let ok = unsafe { ffi::vmm_map(page_addr as *mut core::ffi::c_void, phys, flags) };
                if !ok { return Err(-1); }
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

            if load_count < MAX_LOADS {
                loads[load_count] = LoadSeg64 { p_offset: ph.p_offset, p_vaddr: ph.p_vaddr, p_filesz: ph.p_filesz };
                load_count += 1;
            }
        }
    }

    let mut phdr_vaddr: u64 = 0;
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

/// Find the PT_TLS segment and return (vaddr, memsz).
/// The thread pointer (FS base) on x86_64 = vaddr + memsz (end of TLS block).
pub fn find_tls_info(image: &[u8]) -> Option<(u64, u64)> {
    if image.len() < 16 { return None; }
    if image[0] != 0x7f || image[1] != b'E' || image[2] != b'L' || image[3] != b'F' {
        return None;
    }
    match image[4] {
        1 => {
            if image.len() < core::mem::size_of::<Elf32Ehdr>() { return None; }
            let hdr = unsafe { &*(image.as_ptr() as *const Elf32Ehdr) };
            let phoff = hdr.e_phoff as usize;
            let phentsize = hdr.e_phentsize as usize;
            let phnum = hdr.e_phnum as usize;
            for i in 0..phnum {
                let off = phoff + i * phentsize;
                if off + core::mem::size_of::<Elf32Phdr>() > image.len() { return None; }
                let ph = unsafe { &*(image.as_ptr().add(off) as *const Elf32Phdr) };
                if ph.p_type == PT_TLS {
                    return Some((ph.p_vaddr as u64, ph.p_memsz as u64));
                }
            }
            None
        }
        2 => {
            if image.len() < core::mem::size_of::<Elf64Ehdr>() { return None; }
            let hdr = unsafe { &*(image.as_ptr() as *const Elf64Ehdr) };
            let phoff = hdr.e_phoff as usize;
            let phentsize = hdr.e_phentsize as usize;
            let phnum = hdr.e_phnum as usize;
            for i in 0..phnum {
                let off = phoff + i * phentsize;
                if off + core::mem::size_of::<Elf64Phdr>() > image.len() { return None; }
                let ph = unsafe { &*(image.as_ptr().add(off) as *const Elf64Phdr) };
                if ph.p_type == PT_TLS {
                    return Some((ph.p_vaddr, ph.p_memsz));
                }
            }
            None
        }
        _ => None,
    }
}

/// Parse minimal ELF header fields useful for execve auxv
pub fn parse_elf_header(image: &[u8]) -> Option<(u64, u16, u16)> {
    if image.len() < 16 { return None; }
    if image[0] != 0x7f || image[1] != b'E' || image[2] != b'L' || image[3] != b'F' {
        return None;
    }
    match image[4] {
        1 => {
            let hdr = unsafe { &*(image.as_ptr() as *const Elf32Ehdr) };
            Some((hdr.e_entry as u64, hdr.e_phentsize, hdr.e_phnum))
        }
        2 => {
            let hdr = unsafe { &*(image.as_ptr() as *const Elf64Ehdr) };
            Some((hdr.e_entry, hdr.e_phentsize, hdr.e_phnum))
        }
        _ => None,
    }
}
