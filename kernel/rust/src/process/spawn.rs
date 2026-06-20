use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use crate::process::task::{Task, CpuContext};
use crate::process::Scheduler;
use crate::ffi;

const STACK_BASE: u32 = 0x00C00000;
const STACK_SIZE: u32 = 0x4000;

const PT_LOAD: u32 = 1;
const MAX_LOADS: usize = 32;

/// Load a 32-bit ELF binary into a new user-mode task and add it to the
/// scheduler ready queue. Returns true on success.
///
/// The ELF is loaded into the new process's page directory *without*
/// switching CR3 away from the kernel page directory.  This avoids a
/// class of bugs where kernel-heap data (stored at PDE 4+, e.g.
/// 0x0100_xxxx) becomes inaccessible or where vmm_alloc_region corrupts
/// the global VMM state by mapping into the wrong address space.
pub fn spawn_user_elf(image: &[u8]) -> bool {
    // Pre-parse the ELF header to get the entry point without doing any
    // page mapping.
    let entry = match crate::elf::parse_elf_header(image) {
        Some((e, _, _)) => e,
        None => return false,
    };

    // Allocate a fresh page directory for the new process
    let pd_phys = unsafe { ffi::paging_create_directory_phys() };
    if pd_phys == 0 {
        return false;
    }

    // ── Load ELF segments into pd_phys (kernel PD stays active) ──────
    let page_flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;
    let page_size = 4096usize;

    // Parse program headers from the image (kernel heap is accessible)
    let hdr = unsafe { &*(image.as_ptr() as *const crate::elf::Elf32Ehdr) };
    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    // Track load segments on the stack (no heap alloc) for phdr_vaddr
    #[derive(Clone, Copy, Default)]
    struct LoadSeg { p_offset: u32, p_vaddr: u32, p_filesz: u32 }
    let mut loads: [LoadSeg; MAX_LOADS] = [LoadSeg::default(); MAX_LOADS];
    let mut load_count: usize = 0;

    for i in 0..phnum {
        let phdr_off = phoff + i * phentsize;
        if phdr_off + core::mem::size_of::<crate::elf::Elf32Phdr>() > image.len() {
            unsafe { ffi::paging_destroy_directory(pd_phys); }
            return false;
        }
        let ph = unsafe { &*(image.as_ptr().add(phdr_off) as *const crate::elf::Elf32Phdr) };
        if ph.p_type != PT_LOAD {
            continue;
        }

        let memsz = ph.p_memsz as usize;
        let filesz = ph.p_filesz as usize;
        let vaddr = ph.p_vaddr as usize;

        let aligned_start = vaddr & !(page_size - 1);
        let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
        let alloc_size = aligned_end - aligned_start;

        let mut page_addr = aligned_start;
        while page_addr < aligned_start + alloc_size {
            let phys = unsafe { ffi::pmm_alloc_frame() };
            if phys.is_null() {
                unsafe { ffi::paging_destroy_directory(pd_phys); }
                return false;
            }
            let phys_addr = phys as usize;

            // Map the physical frame into a kernel-window slot so we can
            // write data to it, then unmap and wire it into the new PD.
            let temp = unsafe { ffi::paging_temp_map_frame(phys_addr) };
            let page_off = page_addr.saturating_sub(vaddr);
            if page_off < filesz {
                let copy_from = page_off;
                let copy_len = core::cmp::min(page_size, filesz - copy_from);
                unsafe {
                    let src = image.as_ptr().add(ph.p_offset as usize + copy_from);
                    ptr::copy_nonoverlapping(src, temp as *mut u8, copy_len);
                }
            }
            unsafe { ffi::paging_temp_unmap_frame(); }

            // Map the frame into the new process's page directory
            let ok = unsafe { ffi::paging_map_page_in_pd(pd_phys, page_addr as usize, phys_addr, page_flags) };
            if !ok {
                unsafe { ffi::paging_destroy_directory(pd_phys); }
                return false;
            }
            page_addr += page_size;
        }

        // Record this segment for phdr_vaddr computation
        if load_count < MAX_LOADS {
            loads[load_count] = LoadSeg {
                p_offset: ph.p_offset,
                p_vaddr: ph.p_vaddr,
                p_filesz: ph.p_filesz,
            };
            load_count += 1;
        }
    }

    // ── Allocate and map user stack in the new address space ──────────
    let stack_flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;
    let mut page_addr = STACK_BASE;
    while page_addr < STACK_BASE + STACK_SIZE {
        let phys = unsafe { ffi::pmm_alloc_frame() };
        if phys.is_null() {
            unsafe { ffi::paging_destroy_directory(pd_phys); }
            return false;
        }
        let ok = unsafe { ffi::paging_map_page_in_pd(pd_phys, page_addr as usize, phys as usize, stack_flags) };
        if !ok {
            unsafe { ffi::paging_destroy_directory(pd_phys); }
            return false;
        }
        page_addr += 4096;
    }

    // ── Compute phdr_vaddr and entry point ───────────────────────────
    let mut _phdr_vaddr: u32 = 0;
    for i in 0..load_count {
        let seg = &loads[i];
        let off = hdr.e_phoff;
        if off >= seg.p_offset && off < seg.p_offset + seg.p_filesz {
            _phdr_vaddr = seg.p_vaddr + (off - seg.p_offset);
            break;
        }
    }

    // ── Build user-mode CPU context ──────────────────────────────────
    let ctx = Box::new(CpuContext {
        eax: 0, ebx: 0, ecx: 0, edx: 0,
        esi: 0, edi: 0, ebp: STACK_BASE + STACK_SIZE,
        esp: STACK_BASE + STACK_SIZE,
        eip: entry,
        cs: 0x1B,
        ds: 0x23,
        es: 0x23,
        fs: 0x23,
        gs: 0x23,
        ss: 0x23,
        eflags: 0x202,
        cr3: pd_phys,
    });

    let task = Box::new(Task::from_parts(
        ctx,
        None,
        String::from("compositor"),
        [None; 32],
        0x01000000,
        None,
    ));

    Scheduler::add_task(task);
    true
}
