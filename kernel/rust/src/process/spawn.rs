use alloc::boxed::Box;
use alloc::string::String;
use crate::process::task::{Task, CpuContext};
use crate::process::Scheduler;
use crate::ffi;

const STACK_BASE: u32 = 0x00C00000;
const STACK_SIZE: u32 = 0x4000;

/// Load a 32-bit ELF binary into a new user-mode task and add it to the
/// scheduler ready queue. Returns true on success.
///
/// The ELF is loaded into a fresh page directory after switching CR3, so
/// vmm_map calls operate on the new process's address space.
pub fn spawn_user_elf(image: &[u8]) -> bool {
    // Pre-parse the ELF header to get the entry point without doing any
    // page mapping (which would pollute the kernel's address space).
    let entry = match crate::elf::parse_elf_header(image) {
        Some((e, _, _)) => e,
        None => return false,
    };

    // Allocate a fresh page directory for the new process
    let pd_phys = unsafe { ffi::paging_create_directory_phys() };
    if pd_phys == 0 {
        return false;
    }

    // Switch to the new directory so vmm_map targets the right address space
    let switched = unsafe { ffi::paging_switch_to_directory(pd_phys) };
    if !switched {
        return false;
    }

    // Load the ELF — allocates frames and maps them via vmm_map in the new
    // page directory (current CR3).
    if crate::elf::load_elf_from_bytes(image).is_err() {
        unsafe { ffi::paging_switch_to_directory(ffi::paging_get_kernel_directory_phys()); }
        return false;
    }

    // Allocate and map user stack in the new address space
    let stack_flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;
    let mut page_addr = STACK_BASE;
    while page_addr < STACK_BASE + STACK_SIZE {
        let phys = unsafe { ffi::pmm_alloc_frame() };
        if phys.is_null() {
            return false;
        }
        let ok = unsafe { ffi::vmm_map(page_addr as *mut core::ffi::c_void, phys, stack_flags) };
        if !ok {
            return false;
        }
        page_addr += 4096;
    }

    // Switch back to kernel directory
    let kernel_pd = unsafe { ffi::paging_get_kernel_directory_phys() };
    unsafe { ffi::paging_switch_to_directory(kernel_pd); }

    // Build user-mode CPU context
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
