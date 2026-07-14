//! System call interface for Alloy OS
//!
//! Provides syscall handlers that can be invoked via INT 0x80

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
    Open = 4,
    Read = 5,
    Write = 6,
    Close = 7,
    Dup = 8,
    Lseek = 9,
    Pipe = 10,
    Execve = 11,
    Socket = 12,
    Bind = 13,
    Listen = 14,
    Accept = 15,
    Connect = 16,
    CloseSocket = 17,
    HasPendingConnections = 18,
    Brk = 19,
    Fork = 20,
    Clone = 21,
    WaitPid = 22,
}

/// sys_exit - Terminate the current task
/// arg0: exit code
#[no_mangle]
pub extern "C" fn rust_sys_exit(code: u32) -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_exit called with code ".as_ptr() as *const u8);
        ffi::serial_print(c"\n".as_ptr() as *const u8);
    }

    Scheduler::terminate_current(code);
    code
}

/// sys_yield - Voluntarily give up the CPU
#[no_mangle]
pub extern "C" fn rust_sys_yield() -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_yield called\n".as_ptr() as *const u8);
    }
    Scheduler::yield_cpu();
    0
}

/// sys_getpid - Get the current task ID
#[no_mangle]
pub extern "C" fn rust_sys_getpid() -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_getpid called\n".as_ptr() as *const u8);
    }
    Scheduler::with_current_task(|task| task.id().as_u32()).unwrap_or(1)
}

/// sys_sleep - Sleep for specified milliseconds
#[no_mangle]
pub extern "C" fn rust_sys_sleep(ms: u32) -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_sleep called\n".as_ptr() as *const u8);
    }
    let start = unsafe { ffi::timer_get_uptime_ms_ffi() };
    let target = start + ms as u64;

    while unsafe { ffi::timer_get_uptime_ms_ffi() } < target {
        Scheduler::yield_cpu();
    }
    0
}

/// Minimal open syscall
#[no_mangle]
pub extern "C" fn rust_sys_open(path_ptr: u32, flags: u32, mode: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_open called\n".as_ptr() as *const u8); }
    if path_ptr == 0 { return u32::MAX; }
    let mut buffer = [0u8; 256];
    unsafe {
        if crate::utils::copy_from_user(path_ptr, &mut buffer).is_ok() {
            let len = buffer.iter().position(|&c| c == 0).unwrap_or(256);
            if let Ok(path_str) = core::str::from_utf8(&buffer[..len]) {
                match crate::fs::vfs_open(path_str, flags, mode) {
                    Ok(vnode_id) => {
                        match Scheduler::with_current_task_mut(|task| task.alloc_fd(vnode_id)) {
                            Some(Some(fd)) => return fd,
                            Some(None) => return u32::MAX,
                            None => return u32::MAX,
                        }
                    }
                    Err(_) => return u32::MAX,
                }
            }
        }
    }
    u32::MAX
}

/// Minimal read syscall
#[no_mangle]
pub extern "C" fn rust_sys_read(fd: u32, buf_ptr: u32, len: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_read called\n".as_ptr() as *const u8); }
    if let Some(result) = Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            let vnode_id = entry.0;
            let offset = &mut entry.1;
            crate::fs::vfs_read(vnode_id, offset, buf_ptr, len as usize)
        } else { -1 }
    }) {
        if result >= 0 { return result as u32; }
    }
    u32::MAX
}

/// Minimal write syscall
#[no_mangle]
pub extern "C" fn rust_sys_write(fd: u32, buf_ptr: u32, len: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_write called\n".as_ptr() as *const u8); }
    if fd == 1 {
        let max = core::cmp::min(len as usize, 240usize);
        let mut buffer = [0u8; 241];
        if buf_ptr == 0 { return u32::MAX; }
        unsafe {
            match crate::utils::copy_from_user(buf_ptr, &mut buffer[..max]) {
                Ok(copied) => {
                    buffer[copied] = 0;
                    ffi::serial_print(buffer.as_ptr());
                    return copied as u32;
                }
                Err(_) => return u32::MAX,
            }
        }
    }
    if let Some(result) = Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            let vnode_id = entry.0;
            let offset = &mut entry.1;
            crate::fs::vfs_write(vnode_id, offset, buf_ptr, len as usize)
        } else { -1 }
    }) {
        if result >= 0 { return result as u32; }
    }
    u32::MAX
}

