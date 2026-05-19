// Syscall dispatcher skeleton

use crate::syscall::table;

/// Dispatch a syscall given registers or raw values.
///
/// eax: syscall number
/// ebx, ecx, edx: arguments (convention used by existing mod.rs)
pub fn dispatch_syscall(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    match table::SyscallNumber::from_u32(eax) {
        Some(table::SyscallNumber::Exit) => unsafe { crate::syscall::rust_sys_exit(ebx) },
        Some(table::SyscallNumber::Yield) => unsafe { crate::syscall::rust_sys_yield() },
        Some(table::SyscallNumber::GetPid) => unsafe { crate::syscall::rust_sys_getpid() },
        Some(table::SyscallNumber::Sleep) => unsafe { crate::syscall::rust_sys_sleep(ebx) },
        Some(table::SyscallNumber::Open) => unsafe { crate::syscall::rust_sys_open(ebx, ecx, edx) },
        Some(table::SyscallNumber::Read) => unsafe { crate::syscall::rust_sys_read(ebx, ecx, edx) },
        Some(table::SyscallNumber::Write) => unsafe { crate::syscall::rust_sys_write(ebx, ecx, edx) },
        Some(table::SyscallNumber::Close) => unsafe { crate::syscall::rust_sys_close(ebx) },
        Some(table::SyscallNumber::Dup) => unsafe { crate::syscall::rust_sys_dup(ebx) },
        Some(table::SyscallNumber::Lseek) => unsafe { crate::syscall::rust_sys_lseek(ebx, ecx, edx) },
        Some(table::SyscallNumber::Pipe) => unsafe { crate::syscall::rust_sys_pipe(ebx) },
        Some(table::SyscallNumber::Execve) => unsafe { crate::syscall::rust_sys_execve(ebx) },
Some(table::SyscallNumber::Socket) => unsafe { crate::syscall::rust_sys_socket(ebx as i32, ecx as i32, edx as i32) as u32 },
         Some(table::SyscallNumber::Bind) => unsafe { crate::syscall::rust_sys_bind(ebx as i32, ecx as *const core::ffi::c_void, edx as u32) as u32 },
         Some(table::SyscallNumber::Listen) => unsafe { crate::syscall::rust_sys_listen(ebx as i32, ecx as i32) as u32 },
         Some(table::SyscallNumber::Accept) => unsafe { crate::syscall::rust_sys_accept(ebx as i32) as u32 },
         Some(table::SyscallNumber::Connect) => unsafe { crate::syscall::rust_sys_connect(ebx as i32, ecx as *const core::ffi::c_void, edx as u32) as u32 },
         Some(table::SyscallNumber::CloseSocket) => unsafe { crate::syscall::rust_sys_close_socket(ebx as i32) as u32 },
         Some(table::SyscallNumber::HasPendingConnections) => unsafe { crate::syscall::rust_sys_has_pending_connections(ebx as i32) as u32 },
        None => {
            // Unknown syscall number - return sentinel error code
            core::u32::MAX
        }
    }
}