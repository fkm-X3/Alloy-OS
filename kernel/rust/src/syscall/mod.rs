/// System call interface for Alloy OS
/// 
/// Provides syscall handlers that can be invoked via INT 0x80

use crate::ffi;
use crate::process::Scheduler;

pub mod table;
pub mod dispatcher;

/// Syscall numbers (must match syscall.h)
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum SyscallNumber {
    Exit = 0,
    Yield = 1,
    GetPid = 2,
    Sleep = 3,
}

/// sys_exit - Terminate the current task
/// arg0: exit code
#[no_mangle]
pub extern "C" fn rust_sys_exit(code: u32) -> u32 {
    unsafe {
        ffi::serial_print(b"[Syscall] sys_exit called with code \0".as_ptr());
        ffi::serial_print(b"\n\0".as_ptr());
    }

    // Mark current task as terminated and schedule next task
    let _ = Scheduler::with_current_task_mut(|task| {
        task.set_state(crate::process::task::TaskState::Terminated);
    });

    // Trigger scheduling
    Scheduler::schedule();

    // Should not return, but provide a value if it does
    code
}

/// sys_yield - Voluntarily give up the CPU
#[no_mangle]
pub extern "C" fn rust_sys_yield() -> u32 {
    unsafe {
        ffi::serial_print(b"[Syscall] sys_yield called\n\0".as_ptr());
    }
    
    Scheduler::yield_cpu();
    0 // Success
}

/// sys_getpid - Get the current task ID
#[no_mangle]
pub extern "C" fn rust_sys_getpid() -> u32 {
    unsafe {
        ffi::serial_print(b"[Syscall] sys_getpid called\n\0".as_ptr());
    }
    
    // TODO: Get current task ID from scheduler
    // For now, return dummy value
    1
}

/// sys_sleep - Sleep for specified milliseconds
/// arg0: milliseconds to sleep
#[no_mangle]
pub extern "C" fn rust_sys_sleep(ms: u32) -> u32 {
    unsafe {
        ffi::serial_print(b"[Syscall] sys_sleep called with ms=\0".as_ptr());
        ffi::serial_print(b"\n\0".as_ptr());
    }
    
    // TODO: Implement actual sleep with timer
    // For now, just busy loop (very inefficient!)
    let start = unsafe { ffi::timer_get_uptime_ms_ffi() };
    let target = start + ms as u64;
    
    while unsafe { ffi::timer_get_uptime_ms_ffi() } < target {
        // Yield to other tasks while waiting
        Scheduler::yield_cpu();
    }
    
    0 // Success
}

/// Minimal open syscall - stores a dummy vnode id in task FD table
#[no_mangle]
pub extern "C" fn rust_sys_open(path_ptr: u32, flags: u32, mode: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_open called\n\0".as_ptr()); }
    // Read up to 256 bytes from user pointer and interpret as C string
    if path_ptr == 0 { return core::u32::MAX; }
    let mut buffer = [0u8; 256];
    unsafe {
        if let Ok(_) = crate::utils::copy_from_user(path_ptr, &mut buffer) {
            // Find NUL
            let len = buffer.iter().position(|&c| c == 0).unwrap_or(256);
            if let Ok(path_str) = core::str::from_utf8(&buffer[..len]) {
                // Call VFS open
                match crate::fs::vfs_open(path_str, flags, mode) {
                    Ok(vnode_id) => {
                        match Scheduler::with_current_task_mut(|task| task.alloc_fd(vnode_id)) {
                            Some(fd_opt) => {
                                if let Some(fd) = fd_opt { return fd; } else { return core::u32::MAX; }
                            }
                            None => return core::u32::MAX,
                        }
                    }
                    Err(_) => return core::u32::MAX,
                }
            }
        }
    }
    core::u32::MAX
}

