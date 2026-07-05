// C++ ABI stubs for freestanding Alloy OS environment.
// These satisfy the compiler's implicit references to runtime support functions.

#define UNUSED(x) ((void)(x))

// Called when a pure virtual function is called (should never happen).
void __cxa_pure_virtual(void) {
    while (1) { __builtin_trap(); }
}

// Called by static initializers to register cleanup functions.
// Since we never exit normally, this can be a no-op.
int __cxa_atexit(void (*destructor)(void *), void *arg, void *dso_handle) {
    UNUSED(destructor);
    UNUSED(arg);
    UNUSED(dso_handle);
    return 0;
}

// DSO handle for the main executable.
void *__dso_handle __attribute__((weak)) = (void *)&__dso_handle;

// Personality function for exception handling (no exceptions in this build).
void __gxx_personality_v0(void) {
    __builtin_unreachable();
}

// Unwind resume stub (no exceptions).
void _Unwind_Resume(void *e) {
    UNUSED(e);
    __builtin_unreachable();
}

// Guard variable helpers for thread-safe static initialization.
// In a single-threaded freestanding environment, these are trivial.

// Returns 1 if the guard byte is set (initialization complete), 0 otherwise.
int __cxa_guard_acquire(int *guard) {
    // Check if the low byte is set (initialization complete).
    return (*guard & 1) == 0;
}

void __cxa_guard_release(int *guard) {
    // Set the low byte to indicate initialization is complete.
    *guard = 1;
}

void __cxa_guard_abort(int *guard) {
    UNUSED(guard);
    // Called if initialization throws an exception.
    // No-op when exceptions are disabled.
}
