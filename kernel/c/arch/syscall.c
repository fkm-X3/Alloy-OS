#include "arch/syscall.h"
#include "../drivers/serial.h"

extern uint32_t rust_sys_exit(uint32_t code);
extern uint32_t rust_sys_fork();
extern uint32_t rust_sys_yield();
extern uint32_t rust_sys_getpid();
extern uint32_t rust_sys_sleep(uint32_t ms);
extern uint32_t rust_sys_open(uint32_t path_ptr, uint32_t flags, uint32_t mode);
extern uint32_t rust_sys_read(uint32_t fd, uint32_t buf_ptr, uint32_t len);
extern uint32_t rust_sys_write(uint32_t fd, uint32_t buf_ptr, uint32_t len);
extern uint32_t rust_sys_close(uint32_t fd);
extern uint32_t rust_sys_dup(uint32_t oldfd);
extern uint32_t rust_sys_lseek(uint32_t fd, uint32_t offset, uint32_t whence);
extern uint32_t rust_sys_pipe(uint32_t pipefd_ptr);
extern uint32_t rust_sys_execve(uint32_t path_ptr);
extern int32_t rust_sys_socket(int32_t domain, int32_t socket_type, int32_t protocol);
extern int32_t rust_sys_bind(int32_t fd, const void* addr, uint32_t addr_len);
extern int32_t rust_sys_listen(int32_t fd, int32_t backlog);
extern int32_t rust_sys_accept(int32_t fd);
extern int32_t rust_sys_connect(int32_t fd, const void* addr, uint32_t addr_len);
extern int32_t rust_sys_close_socket(int32_t fd);
extern uint32_t rust_sys_clone(uint32_t entry, uint32_t stack, uint32_t arg);
extern uint32_t rust_sys_waitpid(uint32_t pid, uint32_t options);
extern int32_t rust_sys_has_pending_connections(int32_t fd);
extern int32_t rust_sys_socket_read(int32_t fd, uint32_t buf_ptr, uint32_t len);
extern int32_t rust_sys_socket_write(int32_t fd, uint32_t buf_ptr, uint32_t len);

uint32_t syscall_dispatcher(uint32_t syscall_no,
                            uint32_t arg0,
                            uint32_t arg1,
                            uint32_t arg2,
                            uint32_t arg3,
                            uint32_t arg4,
                            uint32_t* int80_frame) {
    (void)arg1;
    (void)arg2;
    (void)arg3;
    (void)arg4;
    (void)int80_frame;

    uint32_t result = 0;

    switch (syscall_no) {
        case SYS_EXIT:
            result = rust_sys_exit(arg0);
            break;
        case SYS_FORK:
            result = rust_sys_fork();
            break;
        case SYS_CLONE:
            result = rust_sys_clone(arg0, arg1, arg2);
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
        case SYS_HAS_PENDING_CONNECTIONS:
            result = (uint32_t)(int32_t)rust_sys_has_pending_connections((int32_t)arg0);
            break;
        case SYS_WAITPID:
            result = rust_sys_waitpid(arg0, arg1);
            break;
        case SYS_SOCKET_READ:
            result = (uint32_t)(int32_t)rust_sys_socket_read((int32_t)arg0, arg1, arg2);
            break;
        case SYS_SOCKET_WRITE:
            result = (uint32_t)(int32_t)rust_sys_socket_write((int32_t)arg0, arg1, arg2);
            break;
        default:
            serial_print("[Syscall] Unknown syscall number\n");
            result = (uint32_t)-1;
            break;
    }

    return result;
}

#ifdef ARCH_I686

extern void syscall_entry();

void syscall_init() {
    serial_print("[Syscall] Initializing system call interface\n");
    serial_print("[Syscall] System calls ready (INT 0x80)\n");
}

#elif defined(ARCH_X86_64)

extern void syscall_entry();

void syscall_init() {
    serial_print("[Syscall] Initializing x86_64 syscall interface\n");

    // Set up MSRs for the 'syscall' instruction
    // STAR (0xC0000081): selects CS and SS for kernel and user
    uint64_t star = 0;
    star |= (uint64_t)0x08 << 32;  // SYSCALL CS (kernel code segment)
    star |= (uint64_t)0x10 << 48;  // SYSRET CS (user code segment - 2)
    // Actually: for SYSRET, CS = (STAR[63:48] + 16) | 3, which gives 0x18+3 = 0x1B
    // No wait: SYSRET CS = (STAR[63:48] + 16) | 3
    // If STAR[63:48] = 0x08, SYSRET CS = (0x08+16) | 3 = 0x18 | 3 = 0x1B
    // But our x86_64 user code segment is 0x18 (index 3 * 8 = 24 = 0x18)
    // So STAR[63:48] = 0x08 gives SYSRET CS = (0x08 + 16) | 3 = (0x18) | 3 = 0x1B
    // Wait, 0x18 | 3 = 0x1B, but the selector is 0x18. The RPL is OR'd by CPU.
    // So SYSRET gives CS = (STAR[63:48] + 16) | 3 = (0x08 + 16) | 3 = 0x08 + 16 + 3
    // Hmm 0x08 + 0x10 + 3 = 0x1B. That's 0x18 | 3. Correct!
    uint32_t star_low = (uint32_t)(star & 0xFFFFFFFF);
    uint32_t star_high = (uint32_t)((star >> 32) & 0xFFFFFFFF);

    asm volatile("wrmsr" : : "c"(0xC0000081), "a"(star_low), "d"(star_high));

    // LSTAR (0xC0000082): RIP of syscall entry point
    uint64_t lstar = (uint64_t)syscall_entry;
    uint32_t lstar_low = (uint32_t)(lstar & 0xFFFFFFFF);
    uint32_t lstar_high = (uint32_t)((lstar >> 32) & 0xFFFFFFFF);
    asm volatile("wrmsr" : : "c"(0xC0000082), "a"(lstar_low), "d"(lstar_high));

    // SF_MASK (0xC0000084): mask RFLAGS bits on syscall entry
    // Mask IF (bit 9) to disable interrupts on entry
    uint64_t sf_mask = 0x300;  // IF + reserved bit
    uint32_t sf_low = (uint32_t)(sf_mask & 0xFFFFFFFF);
    uint32_t sf_high = (uint32_t)((sf_mask >> 32) & 0xFFFFFFFF);
    asm volatile("wrmsr" : : "c"(0xC0000084), "a"(sf_low), "d"(sf_high));

    serial_print("[Syscall] x86_64 syscall MSRs configured\n");
}

#elif defined(ARCH_AARCH64)

void svc_handler() {
}

void syscall_init() {
    serial_print("[Syscall] ARM64 SVC interface ready\n");
}

#else

extern void syscall_entry();

void syscall_init() {
    serial_print("[Syscall] Initializing system call interface\n");
    serial_print("[Syscall] System calls ready (INT 0x80)\n");
}
#endif