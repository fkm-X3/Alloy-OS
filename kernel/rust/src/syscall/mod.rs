/// System call interface for Alloy OS
/// 
/// Provides syscall handlers that can be invoked via INT 0x80

use crate::ffi;
use crate::process::Scheduler;

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
    
    // TODO: Actually terminate the task
    // For now, just yield to scheduler
    Scheduler::yield_cpu();
    
    code // Return code
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
