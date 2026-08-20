// Minimal ELF loader - supports ET_EXEC with PT_LOAD segments (ELF32 and ELF64)
//
// IMPORTANT: This function may be called with a non-kernel page directory active
// (e.g., during spawn_user_elf). It MUST NOT perform any heap allocations that
// would call vmm_alloc_region, because that function maps pages into the *current*
// page directory and advances the global kernel heap pointer, corrupting VMM state.
// Use fixed-size stack arrays instead of Vec/Box/etc.

use alloy_kernel_hal::elf::{
    parse_elf32_header, parse_elf32_phdr, parse_elf64_header, parse_elf64_phdr,
    elf_class,
};

// Re-export HAL types so downstream code (spawn.rs etc.) can reference them
// without importing the HAL ELF module directly.
pub(crate) use alloy_kernel_hal::elf::Elf32Header;
pub(crate) use alloy_kernel_hal::elf::Elf64Header;

const PT_LOAD: u32 = 1;
const PT_TLS: u32 = 7;

const MAX_LOADS: usize = 32;

/// Load an ELF image from a bytes slice into memory and return (entry point, phdr_vaddr) on success.
/// phdr_vaddr is the runtime virtual address of the program header table (AT_PHDR). If unknown, returns 0.
///
/// **No heap allocations.** Uses a fixed-size stack array to track load segments.
pub fn load_elf_from_bytes(image: &[u8]) -> Result<(u64, u64), i32> {
    let class = elf_class(image).ok_or(-1)?;

    match class {
        1 => load_elf32(image),
        2 => load_elf64(image),
        _ => Err(-1),
    }
}

fn load_elf32(image: &[u8]) -> Result<(u64, u64), i32> {
    let hdr = parse_elf32_header(image).ok_or(-1)?;

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    #[derive(Clone, Copy, Default)]
    struct LoadSeg {
        p_offset: u32,
        p_vaddr: u32,
        p_filesz: u32,
    }
    let mut loads: [LoadSeg; MAX_LOADS] = [LoadSeg::default(); MAX_LOADS];
    let mut load_count: usize = 0;

    for i in 0..phnum {
        let phdr_offset = phoff + i * phentsize;
        let ph = parse_elf32_phdr(image, phdr_offset).ok_or(-1)?;
        if ph.p_type == PT_LOAD {
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr as usize;
            let page_size = 4096usize;
            let aligned_start = vaddr & !(page_size - 1);
            let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
            let alloc_size = aligned_end - aligned_start;
            let flags = alloy_kernel_hal::PageFlags::user_write();

            let mut page_addr = aligned_start;
            while page_addr < aligned_start + alloc_size {
                let frame = alloy_kernel_hal::PhysFrame::alloc().ok_or(-1)?;
                let phys = frame.into_addr(); // owned by the page directory now
                let ok = alloy_kernel_hal::mem::map_page(
                    page_addr,
                    phys,
                    flags,
                );
                if !ok {
                    return Err(-1);
                }
                let page_offset = page_addr.saturating_sub(vaddr);
                if page_offset < filesz {
                    let copy_from = page_offset;
                    let copy_len = core::cmp::min(page_size, filesz - copy_from);
                    let src = &image[ph.p_offset as usize + copy_from..ph.p_offset as usize + copy_from + copy_len];
                    alloy_kernel_hal::mem::copy_to_mapped(page_addr, src);
                }
                page_addr += page_size;
            }

            if load_count < MAX_LOADS {
                loads[load_count] = LoadSeg {
                    p_offset: ph.p_offset,
                    p_vaddr: ph.p_vaddr,
                    p_filesz: ph.p_filesz,
                };
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

fn load_elf64(image: &[u8]) -> Result<(u64, u64), i32> {
    let hdr = parse_elf64_header(image).ok_or(-1)?;

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    #[derive(Clone, Copy, Default)]
    struct LoadSeg64 {
        p_offset: u64,
        p_vaddr: u64,
        p_filesz: u64,
    }
    let mut loads: [LoadSeg64; MAX_LOADS] = [LoadSeg64::default(); MAX_LOADS];
    let mut load_count: usize = 0;

    for i in 0..phnum {
        let phdr_offset = phoff + i * phentsize;
        let ph = parse_elf64_phdr(image, phdr_offset).ok_or(-1)?;
        if ph.p_type == PT_LOAD {
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr as usize;
            let page_size = 4096usize;
            let aligned_start = vaddr & !(page_size - 1);
            let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
            let alloc_size = aligned_end - aligned_start;
            let flags = alloy_kernel_hal::PageFlags::user_write();

            let mut page_addr = aligned_start;
            while page_addr < aligned_start + alloc_size {
                let frame = alloy_kernel_hal::PhysFrame::alloc().ok_or(-1)?;
                let phys = frame.into_addr();
                let ok = alloy_kernel_hal::mem::map_page(
                    page_addr,
                    phys,
                    flags,
                );
                if !ok {
                    return Err(-1);
                }
                let page_offset = page_addr.saturating_sub(vaddr);
                if page_offset < filesz {
                    let copy_from = page_offset;
                    let copy_len = core::cmp::min(page_size, filesz - copy_from);
                    let src = &image[ph.p_offset as usize + copy_from..ph.p_offset as usize + copy_from + copy_len];
                    alloy_kernel_hal::mem::copy_to_mapped(page_addr, src);
                }
                page_addr += page_size;
            }

            if load_count < MAX_LOADS {
                loads[load_count] = LoadSeg64 {
                    p_offset: ph.p_offset,
                    p_vaddr: ph.p_vaddr,
                    p_filesz: ph.p_filesz,
                };
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
    let class = elf_class(image)?;

    match class {
        1 => {
            let hdr = parse_elf32_header(image)?;
            let phoff = hdr.e_phoff as usize;
            let phentsize = hdr.e_phentsize as usize;
            let phnum = hdr.e_phnum as usize;
            for i in 0..phnum {
                let off = phoff + i * phentsize;
                let ph = parse_elf32_phdr(image, off)?;
                if ph.p_type == PT_TLS {
                    return Some((ph.p_vaddr as u64, ph.p_memsz as u64));
                }
            }
            None
        }
        2 => {
            let hdr = parse_elf64_header(image)?;
            let phoff = hdr.e_phoff as usize;
            let phentsize = hdr.e_phentsize as usize;
            let phnum = hdr.e_phnum as usize;
            for i in 0..phnum {
                let off = phoff + i * phentsize;
                let ph = parse_elf64_phdr(image, off)?;
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
    let class = elf_class(image)?;

    match class {
        1 => {
            let hdr = parse_elf32_header(image)?;
            Some((hdr.e_entry as u64, hdr.e_phentsize, hdr.e_phnum))
        }
        2 => {
            let hdr = parse_elf64_header(image)?;
            Some((hdr.e_entry, hdr.e_phentsize, hdr.e_phnum))
        }
        _ => None,
    }
}
