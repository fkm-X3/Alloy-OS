#include <stddef.h>

extern "C" void *malloc(size_t);
extern "C" void free(void *);

namespace std {
enum class align_val_t : size_t {};
}

void *operator new(size_t size) { return malloc(size); }
void *operator new[](size_t size) { return malloc(size); }
void *operator new(size_t size, std::align_val_t al) { (void)al; return malloc(size); }
void *operator new[](size_t size, std::align_val_t al) { (void)al; return malloc(size); }
void operator delete(void *ptr) noexcept { free(ptr); }
void operator delete(void *ptr, size_t) noexcept { free(ptr); }
void operator delete(void *ptr, std::align_val_t al) noexcept { (void)al; free(ptr); }
void operator delete(void *ptr, size_t, std::align_val_t al) noexcept { (void)al; free(ptr); }
void operator delete[](void *ptr) noexcept { free(ptr); }
void operator delete[](void *ptr, size_t) noexcept { free(ptr); }
void operator delete[](void *ptr, std::align_val_t al) noexcept { (void)al; free(ptr); }
void operator delete[](void *ptr, size_t, std::align_val_t al) noexcept { (void)al; free(ptr); }