/// Minimal close syscall
#[no_mangle]
pub extern "C" fn rust_sys_close(fd: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_close called\n".as_ptr() as *const u8); }
    match Scheduler::with_current_task_mut(|task| task.close_fd(fd)) {
        Some(Ok(())) => 0,
        Some(Err(_)) => u32::MAX,
        None => u32::MAX,
    }
}

/// Duplicate a file descriptor
#[no_mangle]
pub extern "C" fn rust_sys_dup(oldfd: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_dup called\n".as_ptr() as *const u8); }
    match Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(oldfd) {
            let vnode = entry.0;
            let offset = entry.1;
            if let Some(newfd) = task.alloc_fd(vnode) {
                if let Some(e) = task.get_fd_entry_mut(newfd) {
                    e.1 = offset;
                    return Some(newfd);
                }
            }
        }
        None
    }) {
        Some(fd_opt) => {
            fd_opt.unwrap_or(u32::MAX)
        }
        None => u32::MAX,
    }
}

/// lseek
#[no_mangle]
pub extern "C" fn rust_sys_lseek(fd: u32, offset: u32, whence: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_lseek called\n".as_ptr() as *const u8); }
    let off = offset as i32;
    match Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            crate::fs::vfs_lseek(entry.0, &mut entry.1, off, whence)
        } else { -1 }
    }) {
        Some(val) if val >= 0 => val as u32,
        _ => u32::MAX,
    }
}

/// pipe
#[no_mangle]
pub extern "C" fn rust_sys_pipe(pipefd_ptr: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_pipe called\n".as_ptr() as *const u8); }
    match crate::fs::vfs_create_pipe() {
        Ok(vnode_id) => {
            match Scheduler::with_current_task_mut(|task| {
                let fd1 = task.alloc_fd(vnode_id);
                let fd2 = task.alloc_fd(vnode_id);
                (fd1, fd2)
            }) {
                Some((Some(f1), Some(f2))) => {
                    let mut arr = [0u32; 2];
                    arr[0] = f1;
                    arr[1] = f2;
                    unsafe {
                        let bytes = core::slice::from_raw_parts(arr.as_ptr() as *const u8, 8);
                        if crate::utils::copy_to_user(pipefd_ptr, bytes).is_ok() {
                            return 0;
                        }
                    }
                    let _ = Scheduler::with_current_task_mut(|task| {
                        let _ = task.close_fd(f1);
                        let _ = task.close_fd(f2);
                    });
                    u32::MAX
                }
                _ => u32::MAX,
            }
        }
        Err(_) => u32::MAX,
    }
}

