#include <stddef.h>

// C++ runtime startup support.
// Calls static initializers (constructors of global/static objects)
// before main() and registers finalizers to run after main().

// These symbols are defined by the linker script to bracket the init/fini arrays.
extern void (*__preinit_array_start[])(int, char **, char **);
extern void (*__preinit_array_end[])(int, char **, char **);
extern void (*__init_array_start[])(void);
extern void (*__init_array_end[])(void);
extern void (*__fini_array_start[])(void);
extern void (*__fini_array_end[])(void);

// Call all C++ static initializers.
void __alloy_init_cpp(void) {
    size_t preinit_count = __preinit_array_end - __preinit_array_start;
    for (size_t i = 0; i < preinit_count; i++) {
        __preinit_array_start[i](0, 0, 0);
    }

    size_t init_count = __init_array_end - __init_array_start;
    for (size_t i = 0; i < init_count; i++) {
        __init_array_start[i]();
    }
}

// Call all C++ finalizers.
void __alloy_fini_cpp(void) {
    size_t fini_count = __fini_array_end - __fini_array_start;
    for (size_t i = fini_count; i > 0; i--) {
        __fini_array_start[i - 1]();
    }
}
