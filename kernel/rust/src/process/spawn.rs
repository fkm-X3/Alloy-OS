use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use crate::process::task::{Task, CpuContext};
use crate::process::Scheduler;
use crate::ffi;
use crate::elf::{Elf32Ehdr, Elf32Phdr, Elf64Ehdr, Elf64Phdr};

const STACK_BASE: u64 = 0x00C00000;
const STACK_SIZE: u64 = 0x4000;

const PT_LOAD: u32 = 1;

/// Load an ELF binary (32 or 64-bit) into a new user-mode task and add it to the
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

    // Detect ELF class
    if image.len() < 16 { unsafe { ffi::paging_destroy_directory(pd_phys); } return false; }
    let loaded = match image[4] {
        1 => load_elf32(image, pd_phys, page_flags, page_size),
        2 => load_elf64(image, pd_phys, page_flags, page_size),
        _ => false,
    };
    if !loaded {
        unsafe { ffi::paging_destroy_directory(pd_phys); }
        return false;
    }

    // ── Build user-mode CPU context ──────────────────────────────────
    #[cfg(feature = "x86_64")]
    let ctx = Box::new(CpuContext {
        rax: 0, rbx: 0, rcx: 0, rdx: 0,
        rsi: 0, rdi: 0, rbp: (STACK_BASE + STACK_SIZE) as u64,
        rsp: (STACK_BASE + STACK_SIZE) as u64,
        r8: 0, r9: 0, r10: 0, r11: 0,
        r12: 0, r13: 0, r14: 0, r15: 0,
        rip: entry as u64,
        cs: 0x23, ds: 0x1B, es: 0x1B, fs: 0x1B, gs: 0x1B, ss: 0x1B,
        rflags: 0x202,
        cr3: pd_phys as u64,
    });
    #[cfg(feature = "aarch64")]
    let ctx = Box::new(CpuContext {
        x19: 0, x20: 0, x21: 0, x22: 0,
        x23: 0, x24: 0, x25: 0, x26: 0,
        x27: 0, x28: 0, fp: (STACK_BASE + STACK_SIZE) as u64,
        lr: 0, sp: (STACK_BASE + STACK_SIZE) as u64,
        elr: entry as u64,
        spsr: 0,
        ttbr0: pd_phys as u64,
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

fn load_elf32(image: &[u8], pd_phys: usize, page_flags: u32, page_size: usize) -> bool {
    let hdr = unsafe { &*(image.as_ptr() as *const Elf32Ehdr) };
    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    for i in 0..phnum {
        let phdr_off = phoff + i * phentsize;
        if phdr_off + core::mem::size_of::<Elf32Phdr>() > image.len() {
            unsafe { ffi::paging_destroy_directory(pd_phys); }
            return false;
        }
        let ph = unsafe { &*(image.as_ptr().add(phdr_off) as *const Elf32Phdr) };
        if ph.p_type != PT_LOAD { continue; }

        if !map_elf_segment(image, ph.p_offset as u64, ph.p_memsz as u64, ph.p_filesz as u64, ph.p_vaddr as u64, pd_phys, page_flags, page_size) {
            return false;
        }
    }
    true
}

fn load_elf64(image: &[u8], pd_phys: usize, page_flags: u32, page_size: usize) -> bool {
    if image.len() < core::mem::size_of::<Elf64Ehdr>() { return false; }
    let hdr = unsafe { &*(image.as_ptr() as *const Elf64Ehdr) };
    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    for i in 0..phnum {
        let phdr_off = phoff + i * phentsize;
        if phdr_off + core::mem::size_of::<Elf64Phdr>() > image.len() {
            unsafe { ffi::paging_destroy_directory(pd_phys); }
            return false;
        }
        let ph = unsafe { &*(image.as_ptr().add(phdr_off) as *const Elf64Phdr) };
        if ph.p_type != PT_LOAD { continue; }

        if !map_elf_segment(image, ph.p_offset, ph.p_memsz, ph.p_filesz, ph.p_vaddr, pd_phys, page_flags, page_size) {
            return false;
        }
    }
    true
}

fn map_elf_segment(
    image: &[u8],
    p_offset: u64, p_memsz: u64, p_filesz: u64, p_vaddr: u64,
    pd_phys: usize, page_flags: u32, page_size: usize,
) -> bool {
    let memsz = p_memsz as usize;
    let filesz = p_filesz as usize;
    let vaddr = p_vaddr as usize;

    let aligned_start = vaddr & !(page_size - 1);
    let aligned_end = (vaddr + memsz).div_ceil(page_size) * page_size;
    let alloc_size = aligned_end - aligned_start;
    let npages = alloc_size / page_size;
    unsafe {
        ffi::serial_print(c"[spawn] seg v=0x".as_ptr() as *const u8);
        ffi::serial_print_hex64(vaddr as u64);
        ffi::serial_print(c" pages=0x".as_ptr() as *const u8);
        ffi::serial_print_hex64(npages as u64);
        ffi::serial_print(c"\n".as_ptr() as *const u8);
    }

    let mut page_addr = aligned_start;
    let mut count: u64 = 0;

    // Disable interrupts for the ENTIRE mapping loop to prevent timer-driven
    // context switches from clobbering the shared window slot (PT_TEMP_IDX)
    // used by win_map/win_unmap inside paging_map_page_in_pd and
    // paging_temp_map_frame.  Serial I/O is polled so it works without ints.
    unsafe { core::arch::asm!("cli"); }

    while page_addr < aligned_start + alloc_size {
        unsafe { ffi::serial_print(c"a".as_ptr() as *const u8); }
        let phys = unsafe { ffi::pmm_alloc_frame() };
        if phys.is_null() {
            unsafe {
                ffi::serial_print(c"[spawn] OOM at page_addr=0x".as_ptr() as *const u8);
                ffi::serial_print_hex64(page_addr as u64);
                ffi::serial_print(c"\n".as_ptr() as *const u8);
                core::arch::asm!("sti");
            }
            return false;
        }
        let phys_addr = phys as usize;

        unsafe { ffi::serial_print(c"b".as_ptr() as *const u8); }
        let temp = unsafe { ffi::paging_temp_map_frame(phys_addr) };
        let page_off = page_addr.saturating_sub(vaddr);
        if page_off < filesz {
            let copy_len = core::cmp::min(page_size, filesz - page_off);
            unsafe {
                let src = image.as_ptr().add(p_offset as usize + page_off);
                ptr::copy_nonoverlapping(src, temp as *mut u8, copy_len);
            }
        }
        unsafe { ffi::paging_temp_unmap_frame(); }

        unsafe { ffi::serial_print(c"c".as_ptr() as *const u8); }
        let ok = unsafe { ffi::paging_map_page_in_pd(pd_phys, page_addr, phys_addr, page_flags) };
        unsafe { ffi::serial_print(c"d".as_ptr() as *const u8); }
        if !ok {
            unsafe {
                ffi::serial_print(c"[spawn] MAP FAIL page_addr=0x".as_ptr() as *const u8);
                ffi::serial_print_hex64(page_addr as u64);
                ffi::serial_print(c"\n".as_ptr() as *const u8);
                core::arch::asm!("sti");
            }
            return false;
        }
        page_addr += page_size;
        count += 1;
        if (count & 0xFF) == 0 {
            unsafe {
                ffi::serial_print(c"[spawn] progress count=0x".as_ptr() as *const u8);
                ffi::serial_print_hex64(count);
                ffi::serial_print(c" page_addr=0x".as_ptr() as *const u8);
                ffi::serial_print_hex64(page_addr as u64);
                ffi::serial_print(c"\n".as_ptr() as *const u8);
            }
        }
    }

    unsafe { core::arch::asm!("sti"); }
    unsafe {
        ffi::serial_print(c"[spawn] seg done v=0x".as_ptr() as *const u8);
        ffi::serial_print_hex64(vaddr as u64);
        ffi::serial_print(c"\n".as_ptr() as *const u8);
    }
    true
}
