#include "arch/syscall.h"

extern "C" void serial_print(const char* str);

// Forward declarations for Rust syscall handlers
extern "C" uint32_t rust_sys_exit(uint32_t code);
extern "C" uint32_t rust_sys_yield();
extern "C" uint32_t rust_sys_getpid();
extern "C" uint32_t rust_sys_sleep(uint32_t ms);
extern "C" uint32_t rust_sys_open(uint32_t path_ptr, uint32_t flags, uint32_t mode);
extern "C" uint32_t rust_sys_read(uint32_t fd, uint32_t buf_ptr, uint32_t len);
extern "C" uint32_t rust_sys_write(uint32_t fd, uint32_t buf_ptr, uint32_t len);
extern "C" uint32_t rust_sys_close(uint32_t fd);
extern "C" uint32_t rust_sys_dup(uint32_t oldfd);
extern "C" uint32_t rust_sys_lseek(uint32_t fd, uint32_t offset, uint32_t whence);
extern "C" uint32_t rust_sys_pipe(uint32_t pipefd_ptr);
extern "C" uint32_t rust_sys_execve(uint32_t path_ptr);
extern "C" int32_t rust_sys_socket(int32_t domain, int32_t socket_type, int32_t protocol);
extern "C" int32_t rust_sys_bind(int32_t fd, const void* addr, uint32_t addr_len);
extern "C" int32_t rust_sys_listen(int32_t fd, int32_t backlog);
extern "C" int32_t rust_sys_accept(int32_t fd);
extern "C" int32_t rust_sys_connect(int32_t fd, const void* addr, uint32_t addr_len);
extern "C" int32_t rust_sys_close_socket(int32_t fd);

// Syscall dispatcher - routes syscalls to handlers
extern "C" uint32_t syscall_dispatcher(uint32_t syscall_no, 
                                       uint32_t arg0,
                                       uint32_t arg1,
                                       uint32_t arg2,
                                       uint32_t arg3,
                                       uint32_t arg4) {
    (void)arg1;
    (void)arg2;
    (void)arg3;
    (void)arg4;
    
    uint32_t result = 0;
    
    switch (syscall_no) {
        case SYS_EXIT:
            result = rust_sys_exit(arg0);
            break;
        case SYS_YIELD:
            result = rust_sys_yield();
            break;
        case SYS_GETPID:
            result = rust_sys_getpid();
            break;
        case SYS_SLEEP:
            result = rust_sys_sleep(arg0);
            break;
        case SYS_OPEN:
            result = rust_sys_open(arg0, arg1, arg2);
            break;
        case SYS_READ:
            result = rust_sys_read(arg0, arg1, arg2);
            break;
        case SYS_WRITE:
            result = rust_sys_write(arg0, arg1, arg2);
            break;
        case SYS_CLOSE:
            result = rust_sys_close(arg0);
            break;
        case SYS_DUP:
            result = rust_sys_dup(arg0);
            break;
        case SYS_LSEEK:
            result = rust_sys_lseek(arg0, arg1, arg2);
            break;
        case SYS_PIPE:
            result = rust_sys_pipe(arg0);
            break;
        case SYS_EXECVE:
            result = rust_sys_execve(arg0);
            break;
        case SYS_SOCKET:
            result = (uint32_t)(int32_t)rust_sys_socket((int32_t)arg0, (int32_t)arg1, (int32_t)arg2);
            break;
        case SYS_BIND:
            result = (uint32_t)(int32_t)rust_sys_bind((int32_t)arg0, (const void*)arg1, (uint32_t)arg2);
            break;
        case SYS_LISTEN:
            result = (uint32_t)(int32_t)rust_sys_listen((int32_t)arg0, (int32_t)arg1);
            break;
        case SYS_ACCEPT:
            result = (uint32_t)(int32_t)rust_sys_accept((int32_t)arg0);
            break;
        case SYS_CONNECT:
            result = (uint32_t)(int32_t)rust_sys_connect((int32_t)arg0, (const void*)arg1, (uint32_t)arg2);
            break;
        case SYS_CLOSE_SOCKET:
            result = (uint32_t)(int32_t)rust_sys_close_socket((int32_t)arg0);
            break;
        default:
            serial_print("[Syscall] Unknown syscall number\n");
            result = (uint32_t)-1;
            break;
    }
    
    return result;
}

#ifdef ARCH_I686
// i686: INT 0x80 syscall interface
extern "C" void syscall_entry();

extern "C" void syscall_init() {
    serial_print("[Syscall] Initializing system call interface\n");
    serial_print("[Syscall] System calls ready (INT 0x80)\n");
}

#elif defined(ARCH_X86_64)
// x86_64: syscall/sysret interface (placeholder)
extern "C" void syscall_init() {
    serial_print("[Syscall] x86_64 syscall not yet implemented\n");
}

#elif defined(ARCH_AARCH64)
// ARM64: SVC #0 syscall interface
extern "C" void svc_handler() {
    // Read syscall number from x8
    // Arguments in x0-x5
    // Return value in x0
}

extern "C" void syscall_init() {
    serial_print("[Syscall] ARM64 SVC interface ready\n");
}

#else
// Default (i686)
extern "C" void syscall_entry();

extern "C" void syscall_init() {
    serial_print("[Syscall] Initializing system call interface\n");
    serial_print("[Syscall] System calls ready (INT 0x80)\n");
}
#endif
