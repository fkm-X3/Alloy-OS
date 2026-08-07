use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
    fn rust_sys_exit(code: uint32_t) -> uint32_t;
    fn rust_sys_fork() -> uint32_t;
    fn rust_sys_yield() -> uint32_t;
    fn rust_sys_getpid() -> uint32_t;
    fn rust_sys_sleep(ms: uint32_t) -> uint32_t;
    fn rust_sys_open(path_ptr: uint32_t, flags: uint32_t, mode: uint32_t) -> uint32_t;
    fn rust_sys_read(fd: uint32_t, buf_ptr: uint32_t, len: uint32_t) -> uint32_t;
    fn rust_sys_write(fd: uint32_t, buf_ptr: uint32_t, len: uint32_t) -> uint32_t;
    fn rust_sys_close(fd: uint32_t) -> uint32_t;
    fn rust_sys_dup(oldfd: uint32_t) -> uint32_t;
    fn rust_sys_lseek(fd: uint32_t, offset: uint32_t, whence: uint32_t) -> uint32_t;
    fn rust_sys_pipe(pipefd_ptr: uint32_t) -> uint32_t;
    fn rust_sys_execve(path_ptr: uint32_t) -> uint32_t;
    fn rust_sys_socket(domain: int32_t, socket_type: int32_t, protocol: int32_t) -> int32_t;
    fn rust_sys_bind(fd: int32_t, addr: *const ::core::ffi::c_void, addr_len: uint32_t) -> int32_t;
    fn rust_sys_listen(fd: int32_t, backlog: int32_t) -> int32_t;
    fn rust_sys_accept(fd: int32_t) -> int32_t;
    fn rust_sys_connect(
        fd: int32_t,
        addr: *const ::core::ffi::c_void,
        addr_len: uint32_t,
    ) -> int32_t;
    fn rust_sys_close_socket(fd: int32_t) -> int32_t;
    fn rust_sys_clone(entry: uint32_t, stack: uint32_t, arg: uint32_t) -> uint32_t;
    fn rust_sys_waitpid(pid: uint32_t, options: uint32_t) -> uint32_t;
    fn rust_sys_has_pending_connections(fd: int32_t) -> int32_t;
    fn rust_sys_socket_read(fd: int32_t, buf_ptr: uint32_t, len: uint32_t) -> int32_t;
    fn rust_sys_socket_write(fd: int32_t, buf_ptr: uint32_t, len: uint32_t) -> int32_t;
    fn rust_sys_dup2(oldfd: uint32_t, newfd: uint32_t) -> uint32_t;
    fn rust_sys_kill(pid: uint32_t, sig: uint32_t) -> uint32_t;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    fn serial_print_hex64(value: uint64_t);
    fn syscall_entry();
    static mut kernel_stack_top_alias: uint64_t;
    static mut kernel_stack_top: uint64_t;
}
pub type uint32_t = u32;
pub type int32_t = i32;
#[cfg(target_arch = "x86_64")]
pub type uint64_t = u64;
#[cfg(target_arch = "x86_64")]
pub type uintptr_t = usize;
pub const SYS_EXIT: uint32_t = 0 as uint32_t;
pub const SYS_YIELD: uint32_t = 1 as uint32_t;
pub const SYS_GETPID: uint32_t = 2 as uint32_t;
pub const SYS_SLEEP: uint32_t = 3 as uint32_t;
pub const SYS_OPEN: uint32_t = 4 as uint32_t;
pub const SYS_READ: uint32_t = 5 as uint32_t;
pub const SYS_WRITE: uint32_t = 6 as uint32_t;
pub const SYS_CLOSE: uint32_t = 7 as uint32_t;
pub const SYS_DUP: uint32_t = 8 as uint32_t;
pub const SYS_LSEEK: uint32_t = 9 as uint32_t;
pub const SYS_PIPE: uint32_t = 10 as uint32_t;
pub const SYS_EXECVE: uint32_t = 11 as uint32_t;
pub const SYS_SOCKET: uint32_t = 12 as uint32_t;
pub const SYS_BIND: uint32_t = 13 as uint32_t;
pub const SYS_LISTEN: uint32_t = 14 as uint32_t;
pub const SYS_ACCEPT: uint32_t = 15 as uint32_t;
pub const SYS_CONNECT: uint32_t = 16 as uint32_t;
pub const SYS_CLOSE_SOCKET: uint32_t = 17 as uint32_t;
pub const SYS_HAS_PENDING_CONNECTIONS: uint32_t = 18 as uint32_t;
pub const SYS_FORK: uint32_t = 20 as uint32_t;
pub const SYS_CLONE: uint32_t = 21 as uint32_t;
pub const SYS_WAITPID: uint32_t = 22 as uint32_t;
pub const SYS_SOCKET_READ: uint32_t = 23 as uint32_t;
pub const SYS_SOCKET_WRITE: uint32_t = 24 as uint32_t;
pub const SYS_DUP2: uint32_t = 29 as uint32_t;
pub const SYS_KILL: uint32_t = 30 as uint32_t;
#[no_mangle]
pub unsafe extern "C" fn syscall_dispatcher(
    mut syscall_no: uint32_t,
    mut arg0: uint32_t,
    mut arg1: uint32_t,
    mut arg2: uint32_t,
    mut arg3: uint32_t,
    mut arg4: uint32_t,
) -> uint32_t {
    let mut result: uint32_t = 0 as uint32_t;
    match syscall_no {
        0 => {
            result = rust_sys_exit(arg0);
        }
        20 => {
            result = rust_sys_fork();
        }
        21 => {
            result = rust_sys_clone(arg0, arg1, arg2);
        }
        1 => {
            result = rust_sys_yield();
        }
        2 => {
            result = rust_sys_getpid();
        }
        3 => {
            result = rust_sys_sleep(arg0);
        }
        4 => {
            result = rust_sys_open(arg0, arg1, arg2);
        }
        5 => {
            result = rust_sys_read(arg0, arg1, arg2);
        }
        6 => {
            result = rust_sys_write(arg0, arg1, arg2);
        }
        7 => {
            result = rust_sys_close(arg0);
        }
        8 => {
            result = rust_sys_dup(arg0);
        }
        9 => {
            result = rust_sys_lseek(arg0, arg1, arg2);
        }
        10 => {
            result = rust_sys_pipe(arg0);
        }
        11 => {
            result = rust_sys_execve(arg0);
        }
        12 => {
            result = rust_sys_socket(arg0 as int32_t, arg1 as int32_t, arg2 as int32_t) as uint32_t;
        }
        13 => {
            result = rust_sys_bind(arg0 as int32_t, arg1 as *const ::core::ffi::c_void, arg2)
                as uint32_t;
        }
        14 => {
            result = rust_sys_listen(arg0 as int32_t, arg1 as int32_t) as uint32_t;
        }
        15 => {
            result = rust_sys_accept(arg0 as int32_t) as uint32_t;
        }
        16 => {
            result = rust_sys_connect(arg0 as int32_t, arg1 as *const ::core::ffi::c_void, arg2)
                as uint32_t;
        }
        17 => {
            result = rust_sys_close_socket(arg0 as int32_t) as uint32_t;
        }
        18 => {
            result = rust_sys_has_pending_connections(arg0 as int32_t) as uint32_t;
        }
        22 => {
            result = rust_sys_waitpid(arg0, arg1);
        }
        23 => {
            result = rust_sys_socket_read(arg0 as int32_t, arg1, arg2) as uint32_t;
        }
        24 => {
            result = rust_sys_socket_write(arg0 as int32_t, arg1, arg2) as uint32_t;
        }
        29 => {
            result = rust_sys_dup2(arg0, arg1);
        }
        30 => {
            result = rust_sys_kill(arg0, arg1);
        }
        _ => {
            serial_print(
                b"[Syscall] Unknown syscall number: 0x\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            serial_print_hex(syscall_no);
            serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
            result = -(1 as ::core::ffi::c_int) as uint32_t;
        }
    }
    return result;
}
#[cfg(target_arch = "x86_64")]
static mut syscall_gs_save_area: [uint64_t; 3] = [
    0 as ::core::ffi::c_int as uint64_t,
    0 as ::core::ffi::c_int as uint64_t,
    0 as ::core::ffi::c_int as uint64_t,
];
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub static mut g_kernel_gs_base: uint64_t = 0 as uint64_t;
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn syscall_init() {
    serial_print(
        b"[Syscall] Initializing x86_64 syscall interface\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    kernel_stack_top = &raw mut kernel_stack_top_alias as uint64_t;
    serial_print(b"[Syscall] Kernel stack top: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex64(kernel_stack_top);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut star: uint64_t = 0 as uint64_t;
    star |= (0x8 as ::core::ffi::c_int as uint64_t) << 32 as ::core::ffi::c_int;
    star |= (0x10 as ::core::ffi::c_int as uint64_t) << 48 as ::core::ffi::c_int;
    let mut star_low: uint32_t = (star & 0xffffffff as uint64_t) as uint32_t;
    let mut star_high: uint32_t =
        (star >> 32 as ::core::ffi::c_int & 0xffffffff as uint64_t) as uint32_t;
    asm!(
        "wrmsr\n", inlateout("cx") 0xc0000081 as ::core::ffi::c_uint => _,
        inlateout("ax") star_low => _, inlateout("dx") star_high => _,
        options(preserves_flags, att_syntax)
    );
    let mut lstar: uint64_t = ::core::mem::transmute::<
        Option<unsafe extern "C" fn() -> ()>,
        uint64_t,
    >(Some(syscall_entry));
    let mut lstar_low: uint32_t = (lstar & 0xffffffff as uint64_t) as uint32_t;
    let mut lstar_high: uint32_t =
        (lstar >> 32 as ::core::ffi::c_int & 0xffffffff as uint64_t) as uint32_t;
    asm!(
        "wrmsr\n", inlateout("dx") lstar_high => _, inlateout("cx") 0xc0000082 as
        ::core::ffi::c_uint => _, inlateout("ax") lstar_low => _,
        options(preserves_flags, att_syntax)
    );
    let mut sf_mask: uint64_t = 0x300 as uint64_t;
    let mut sf_low: uint32_t = (sf_mask & 0xffffffff as uint64_t) as uint32_t;
    let mut sf_high: uint32_t =
        (sf_mask >> 32 as ::core::ffi::c_int & 0xffffffff as uint64_t) as uint32_t;
    asm!(
        "wrmsr\n", inlateout("cx") 0xc0000084 as ::core::ffi::c_uint => _,
        inlateout("ax") sf_low => _, inlateout("dx") sf_high => _,
        options(preserves_flags, att_syntax)
    );
    let mut gs_base: uint64_t = &raw mut syscall_gs_save_area as uintptr_t as uint64_t;
    let mut kgs_low: uint32_t = (gs_base & 0xffffffff as uint64_t) as uint32_t;
    let mut kgs_high: uint32_t =
        (gs_base >> 32 as ::core::ffi::c_int & 0xffffffff as uint64_t) as uint32_t;
    asm!(
        "wrmsr\n", inlateout("ax") kgs_low => _, inlateout("cx") 0xc0000102 as
        ::core::ffi::c_uint => _, inlateout("dx") kgs_high => _, options(preserves_flags,
        att_syntax)
    );
    g_kernel_gs_base = gs_base;
    serial_print(
        b"[Syscall] x86_64 syscall MSRs configured\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn syscall_init() {
    serial_print(
        b"[Syscall] ARM64 SVC interface ready\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