/// Minimal read syscall - not yet implemented
#[no_mangle]
pub extern "C" fn rust_sys_read(fd: u32, buf_ptr: u32, len: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_read called\n\0".as_ptr()); }
    // Use scheduler helper to get mutable fd entry
    if let Some(result) = Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            let vnode_id = entry.0;
            let offset = &mut entry.1;
            crate::fs::vfs_read(vnode_id, offset, buf_ptr, len as usize)
        } else {
            -1
        }
    }) {
        if result >= 0 { return result as u32; }
    }
    core::u32::MAX
}

/// Minimal write syscall - copies user buffer to serial for fd==1 (stdout)
#[no_mangle]
pub extern "C" fn rust_sys_write(fd: u32, buf_ptr: u32, len: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_write called\n\0".as_ptr()); }
    if fd == 1 {
        // Copy up to 240 bytes into a temporary buffer and null-terminate for serial_print
        let max = core::cmp::min(len as usize, 240usize);
        let mut buffer = [0u8; 241];
        if buf_ptr == 0 {
            return core::u32::MAX;
        }
        // Use copy_from_user helper
        unsafe {
            match crate::utils::copy_from_user(buf_ptr, &mut buffer[..max]) {
                Ok(copied) => {
                    buffer[copied] = 0;
                    ffi::serial_print(buffer.as_ptr());
                    return copied as u32;
                }
                Err(_) => return core::u32::MAX,
            }
        }
    }

    // Otherwise, try VFS write using per-fd offset
    if let Some(result) = Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            let vnode_id = entry.0;
            let offset = &mut entry.1;
            crate::fs::vfs_write(vnode_id, offset, buf_ptr, len as usize)
        } else { -1 }
    }) {
        if result >= 0 { return result as u32; }
    }
    core::u32::MAX
}

/// Minimal close syscall
#[no_mangle]
pub extern "C" fn rust_sys_close(fd: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_close called\n\0".as_ptr()); }
    match Scheduler::with_current_task_mut(|task| task.close_fd(fd)) {
        Some(Ok(())) => 0,
        Some(Err(())) => core::u32::MAX,
        None => core::u32::MAX,
    }
}

/// Duplicate a file descriptor (simple implementation returns new fd or error)
#[no_mangle]
pub extern "C" fn rust_sys_dup(oldfd: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_dup called\n\0".as_ptr()); }
    match Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(oldfd) {
            let vnode = entry.0;
            let offset = entry.1;
            if let Some(newfd) = task.alloc_fd(vnode) {
                // adjust offset for new fd
                if let Some(e) = task.get_fd_entry_mut(newfd) {
                    e.1 = offset;
                    return Some(newfd as u32);
                }
            }
        }
        None
    }) {
        Some(fd_opt) => {
            if let Some(fd) = fd_opt { fd } else { core::u32::MAX }
        }
        None => core::u32::MAX,
    }
}

/// lseek: fd, offset (signed 32-bit), whence
#[no_mangle]
pub extern "C" fn rust_sys_lseek(fd: u32, offset: u32, whence: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_lseek called\n\0".as_ptr()); }
    let off = offset as i32;
    match Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            // call into VFS to compute new offset
            crate::fs::vfs_lseek(entry.0, &mut entry.1, off, whence)
        } else { -1 }
    }) {
        Some(val) if val >= 0 => val as u32,
        _ => core::u32::MAX,
    }
}

/// pipe: create an anonymous pipe and write two fds into user pointer (int pipefd[2])
#[no_mangle]
pub extern "C" fn rust_sys_pipe(pipefd_ptr: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_pipe called\n\0".as_ptr()); }
    // Create pipe vnode
    match crate::fs::vfs_create_pipe() {
        Ok(vnode_id) => {
            // allocate two fds in current task
            match Scheduler::with_current_task_mut(|task| {
                let fd1 = task.alloc_fd(vnode_id);
                let fd2 = task.alloc_fd(vnode_id);
                (fd1, fd2)
            }) {
                Some((Some(f1), Some(f2))) => {
                    // write back to user pointer as two 32-bit ints
                    let mut arr = [0u32; 2];
                    arr[0] = f1;
                    arr[1] = f2;
                    unsafe {
                        // write the u32 array into user memory
                        let bytes = core::slice::from_raw_parts(arr.as_ptr() as *const u8, 8);
                        if let Ok(_) = crate::utils::copy_to_user(pipefd_ptr, bytes) {
                            return 0;
                        }
                    }
                    // If write failed, close fds
                    let _ = Scheduler::with_current_task_mut(|task| { let _ = task.close_fd(f1); let _ = task.close_fd(f2); });
                    core::u32::MAX
                }
                _ => core::u32::MAX,
            }
        }
        Err(_) => core::u32::MAX,
    }
}

