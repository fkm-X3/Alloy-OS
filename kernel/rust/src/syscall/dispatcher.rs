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
        None => {
            // Unknown syscall number - for now, return a sentinel error code (e.g., u32::MAX)
            core::u32::MAX
        }
    }
}