/// execve
#[no_mangle]
pub extern "C" fn rust_sys_execve(path_ptr: u32) -> u32 {
    unsafe { ffi::serial_print(c"[Syscall] sys_execve called\n".as_ptr() as *const u8); }
    if path_ptr == 0 { return u32::MAX; }
    let mut buf = [0u8; 256];
    unsafe {
        if crate::utils::copy_from_user(path_ptr, &mut buf).is_err() { return u32::MAX; }
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(256);
    let path = match core::str::from_utf8(&buf[..len]) {
        Ok(s) => s,
        Err(_) => return u32::MAX,
    };
    unsafe { ffi::serial_print(c"[Syscall] Opening file\n".as_ptr() as *const u8); }
    let vnode = match crate::fs::vfs_open(path, 0, 0) {
        Ok(id) => id,
        Err(_) => return u32::MAX,
    };
    unsafe { ffi::serial_print(c"[Syscall] Reading file\n".as_ptr() as *const u8); }
    let image = match crate::fs::vfs_read_all(vnode) {
        Some(v) => v,
        None => return u32::MAX,
    };
    unsafe { ffi::serial_print(c"[Syscall] Creating page directory\n".as_ptr() as *const u8); }
    let pd_phys = unsafe { ffi::paging_create_directory_phys() };
    if pd_phys == 0 { return u32::MAX; }
    unsafe { ffi::serial_print(c"[Syscall] Loading ELF\n".as_ptr() as *const u8); }
    match crate::elf::load_elf_from_bytes(&image) {
        Ok((entry, phdr_vaddr)) => {
            unsafe { ffi::serial_print(c"[Syscall] ELF loaded successfully\n".as_ptr() as *const u8); }
            let switched = unsafe { ffi::paging_switch_to_directory(pd_phys) };
            if !switched { return u32::MAX; }
            unsafe { ffi::serial_print(c"[Syscall] Switched to user directory\n".as_ptr() as *const u8); }

            const STACK_BASE: u32 = 0x00C00000;
            const STACK_SIZE: u32 = 0x4000;

            let stack_flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;
            let mut page_addr = STACK_BASE;
            while page_addr < STACK_BASE + STACK_SIZE {
                let phys = unsafe { ffi::pmm_alloc_frame() };
                if phys.is_null() { return u32::MAX; }
                let ok = unsafe { ffi::vmm_map(page_addr as *mut core::ffi::c_void, phys, stack_flags) };
                if !ok { return u32::MAX; }
                page_addr += 4096;
            }
            let stack_ptr = STACK_BASE;

            let prog_name = path.rsplit('/').next().unwrap_or(path);
            let name_bytes = prog_name.as_bytes();
            let name_len = name_bytes.len() + 1;

            let stack_top = stack_ptr + STACK_SIZE;
            let argv_count = 1u32;
            let argv_ptrs_size = (argv_count + 1) * 4;
            let envp_ptrs_size = 4;
            let auxv_size = 6 * 8;
            let total_size = 4u32 + argv_ptrs_size + (envp_ptrs_size as u32) + (auxv_size as u32) + (name_len as u32);

            let mut block_start = stack_top.wrapping_sub(total_size);
            block_start &= !0xF;

            let argc_addr = block_start;
            let argv_array_addr = argc_addr + 4;
            let envp_array_addr = argv_array_addr + argv_ptrs_size;
            let auxv_addr = envp_array_addr + (envp_ptrs_size as u32);
            let string_addr = auxv_addr + (auxv_size as u32);

            if string_addr + (name_len as u32) > stack_top { return u32::MAX; }

            unsafe {
                for (i, &b) in name_bytes.iter().enumerate() {
                    let byte_addr = string_addr + (i as u32);
                    let bytes_slice = core::slice::from_raw_parts(&b, 1);
                    if crate::utils::copy_to_user(byte_addr, bytes_slice).is_err() { return u32::MAX; }
                }
                let nul_byte: u8 = 0;
                let nul_slice = core::slice::from_raw_parts(&nul_byte, 1);
                if crate::utils::copy_to_user(string_addr + (name_bytes.len() as u32), nul_slice).is_err() { return u32::MAX; }
            }

            let mut ptrs = [0u32; 2];
            ptrs[0] = string_addr as u32;
            ptrs[1] = 0;
            let ptrs_bytes = unsafe { core::slice::from_raw_parts(ptrs.as_ptr() as *const u8, argv_ptrs_size as usize) };
            unsafe {
                if crate::utils::copy_to_user(argv_array_addr, ptrs_bytes).is_err() { return u32::MAX; }
            }

            let zero_ptr: u32 = 0;
            let zero_bytes = unsafe { core::slice::from_raw_parts((&zero_ptr) as *const u32 as *const u8, 4) };
            unsafe {
                if crate::utils::copy_to_user(envp_array_addr, zero_bytes).is_err() { return u32::MAX; }
            }

            let (entry_val, phentsize, phnum) = match crate::elf::parse_elf_header(&image) {
                Some(t) => t,
                None => return u32::MAX,
            };

            const AT_PHDR: u32 = 3;
            const AT_PHENT: u32 = 4;
            const AT_PHNUM: u32 = 5;
            const AT_PAGESZ: u32 = 6;
            const AT_ENTRY: u32 = 9;
            const AT_NULL: u32 = 0;

            let mut aux = [0u32; 12];
            let mut ai = 0;
            aux[ai] = AT_PHDR; ai += 1; aux[ai] = phdr_vaddr as u32; ai += 1;
            aux[ai] = AT_PHENT; ai += 1; aux[ai] = phentsize as u32; ai += 1;
            aux[ai] = AT_PHNUM; ai += 1; aux[ai] = phnum as u32; ai += 1;
            aux[ai] = AT_PAGESZ; ai += 1; aux[ai] = 4096u32; ai += 1;
            aux[ai] = AT_ENTRY; ai += 1; aux[ai] = entry_val as u32; ai += 1;
            aux[ai] = AT_NULL; ai += 1; aux[ai] = 0u32; ai += 1;

            let auxv_bytes = unsafe { core::slice::from_raw_parts(aux.as_ptr() as *const u8, ai * 4) };
            unsafe {
                if crate::utils::copy_to_user(auxv_addr, auxv_bytes).is_err() { return u32::MAX; }
            }

            let argc_val: u32 = 1;
            let argc_bytes = unsafe { core::slice::from_raw_parts((&argc_val) as *const u32 as *const u8, 4) };
            unsafe {
                if crate::utils::copy_to_user(argc_addr, argc_bytes).is_err() { return u32::MAX; }
            }

            let _ = Scheduler::with_current_task_mut(|task| {
                let ctx = task.context_mut();
                #[cfg(feature = "x86_64")]
                {
                    ctx.rip = entry as u64;
                    ctx.rsp = argc_addr as u64;
                    ctx.cr3 = pd_phys as u64;
                    ctx.cs = 0x1B;
                    ctx.ds = 0x23;
                    ctx.es = 0x23;
                    ctx.fs = 0x23;
                    ctx.gs = 0x23;
                    ctx.ss = 0x23;
                }
                #[cfg(feature = "aarch64")]
                {
                    ctx.ttbr0 = pd_phys as u64;
                    ctx.spsr = 0;
                }
            });

            0
        }
        Err(_) => u32::MAX,
    }
}

/// Socket syscall - creates a new socket
#[no_mangle]
pub extern "C" fn rust_sys_socket(domain: i32, socket_type: i32, protocol: i32) -> i32 {
    crate::net::socket_create(domain, socket_type, protocol)
}

/// Bind syscall - binds socket to address
///
/// # Safety
/// `addr` must be a valid pointer to a sockaddr structure of at least `addr_len` bytes.
/// The address family byte at `addr[0..1]` must be readable.
#[no_mangle]
pub unsafe extern "C" fn rust_sys_bind(fd: i32, addr: *const core::ffi::c_void, addr_len: u32) -> i32 {
    if addr.is_null() || addr_len < 2 {
        return -1;
    }
    let path_bytes = unsafe { core::slice::from_raw_parts(addr.add(2) as *const u8, (addr_len - 2) as usize) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    crate::net::socket_bind(fd, path)
}

/// Listen syscall - listens for connections on socket
#[no_mangle]
pub extern "C" fn rust_sys_listen(fd: i32, backlog: i32) -> i32 {
    crate::net::socket_listen(fd, backlog)
}

/// Accept syscall - accepts a connection on a listening socket
#[no_mangle]
pub extern "C" fn rust_sys_accept(fd: i32) -> i32 {
    crate::net::socket_accept(fd)
}

/// Connect syscall - connects socket to an address
///
/// # Safety
/// `addr` must be a valid pointer to a sockaddr structure of at least `addr_len` bytes.
/// The address family byte at `addr[0..1]` must be readable.
#[no_mangle]
pub unsafe extern "C" fn rust_sys_connect(fd: i32, addr: *const core::ffi::c_void, addr_len: u32) -> i32 {
    if addr.is_null() || addr_len < 2 {
        return -1;
    }
    let path_bytes = unsafe { core::slice::from_raw_parts(addr.add(2) as *const u8, (addr_len - 2) as usize) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    crate::net::socket_connect(fd, path)
}

/// Close socket syscall
#[no_mangle]
pub extern "C" fn rust_sys_close_socket(fd: i32) -> i32 {
     crate::net::socket_close(fd)
}

/// Has pending connections syscall
#[no_mangle]
pub extern "C" fn rust_sys_has_pending_connections(fd: i32) -> i32 {
    crate::net::socket_has_pending_connections(fd)
}

/// Socket read syscall — read data from a connected socket
#[no_mangle]
pub extern "C" fn rust_sys_socket_read(fd: i32, buf_ptr: u32, len: u32) -> i32 {
    if buf_ptr == 0 || len == 0 {
        return -1;
    }
    let mut buf = alloc::vec![0u8; len as usize];
    let result = crate::net::socket_read(fd, &mut buf);
    if result < 0 {
        return result as i32;
    }
    let to_copy = core::cmp::min(result as usize, len as usize);
    unsafe {
        if crate::utils::copy_to_user(buf_ptr, &buf[..to_copy]).is_err() {
            return -1;
        }
    }
    to_copy as i32
}

/// Socket write syscall — write data to a connected socket
#[no_mangle]
pub extern "C" fn rust_sys_socket_write(fd: i32, buf_ptr: u32, len: u32) -> i32 {
    if buf_ptr == 0 || len == 0 {
        return -1;
    }
    let max = core::cmp::min(len as usize, 4096usize);
    let mut buf = [0u8; 4096];
    unsafe {
        if crate::utils::copy_from_user(buf_ptr, &mut buf[..max]).is_err() {
            return -1;
        }
    }
    crate::net::socket_write(fd, &buf[..max]) as i32
}

/// sys_clone - Create a new task running `entry(arg)` with `stack`.
/// Returns child PID on success, !0 on error.
#[no_mangle]
pub extern "C" fn rust_sys_clone(entry: u32, stack: u32, arg: u32) -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_clone called\n".as_ptr() as *const u8);
    }
    crate::process::Scheduler::clone_task(entry, stack, arg)
}

/// sys_fork - Create a child process with COW-shared address space.
/// Returns child PID to parent, 0 to child.
#[no_mangle]
pub extern "C" fn rust_sys_fork() -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_fork called\n".as_ptr() as *const u8);
    }
    crate::process::Scheduler::fork_current()
}

