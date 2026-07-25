// ── Qt6 Platform Stubs (for CMAKE_SYSTEM_NAME=Generic builds) ────────────
// These provide the platform-specific functions that are normally compiled
// from Unix/*nix-specific source files. They exist only to satisfy the
// linker and should NOT be called at runtime.
//
// Compiled with: x86_64-elf-g++-qt6 -m64 -std=c++17 -fno-rtti -fno-exceptions

#include <cstddef>
#include <cstdint>

extern "C" {

// ── QFileSystemIterator ──────────────────────────────────────────────────
// Normally in qfilesystemiterator_unix.cpp

void _ZN19QFileSystemIteratorC1ERK16QFileSystemEntry6QFlagsIN4QDir6FilterEERK5QListI7QStringES3_IN12QDirIterator12IteratorFlagEE(
    void* this_, void* entry, int filters, void* nameFilters, int flags) {
    (void)this_; (void)entry; (void)filters; (void)nameFilters; (void)flags;
}

void _ZN19QFileSystemIteratorD1Ev(void* this_) {
    (void)this_;
}

int _ZN19QFileSystemIterator7advanceER16QFileSystemEntryR19QFileSystemMetaData(
    void* this_, void* entry, void* metaData) {
    (void)this_; (void)entry; (void)metaData;
    return 0;
}

// ── QFileSystemEngine (REMOVED) ────────────────────────────────────────────
// All QFileSystemEngine stubs removed because they shadow real implementations
// in libQt6Core.a. The GNU linker resolves .o before .a archives.

// ── QStandardPaths ───────────────────────────────────────────────────────
// Normally in qstandardpaths_unix.cpp

void _ZN14QStandardPaths16writableLocationENS_16StandardLocationE(
    void* ret, int location) {
    (void)ret; (void)location;
}

void _ZN14QStandardPaths17standardLocationsENS_16StandardLocationE(
    void* ret, int location) {
    (void)ret; (void)location;
}

// ── QFSFileEngine (REMOVED) ───────────────────────────────────────────────
// All QFSFileEngine stubs removed because they shadow real implementations
// in libQt6Core.a. The GNU linker resolves .o before .a archives.

// ── QFSFileEnginePrivate (REMOVED) ────────────────────────────────────────
// All QFSFileEnginePrivate stubs removed because they shadow real implementations
// in libQt6Core.a. The GNU linker resolves .o before .a archives.

// ── QThread / QThreadData / QThreadPrivate (REMOVED) ──────────────────────
// QThreadPrivate::createEventDispatcher and QThreadPrivate::setPriority
// removed because they shadow real implementations in libQt6Core.a.

// ── QSystemLocale ────────────────────────────────────────────────────────
// Normally in qsystemlocale_unix.cpp

void _ZNK13QSystemLocale14fallbackLocaleEv(void* ret, void* this_) {
    (void)ret; (void)this_;
}

void _ZNK13QSystemLocale5queryENS_9QueryTypeE8QVariant(
    void* ret, void* this_, int type, void* variant) {
    (void)ret; (void)this_; (void)type; (void)variant;
}

// ── QTzTimeZonePrivate (REMOVED) ──────────────────────────────────────────
// QTzTimeZonePrivate constructors removed because they shadow real
// implementations in libQt6Core.a.

// ── qt_readlink — REMOVED: real implementation in libQt6Core.a ─────────────
// Normally in qfilesystemengine_unix.cpp (wraps POSIX readlink)

// ── Plugin registration ──────────────────────────────────────────────────
// Called by Q_IMPORT_PLUGIN(AlloyIntegrationPlugin)
// TODO: actually initialize and register the AlloyIntegrationPlugin
// Uses C++-mangled name because Q_IMPORT_PLUGIN generates a C++ call, not extern "C"

void _Z39qt_static_plugin_AlloyIntegrationPluginv() {
}

// ── Qt class type_info objects ──────────────────────────────────────────────
// These are normally emitted by the C++ compiler when RTTI is enabled.
// Since libQt6Core.a and parts of libQt6Gui.a were compiled with -fno-rtti,
// these typeinfo symbols are missing. We provide them as data stubs.
//
// Typeinfo layout (Itanium C++ ABI):
//   __class_type_info:       [vtable_ptr+16, name_ptr]                 (16 bytes)
//   __si_class_type_info:    [vtable_ptr+16, name_ptr, base_ptr]      (24 bytes)
//   __vmi_class_type_info:   [vtable_ptr+16, name_ptr, flags, count, ...]
//
// The vtable_ptr points to the __cxxabiv1 vtable + 16 bytes (skipping
// offset_to_top and typeinfo_ptr entries, per Itanium ABI convention).

// Forward declarations for the __cxxabiv1 vtables (defined in cxxabi_stubs.cpp)
extern void* _ZTVN10__cxxabiv117__class_type_infoE[11];
extern void* _ZTVN10__cxxabiv120__si_class_type_infoE[11];

// ── Type name strings (_ZTS symbols) ──────────────────────────────────────

extern "C" const char _ZTS7QObject[]                  = "7QObject";
extern "C" const char _ZTS14QObjectPrivate[]          = "14QObjectPrivate";
extern "C" const char _ZTS12QInputDevice[]            = "12QInputDevice";
extern "C" const char _ZTS19QInputDevicePrivate[]     = "19QInputDevicePrivate";
extern "C" const char _ZTS15QImageIOHandler[]         = "15QImageIOHandler";
extern "C" const char _ZTS18QPaintDeviceWindow[]      = "18QPaintDeviceWindow";
extern "C" const char _ZTS25QPaintDeviceWindowPrivate[] = "25QPaintDeviceWindowPrivate";

// ── __class_type_info objects (root classes) ───────────────────────────────
// Layout: [vtable_ptr+16, name_ptr]

void* _ZTI7QObject[2] = {
    (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
    (void*)_ZTS7QObject
};

void* _ZTI14QObjectPrivate[2] = {
    (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
    (void*)_ZTS14QObjectPrivate
};

// ── __si_class_type_info objects (single-inheritance classes) ─────────────
// Layout: [vtable_ptr+16, name_ptr, base_typeinfo_ptr]
// The base_typeinfo_ptr references another _ZTI symbol (defined above or in Qt6 libs).

void* _ZTI12QInputDevice[3] = {
    (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
    (void*)_ZTS12QInputDevice,
    (void*)_ZTI7QObject                    // base: QObject
};

void* _ZTI19QInputDevicePrivate[3] = {
    (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
    (void*)_ZTS19QInputDevicePrivate,
    (void*)_ZTI14QObjectPrivate            // base: QObjectPrivate
};

void* _ZTI15QImageIOHandler[3] = {
    (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
    (void*)_ZTS15QImageIOHandler,
    (void*)_ZTI7QObject                    // base: QObject
};

// QPaintDeviceWindow inherits QWindow (which uses __vmi_class_type_info for
// its own typeinfo, but QPaintDeviceWindow's direct base is just QWindow).
// _ZTI7QWindow is defined as V (weak) in libQt6Gui.a.
extern "C" void* _ZTI7QWindow[];           // defined in libQt6Gui.a (weak)

void* _ZTI18QPaintDeviceWindow[3] = {
    (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
    (void*)_ZTS18QPaintDeviceWindow,
    (void*)_ZTI7QWindow                    // base: QWindow (from Qt6Gui)
};

// QPaintDeviceWindowPrivate inherits QWindowPrivate (single inheritance).
// _ZTI14QWindowPrivate is defined as V (weak) in libQt6Gui.a.
extern "C" void* _ZTI14QWindowPrivate[];  // defined in libQt6Gui.a (weak)

void* _ZTI25QPaintDeviceWindowPrivate[3] = {
    (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
    (void*)_ZTS25QPaintDeviceWindowPrivate,
    (void*)_ZTI14QWindowPrivate            // base: QWindowPrivate (from Qt6Gui)
};

} // extern "C"
