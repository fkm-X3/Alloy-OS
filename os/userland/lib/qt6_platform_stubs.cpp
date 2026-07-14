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

// ── QFileSystemEngine ────────────────────────────────────────────────────
// Normally in qfilesystemengine_unix.cpp

void _ZN17QFileSystemEngine10removeFileERK16QFileSystemEntryR12QSystemError(
    void* entry, void* error) {
    (void)entry; (void)error;
}

void _ZN17QFileSystemEngine10renameFileERK16QFileSystemEntryS2_R12QSystemError(
    void* oldName, void* newName, void* error) {
    (void)oldName; (void)newName; (void)error;
}

void _ZN17QFileSystemEngine11currentPathEv(void* ret) {
    (void)ret;
}

void _ZN17QFileSystemEngine12absoluteNameERK16QFileSystemEntry(
    void* ret, void* entry) {
    (void)ret; (void)entry;
}

void _ZN17QFileSystemEngine12fillMetaDataERK16QFileSystemEntryR19QFileSystemMetaData6QFlagsINS3_12MetaDataFlagEE(
    void* entry, void* metaData, int flags) {
    (void)entry; (void)metaData; (void)flags;
}

void _ZN17QFileSystemEngine13canonicalNameERK16QFileSystemEntryR19QFileSystemMetaData(
    void* ret, void* entry, void* metaData) {
    (void)ret; (void)entry; (void)metaData;
}

void _ZN17QFileSystemEngine13getLinkTargetERK16QFileSystemEntryR19QFileSystemMetaData(
    void* ret, void* entry, void* metaData) {
    (void)ret; (void)entry; (void)metaData;
}

void _ZN17QFileSystemEngine14setCurrentPathERK16QFileSystemEntry(
    void* entry) {
    (void)entry;
}

void _ZN17QFileSystemEngine15createDirectoryERK16QFileSystemEntrybSt8optionalI6QFlagsIN11QFileDevice10PermissionEEE(
    void* path, int /*bool*/ treatRootAsDir, void* permissions) {
    (void)path; (void)treatRootAsDir; (void)permissions;
}

void _ZN17QFileSystemEngine15removeDirectoryERK16QFileSystemEntryb(
    void* entry, int recurse) {
    (void)entry; (void)recurse;
}

void _ZN17QFileSystemEngine19renameOverwriteFileERK16QFileSystemEntryS2_R12QSystemError(
    void* oldName, void* newName, void* error) {
    (void)oldName; (void)newName; (void)error;
}

void _ZN17QFileSystemEngine8copyFileERK16QFileSystemEntryS2_R12QSystemError(
    void* src, void* dest, void* error) {
    (void)src; (void)dest; (void)error;
}

void _ZN17QFileSystemEngine8homePathEv(void* ret) {
    (void)ret;
}

void _ZN17QFileSystemEngine8rootPathEv(void* ret) {
    (void)ret;
}

void _ZN17QFileSystemEngine8tempPathEv(void* ret) {
    (void)ret;
}

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

// ── QFSFileEngine ────────────────────────────────────────────────────────
// Normally in qfsfileengine_unix.cpp

void _ZNK13QFSFileEngine8fileNameEN19QAbstractFileEngine8FileNameE(
    void* ret, void* this_, int file) {
    (void)ret; (void)this_; (void)file;
}

int _ZN13QFSFileEngine4linkERK7QString(void* this_, void* newName) {
    (void)this_; (void)newName;
    return 0;
}

int _ZN13QFSFileEngine7setSizeEx(void* this_, long long size) {
    (void)this_; (void)size;
    return 0;
}

int _ZNK13QFSFileEngine13caseSensitiveEv(void* this_) {
    (void)this_;
    return 0;
}

int _ZNK13QFSFileEngine14isRelativePathEv(void* this_) {
    (void)this_;
    return 0;
}

uint64_t _ZNK13QFSFileEngine9fileFlagsE6QFlagsIN19QAbstractFileEngine8FileFlagEE(
    void* this_, int type) {
    (void)this_; (void)type;
    return 0;
}

int _ZN13QFSFileEngine14setPermissionsEj(void* this_, unsigned int perms) {
    (void)this_; (void)perms;
    return 0;
}