/// execve: path_ptr (user), returns 0 on success; replaces current task image
#[no_mangle]
pub extern "C" fn rust_sys_execve(path_ptr: u32) -> u32 {
    unsafe { ffi::serial_print(b"[Syscall] sys_execve called\n\0".as_ptr()); }
    if path_ptr == 0 { return core::u32::MAX; }
    let mut buf = [0u8; 256];
    unsafe {
        if let Err(_) = crate::utils::copy_from_user(path_ptr, &mut buf) { return core::u32::MAX; }
    }
    let len = buf.iter().position(|&c| c==0).unwrap_or(256);
    let path = match core::str::from_utf8(&buf[..len]) {
        Ok(s) => s,
        Err(_) => return core::u32::MAX,
    };
    unsafe { ffi::serial_print(b"[Syscall] Opening file\n\0".as_ptr()); }
    // Open file in VFS
    let vnode = match crate::fs::vfs_open(path, 0, 0) {
        Ok(id) => id,
        Err(_) => return core::u32::MAX,
    };
    unsafe { ffi::serial_print(b"[Syscall] Reading file\n\0".as_ptr()); }
    // Read full file
    let image = match crate::fs::vfs_read_all(vnode) {
        Some(v) => v,
        None => return core::u32::MAX,
    };
    unsafe { ffi::serial_print(b"[Syscall] Creating page directory\n\0".as_ptr()); }
    
    // Create a fresh page directory for the new process image
    let pd_phys = unsafe { ffi::paging_create_directory_phys() };
    if pd_phys == 0 { return core::u32::MAX; }
    unsafe { ffi::serial_print(b"[Syscall] Loading ELF\n\0".as_ptr()); }
    match crate::elf::load_elf_from_bytes(&image) {
        Ok((entry, phdr_vaddr)) => {
            unsafe { ffi::serial_print(b"[Syscall] ELF loaded successfully\n\0".as_ptr()); }
            // Allocate stack at a fixed low address BEFORE switching
            const STACK_BASE: u32 = 0x00C00000;
            const STACK_SIZE: u32 = 0x4000; // 16KB
            
            let stack_flags = (ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER) as u32;
            let mut page_addr = STACK_BASE;
            while page_addr < STACK_BASE + STACK_SIZE {
                let phys = unsafe { ffi::pmm_alloc_frame() };
                if phys.is_null() { return core::u32::MAX; }
                let ok = unsafe { ffi::vmm_map(page_addr as *mut core::ffi::c_void, phys, stack_flags) };
                if !ok { return core::u32::MAX; }
                page_addr += 4096;
            }
            let stack_ptr = STACK_BASE;
            unsafe { ffi::serial_print(b"[Syscall] Stack allocated\n\0".as_ptr()); }

            // Switch to the new page directory
            let switched = unsafe { ffi::paging_switch_to_directory(pd_phys) };
            if !switched { return core::u32::MAX; }
            unsafe { ffi::serial_print(b"[Syscall] Switched to user directory\n\0".as_ptr()); }

            // Prepare minimal argv (argv[0] = basename, envp empty)
            unsafe { ffi::serial_print(b"[Syscall] Preparing argv\n\0".as_ptr()); }
            let prog_name = path.rsplit('/').next().unwrap_or(path);
            unsafe { ffi::serial_print(b"[Syscall] Got prog_name\n\0".as_ptr()); }
            let name_bytes = prog_name.as_bytes();
            unsafe { ffi::serial_print(b"[Syscall] Got name_bytes\n\0".as_ptr()); }
            let name_len = name_bytes.len() + 1; // NUL
            unsafe { ffi::serial_print(b"[Syscall] Got name_len\n\0".as_ptr()); }

            // Compute stack layout using a single aligned block. Layout (low->high): argc, argv ptrs, envp ptrs, auxv, strings
            unsafe { ffi::serial_print(b"[Syscall] Computing stack layout\n\0".as_ptr()); }
            let stack_top = stack_ptr + STACK_SIZE;
            unsafe { ffi::serial_print(b"[Syscall] Got stack_top\n\0".as_ptr()); }

            // Sizes
            let argv_count = 1u32;
            let argv_ptrs_size = (argv_count + 1) * 4; // argv + NULL
            let envp_ptrs_size = 1 * 4; // just NULL
            let auxv_entries = 6; // we'll create up to 6 u32 pairs (but only use some); compute realistically below
            let auxv_size = 6 * 8; // allocate room for up to 6 pairs (48 bytes)
            // But actual used auxv entries (we'll write only what we need)
            let used_auxv_size = 6 * 8; // keep consistent for writing

            // Total block size: argc (4) + argv_ptrs + envp_ptrs + auxv + strings
            let total_size = 4u32 + (argv_ptrs_size as u32) + (envp_ptrs_size as u32) + (used_auxv_size as u32) + (name_len as u32);

            // Align the start of the block to 16 bytes
            let mut block_start = stack_top.wrapping_sub(total_size);
            block_start = block_start & !0xF;

            let argc_addr = block_start;
            let argv_array_addr = argc_addr + 4;
            let envp_array_addr = argv_array_addr + (argv_ptrs_size as u32);
            let auxv_addr = envp_array_addr + (envp_ptrs_size as u32);
            let string_addr = auxv_addr + (used_auxv_size as u32);

            // Sanity: ensure string fits
            if string_addr + (name_len as u32) > stack_top { return core::u32::MAX; }

            // Write string directly to user stack (use copy_to_user which handles user context)
            unsafe {
                ffi::serial_print(b"[Syscall] Writing program name\n\0".as_ptr());
                // Create the NUL-terminated string by writing name bytes then NUL
                // We need to be careful here - name_bytes is from kernel memory
                for (i, &b) in name_bytes.iter().enumerate() {
                    let byte_addr = string_addr + (i as u32);
                    let bytes_slice = core::slice::from_raw_parts(&b, 1);
                    if let Err(_) = crate::utils::copy_to_user(byte_addr, bytes_slice) { return core::u32::MAX; }
                }
                // Write NUL terminator
                let nul_byte: u8 = 0;
                let nul_slice = core::slice::from_raw_parts(&nul_byte, 1);
                if let Err(_) = crate::utils::copy_to_user(string_addr + (name_bytes.len() as u32), nul_slice) { return core::u32::MAX; }
                ffi::serial_print(b"[Syscall] Program name written\n\0".as_ptr());
            }

            // Write argv pointers (argv0, NULL)
            let mut ptrs = [0u32; 2];
            ptrs[0] = string_addr as u32;
            ptrs[1] = 0;
            let ptrs_bytes = unsafe { core::slice::from_raw_parts(ptrs.as_ptr() as *const u8, argv_ptrs_size as usize) };
            unsafe {
                if let Err(_) = crate::utils::copy_to_user(argv_array_addr, ptrs_bytes) { return core::u32::MAX; }
            }

            // Write envp NULL
            let zero_ptr: u32 = 0;
            let zero_bytes = unsafe { core::slice::from_raw_parts((&zero_ptr) as *const u32 as *const u8, 4) };
            unsafe {
                if let Err(_) = crate::utils::copy_to_user(envp_array_addr, zero_bytes) { return core::u32::MAX; }
            }

            // Parse ELF header for e_phentsize and e_phnum and e_entry
            let (entry_val, phentsize, phnum) = match crate::elf::parse_elf_header(&image) {
                Some(t) => t,
                None => return core::u32::MAX,
            };

            const AT_PHDR: u32 = 3;
            const AT_PHENT: u32 = 4;
            const AT_PHNUM: u32 = 5;
            const AT_PAGESZ: u32 = 6;
            const AT_ENTRY: u32 = 9;
            const AT_NULL: u32 = 0;

            // Build aux vector: pairs of u32 (type, value)
            // We'll prepare an array sized to used_auxv_size
            let mut aux = [0u32; 12];
            let mut ai = 0;
            // AT_PHDR -> phdr virtual address returned by loader when available
            aux[ai] = AT_PHDR; ai += 1; aux[ai] = phdr_vaddr as u32; ai += 1;
            // AT_PHENT
            aux[ai] = AT_PHENT; ai += 1; aux[ai] = phentsize as u32; ai += 1;
            // AT_PHNUM
            aux[ai] = AT_PHNUM; ai += 1; aux[ai] = phnum as u32; ai += 1;
            // AT_PAGESZ
            aux[ai] = AT_PAGESZ; ai += 1; aux[ai] = 4096u32; ai += 1;
            // AT_ENTRY
            aux[ai] = AT_ENTRY; ai += 1; aux[ai] = entry_val as u32; ai += 1;
            // AT_NULL
            aux[ai] = AT_NULL; ai += 1; aux[ai] = 0u32; ai += 1;

            let auxv_bytes = unsafe { core::slice::from_raw_parts(aux.as_ptr() as *const u8, (ai * 4) as usize) };
            unsafe {
                if let Err(_) = crate::utils::copy_to_user(auxv_addr, auxv_bytes) { return core::u32::MAX; }
            }

            // Write argc
            let argc_val: u32 = 1;
            let argc_bytes = unsafe { core::slice::from_raw_parts((&argc_val) as *const u32 as *const u8, 4) };
            unsafe {
                if let Err(_) = crate::utils::copy_to_user(argc_addr, argc_bytes) { return core::u32::MAX; }
            }

            // Update current task context
            let _ = Scheduler::with_current_task_mut(|task| {
                let ctx = task.context_mut();
                ctx.eip = entry;
                // Set user segments (typical selectors)
                ctx.cs = 0x1B;
                ctx.ds = 0x23;
                ctx.es = 0x23;
                ctx.fs = 0x23;
                ctx.gs = 0x23;
                ctx.ss = 0x23;
                // Update CR3 in context for future context switches
                ctx.cr3 = pd_phys;
                // Set user stack to point to argc on stack
                ctx.esp = argc_addr;
            });

            0
        }
        Err(_) => core::u32::MAX,
    }
}

/// Invoke a syscall (for testing/internal use)
/// This is a safe Rust wrapper around INT 0x80
#[allow(dead_code)]
pub fn syscall(num: SyscallNumber, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("eax") num as u32,
            in("ebx") arg0,
            in("ecx") arg1,
            in("edx") arg2,
            lateout("eax") result,
        );
    }
    result
}

/// Convenience wrappers for syscalls
#[allow(dead_code)]
pub fn exit(code: u32) -> ! {
    syscall(SyscallNumber::Exit, code, 0, 0);
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[allow(dead_code)]
pub fn yield_cpu() {
    syscall(SyscallNumber::Yield, 0, 0, 0);
}

#[allow(dead_code)]
pub fn getpid() -> u32 {
    syscall(SyscallNumber::GetPid, 0, 0, 0)
}

#[allow(dead_code)]
pub fn sleep(ms: u32) {
    syscall(SyscallNumber::Sleep, ms, 0, 0);
}

/// Dispatcher wrapper callable from C/C++: routes raw registers to Rust dispatcher
#[no_mangle]
pub extern "C" fn rust_dispatcher(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    dispatcher::dispatch_syscall(eax, ebx, ecx, edx)
}
