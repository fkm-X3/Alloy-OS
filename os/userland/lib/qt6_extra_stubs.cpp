// qt6_extra_stubs.cpp — Additional stubs needed by Qt6Qml + Qt6Quick
// Provides linker-only stubs for glib, ICU, zstd, dlopen, POSIX, and C++ ABI.

#include <cstddef>
#include <cstdint>

extern "C" {

// ── C++ ABI exception handling stubs ────────────────────────────────────────
// Qt libraries were compiled with exceptions; we provide no-op stubs.

void* __cxa_allocate_exception(unsigned long sz) {
    (void)sz;
    __builtin_trap();
}
void __cxa_free_exception(void* ptr) { (void)ptr; }
void __cxa_throw(void* ptr, void* type, void (*dtor)(void*)) {
    (void)ptr; (void)type; (void)dtor;
    __builtin_trap();
}
void __cxa_begin_catch(void* exceptionObject) { (void)exceptionObject; }
void __cxa_end_catch(void) {}
void* __cxa_rethrow(void) { __builtin_trap(); return 0; }
void __cxa_bad_cast(void) { __builtin_trap(); }
void __cxa_bad_typeid(void) { __builtin_trap(); }
void __cxa_throw_bad_array_new_length(void) { __builtin_trap(); }
char* __cxa_demangle(const char* mangled, char* buf, size_t* len, int* status) {
    (void)mangled; (void)buf; (void)len;
    if (status) *status = -1;
    return 0;
}

// ── Unwind stubs (no exceptions) ───────────────────────────────────────────

struct _Unwind_Exception { unsigned long long exception_class; void (*exception_cleanup)(int, void*); };
struct _Unwind_Context;

int _Unwind_RaiseException(struct _Unwind_Exception* exc) {
    (void)exc;
    __builtin_trap();
    return 0;
}
void _Unwind_DeleteException(struct _Unwind_Exception* exc) { (void)exc; }
void* _Unwind_GetGR(struct _Unwind_Context* context, int index) { (void)context; (void)index; return 0; }
void _Unwind_SetGR(struct _Unwind_Context* context, int index, void* value) { (void)context; (void)index; (void)value; }
void* _Unwind_GetIP(struct _Unwind_Context* context) { (void)context; return 0; }
void _Unwind_SetIP(struct _Unwind_Context* context, void* value) { (void)context; (void)value; }
void* _Unwind_GetLanguageSpecificData(struct _Unwind_Context* context) { (void)context; return 0; }
void* _Unwind_GetRegionStart(struct _Unwind_Context* context) { (void)context; return 0; }
unsigned long _Unwind_GetCFA(struct _Unwind_Context* context) { (void)context; return 0; }
void _Unwind_Resume(struct _Unwind_Exception* exc) { (void)exc; __builtin_trap(); }

// typeinfo for __cxxabiv1::__forced_unwind (needed by exception handling)
extern const char _ZTSN10__cxxabiv115__forced_unwindE[];
extern void* _ZTVN10__cxxabiv117__class_type_infoE[];
void* _ZTIN10__cxxabiv115__forced_unwindE[3] = {
    (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
    (void*)_ZTSN10__cxxabiv115__forced_unwindE,
    (void*)0
};
const char _ZTSN10__cxxabiv115__forced_unwindE[] = "N10__cxxabiv115__forced_unwindE";

// typeinfo/vtable for std::bad_alloc
extern void* _ZTVN10__cxxabiv120__si_class_type_infoE[];

// ── dlopen / dlsym stubs ───────────────────────────────────────────────────

void* dlopen(const char* filename, int flags) { (void)filename; (void)flags; return 0; }
int dlclose(void* handle) { (void)handle; return -1; }
void* dlsym(void* handle, const char* symbol) { (void)handle; (void)symbol; return 0; }
char* dlerror(void) { return "dlopen not supported"; }
int dladdr(const void* addr, void* info) { (void)addr; (void)info; return 0; }

// ── glib stubs ──────────────────────────────────────────────────────────────
// QEventDispatcherGlib uses glib. We stub it all out since we use our own
// event dispatch via the Wayland QPA plugin.

struct _GMainContext;
struct _GSource;
struct _GPollFD;
typedef int (*GSourceFunc)(void*);
typedef struct _GSourceFuncs { int (*prepare)(void*, int*); int (*check)(void*, void*); int (*dispatch)(void*, GSourceFunc, void*); void (*finalize)(void*); } GSourceFuncs;

void* g_main_context_default(void) { return 0; }
void* g_main_context_new(void) { return 0; }
void g_main_context_ref(void* context) { (void)context; }
void g_main_context_unref(void* context) { (void)context; }
int g_main_context_iteration(void* context, int may_block) { (void)context; (void)may_block; return 0; }
void g_main_context_push_thread_default(void* context) { (void)context; }
void g_main_context_pop_thread_default(void* context) { (void)context; }
void g_main_context_wakeup(void* context) { (void)context; }

void* g_source_new(GSourceFuncs* source_funcs, unsigned int struct_size) {
    (void)source_funcs; (void)struct_size;
    static char dummy[256];
    return dummy;
}
void g_source_set_name(void* source, const char* name) { (void)source; (void)name; }
void g_source_set_can_recurse(void* source, int can_recurse) { (void)source; (void)can_recurse; }
unsigned int g_source_attach(void* source, void* context) { (void)source; (void)context; return 1; }
void g_source_destroy(void* source) { (void)source; }
void g_source_unref(void* source) { (void)source; }
void g_source_add_poll(void* source, void* fd) { (void)source; (void)fd; }
void g_source_remove_poll(void* source, void* fd) { (void)source; (void)fd; }

// ── ICU stubs ───────────────────────────────────────────────────────────────
// Qt6 uses ICU for text encoding/collation. We provide no-op stubs.

typedef void* UConverter;
typedef int32_t UChar;
typedef unsigned int UErrorCode;

void* ucnv_open_74(const char* converterName, UErrorCode* err) {
    (void)converterName; if (err) *err = 0; return 0;
}
void ucnv_close_74(void* converter) { (void)converter; }
void ucnv_setFromUCallBack_74(void* converter, void* newAction, void* oldAction, void* err) {
    (void)converter; (void)newAction; (void)oldAction; (void)err;
}
void ucnv_setToUCallBack_74(void* converter, void* newAction, void* oldAction, void* err) {
    (void)converter; (void)newAction; (void)oldAction; (void)err;
}
int32_t ucnv_fromUnicode_74(void* converter, char* target, int32_t targetCapacity,
    const void** source, const void* sourceLimit, int32_t* offsets, int32_t flush, UErrorCode* err) {
    (void)converter; (void)target; (void)targetCapacity; (void)source;
    (void)sourceLimit; (void)offsets; (void)flush; if (err) *err = 0; return 0;
}
int32_t ucnv_toUnicode_74(void* converter, void* target, int32_t targetCapacity,
    const char** source, const char* sourceLimit, int32_t* offsets, int32_t flush, UErrorCode* err) {
    (void)converter; (void)target; (void)targetCapacity; (void)source;
    (void)sourceLimit; (void)offsets; (void)flush; if (err) *err = 0; return 0;
}
void ucnv_reset_74(void* converter) { (void)converter; }
int32_t ucnv_getMaxCharSize_74(void* converter) { (void)converter; return 1; }
const char* ucnv_getName_74(void* converter, UErrorCode* err) { (void)converter; if (err) *err = 0; return "US-ASCII"; }
const char* ucnv_getStandardName_74(const char* name, const char* standard, UErrorCode* err) {
    (void)name; (void)standard; if (err) *err = 0; return "US-ASCII";
}
void* ucnv_getFromUCallBack_74(void* converter, int* err) { (void)converter; (void)err; return 0; }
void* ucnv_getToUCallBack_74(void* converter, int* err) { (void)converter; (void)err; return 0; }
void ucnv_cbFromUWriteUChars_74(void* converter, const void* srcStart, const void* srcLimit,
    void** target, const void* targetLimit, void* offsets, int32_t flush, int* err) {
    (void)converter; (void)srcStart; (void)srcLimit; (void)target;
    (void)targetLimit; (void)offsets; (void)flush; (void)err;
}
void ucnv_cbToUWriteUChars_74(void* converter, const void* source, int32_t length,
    int32_t numberCharsWritten, void* offsets, int32_t flush, int* err) {
    (void)converter; (void)source; (void)length; (void)numberCharsWritten;
    (void)offsets; (void)flush; (void)err;
}
void ucnv_cbFromUWriteBytes_74(void* converter, const char* source, int32_t length,
    int32_t numberBytesWritten, void* offsets, int32_t flush, int* err) {
    (void)converter; (void)source; (void)length; (void)numberBytesWritten;
    (void)offsets; (void)flush; (void)err;
}
int32_t ucnv_fromUCountPending_74(void* converter, UErrorCode* err) {
    (void)converter; if (err) *err = 0; return 0;
}
int32_t ucnv_toUCountPending_74(void* converter, UErrorCode* err) {
    (void)converter; if (err) *err = 0; return 0;
}

// ICU collation stubs
void* ucol_open_74(const char* loc, int* status) { (void)loc; if (status) *status = 0; return 0; }
void ucol_close_74(void* coll) { (void)coll; }
void ucol_setAttribute_74(void* coll, int type, int value, int* status) {
    (void)coll; (void)type; (void)value; if (status) *status = 0;
}
int32_t ucol_strcoll_74(void* coll, const void* source, int32_t sourceLength,
    const void* target, int32_t targetLength) {
    (void)coll; (void)source; (void)sourceLength; (void)target; (void)targetLength;
    return 0;
}

// ICU callback data symbols (function pointers used as default callbacks)
void UCNV_FROM_U_CALLBACK_SUBSTITUTE_74() {}
void UCNV_TO_U_CALLBACK_SUBSTITUTE_74() {}

// ICU calendar stubs (needed by QIcuTimeZonePrivate)
void* ucal_open_74(const void* tzID, int32_t tzIDLength, const char* locale, int32_t type, int* status) {
    (void)tzID; (void)tzIDLength; (void)locale; (void)type; if (status) *status = 0; return 0;
}
void ucal_close_74(void* cal) { (void)cal; }
void* ucal_clone_74(void* cal, int* status) { (void)cal; if (status) *status = 0; return 0; }
void ucal_setMillis_74(void* cal, double millis, int* status) { (void)cal; (void)millis; if (status) *status = 0; }
void ucal_get_74(void* cal, int32_t field, int* status) { (void)cal; (void)field; if (status) *status = 0; }
int32_t ucal_inDaylightTime_74(void* cal, int* status) { (void)cal; if (status) *status = 0; return 0; }
int32_t ucal_getDSTSavings_74(void* tz, int* status) { (void)tz; if (status) *status = 0; return 0; }
void ucal_openTimeZones_74(int* status) { if (status) *status = 0; }
void ucal_openCountryTimeZones_74(const char* country, int* status) { (void)country; if (status) *status = 0; }
void* ucal_openTimeZoneIDEnumeration_74(int32_t zoneType, void* region, const void* filter, int* status) {
    (void)zoneType; (void)region; (void)filter; if (status) *status = 0; return 0;
}
void ucal_getDefaultTimeZone_74(void* resultID, int32_t resultIDCapacity, int* status) {
    (void)resultID; (void)resultIDCapacity; if (status) *status = 0;
}
int32_t ucal_getTimeZoneDisplayName_74(void* cal, int32_t type, int32_t nameStyle, const void* locale, void* result, int32_t resultLength, int* status) {
    (void)cal; (void)type; (void)nameStyle; (void)locale; (void)result; (void)resultLength; if (status) *status = 0; return 0;
}
int32_t ucal_getTimeZoneTransitionDate_74(void* cal, int32_t type, double* transition, int* status) {
    (void)cal; (void)type; if (transition) *transition = 0; if (status) *status = 0; return 0;
}

// ICU enumeration stubs
void* uenum_next_74(void* en, int32_t* resultLength, int* status) {
    (void)en; if (resultLength) *resultLength = 0; if (status) *status = 0; return 0;
}
void uenum_close_74(void* en) { (void)en; }

} // extern "C"

// ── std::bad_alloc — RTTI + vtable + destructor ───────────────────────────
// Qt6Core was compiled with RTTI; these symbols must exist for vtable/typeinfo
// relocation in .data.rel.ro. We provide the raw data manually because we
// compile userland with -fno-rtti and don't have <exception> in freestanding.

extern "C" {

// typeinfo name: "St9bad_alloc"
static const char _ZTSSt9bad_alloc_val[] = "St9bad_alloc";
extern "C" void* _ZTVN10__cxxabiv117__class_type_infoE[];

// typeinfo for std::bad_alloc (__class_type_info, no base)
void* _ZTISt9bad_alloc[2] = {
    (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
    (void*)_ZTSSt9bad_alloc_val
};

// vtable for std::bad_alloc: [offset_to_top=0, typeinfo, what(), ~bad_alloc()]
// We need at least 3 entries (vcall offset 0, offset_to_top, typeinfo)
void* _ZTVSt9bad_alloc[4] = {
    (void*)0,                // [0]: vcall offset (unused)
    (void*)0,                // [1]: offset_to_top = 0
    (void*)_ZTISt9bad_alloc, // [2]: typeinfo ptr
    (void*)0,                // [3]: virtual destructor placeholder
};

// std::bad_alloc destructor
void _ZNSt9bad_allocD1Ev(void*) {}
void _ZNSt9bad_allocD0Ev(void*) {}

// std::bad_alloc::what()
const char* _ZNKSt9bad_alloc4whatEv() { return "std::bad_alloc"; }

// __gxx_personality_v0 — exception handling personality routine (never invoked
// because we use -fno-exceptions, but referenced by .gcc_except_table in Qt6)
int __gxx_personality_v0(...) { return 0; }
int __gxx_personality_v1(...) { return 0; }

} // extern "C" (std::bad_alloc + personality)

extern "C" {

// ── ZSTD stubs ──────────────────────────────────────────────────────────────

typedef unsigned long long ZSTD_DDict;
typedef unsigned long long ZSTD_DStream;
typedef size_t ZSTD_ErrorCode;

void* ZSTD_createDCtx(void) { return 0; }
size_t ZSTD_freeDCtx(void* cctx) { (void)cctx; return 0; }
size_t ZSTD_decompress(void* dst, size_t dstCapacity, const void* src, size_t compressedSize) {
    (void)dst; (void)dstCapacity; (void)src; (void)compressedSize; return 0;
}
size_t ZSTD_getFrameContentSize(const void* src, size_t srcSize) { (void)src; (void)srcSize; return 0; }
int ZSTD_isError(size_t code) { (void)code; return 0; }
const char* ZSTD_getErrorName(size_t code) { (void)code; return "zstd not supported"; }
size_t ZSTD_findFrameCompressedSize(const void* src, size_t srcSize) { (void)src; (void)srcSize; return 0; }

// ── zlib stubs (for inflate used by Qt Quick) ──────────────────────────────

typedef unsigned char Bytef;
typedef unsigned long uInt;
typedef unsigned long uLong;
typedef void* z_streamp;
struct internal_state { int dummy; };

int inflateInit_(void* strm, const char* version, int stream_size) {
    (void)strm; (void)version; (void)stream_size; return 0;
}
int inflate(void* strm, int flush) { (void)strm; (void)flush; return 0; }
int inflateEnd(void* strm) { (void)strm; return 0; }
int inflateInit2_(void* strm, int windowBits, const char* version, int stream_size) {
    (void)strm; (void)windowBits; (void)version; (void)stream_size; return 0;
}
int inflateSync(void* strm) { (void)strm; return 0; }
int inflateReset(void* strm) { (void)strm; return 0; }
int uncompress(Bytef* dest, uLong* destLen, const Bytef* source, uLong sourceLen) {
    (void)dest; (void)destLen; (void)source; (void)sourceLen; return 0;
}
int uncompress2(Bytef* dest, uLong* destLen, const Bytef* source, uLong* sourceLen) {
    (void)dest; (void)destLen; (void)source; (void)sourceLen; return 0;
}

int z_deflate(void* strm, int flush) { (void)strm; (void)flush; return 0; }
int z_deflateEnd(void* strm) { (void)strm; return 0; }
int z_deflateInit_(void* strm, int level, const char* version, int stream_size) {
    (void)strm; (void)level; (void)version; (void)stream_size; return 0;
}
int z_inflate(void* strm, int flush) { (void)strm; (void)flush; return 0; }
int z_inflateEnd(void* strm) { (void)strm; return 0; }
int z_inflateInit_(void* strm, const char* version, int stream_size) {
    (void)strm; (void)version; (void)stream_size; return 0;
}
int z_inflateInit2_(void* strm, int windowBits, const char* version, int stream_size) {
    (void)strm; (void)windowBits; (void)version; (void)stream_size; return 0;
}
unsigned long z_compressBound(unsigned long sourceLen) { (void)sourceLen; return 0; }
int z_uncompress(Bytef* dest, uLong* destLen, const Bytef* source, uLong sourceLen) {
    (void)dest; (void)destLen; (void)source; (void)sourceLen; return 0;
}

// ── POSIX stubs missing from posix_stubs.c ──────────────────────────────────

int clock_getres(int clk_id, void* res) { (void)clk_id; (void)res; return 0; }
int ppoll(void* fds, unsigned long nfds, const void* timeout, const void* sigmask) {
    (void)fds; (void)nfds; (void)timeout; (void)sigmask; return 0;
}
int pipe2(int pipefd[2], int flags) {
    (void)flags;
    // Use the kernel SYS_PIPE syscall (number 10) via x86_64 syscall instruction
    long ret;
    asm volatile (
        "syscall"
        : "=a" (ret)
        : "a" (10), "D" ((long)pipefd), "S" (0), "d" (0)
        : "rcx", "r11", "memory"
    );
    return (int)ret;
}
int eventfd(unsigned int initval, int flags) { (void)initval; (void)flags; return -1; }
int linkat(int olddirfd, const char* oldpath, int newdirfd, const char* newpath, int flags) {
    (void)olddirfd; (void)oldpath; (void)newdirfd; (void)newpath; (void)flags; return -1;
}
int sched_yield(void) { return 0; }
int times(void* buf) { (void)buf; return 0; }
void perror(const char* s) { (void)s; }
int getentropy(void* buffer, unsigned long length) { (void)buffer; (void)length; return 0; }
void* getauxval(unsigned long type) { (void)type; return 0; }
struct tm* gmtime_r(const void* timer, void* buf) { (void)timer; (void)buf; return 0; }

int madvise(void* addr, unsigned long length, int advice) { (void)addr; (void)length; (void)advice; return 0; }
int mprotect(void* addr, unsigned long length, int prot) { (void)addr; (void)length; (void)prot; return 0; }

// ── POSIX stubs needed by real Qt6 implementations ──────────────────────────

int fileno(void* stream) { (void)stream; return -1; }
int ftruncate64(int fd, long long length) { (void)fd; (void)length; return -1; }
long long truncate64(const char* path, long long length) { (void)path; (void)length; return -1; }
int flock(int fd, int operation) { (void)fd; (void)operation; return 0; }
int fdatasync(int fd) { (void)fd; return 0; }
int fchmod(int fd, unsigned int mode) { (void)fd; (void)mode; return -1; }
int chmod(const char* path, unsigned int mode) { (void)path; (void)mode; return -1; }
int symlink(const char* target, const char* linkpath) { (void)target; (void)linkpath; return -1; }
int futimens(int fd, const void* times) { (void)fd; (void)times; return -1; }
int renameat2(int olddirfd, const char* oldpath, int newdirfd, const char* newpath, unsigned int flags) {
    (void)olddirfd; (void)oldpath; (void)newdirfd; (void)newpath; (void)flags; return -1;
}
int link(const char* oldpath, const char* newpath) { (void)oldpath; (void)newpath; return -1; }
int lstat64(const char* pathname, void* statbuf) { (void)pathname; (void)statbuf; return -1; }
char* realpath(const char* path, char* resolved_path) { (void)path; (void)resolved_path; return 0; }
int fgetc(void* stream) { (void)stream; return -1; }
unsigned long getpagesize(void) { return 4096; }

struct statx { int dummy; };
int statx(int dirfd, const char* pathname, unsigned flags, unsigned mask, struct statx* buf) {
    (void)dirfd; (void)pathname; (void)flags; (void)mask; (void)buf; return -1;
}
long long sendfile(int out_fd, int in_fd, long long* offset, unsigned long count) {
    (void)out_fd; (void)in_fd; (void)offset; (void)count; return -1;
}

// ── sched / pthread stubs ───────────────────────────────────────────────────

int sched_getaffinity(int pid, unsigned int cpusetsize, void* cpuset) {
    (void)pid; (void)cpusetsize; (void)cpuset; return -1;
}
int __sched_cpucount(unsigned long cpusetsize, const void* cpuset) {
    (void)cpusetsize; (void)cpuset; return 1;
}
int sched_get_priority_min(int policy) { (void)policy; return 0; }
int sched_get_priority_max(int policy) { (void)policy; return 0; }

int pthread_attr_setdetachstate(void* attr, int detachstate) { (void)attr; (void)detachstate; return 0; }
int pthread_attr_getschedpolicy(const void* attr, int* policy) { (void)attr; if (policy) *policy = 0; return 0; }
int pthread_attr_setinheritsched(void* attr, int inherit) { (void)attr; (void)inherit; return 0; }
int pthread_attr_setschedpolicy(void* attr, int policy) { (void)attr; (void)policy; return 0; }
int pthread_attr_setstacksize(void* attr, unsigned long stacksize) { (void)attr; (void)stacksize; return 0; }
int pthread_attr_setschedparam(void* attr, const void* param) { (void)attr; (void)param; return 0; }
int pthread_getschedparam(int thread, int* policy, void* param) {
    (void)thread; if (policy) *policy = 0; (void)param; return 0;
}
int pthread_setschedparam(int thread, int policy, const void* param) {
    (void)thread; (void)policy; (void)param; return 0;
}
int pthread_setcancelstate(int state, int* oldstate) {
    (void)state; if (oldstate) *oldstate = 0; return 0;
}
int pthread_testcancel(void) { return 0; }

int pthread_condattr_init(void* attr) { (void)attr; return 0; }
int pthread_condattr_destroy(void* attr) { (void)attr; return 0; }
int pthread_condattr_setclock(void* attr, int clockid) { (void)attr; (void)clockid; return 0; }

int prctl(int option, ...) { (void)option; return 0; }

// ── User/group lookup stubs (needed by QFileSystemEngine) ───────────────────

struct passwd { char* pw_name; char* pw_passwd; int pw_uid; int pw_gid; char* pw_gecos; char* pw_dir; char* pw_shell; };
struct group { char* gr_name; char* gr_passwd; int gr_gid; char** gr_mem; };

int getpwuid_r(int uid, struct passwd* pwd, char* buf, unsigned long buflen, struct passwd** result) {
    (void)uid; (void)pwd; (void)buf; (void)buflen;
    if (result) *result = 0;
    return 0;
}

int getgrgid_r(int gid, struct group* grp, char* buf, unsigned long buflen, struct group** result) {
    (void)gid; (void)grp; (void)buf; (void)buflen;
    if (result) *result = 0;
    return 0;
}

// ── Math stubs missing from posix_stubs.c ───────────────────────────────────

double acosh(double x) { (void)x; return 0; }
double asinh(double x) { (void)x; return 0; }
double atanh(double x) { (void)x; return 0; }
double cbrt(double x) { (void)x; return 0; }
double expm1(double x) { (void)x; return 0; }
double log1p(double x) { (void)x; return 0; }

// ── __strcat_chk (fortified) ────────────────────────────────────────────────

char* __strcat_chk(char* dest, const char* src, unsigned long destlen) {
    (void)destlen;
    char* d = dest;
    while (*d) d++;
    while ((*d++ = *src++));
    return dest;
}

// ── eventfd_read/write stubs ────────────────────────────────────────────────

typedef struct { unsigned long long val; } eventfd_t;
int eventfd_read(int fd, eventfd_t* value) { (void)fd; if (value) value->val = 0; return -1; }
int eventfd_write(int fd, unsigned long long value) { (void)fd; (void)value; return -1; }

// ── backtrace stub ──────────────────────────────────────────────────────────

int backtrace(void** buffer, int size) { (void)buffer; (void)size; return 0; }
char** backtrace_symbols(void* const* buffer, int size) { (void)buffer; (void)size; return 0; }
void backtrace_symbols_fd(void* const* buffer, int size, int fd) { (void)buffer; (void)size; (void)fd; }

// ── HarfBuzz stubs ──────────────────────────────────────────────────────────

typedef void* hb_blob_t;
typedef void* hb_face_t;
typedef void* hb_font_t;
typedef void* hb_buffer_t;
typedef void* hb_unicode_funcs_t;
typedef void* hb_font_funcs_t;

hb_blob_t* hb_blob_create(const char* data, unsigned int len, int mode, void* user_data, void* destroy) {
    (void)data; (void)len; (void)mode; (void)user_data; (void)destroy; return nullptr;
}
hb_blob_t* hb_blob_get_empty() { return nullptr; }
hb_buffer_t* hb_buffer_create() { return nullptr; }
hb_buffer_t* hb_buffer_pre_allocate(hb_buffer_t* b, int size) { (void)b; (void)size; return nullptr; }
int hb_buffer_allocation_successful(hb_buffer_t* b) { (void)b; return 0; }
void hb_buffer_clear_contents(hb_buffer_t* b) { (void)b; }
void hb_buffer_add_utf16(hb_buffer_t* b, const unsigned short* text, int length, unsigned int offset, unsigned int count) {
    (void)b; (void)text; (void)length; (void)offset; (void)count;
}
void hb_buffer_set_segment_properties(hb_buffer_t* b, void* seg) { (void)b; (void)seg; }
void hb_buffer_set_flags(hb_buffer_t* b, unsigned int flags) { (void)b; (void)flags; }
void hb_buffer_set_unicode_funcs(hb_buffer_t* b, hb_unicode_funcs_t* uf) { (void)b; (void)uf; }
hb_unicode_funcs_t* hb_unicode_funcs_create(void* parent) { (void)parent; return nullptr; }
void hb_unicode_funcs_destroy(hb_unicode_funcs_t* uf) { (void)uf; }
void hb_unicode_funcs_set_combining_class_func(hb_unicode_funcs_t* uf, void* func, void* data, void* destroy) {
    (void)uf; (void)func; (void)data; (void)destroy;
}
void hb_unicode_funcs_set_compose_func(hb_unicode_funcs_t* uf, void* func, void* data, void* destroy) {
    (void)uf; (void)func; (void)data; (void)destroy;
}
void hb_unicode_funcs_set_decompose_func(hb_unicode_funcs_t* uf, void* func, void* data, void* destroy) {
    (void)uf; (void)func; (void)data; (void)destroy;
}
void hb_unicode_funcs_set_general_category_func(hb_unicode_funcs_t* uf, void* func, void* data, void* destroy) {
    (void)uf; (void)func; (void)data; (void)destroy;
}
void hb_unicode_funcs_set_mirroring_func(hb_unicode_funcs_t* uf, void* func, void* data, void* destroy) {
    (void)uf; (void)func; (void)data; (void)destroy;
}
void hb_unicode_funcs_set_script_func(hb_unicode_funcs_t* uf, void* func, void* data, void* destroy) {
    (void)uf; (void)func; (void)data; (void)destroy;
}
hb_face_t* hb_face_create_for_tables(void* func, void* user_data) { (void)func; (void)user_data; return nullptr; }
void hb_face_destroy(hb_face_t* f) { (void)f; }
void hb_face_set_index(hb_face_t* f, unsigned int idx) { (void)f; (void)idx; }
void hb_face_set_upem(hb_face_t* f, unsigned int upem) { (void)f; (void)upem; }
hb_font_t* hb_font_create(hb_face_t* face) { (void)face; return nullptr; }
void hb_font_destroy(hb_font_t* f) { (void)f; }
hb_font_funcs_t* hb_font_funcs_create(void* alloc) { (void)alloc; return nullptr; }
void hb_font_funcs_destroy(hb_font_funcs_t* ff) { (void)ff; }
void hb_font_funcs_make_immutable(hb_font_funcs_t* ff) { (void)ff; }
void hb_font_funcs_set_font_h_extents_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void hb_font_funcs_set_glyph_contour_point_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void hb_font_funcs_set_glyph_extents_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void hb_font_funcs_set_glyph_h_advance_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void hb_font_funcs_set_glyph_h_kerning_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void hb_font_funcs_set_nominal_glyph_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void hb_font_funcs_set_variation_glyph_func(hb_font_funcs_t* ff, void* func, void* data, void* destroy) {
    (void)ff; (void)func; (void)data; (void)destroy;
}
void* hb_font_get_user_data(hb_font_t* f, void* key) { (void)f; (void)key; return nullptr; }
void hb_font_set_funcs(hb_font_t* font, hb_font_funcs_t* ff, void* font_data, void* destroy) {
    (void)font; (void)ff; (void)font_data; (void)destroy;
}
void hb_font_set_ppem(hb_font_t* font, unsigned int x_ppem, unsigned int y_ppem) { (void)font; (void)x_ppem; (void)y_ppem; }
void hb_font_set_ptem(hb_font_t* font, float ptem) { (void)font; (void)ptem; }
void hb_font_set_scale(hb_font_t* font, int x_scale, int y_scale) { (void)font; (void)x_scale; (void)y_scale; }
void* hb_font_set_user_data(hb_font_t* font, void* key, void* data, void* destroy, int replace) {
    (void)font; (void)key; (void)data; (void)destroy; (void)replace; return nullptr;
}
void* hb_language_get_default() { return nullptr; }
void hb_ot_layout_table_select_script(void* face, unsigned int table_tag, unsigned int script_index, void* script_tag, unsigned int* index) {
    (void)face; (void)table_tag; (void)script_index; (void)script_tag; if (index) *index = 0;
}
void hb_ot_tags_from_script_and_language(void* script, void* language, unsigned int* script_count, unsigned int* scripts, unsigned int* language_count, unsigned int* languages) {
    (void)script; (void)language;
    if (script_count) *script_count = 0;
    if (language_count) *language_count = 0;
    (void)scripts; (void)languages;
}
void hb_shape_full(hb_font_t* font, hb_buffer_t* buffer, void* feature_list, unsigned int feature_count) {
    (void)font; (void)buffer; (void)feature_list; (void)feature_count;
}
unsigned int hb_buffer_get_length(hb_buffer_t* b) { (void)b; return 0; }
void* hb_buffer_get_glyph_infos(hb_buffer_t* b, unsigned int* len) { (void)b; if (len) *len = 0; return nullptr; }
void* hb_buffer_get_glyph_positions(hb_buffer_t* b, unsigned int* len) { (void)b; if (len) *len = 0; return nullptr; }
void hb_buffer_destroy(hb_buffer_t* b) { (void)b; }
void hb_buffer_reverse(hb_buffer_t* b) { (void)b; }

// ── PCRE2 stubs (16-bit) ───────────────────────────────────────────────────

typedef void pcre2_code_16;
typedef void pcre2_match_data_16;
typedef void pcre2_match_context_16;
typedef void pcre2_jit_stack_16;

pcre2_code_16* pcre2_compile_16(const unsigned short* pattern, int length, unsigned int options, int* error_code, unsigned int* error_offset, void* gctx) {
    (void)pattern; (void)length; (void)options; if (error_code) *error_code = 0; if (error_offset) *error_offset = 0; (void)gctx; return nullptr;
}
void pcre2_code_free_16(pcre2_code_16* code) { (void)code; }
int pcre2_config_16(int what, void* where) { (void)what; (void)where; return 0; }
int pcre2_get_error_message_16(int error_code, unsigned short* buffer, int buffer_len) {
    (void)error_code; (void)buffer; (void)buffer_len; return -1;
}
unsigned int* pcre2_get_ovector_pointer_16(pcre2_match_data_16* md) { (void)md; return nullptr; }
int pcre2_jit_compile_16(pcre2_code_16* code, unsigned int options) { (void)code; (void)options; return -1; }
void pcre2_jit_stack_assign_16(pcre2_match_context_16* mctx, void* callback, void* user_data) {
    (void)mctx; (void)callback; (void)user_data;
}
pcre2_jit_stack_16* pcre2_jit_stack_create_16(int startsize, int maxsize, void* gctx) {
    (void)startsize; (void)maxsize; (void)gctx; return nullptr;
}
void pcre2_jit_stack_free_16(pcre2_jit_stack_16* stack) { (void)stack; }
int pcre2_match_16(const pcre2_code_16* code, const unsigned short* subject, int length,
    unsigned int startoffset, unsigned int options, pcre2_match_data_16* match_data, pcre2_match_context_16* mctx) {
    (void)code; (void)subject; (void)length; (void)startoffset; (void)options; (void)match_data; (void)mctx; return -1;
}
pcre2_match_context_16* pcre2_match_context_create_16(void* gctx) { (void)gctx; return nullptr; }
void pcre2_match_context_free_16(pcre2_match_context_16* mctx) { (void)mctx; }
pcre2_match_data_16* pcre2_match_data_create_from_pattern_16(const pcre2_code_16* code, void* gctx) {
    (void)code; (void)gctx; return nullptr;
}
void pcre2_match_data_free_16(pcre2_match_data_16* md) { (void)md; }
int pcre2_pattern_info_16(const pcre2_code_16* code, unsigned int what, void* where) {
    (void)code; (void)what; (void)where; return -1;
}

// ── libpng stubs ────────────────────────────────────────────────────────────

struct png_struct_def { int dummy; };
struct png_info_def { int dummy; };
typedef struct png_struct_def* png_structp;
typedef struct png_info_def* png_infop;
typedef unsigned char png_byte;

void png_error(png_structp png_ptr, const char* error_msg) { (void)png_ptr; (void)error_msg; }
void png_set_error_fn(png_structp png_ptr, void* error_ptr, void* error_fn, void* warning_fn) {
    (void)png_ptr; (void)error_ptr; (void)error_fn; (void)warning_fn;
}
void png_set_longjmp_fn(png_structp png_ptr, void* longjmp_fn, int jump_size) { (void)png_ptr; (void)longjmp_fn; (void)jump_size; }

png_structp png_create_read_struct(const char* user_png_ver, void* error_ptr, void* error_fn, void* warning_fn) {
    (void)user_png_ver; (void)error_ptr; (void)error_fn; (void)warning_fn; return nullptr;
}
png_structp png_create_write_struct(const char* user_png_ver, void* error_ptr, void* error_fn, void* warning_fn) {
    (void)user_png_ver; (void)error_ptr; (void)error_fn; (void)warning_fn; return nullptr;
}
png_infop png_create_info_struct(png_structp png_ptr) { (void)png_ptr; return nullptr; }
void png_destroy_read_struct(png_structp* png_ptr_ptr, png_infop* info_ptr_ptr, png_infop* end_info_ptr) {
    (void)png_ptr_ptr; (void)info_ptr_ptr; (void)end_info_ptr;
}
void png_destroy_write_struct(png_structp* png_ptr_ptr, png_infop* info_ptr_ptr) {
    (void)png_ptr_ptr; (void)info_ptr_ptr;
}
void* png_get_io_ptr(png_structp png_ptr) { (void)png_ptr; return nullptr; }
void png_set_read_fn(png_structp png_ptr, void* io_ptr, void* read_data_fn) { (void)png_ptr; (void)io_ptr; (void)read_data_fn; }
void png_set_write_fn(png_structp png_ptr, void* io_ptr, void* write_data_fn, void* output_flush_fn) {
    (void)png_ptr; (void)io_ptr; (void)write_data_fn; (void)output_flush_fn;
}
void png_set_sig_bytes(png_structp png_ptr, int num_bytes) { (void)png_ptr; (void)num_bytes; }
void png_read_info(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; }
void png_read_image(png_structp png_ptr, png_byte** row_pointers) { (void)png_ptr; (void)row_pointers; }
void png_read_row(png_structp png_ptr, png_byte* row, png_byte* sparse_row) { (void)png_ptr; (void)row; (void)sparse_row; }
void png_read_end(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; }
void png_read_update_info(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; }
void png_write_info(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; }
void png_write_image(png_structp png_ptr, png_byte** image) { (void)png_ptr; (void)image; }
void png_write_end(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; }
void png_write_rows(png_structp png_ptr, png_byte** row_pointers, unsigned int num_rows) {
    (void)png_ptr; (void)row_pointers; (void)num_rows;
}
void png_write_chunk(png_structp png_ptr, const unsigned char* chunk_name, const png_byte* data, unsigned int length) {
    (void)png_ptr; (void)chunk_name; (void)data; (void)length;
}
int png_get_IHDR(png_structp png_ptr, png_infop info_ptr, unsigned int* width, unsigned int* height,
    int* bit_depth, int* color_type, int* interlace_method, int* compression_method, int* filter_method) {
    (void)png_ptr; (void)info_ptr;
    if (width) *width = 0; if (height) *height = 0;
    if (bit_depth) *bit_depth = 8; if (color_type) *color_type = 2;
    if (interlace_method) *interlace_method = 0;
    if (compression_method) *compression_method = 0; if (filter_method) *filter_method = 0;
    return 0;
}
void png_set_IHDR(png_structp png_ptr, png_infop info_ptr, unsigned int width, unsigned int height,
    int bit_depth, int color_type, int interlace_method, int compression_method, int filter_method) {
    (void)png_ptr; (void)info_ptr; (void)width; (void)height;
    (void)bit_depth; (void)color_type; (void)interlace_method; (void)compression_method; (void)filter_method;
}
unsigned int png_get_image_width(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 0; }
unsigned int png_get_image_height(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 0; }
int png_get_bit_depth(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 8; }
int png_get_color_type(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 2; }
int png_get_channels(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 3; }
unsigned int png_get_valid(png_structp png_ptr, png_infop info_ptr, unsigned int flag) { (void)png_ptr; (void)info_ptr; (void)flag; return 0; }
void png_set_PLTE(png_structp png_ptr, png_infop info_ptr, void* palette, int num_palette) {
    (void)png_ptr; (void)info_ptr; (void)palette; (void)num_palette;
}
int png_get_PLTE(png_structp png_ptr, png_infop info_ptr, void** palette, int* num_palette) {
    (void)png_ptr; (void)info_ptr; if (palette) *palette = nullptr; if (num_palette) *num_palette = 0; return 0;
}
int png_get_tRNS(png_structp png_ptr, png_infop info_ptr, unsigned char** trans, int* num_trans, void** trans_values) {
    (void)png_ptr; (void)info_ptr; if (trans) *trans = nullptr; if (num_trans) *num_trans = 0; if (trans_values) *trans_values = nullptr; return 0;
}
void png_set_tRNS(png_structp png_ptr, png_infop info_ptr, const unsigned char* trans, int num_trans, void* trans_values) {
    (void)png_ptr; (void)info_ptr; (void)trans; (void)num_trans; (void)trans_values;
}
int png_get_cHRM(png_structp png_ptr, png_infop info_ptr, void* white_x, void* white_y, void* red_x, void* red_y,
    void* green_x, void* green_y, void* blue_x, void* blue_y) {
    (void)png_ptr; (void)info_ptr;
    (void)white_x; (void)white_y; (void)red_x; (void)red_y;
    (void)green_x; (void)green_y; (void)blue_x; (void)blue_y;
    return 0;
}
int png_get_gAMA(png_structp png_ptr, png_infop info_ptr, double* gamma) { (void)png_ptr; (void)info_ptr; if (gamma) *gamma = 1.0; return 0; }
void png_set_gAMA(png_structp png_ptr, png_infop info_ptr, double gamma) { (void)png_ptr; (void)info_ptr; (void)gamma; }
int png_get_iCCP(png_structp png_ptr, png_infop info_ptr, void** profile_name, int* compression_type, unsigned char** profile, unsigned int* profile_length) {
    (void)png_ptr; (void)info_ptr;
    if (profile_name) *profile_name = nullptr; if (compression_type) *compression_type = 0;
    if (profile) *profile = nullptr; if (profile_length) *profile_length = 0; return 0;
}
void png_set_iCCP(png_structp png_ptr, png_infop info_ptr, const void* name, int compression_type, const unsigned char* profile, unsigned int profile_length) {
    (void)png_ptr; (void)info_ptr; (void)name; (void)compression_type; (void)profile; (void)profile_length;
}
int png_get_sRGB(png_structp png_ptr, png_infop info_ptr, int* srgb_intent) { (void)png_ptr; (void)info_ptr; if (srgb_intent) *srgb_intent = 0; return 0; }
void png_set_sRGB(png_structp png_ptr, png_infop info_ptr, int intent) { (void)png_ptr; (void)info_ptr; (void)intent; }
int png_get_oFFs(png_structp png_ptr, png_infop info_ptr, int* offset_x, int* offset_y, int* unit_type) {
    (void)png_ptr; (void)info_ptr;
    if (offset_x) *offset_x = 0; if (offset_y) *offset_y = 0; if (unit_type) *unit_type = 0; return 0;
}
void png_set_oFFs(png_structp png_ptr, png_infop info_ptr, int offset_x, int offset_y, int unit_type) {
    (void)png_ptr; (void)info_ptr; (void)offset_x; (void)offset_y; (void)unit_type;
}
int png_get_pHYs(png_structp png_ptr, png_infop info_ptr, unsigned int* res_x, unsigned int* res_y, int* unit_type) {
    (void)png_ptr; (void)info_ptr; if (res_x) *res_x = 0; if (res_y) *res_y = 0; if (unit_type) *unit_type = 0; return 0;
}
void png_set_pHYs(png_structp png_ptr, png_infop info_ptr, unsigned int res_x, unsigned int res_y, int unit_type) {
    (void)png_ptr; (void)info_ptr; (void)res_x; (void)res_y; (void)unit_type;
}
int png_get_x_pixels_per_meter(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 0; }
int png_get_y_pixels_per_meter(png_structp png_ptr, png_infop info_ptr) { (void)png_ptr; (void)info_ptr; return 0; }
int png_get_text(png_structp png_ptr, png_infop info_ptr, void** text_ptr, int* num_text) {
    (void)png_ptr; (void)info_ptr; if (text_ptr) *text_ptr = nullptr; if (num_text) *num_text = 0; return 0;
}
void png_set_text(png_structp png_ptr, png_infop info_ptr, void* text_ptr, int num_text) {
    (void)png_ptr; (void)info_ptr; (void)text_ptr; (void)num_text;
}
int png_set_interlace_handling(png_structp png_ptr) { (void)png_ptr; return 0; }
void png_set_expand(png_structp png_ptr) { (void)png_ptr; }
void png_set_strip_16(png_structp png_ptr) { (void)png_ptr; }
void png_set_gray_to_rgb(png_structp png_ptr) { (void)png_ptr; }
void png_set_filler(png_structp png_ptr, unsigned char filler, int filler_loc) { (void)png_ptr; (void)filler; (void)filler_loc; }
void png_set_add_alpha(png_structp png_ptr, unsigned char filler, int filler_loc) { (void)png_ptr; (void)filler; (void)filler_loc; }
void png_set_swap(png_structp png_ptr) { (void)png_ptr; }
void png_set_packing(png_structp png_ptr) { (void)png_ptr; }
void png_set_packswap(png_structp png_ptr) { (void)png_ptr; }
void png_set_bgr(png_structp png_ptr) { (void)png_ptr; }
void png_set_invert_mono(png_structp png_ptr) { (void)png_ptr; }
void png_set_swap_alpha(png_structp png_ptr) { (void)png_ptr; }
void png_set_tRNS_to_alpha(png_structp png_ptr) { (void)png_ptr; }
void png_set_gamma(png_structp png_ptr, double screen_gamma, void* sRGB_inverse) { (void)png_ptr; (void)screen_gamma; (void)sRGB_inverse; }
void png_set_option(png_structp png_ptr, int option, int onoff) { (void)png_ptr; (void)option; (void)onoff; }
void png_set_benign_errors(png_structp png_ptr, int allowed) { (void)png_ptr; (void)allowed; }
void png_set_compression_level(png_structp png_ptr, int level) { (void)png_ptr; (void)level; }
void png_set_compression_strategy(png_structp png_ptr, int strategy) { (void)png_ptr; (void)strategy; }
void png_set_compression_mem_level(png_structp png_ptr, int mem_level) { (void)png_ptr; (void)mem_level; }
void png_set_compression_buffer_size(png_structp png_ptr, unsigned int buffer_size) { (void)png_ptr; (void)buffer_size; }
void png_set_filter(png_structp png_ptr, int method, int filters) { (void)png_ptr; (void)method; (void)filters; }
void png_set_shift(png_structp png_ptr, void* shift) { (void)png_ptr; (void)shift; }
void png_set_interlace(png_structp png_ptr) { (void)png_ptr; }
void png_set_flush(png_structp png_ptr, int num_rows) { (void)png_ptr; (void)num_rows; }
void png_write_png(png_structp png_ptr, png_infop info_ptr, int transforms, void* params) { (void)png_ptr; (void)info_ptr; (void)transforms; (void)params; }
png_structp png_create_read_struct_2(const char* ver, void* err_ptr, void* err_fn, void* warn_fn, void* mem_ptr, void* malloc_fn, void* free_fn) {
    (void)ver; (void)err_ptr; (void)err_fn; (void)warn_fn; (void)mem_ptr; (void)malloc_fn; (void)free_fn; return nullptr;
}
png_structp png_create_write_struct_2(const char* ver, void* err_ptr, void* err_fn, void* warn_fn, void* mem_ptr, void* malloc_fn, void* free_fn) {
    (void)ver; (void)err_ptr; (void)err_fn; (void)warn_fn; (void)mem_ptr; (void)malloc_fn; (void)free_fn; return nullptr;
}
void png_set_cHRM(png_structp png_ptr, png_infop info_ptr, double white_x, double white_y, double red_x, double red_y,
    double green_x, double green_y, double blue_x, double blue_y) {
    (void)png_ptr; (void)info_ptr; (void)white_x; (void)white_y; (void)red_x; (void)red_y;
    (void)green_x; (void)green_y; (void)blue_x; (void)blue_y;
}
void png_set_IHDR_2(png_structp png_ptr, png_infop info_ptr, unsigned int width, unsigned int height,
    int bit_depth, int color_type, int interlace_method, int compression_method, int filter_method) {
    (void)png_ptr; (void)info_ptr; (void)width; (void)height;
    (void)bit_depth; (void)color_type; (void)interlace_method; (void)compression_method; (void)filter_method;
}
int png_get_sPLT(png_structp png_ptr, png_infop info_ptr, void** spalettes) { (void)png_ptr; (void)info_ptr; if (spalettes) *spalettes = nullptr; return 0; }
void png_set_sPLT(png_structp png_ptr, png_infop info_ptr, void* palettes, int num_palettes) {
    (void)png_ptr; (void)info_ptr; (void)palettes; (void)num_palettes;
}
void png_set_chunk_malloc_max(png_structp png_ptr, unsigned long user_chunk_malloc_max) { (void)png_ptr; (void)user_chunk_malloc_max; }
void png_set_keep_unknown_chunks(png_structp png_ptr, int keep, void* chunk_list, unsigned int num_chunks) {
    (void)png_ptr; (void)keep; (void)chunk_list; (void)num_chunks;
}
void png_set_read_user_chunk_fn(png_structp png_ptr, void* user_chunk_ptr, void* read_user_chunk_fn, void** unknown_chunk_copy) {
    (void)png_ptr; (void)user_chunk_ptr; (void)read_user_chunk_fn; (void)unknown_chunk_copy;
}

// ── zlib deflate stubs ──────────────────────────────────────────────────────

unsigned long compressBound(unsigned long sourceLen) { (void)sourceLen; return 0; }

// ── stdio with __vfprintf_chk ──────────────────────────────────────────────
int __vfprintf_chk(void* /*stream*/, int /*flag*/, const char* format, ...) {
    (void)format; return 0;
}
int fseeko64(void* stream, long long offset, int whence) { (void)stream; (void)offset; (void)whence; return -1; }
long long ftello64(void* stream) { (void)stream; return -1; }

// ── Math stubs (C-linkage but may be called from C++) ──────────────────────
float asinf(float x) { (void)x; return 0.0f; }
float atan2f(float y, float x) { (void)y; (void)x; return 0.0f; }
void sincosf(float x, float* sinval, float* cosval) { (void)x; if (sinval) *sinval = 0.0f; if (cosval) *cosval = 1.0f; }
float sqrtf(float x) { (void)x; return 0.0f; }

} // extern "C" (math stubs)

// ── C++ RTTI / typeinfo stubs ──────────────────────────────────────────────

extern "C" void* __dynamic_cast(const void*, const void*, const void*, long) { return nullptr; }

extern "C" void* _ZTVN10__cxxabiv117__class_type_infoE[];
extern "C" void* _ZTVN10__cxxabiv120__si_class_type_infoE[];

extern "C" {
    // QEvent (root)
    static const char _ts_6QEvent[] = "6QEvent";
    void* _ZTI6QEvent[2] = {
        (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
        (void*)_ts_6QEvent
    };

    // QThread : QObject
    static const char _ts_7QThread[] = "7QThread";
    extern void* _ZTI7QObject[];
    void* _ZTI7QThread[3] = {
        (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
        (void*)_ts_7QThread,
        (void*)_ZTI7QObject
    };

    // QPlatformCursor (root)
    static const char _ts_15QPlatformCursor[] = "15QPlatformCursor";
    void* _ZTI15QPlatformCursor[2] = {
        (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
        (void*)_ts_15QPlatformCursor
    };

    // QDynamicMetaObjectData (root)
    static const char _ts_24QDynamicMetaObjectData[] = "24QDynamicMetaObjectData";
    void* _ZTI24QDynamicMetaObjectData[2] = {
        (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
        (void*)_ts_24QDynamicMetaObjectData
    };

    // QAbstractDynamicMetaObject : QDynamicMetaObjectData
    static const char _ts_27QAbstractDynamicMetaObject[] = "27QAbstractDynamicMetaObject";
    void* _ZTI27QAbstractDynamicMetaObject[3] = {
        (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
        (void*)_ts_27QAbstractDynamicMetaObject,
        (void*)_ZTI24QDynamicMetaObjectData
    };

    // QNativeInterface::Private::QEvdevKeyMapper (root)
    static const char _ts_QEvdevKeyMapper[] = "N16QNativeInterface7Private15QEvdevKeyMapperE";
    void* _ZTIN16QNativeInterface7Private15QEvdevKeyMapperE[2] = {
        (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
        (void*)_ts_QEvdevKeyMapper
    };
}

// ── QElapsedTimer — REMOVED: real implementation in libQt6Core.a ────────────

// ── QFSFileEngine, QFileSystemEngine, QLockFilePrivate, QThread, QAdoptedThread
// REMOVED: all shadowing stubs deleted to let real implementations from
// libQt6Core.a be linked instead. The GNU linker resolves .o before .a
// archives, so these were preventing the real code from being used.

// ── QtGenericUnixDispatcher ─────────────────────────────────────────────────
// REMOVED: The real createUnixEventDispatcher() from libQt6Core.a should be
// linked instead of this nullptr stub. If the real one fails to link, we'll
// need to provide a working implementation.

// ── vtable for QPlatformNativeInterface (weak — real vtable in libQt6Gui) ───
extern "C" {
    extern void* _ZTVN10__cxxabiv117__class_type_infoE[];
    static const char _ts_24QPlatformNativeInterface[] = "24QPlatformNativeInterface";
    __attribute__((weak)) void* _ZTI24QPlatformNativeInterface[2] = {
        (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
        (void*)_ts_24QPlatformNativeInterface
    };
}
__attribute__((weak)) void* _ZTV24QPlatformNativeInterface[3] = {
    nullptr,
    nullptr,
    nullptr
};

// ── QQmlApplicationEngine stubs ────────────────────────────────────────────
// REMOVED: The real QQmlApplicationEngine implementations from libQt6Qml.a
// should be linked. These weak stubs were shadowing the real constructors,
// destructor, and loadData(), causing the QML engine to do nothing.