// ── QFSFileEnginePrivate ─────────────────────────────────────────────────
// Normally in qfsfileengine_unix.cpp

int _ZN20QFSFileEnginePrivate10nativeOpenE6QFlagsIN13QIODeviceBase12OpenModeFlagEESt8optionalIS0_IN11QFileDevice10PermissionEEE(
    void* this_, int openMode, void* permissions) {
    (void)this_; (void)openMode; (void)permissions;
    return 0;
}

long long _ZN20QFSFileEnginePrivate10nativeReadEPcx(
    void* this_, char* data, long long maxlen) {
    (void)this_; (void)data; (void)maxlen;
    return -1;
}

int _ZN20QFSFileEnginePrivate10nativeSeekEx(void* this_, long long pos) {
    (void)this_; (void)pos;
    return 0;
}

void _ZN20QFSFileEnginePrivate11nativeCloseEv(void* this_) {
    (void)this_;
}

void _ZN20QFSFileEnginePrivate11nativeFlushEv(void* this_) {
    (void)this_;
}

long long _ZN20QFSFileEnginePrivate11nativeWriteEPKcx(
    void* this_, const char* data, long long len) {
    (void)this_; (void)data; (void)len;
    return -1;
}

long long _ZN20QFSFileEnginePrivate14nativeReadLineEPcx(
    void* this_, char* data, long long maxlen) {
    (void)this_; (void)data; (void)maxlen;
    return -1;
}

void _ZN20QFSFileEnginePrivate16nativeSyncToDiskEv(void* this_) {
    (void)this_;
}

void* _ZN20QFSFileEnginePrivate3mapExx6QFlagsIN11QFileDevice13MemoryMapFlagEE(
    void* this_, long long offset, long long size, int flags) {
    (void)this_; (void)offset; (void)size; (void)flags;
    return nullptr;
}

void _ZN20QFSFileEnginePrivate5unmapEPh(void* this_, unsigned char* addr) {
    (void)this_; (void)addr;
}

long long _ZNK20QFSFileEnginePrivate10nativeSizeEv(void* this_) {
    (void)this_;
    return -1;
}

long long _ZNK20QFSFileEnginePrivate12nativeHandleEv(void* this_) {
    (void)this_;
    return -1;
}

int _ZNK20QFSFileEnginePrivate18nativeIsSequentialEv(void* this_) {
    (void)this_;
    return 0;
}

long long _ZNK20QFSFileEnginePrivate6doStatE6QFlagsIN19QFileSystemMetaData12MetaDataFlagEE(
    void* this_, int flags) {
    (void)this_; (void)flags;
    return -1;
}

long long _ZNK20QFSFileEnginePrivate9nativePosEv(void* this_) {
    (void)this_;
    return 0;
}

// ── QThread / QThreadData / QThreadPrivate ───────────────────────────────
// Normally in qthread_unix.cpp

void _ZN7QThread9terminateEv(void* this_) {
    (void)this_;
}

void _ZN11QThreadData22clearCurrentThreadDataEv() {
}

void* _ZN14QThreadPrivate21createEventDispatcherEP11QThreadData(
    void* this_, void* data) {
    (void)this_; (void)data;
    return nullptr;
}

void _ZN14QThreadPrivate11setPriorityEN7QThread8PriorityE(
    void* this_, int priority) {
    (void)this_; (void)priority;
}

// ── QSystemLocale ────────────────────────────────────────────────────────
// Normally in qsystemlocale_unix.cpp

void _ZNK13QSystemLocale14fallbackLocaleEv(void* ret, void* this_) {
    (void)ret; (void)this_;
}

void _ZNK13QSystemLocale5queryENS_9QueryTypeE8QVariant(
    void* ret, void* this_, int type, void* variant) {
    (void)ret; (void)this_; (void)type; (void)variant;
}

// ── QTzTimeZonePrivate ───────────────────────────────────────────────────
// Normally in qtztimezoneprivate.cpp

void _ZN18QTzTimeZonePrivateC1ERK10QByteArray(void* this_, void* id) {
    (void)this_; (void)id;
}

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
