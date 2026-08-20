//! System call interface for Alloy OS
//!
//! Provides syscall handlers that can be invoked via INT 0x80

use crate::process::Scheduler;
use alloy_kernel_hal::mem::{self, AddressSpace, PageFlags, PhysFrame};

pub mod dispatcher;
pub mod table;

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
pub extern "C" fn rust_sys_exit(code: u32) -> u32 {
    crate::println!("[Syscall] sys_exit called with code {code}");

    Scheduler::terminate_current(code);
    code
}

/// sys_yield - Voluntarily give up the CPU
pub extern "C" fn rust_sys_yield() -> u32 {
    crate::println!("[Syscall] sys_yield called");
    Scheduler::yield_cpu();
    0
}

/// sys_getpid - Get the current task ID
pub extern "C" fn rust_sys_getpid() -> u32 {
    crate::println!("[Syscall] sys_getpid called");
    Scheduler::with_current_task(|task| task.id().as_u32()).unwrap_or(1)
}

/// sys_sleep - Sleep for specified milliseconds
pub extern "C" fn rust_sys_sleep(ms: u32) -> u32 {
    crate::println!("[Syscall] sys_sleep called");
    let start = crate::SystemTimer::uptime_ms();
    let target = start + ms as u64;

    while crate::SystemTimer::uptime_ms() < target {
        Scheduler::yield_cpu();
    }
    0
}

/// Minimal open syscall
pub extern "C" fn rust_sys_open(path_ptr: u32, flags: u32, mode: u32) -> u32 {
    crate::println!("[Syscall] sys_open called");
    if path_ptr == 0 {
        return u32::MAX;
    }
    let mut buffer = [0u8; 256];
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
    u32::MAX
}

/// Minimal read syscall
pub extern "C" fn rust_sys_read(fd: u32, buf_ptr: u32, len: u32) -> u32 {
    crate::println!("[Syscall] sys_read called");
    if let Some(result) = Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            let vnode_id = entry.0;
            let offset = &mut entry.1;
            crate::fs::vfs_read(vnode_id, offset, buf_ptr, len as usize)
        } else {
            -1
        }
    }) {
        if result >= 0 {
            return result as u32;
        }
    }
    u32::MAX
}

/// Minimal write syscall
pub extern "C" fn rust_sys_write(fd: u32, buf_ptr: u32, len: u32) -> u32 {
    crate::println!("[Syscall] sys_write fd={fd:08X} buf=0x{buf_ptr:016X} len={len:08X}");
    if fd == 1 {
        let max = core::cmp::min(len as usize, 240usize);
        let mut buffer = [0u8; 241];
        if buf_ptr == 0 {
            return u32::MAX;
        }
        match crate::utils::copy_from_user(buf_ptr, &mut buffer[..max]) {
            Ok(copied) => {
                crate::Serial::write_bytes(&buffer[..copied]);
                return copied as u32;
            }
            Err(_) => return u32::MAX,
        }
    }
    if let Some(result) = Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            let vnode_id = entry.0;
            let offset = &mut entry.1;
            crate::fs::vfs_write(vnode_id, offset, buf_ptr, len as usize)
        } else {
            -1
        }
    }) {
        if result >= 0 {
            return result as u32;
        }
    }
    u32::MAX
}

/// Minimal close syscall
pub extern "C" fn rust_sys_close(fd: u32) -> u32 {
    crate::println!("[Syscall] sys_close called");
    match Scheduler::with_current_task_mut(|task| task.close_fd(fd)) {
        Some(Ok(())) => 0,
        Some(Err(_)) => u32::MAX,
        None => u32::MAX,
    }
}

/// Duplicate a file descriptor
pub extern "C" fn rust_sys_dup(oldfd: u32) -> u32 {
    crate::println!("[Syscall] sys_dup called");
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
        Some(fd_opt) => fd_opt.unwrap_or(u32::MAX),
        None => u32::MAX,
    }
}

