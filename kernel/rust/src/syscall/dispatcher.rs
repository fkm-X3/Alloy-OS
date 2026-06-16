// Syscall dispatcher skeleton

use crate::syscall::table;

/// Dispatch a syscall given registers or raw values.
///
/// eax: syscall number
/// ebx, ecx, edx: arguments (convention used by existing mod.rs)
pub fn dispatch_syscall(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    match table::SyscallNumber::from_u32(eax) {
        Some(table::SyscallNumber::Exit) => crate::syscall::rust_sys_exit(ebx),
        Some(table::SyscallNumber::Yield) => crate::syscall::rust_sys_yield(),
        Some(table::SyscallNumber::GetPid) => crate::syscall::rust_sys_getpid(),
        Some(table::SyscallNumber::Sleep) => crate::syscall::rust_sys_sleep(ebx),
        Some(table::SyscallNumber::Open) => crate::syscall::rust_sys_open(ebx, ecx, edx),
        Some(table::SyscallNumber::Read) => crate::syscall::rust_sys_read(ebx, ecx, edx),
        Some(table::SyscallNumber::Write) => crate::syscall::rust_sys_write(ebx, ecx, edx),
        Some(table::SyscallNumber::Close) => crate::syscall::rust_sys_close(ebx),
        Some(table::SyscallNumber::Dup) => crate::syscall::rust_sys_dup(ebx),
        Some(table::SyscallNumber::Lseek) => crate::syscall::rust_sys_lseek(ebx, ecx, edx),
        Some(table::SyscallNumber::Pipe) => crate::syscall::rust_sys_pipe(ebx),
        Some(table::SyscallNumber::Execve) => crate::syscall::rust_sys_execve(ebx),
        Some(table::SyscallNumber::Socket) => crate::syscall::rust_sys_socket(ebx as i32, ecx as i32, edx as i32) as u32,
        Some(table::SyscallNumber::Bind) => unsafe { crate::syscall::rust_sys_bind(ebx as i32, ecx as *const core::ffi::c_void, edx) as u32 },
        Some(table::SyscallNumber::Listen) => crate::syscall::rust_sys_listen(ebx as i32, ecx as i32) as u32,
        Some(table::SyscallNumber::Accept) => crate::syscall::rust_sys_accept(ebx as i32) as u32,
        Some(table::SyscallNumber::Connect) => unsafe { crate::syscall::rust_sys_connect(ebx as i32, ecx as *const core::ffi::c_void, edx) as u32 },
        Some(table::SyscallNumber::CloseSocket) => crate::syscall::rust_sys_close_socket(ebx as i32) as u32,
        Some(table::SyscallNumber::HasPendingConnections) => crate::syscall::rust_sys_has_pending_connections(ebx as i32) as u32,
        Some(table::SyscallNumber::SocketRead) => crate::syscall::rust_sys_socket_read(ebx as i32, ecx, edx) as u32,
        Some(table::SyscallNumber::SocketWrite) => crate::syscall::rust_sys_socket_write(ebx as i32, ecx, edx) as u32,
        Some(table::SyscallNumber::Brk) => crate::syscall::rust_sys_brk(ebx),
        Some(table::SyscallNumber::Fork) => crate::syscall::rust_sys_fork(),
        Some(table::SyscallNumber::Clone) => crate::syscall::rust_sys_clone(ebx, ecx, edx),
        Some(table::SyscallNumber::WaitPid) => crate::syscall::rust_sys_waitpid(ebx, ecx),
        Some(table::SyscallNumber::AllocShm) => crate::syscall::rust_sys_alloc_shm(ebx, ecx, edx),
        Some(table::SyscallNumber::ShmUserVaddr) => crate::syscall::rust_sys_shm_user_vaddr(ebx),
        None => {
            u32::MAX
        }
    }
}