/// sys_waitpid - Wait for a child process to exit.
/// Returns (child_pid << 16) | (exit_code & 0xFFFF), or u32::MAX on error.
#[no_mangle]
pub extern "C" fn rust_sys_waitpid(_pid: u32, _options: u32) -> u32 {
    unsafe {
        ffi::serial_print(c"[Syscall] sys_waitpid called\n".as_ptr() as *const u8);
    }
    let (child_pid, exit_code) = Scheduler::wait_for_child();
    if child_pid == u32::MAX {
        return u32::MAX;
    }
    // Pack PID and exit code: higher 16 bits = PID, lower 16 bits = exit code
    ((child_pid & 0xFFFF) << 16) | (exit_code & 0xFFFF)
}

/// sys_brk - Set the program break (heap end) for the current task.
/// If addr is 0, return the current break. Otherwise, try to extend/shrink
/// the heap to the given address. Returns the new program break on success,
/// or !0 on failure.
#[no_mangle]
pub extern "C" fn rust_sys_brk(addr: u32) -> u32 {
    let current_break = crate::process::Scheduler::with_current_task_mut(|task| {
        task.heap_break()
    }).unwrap_or(0);

    if addr == 0 {
        return current_break;
    }

    let page_ceil = |x: u32| (x + 0xFFF) & !0xFFF;

    let old_brk = current_break;
    let new_brk = core::cmp::max(addr, 0x01000000);

    let old_page = page_ceil(old_brk);
    let new_page = page_ceil(new_brk);

    if new_page > old_page {
        let alloc_size = new_page - old_page;
        let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;
        let ptr = unsafe { ffi::vmm_alloc_region(alloc_size as usize, flags) };
        if ptr.is_null() {
            return u32::MAX;
        }
    } else if new_page < old_page {
        let free_start = new_page;
        let free_size = old_page - new_page;
        unsafe { ffi::vmm_free_region(free_start as *mut core::ffi::c_void, free_size as usize); }
    }

    let _ = crate::process::Scheduler::with_current_task_mut(|task| {
        task.set_heap_break(new_brk);
    });

    new_brk
}