/// dup2 - Duplicate a file descriptor to a specific target fd.
/// If newfd is already open, it is closed first.
pub extern "C" fn rust_sys_dup2(oldfd: u32, newfd: u32) -> u32 {
    crate::println!("[Syscall] sys_dup2 called");
    if oldfd == newfd {
        return newfd;
    }
    match Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(oldfd) {
            let vnode = entry.0;
            let offset = entry.1;
            let _ = task.close_fd(newfd);
            task.set_fd_at(newfd, vnode, offset);
            return Some(newfd);
        }
        None
    }) {
        Some(fd_opt) => fd_opt.unwrap_or(u32::MAX),
        None => u32::MAX,
    }
}

/// lseek
pub extern "C" fn rust_sys_lseek(fd: u32, offset: u32, whence: u32) -> u32 {
    crate::println!("[Syscall] sys_lseek called");
    let off = offset as i32;
    match Scheduler::with_current_task_mut(|task| {
        if let Some(entry) = task.get_fd_entry_mut(fd) {
            crate::fs::vfs_lseek(entry.0, &mut entry.1, off, whence)
        } else {
            -1
        }
    }) {
        Some(val) if val >= 0 => val as u32,
        _ => u32::MAX,
    }
}

/// pipe
pub extern "C" fn rust_sys_pipe(pipefd_ptr: u32) -> u32 {
    crate::println!("[Syscall] sys_pipe called");
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
                    let bytes = crate::utils::as_byte_slice_of(&arr);
                    if crate::utils::copy_to_user(pipefd_ptr, bytes).is_ok() {
                        return 0;
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
pub extern "C" fn rust_sys_execve(path_ptr: u32) -> u32 {
    crate::println!("[Syscall] sys_execve called");
    if path_ptr == 0 {
        return u32::MAX;
    }
    let mut buf = [0u8; 256];
    if crate::utils::copy_from_user(path_ptr, &mut buf).is_err() {
        return u32::MAX;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(256);
    let path = match core::str::from_utf8(&buf[..len]) {
        Ok(s) => s,
        Err(_) => return u32::MAX,
    };
    crate::println!("[Syscall] Opening file");
    let vnode = match crate::fs::vfs_open(path, 0, 0) {
        Ok(id) => id,
        Err(_) => return u32::MAX,
    };
    crate::println!("[Syscall] Reading file");
    let image = match crate::fs::vfs_read_all(vnode) {
        Some(v) => v,
        None => return u32::MAX,
    };
    crate::println!("[Syscall] Creating page directory");
    let Some(aspace) = AddressSpace::create() else {
        return u32::MAX;
    };
    crate::println!("[Syscall] Loading ELF");
    match crate::elf::load_elf_from_bytes(&image) {
        Ok((entry, phdr_vaddr)) => {
            crate::println!("[Syscall] ELF loaded successfully");
            if !aspace.switch() {
                return u32::MAX;
            }
            aspace.set_current_user();
            crate::println!("[Syscall] Switched to user directory");

            const STACK_BASE: u32 = 0x00C00000;
            const STACK_SIZE: u32 = 0x4000;

            let stack_flags = PageFlags::user_write();
            let mut page_addr = STACK_BASE;
            while page_addr < STACK_BASE + STACK_SIZE {
                let Some(frame) = PhysFrame::alloc() else {
                    return u32::MAX;
                };
                if !mem::map_frame(page_addr as usize, frame, stack_flags) {
                    return u32::MAX;
                }
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
            let total_size = 4u32
                + argv_ptrs_size
                + (envp_ptrs_size as u32)
                + (auxv_size as u32)
                + (name_len as u32);

            let mut block_start = stack_top.wrapping_sub(total_size);
            block_start &= !0xF;

            let argc_addr = block_start;
            let argv_array_addr = argc_addr + 4;
            let envp_array_addr = argv_array_addr + argv_ptrs_size;
            let auxv_addr = envp_array_addr + (envp_ptrs_size as u32);
            let string_addr = auxv_addr + (auxv_size as u32);

            if string_addr + (name_len as u32) > stack_top {
                return u32::MAX;
            }

            for (i, &b) in name_bytes.iter().enumerate() {
                let byte_addr = string_addr + (i as u32);
                let bytes_slice = core::slice::from_ref(&b);
                if crate::utils::copy_to_user(byte_addr, bytes_slice).is_err() {
                    return u32::MAX;
                }
            }
            let nul_byte: u8 = 0;
            let nul_slice = core::slice::from_ref(&nul_byte);
            if crate::utils::copy_to_user(string_addr + (name_bytes.len() as u32), nul_slice)
                .is_err()
            {
                return u32::MAX;
            }

            let mut ptrs = [0u32; 2];
            ptrs[0] = string_addr as u32;
            ptrs[1] = 0;
            let ptrs_bytes = crate::utils::as_byte_slice_of(&ptrs[..argv_ptrs_size as usize / 4]);
            if crate::utils::copy_to_user(argv_array_addr, ptrs_bytes).is_err() {
                return u32::MAX;
            }

            let zero_ptr: u32 = 0;
            let zero_bytes = crate::utils::as_byte_slice(&zero_ptr);
            if crate::utils::copy_to_user(envp_array_addr, zero_bytes).is_err() {
                return u32::MAX;
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
            aux[ai] = AT_PHDR;
            ai += 1;
            aux[ai] = phdr_vaddr as u32;
            ai += 1;
            aux[ai] = AT_PHENT;
            ai += 1;
            aux[ai] = phentsize as u32;
            ai += 1;
            aux[ai] = AT_PHNUM;
            ai += 1;
            aux[ai] = phnum as u32;
            ai += 1;
            aux[ai] = AT_PAGESZ;
            ai += 1;
            aux[ai] = 4096u32;
            ai += 1;
            aux[ai] = AT_ENTRY;
            ai += 1;
            aux[ai] = entry_val as u32;
            ai += 1;
            aux[ai] = AT_NULL;
            ai += 1;
            aux[ai] = 0u32;
            ai += 1;

            let auxv_bytes = crate::utils::as_byte_slice_of(&aux[..ai]);
            if crate::utils::copy_to_user(auxv_addr, auxv_bytes).is_err() {
                return u32::MAX;
            }

            let argc_val: u32 = 1;
            let argc_bytes = crate::utils::as_byte_slice(&argc_val);
            if crate::utils::copy_to_user(argc_addr, argc_bytes).is_err() {
                return u32::MAX;
            }

            let aspace_addr = aspace.addr();
            let _ = Scheduler::with_current_task_mut(|task| {
                let ctx = task.context_mut();
                #[cfg(feature = "x86_64")]
                {
                    ctx.rip = entry as u64;
                    ctx.rsp = argc_addr as u64;
                    ctx.cr3 = aspace_addr as u64;
                    ctx.cs = 0x23;
                    ctx.ds = 0x1B;
                    ctx.es = 0x1B;
                    ctx.fs = 0x1B;
                    ctx.gs = 0x1B;
                    ctx.ss = 0x1B;
                }
                #[cfg(feature = "aarch64")]
                {
                    ctx.ttbr0 = aspace_addr as u64;
                    ctx.spsr = 0;
                }
                task.set_address_space(aspace);
            });

            0
        }
        Err(_) => u32::MAX,
    }
}

/// Socket syscall - creates a new socket
pub extern "C" fn rust_sys_socket(domain: i32, socket_type: i32, protocol: i32) -> i32 {
    crate::net::socket_create(domain, socket_type, protocol)
}

/// Core bind logic — takes raw sockaddr bytes (already in kernel memory).
pub fn sys_bind_inner(fd: i32, sockaddr: &[u8], addr_len: u32) -> i32 {
    if addr_len < 2 {
        return -1;
    }
    let path_len = (addr_len - 2) as usize;
    let max_path = if path_len > 256 { 256 } else { path_len };
    let path_end = core::cmp::min(2 + max_path, sockaddr.len());
    if path_end <= 2 {
        return -1;
    }
    let path = match core::str::from_utf8(&sockaddr[2..path_end]) {
        Ok(s) => s.trim_end_matches('\0'),
        Err(_) => return -1,
    };
    crate::net::socket_bind(fd, path)
}

/// Bind syscall - binds socket to address (called from syscall dispatch with user pointer)
pub extern "C" fn rust_sys_bind(
    fd: i32,
    addr: u32,
    addr_len: u32,
) -> i32 {
    if addr == 0 || addr_len < 2 {
        return -1;
    }
    let mut buf = [0u8; 110];
    let copy_len = core::cmp::min(addr_len as usize, buf.len());
    if crate::utils::copy_from_user(addr, &mut buf[..copy_len]).is_err() {
        return -1;
    }
    sys_bind_inner(fd, &buf[..copy_len], addr_len)
}

/// Listen syscall - listens for connections on socket
pub extern "C" fn rust_sys_listen(fd: i32, backlog: i32) -> i32 {
    crate::net::socket_listen(fd, backlog)
}

/// Accept syscall - accepts a connection on a listening socket
pub extern "C" fn rust_sys_accept(fd: i32) -> i32 {
    crate::net::socket_accept(fd)
}

/// Core connect logic — takes raw sockaddr bytes (already in kernel memory).
pub fn sys_connect_inner(fd: i32, sockaddr: &[u8], addr_len: u32) -> i32 {
    if addr_len < 2 {
        return -1;
    }
    let path_len = (addr_len - 2) as usize;
    let max_path = if path_len > 256 { 256 } else { path_len };
    let path_end = core::cmp::min(2 + max_path, sockaddr.len());
    if path_end <= 2 {
        return -1;
    }
    let path = match core::str::from_utf8(&sockaddr[2..path_end]) {
        Ok(s) => s.trim_end_matches('\0'),
        Err(_) => return -1,
    };
    crate::net::socket_connect(fd, path)
}

/// Connect syscall - connects socket to an address (called from syscall dispatch with user pointer)
pub extern "C" fn rust_sys_connect(
    fd: i32,
    addr: u32,
    addr_len: u32,
) -> i32 {
    if addr == 0 || addr_len < 2 {
        return -1;
    }
    let mut buf = [0u8; 110];
    let copy_len = core::cmp::min(addr_len as usize, buf.len());
    if crate::utils::copy_from_user(addr, &mut buf[..copy_len]).is_err() {
        return -1;
    }
    sys_connect_inner(fd, &buf[..copy_len], addr_len)
}

/// Close socket syscall
pub extern "C" fn rust_sys_close_socket(fd: i32) -> i32 {
    crate::net::socket_close(fd)
}

/// Has pending connections syscall
pub extern "C" fn rust_sys_has_pending_connections(fd: i32) -> i32 {
    crate::net::socket_has_pending_connections(fd)
}

/// Socket read syscall — read data from a connected socket
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
    if crate::utils::copy_to_user(buf_ptr, &buf[..to_copy]).is_err() {
        return -1;
    }
    to_copy as i32
}

/// Socket write syscall — write data to a connected socket
pub extern "C" fn rust_sys_socket_write(fd: i32, buf_ptr: u32, len: u32) -> i32 {
    if buf_ptr == 0 || len == 0 {
        return -1;
    }
    let max = core::cmp::min(len as usize, 4096usize);
    let mut buf = [0u8; 4096];
    if crate::utils::copy_from_user(buf_ptr, &mut buf[..max]).is_err() {
        return -1;
    }
    crate::net::socket_write(fd, &buf[..max]) as i32
}

/// sys_clone - Create a new task running `entry(arg)` with `stack`.
/// Returns child PID on success, !0 on error.
pub extern "C" fn rust_sys_clone(entry: u32, stack: u32, arg: u32) -> u32 {
    crate::println!("[Syscall] sys_clone called");
    crate::process::Scheduler::clone_task(entry, stack, arg)
}

/// sys_fork - Create a child process with COW-shared address space.
/// Returns child PID to parent, 0 to child.
pub extern "C" fn rust_sys_fork() -> u32 {
    crate::println!("[Syscall] sys_fork called");
    crate::process::Scheduler::fork_current()
}

/// sys_waitpid - Wait for a child process to exit.
/// Returns (child_pid << 16) | (exit_code & 0xFFFF), or u32::MAX on error.
pub extern "C" fn rust_sys_waitpid(_pid: u32, _options: u32) -> u32 {
    crate::println!("[Syscall] sys_waitpid called");
    let (child_pid, exit_code) = Scheduler::wait_for_child();
    if child_pid == u32::MAX {
        return u32::MAX;
    }
    // Pack PID and exit code: higher 16 bits = PID, lower 16 bits = exit code
    ((child_pid & 0xFFFF) << 16) | (exit_code & 0xFFFF)
}

/// sys_kill - Send a signal to a process.
/// For now, only supports SIGTERM (signal 15) which terminates the process.
pub extern "C" fn rust_sys_kill(pid: u32, sig: u32) -> u32 {
    crate::println!("[Syscall] sys_kill called");
    if sig == 15 || sig == 9 {
        Scheduler::terminate_pid(pid, sig)
    } else {
        u32::MAX
    }
}

/// sys_brk - Set the program break (heap end) for the current task.
/// If addr is 0, return the current break. Otherwise, try to extend/shrink
/// the heap to the given address. Returns the new program break on success,
/// or !0 on failure.
pub extern "C" fn rust_sys_brk(addr: u32) -> u32 {
    let current_break =
        crate::process::Scheduler::with_current_task_mut(|task| task.heap_break()).unwrap_or(0);

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
        let ptr = alloy_kernel_hal::mem::VmRegion::alloc(
            alloc_size as usize,
            alloy_kernel_hal::PageFlags::user_write(),
        )
        .map(|r| r.leak())
        .unwrap_or(0);
        if ptr == 0 {
            return u32::MAX;
        }
    } else if new_page < old_page {
        let free_start = new_page;
        let free_size = old_page - new_page;
        alloy_kernel_hal::mem::free_region(free_start as usize, free_size as usize);
    }

    let _ = crate::process::Scheduler::with_current_task_mut(|task| {
        task.set_heap_break(new_brk);
    });

    new_brk
}

/// Safe delegate to the unsafe-core `syscall_invoke` (x86 `int 0x80`).
#[cfg(feature = "x86_64")]
fn syscall(num: SyscallNumber, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    alloy_kernel_hal::syscall_invoke(num as u32, arg0, arg1, arg2)
}

/// Convenience wrappers for syscalls (x86 only)
#[cfg(feature = "x86_64")]
#[allow(dead_code)]
pub fn exit(code: u32) -> ! {
    syscall(SyscallNumber::Exit, code, 0, 0);
    loop {
        alloy_kernel_hal::cpu_halt();
    }
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
    syscall(
        SyscallNumber::Socket,
        domain as u32,
        socket_type as u32,
        protocol as u32,
    ) as i32
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
pub extern "C" fn rust_sys_alloc_shm(width: u32, height: u32, bpp: u32) -> u32 {
    let fd = crate::shm_alloc::shm_alloc(width, height, bpp);
    if fd < 0 {
        u32::MAX
    } else {
        fd as u32
    }
}

/// Get user virtual address of an SHM buffer
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
pub extern "C" fn rust_sys_mmap(hint: u32, length: u32, flags: u32) -> u32 {
    if length == 0 {
        return 0xFFFFFFFF;
    }

    let map_anonymous = (flags & 0x10) != 0;
    let page_ceil = |x: u32| (x + 0xFFF) & !0xFFF;
    let alloc_pages = page_ceil(length);
    let num_pages = (alloc_pages / 4096) as usize;

    if map_anonymous {
        // For anonymous mappings, use the mmap region above the brk region.
        // Find a free virtual address range and map physical frames to it.
        let ptr = alloy_kernel_hal::mem::VmRegion::alloc(
            alloc_pages as usize,
            alloy_kernel_hal::PageFlags::user_write(),
        )
        .map(|r| r.leak())
        .unwrap_or(0);
        if ptr == 0 {
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
pub extern "C" fn rust_sys_gettimeofday(timeval_ptr: u32) -> u32 {
    if timeval_ptr == 0 {
        return 0xFFFFFFFF;
    }

    let uptime_ms = crate::SystemTimer::uptime_ms();
    let tv_sec = (uptime_ms / 1000) as u32;
    let tv_usec = ((uptime_ms % 1000) * 1000) as u32;

    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&tv_sec.to_ne_bytes());
    buf[4..8].copy_from_slice(&tv_usec.to_ne_bytes());

    if crate::utils::copy_to_user(timeval_ptr, &buf).is_err() {
        return 0xFFFFFFFF;
    }
    0
}

/// Register every syscall handler with the HAL callback table.
///
/// Called once from `rust_main` before any userland exists. The ported
/// `syscall_dispatcher` in `unsafe-core` invokes these through the table;
/// it no longer calls `rust_sys_*` by symbol.
///
/// Only the numbers the translated C dispatcher routed are registered, so
/// userland-visible behavior is byte-identical (19, 25–28 still fall to the
/// dispatcher's "unknown syscall" path). Registering the remaining handlers
/// lands with their subsystems in a later code.
pub fn register_all() {
    use alloy_kernel_hal::{SyscallHandler, SyscallTable};

    let reg = |no: u32, handler: SyscallHandler| {
        assert!(
            SyscallTable::register(no, handler),
            "syscall number {no} out of range"
        );
    };

    reg(0, |a0, _, _, _, _| rust_sys_exit(a0));
    reg(1, |_, _, _, _, _| rust_sys_yield());
    reg(2, |_, _, _, _, _| rust_sys_getpid());
    reg(3, |a0, _, _, _, _| rust_sys_sleep(a0));
    reg(4, |a0, a1, a2, _, _| rust_sys_open(a0, a1, a2));
    reg(5, |a0, a1, a2, _, _| rust_sys_read(a0, a1, a2));
    reg(6, |a0, a1, a2, _, _| rust_sys_write(a0, a1, a2));
    reg(7, |a0, _, _, _, _| rust_sys_close(a0));
    reg(8, |a0, _, _, _, _| rust_sys_dup(a0));
    reg(9, |a0, a1, a2, _, _| rust_sys_lseek(a0, a1, a2));
    reg(10, |a0, _, _, _, _| rust_sys_pipe(a0));
    reg(11, |a0, _, _, _, _| rust_sys_execve(a0));
    reg(12, |a0, a1, a2, _, _| {
        rust_sys_socket(a0 as i32, a1 as i32, a2 as i32) as u32
    });
    reg(13, |a0, a1, a2, _, _| {
        rust_sys_bind(a0 as i32, a1, a2) as u32
    });
    reg(14, |a0, a1, _, _, _| {
        rust_sys_listen(a0 as i32, a1 as i32) as u32
    });
    reg(15, |a0, _, _, _, _| rust_sys_accept(a0 as i32) as u32);
    reg(16, |a0, a1, a2, _, _| {
        rust_sys_connect(a0 as i32, a1, a2) as u32
    });
    reg(17, |a0, _, _, _, _| rust_sys_close_socket(a0 as i32) as u32);
    reg(18, |a0, _, _, _, _| {
        rust_sys_has_pending_connections(a0 as i32) as u32
    });
    reg(20, |_, _, _, _, _| rust_sys_fork());
    reg(21, |a0, a1, a2, _, _| rust_sys_clone(a0, a1, a2));
    reg(22, |a0, a1, _, _, _| rust_sys_waitpid(a0, a1));
    reg(23, |a0, a1, a2, _, _| {
        rust_sys_socket_read(a0 as i32, a1, a2) as u32
    });
    reg(24, |a0, a1, a2, _, _| {
        rust_sys_socket_write(a0 as i32, a1, a2) as u32
    });
    reg(29, |a0, a1, _, _, _| rust_sys_dup2(a0, a1));
    reg(30, |a0, a1, _, _, _| rust_sys_kill(a0, a1));

    // Register the syscall dispatcher trampoline so unsafe-core's boot main
    // can route C/asm `rust_dispatcher(eax,ebx,ecx,edx)` calls here.
    alloy_kernel_hal::set_syscall_dispatcher(dispatcher::dispatch_syscall);
}
