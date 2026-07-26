#include <stddef.h>

extern void (*__preinit_array_start[])(int, char **, char **);
extern void (*__preinit_array_end[])(int, char **, char **);
extern void (*__init_array_start[])(void);
extern void (*__init_array_end[])(void);
extern void (*__fini_array_start[])(void);
extern void (*__fini_array_end[])(void);

#define SYS_WRITE 6

static inline void debug_write(char c) {
    char buf[1] = {c};
    register long r10 asm("r10") = 0;
    register long r8 asm("r8") = 0;
    asm volatile(
        "syscall"
        : : "a"((long)SYS_WRITE), "D"((long)1), "S"(buf), "d"((long)1), "r"(r10), "r"(r8)
        : "rcx", "r11", "r9", "memory"
    );
}

void __alloy_init_cpp(void) {
    debug_write('P');

    size_t preinit_count = __preinit_array_end - __preinit_array_start;
    for (size_t i = 0; i < preinit_count; i++) {
        debug_write('0' + (i < 9 ? i : 9));
        __preinit_array_start[i](0, 0, 0);
    }

    debug_write('I');

    size_t init_count = __init_array_end - __init_array_start;
    for (size_t i = 0; i < init_count; i++) {
        debug_write('a' + (i < 25 ? i : 25));
        __init_array_start[i]();
    }

    debug_write('D');
}

void __alloy_fini_cpp(void) {
    size_t fini_count = __fini_array_end - __fini_array_start;
    for (size_t i = fini_count; i > 0; i--) {
        __fini_array_start[i - 1]();
    }
}