/// Invoke a syscall (for testing/internal use) — x86 only
#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn syscall(num: SyscallNumber, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "push rbx",
            "mov ebx, {0:e}",
            "int 0x80",
            "pop rbx",
            in(reg) arg0,
            inlateout("eax") num as u32 => result,
            in("ecx") arg1,
            in("edx") arg2,
            lateout("ecx") _,
            lateout("edx") _,
            options(preserves_flags),
        );
    }
    result
}

/// Convenience wrappers for syscalls (x86 only)
#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn exit(code: u32) -> ! {
    syscall(SyscallNumber::Exit, code, 0, 0);
    loop { unsafe { core::arch::asm!("hlt"); } }
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn yield_cpu() {
    syscall(SyscallNumber::Yield, 0, 0, 0);
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn getpid() -> u32 {
    syscall(SyscallNumber::GetPid, 0, 0, 0)
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sleep(ms: u32) {
    syscall(SyscallNumber::Sleep, ms, 0, 0);
}

/// Socket convenience wrappers (x86 only)
#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sys_socket(domain: i32, socket_type: i32, protocol: i32) -> i32 {
    syscall(SyscallNumber::Socket, domain as u32, socket_type as u32, protocol as u32) as i32
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sys_bind(fd: i32, addr: u32, addr_len: u32) -> i32 {
    syscall(SyscallNumber::Bind, fd as u32, addr, addr_len) as i32
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sys_listen(fd: i32, backlog: i32) -> i32 {
    syscall(SyscallNumber::Listen, fd as u32, backlog as u32, 0) as i32
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sys_accept(fd: i32) -> i32 {
    syscall(SyscallNumber::Accept, fd as u32, 0, 0) as i32
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sys_connect(fd: i32, addr: u32, addr_len: u32) -> i32 {
    syscall(SyscallNumber::Connect, fd as u32, addr, addr_len) as i32
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sys_close_socket(fd: i32) -> i32 {
    syscall(SyscallNumber::CloseSocket, fd as u32, 0, 0) as i32
}

#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn sbrk(incr: i32) -> u32 {
    let current = syscall(SyscallNumber::Brk, 0, 0, 0);
    if incr == 0 {
        return current;
    }
    let new = (current as i32).checked_add(incr).unwrap_or(i32::MAX) as u32;
    syscall(SyscallNumber::Brk, new, 0, 0)
}

/// Allocate shared memory buffer for Wayland SHM
#[no_mangle]
pub extern "C" fn rust_sys_alloc_shm(width: u32, height: u32, bpp: u32) -> u32 {
    let fd = crate::shm_alloc::shm_alloc(width, height, bpp);
    if fd < 0 { u32::MAX } else { fd as u32 }
}

/// Get user virtual address of an SHM buffer
#[no_mangle]
pub extern "C" fn rust_sys_shm_user_vaddr(fd: u32) -> u32 {
    crate::shm_alloc::shm_user_vaddr(fd as i32)
}

/// sys_mmap - Map memory pages for the current task.
/// arg0: hint address (or 0 for auto)
/// arg1: length in bytes
/// arg2: mmap flags (bitmask)
/// Returns: user virtual address on success, 0xFFFFFFFF on failure
///
/// Flags (matching Linux mmap):
///   bit 0 (0x01): MAP_SHARED
///   bit 1 (0x02): MAP_PRIVATE
///   bit 4 (0x10): MAP_ANONYMOUS
#[no_mangle]
pub extern "C" fn rust_sys_mmap(hint: u32, length: u32, flags: u32) -> u32 {
    if length == 0 { return 0xFFFFFFFF; }

    let map_anonymous = (flags & 0x10) != 0;
    let page_ceil = |x: u32| (x + 0xFFF) & !0xFFF;
    let alloc_pages = page_ceil(length);
    let num_pages = (alloc_pages / 4096) as usize;

    if map_anonymous {
        // For anonymous mappings, use the mmap region above the brk region.
        // Find a free virtual address range and map physical frames to it.
        let flags = ffi::PAGE_PRESENT | ffi::PAGE_WRITE | ffi::PAGE_USER;
        let ptr = unsafe { ffi::vmm_alloc_region(alloc_pages as usize, flags) };
        if ptr.is_null() {
            return 0xFFFFFFFF;
        }
        return ptr as u32;
    }

    // For non-anonymous mmap, return failure for now
    0xFFFFFFFF
}

/// sys_gettimeofday - Get current time.
/// arg0: pointer to user timeval struct { tv_sec, tv_usec }
/// Returns: 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn rust_sys_gettimeofday(timeval_ptr: u32) -> u32 {
    if timeval_ptr == 0 { return 0xFFFFFFFF; }

    let uptime_ms = unsafe { crate::ffi::timer_get_uptime_ms_ffi() };
    let tv_sec = (uptime_ms / 1000) as u32;
    let tv_usec = ((uptime_ms % 1000) * 1000) as u32;

    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&tv_sec.to_ne_bytes());
    buf[4..8].copy_from_slice(&tv_usec.to_ne_bytes());

    unsafe {
        if crate::utils::copy_to_user(timeval_ptr, &buf).is_err() {
            return 0xFFFFFFFF;
        }
    }
    0
}

/// Dispatcher wrapper callable from C/C++: routes raw registers to Rust dispatcher
#[no_mangle]
pub extern "C" fn rust_dispatcher(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    dispatcher::dispatch_syscall(eax, ebx, ecx, edx)
}