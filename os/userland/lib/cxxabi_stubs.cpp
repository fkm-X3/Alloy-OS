// cxxabi_stubs.cpp — Minimal freestanding C++ standard library + C++ ABI stubs for Qt6
//
// Provides all std:: symbols that Qt6Core needs at link time so we can avoid
// pulling in the host libstdc++ (which cascades into libc, libm, etc.).
// Compile with -ffreestanding -fno-rtti -fno-exceptions — no standard headers.

// ── Red-black tree node (layout matches libstdc++) ──────────────────────────

struct _Rb_tree_node_base {
    int _M_color;               // 0 = red, 1 = black
    _Rb_tree_node_base* _M_parent;
    _Rb_tree_node_base* _M_left;
    _Rb_tree_node_base* _M_right;
};

// ── pair<bool, size_t> (returned by _M_need_rehash) ────────────────────────

struct _Pair_bool_size {
    unsigned char first;
    unsigned long second;
};

// ── Rb_tree internal helpers ───────────────────────────────────────────────

static _Rb_tree_node_base*
_Rb_tree_increment_impl(_Rb_tree_node_base* x) {
    if (x->_M_right) {
        x = x->_M_right;
        while (x->_M_left) x = x->_M_left;
    } else {
        _Rb_tree_node_base* y = x->_M_parent;
        while (x == y->_M_right) { x = y; y = y->_M_parent; }
        if (x->_M_right != y) x = y;
    }
    return x;
}

static _Rb_tree_node_base*
_Rb_tree_decrement_impl(_Rb_tree_node_base* x) {
    if (x->_M_color == 1 && x->_M_parent->_M_parent == x)
        x = x->_M_right;
    else if (x->_M_left) {
        _Rb_tree_node_base* y = x->_M_left;
        while (y->_M_right) y = y->_M_right;
        x = y;
    } else {
        _Rb_tree_node_base* y = x->_M_parent;
        while (x == y->_M_left) { x = y; y = y->_M_parent; }
        x = y;
    }
    return x;
}

static void
_Rb_tree_rotate_left(_Rb_tree_node_base* x, _Rb_tree_node_base*& root) {
    _Rb_tree_node_base* y = x->_M_right;
    x->_M_right = y->_M_left;
    if (y->_M_left) y->_M_left->_M_parent = x;
    y->_M_parent = x->_M_parent;
    if (!x->_M_parent) root = y;
    else if (x == x->_M_parent->_M_left) x->_M_parent->_M_left = y;
    else x->_M_parent->_M_right = y;
    y->_M_left = x;
    x->_M_parent = y;
}

static void
_Rb_tree_rotate_right(_Rb_tree_node_base* x, _Rb_tree_node_base*& root) {
    _Rb_tree_node_base* y = x->_M_left;
    x->_M_left = y->_M_right;
    if (y->_M_right) y->_M_right->_M_parent = x;
    y->_M_parent = x->_M_parent;
    if (!x->_M_parent) root = y;
    else if (x == x->_M_parent->_M_right) x->_M_parent->_M_right = y;
    else x->_M_parent->_M_left = y;
    y->_M_right = x;
    x->_M_parent = y;
}

// ── _ZNKSt6locale2id5_M_idEv ────────────────────────────────────────────────

extern "C" unsigned _ZNKSt6locale2id5_M_idEv() { return 0; }

// ── _ZNKSt8__detail20_Prime_rehash_policy11_M_next_bktEm ─────────────────────

extern "C" unsigned long
_ZNKSt8__detail20_Prime_rehash_policy11_M_next_bktEm(unsigned long __n) {
    if (__n == 0) return 1;
    return __n;
}

// ── _ZNKSt8__detail20_Prime_rehash_policy14_M_need_rehashEmmm ──────────────

extern "C" _Pair_bool_size
_ZNKSt8__detail20_Prime_rehash_policy14_M_need_rehashEmmm(
    unsigned long, unsigned long, unsigned long) {
    _Pair_bool_size r = { 0, 0 };
    return r;
}

// ── _ZNSt15__exception_ptr13exception_ptr9_M_addrefEv ───────────────────────

extern "C" void _ZNSt15__exception_ptr13exception_ptr9_M_addrefEv() {}

// ── condition_variable ──────────────────────────────────────────────────────

extern "C" void _ZNSt18condition_variable10notify_allEv() {}
extern "C" void _ZNSt18condition_variable10notify_oneEv() {}
extern "C" void _ZNSt18condition_variable4waitERSt11unique_lockISt5mutexE(
    void*) {}
extern "C" void _ZNSt18condition_variableC1Ev() {}
extern "C" void _ZNSt18condition_variableD1Ev() {}

// ── _ZNSt19_Sp_make_shared_tag5_S_eqERKSt9type_info ─────────────────────────

extern "C" int _ZNSt19_Sp_make_shared_tag5_S_eqERKSt9type_info(
    const void*) { return 0; }

// ── _ZNSt28__atomic_futex_unsigned_base19_M_futex_wait_untilEPjjbNSt6chrono...
//     (long long is wrong here but avoids pulling in <chrono>) ───────────────

struct _Chrono_dur64 {
    long long __rep;
};
struct _Chrono_dur_ns {
    long long __rep;
};

extern "C" void
_ZNSt28__atomic_futex_unsigned_base19_M_futex_wait_untilEPjjbNSt6chrono8durationIlSt5ratioILl1ELl1EEEENS2_IlS3_ILl1ELl1000000000EEEE(
    void*, unsigned*, unsigned, int,
    _Chrono_dur64, _Chrono_dur_ns) {}

// ── std::pmr ────────────────────────────────────────────────────────────────

extern "C" void* _ZNSt3pmr20get_default_resourceEv() { return 0; }

extern "C" void
_ZNSt3pmr25monotonic_buffer_resource13_M_new_bufferEmm(
    void*, unsigned long, unsigned long) {}

extern "C" void _ZNSt3pmr25monotonic_buffer_resourceD1Ev() {}

void* _ZTVNSt3pmr25monotonic_buffer_resourceE[5] = { 0, 0, 0, 0, 0 };

// ── std::ctype<char>::id ─────────────────────────────────────────────────────

void* _ZNSt5ctypeIcE2idE = 0;

// ── std::chrono::steady_clock::now() ─────────────────────────────────────────

extern "C" long long _ZNSt6chrono3_V212steady_clock3nowEv() { return 0; }

// ── std::locale::classic() ───────────────────────────────────────────────────

extern "C" void* _ZNSt6locale7classicEv() { return 0; }

// ── std::string::_M_create ───────────────────────────────────────────────────

extern "C" void*
_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm(
    void*, unsigned long&, unsigned long) { return 0; }

// ── Throw stubs (never called — no exceptions in this build) ────────────────

extern "C" void _ZSt16__throw_bad_castv() { __builtin_trap(); }
extern "C" void _ZSt17__throw_bad_allocv() { __builtin_trap(); }
extern "C" void _ZSt17rethrow_exceptionNSt15__exception_ptr13exception_ptrE(
    void*) { __builtin_trap(); }
extern "C" void _ZSt20__throw_future_errori(int) { (void)0; __builtin_trap(); }
extern "C" void _ZSt20__throw_length_errorPKc(const char*) {
    (void)0; __builtin_trap();
}
extern "C" void _ZSt20__throw_system_errori(int) { (void)0; __builtin_trap(); }
extern "C" void _ZSt24__throw_out_of_range_fmtPKcz(const char*, ...) {
    (void)0; __builtin_trap();
}
extern "C" void _ZSt25__throw_bad_function_callv() { __builtin_trap(); }
extern "C" void _ZSt28__throw_bad_array_new_lengthv() { __builtin_trap(); }

// ── std::terminate ───────────────────────────────────────────────────────────

extern "C" void _ZSt9terminatev() { __builtin_trap(); }

// ── std::nothrow ─────────────────────────────────────────────────────────────

extern "C" const int _ZSt7nothrow = 0;

// ── Rb_tree: increment/decrement ─────────────────────────────────────────────

extern "C" _Rb_tree_node_base*
_ZSt18_Rb_tree_incrementPSt18_Rb_tree_node_base(_Rb_tree_node_base* x) {
    return _Rb_tree_increment_impl(x);
}

extern "C" _Rb_tree_node_base*
_ZSt18_Rb_tree_incrementPKSt18_Rb_tree_node_base(_Rb_tree_node_base* x) {
    return _Rb_tree_increment_impl(x);
}

extern "C" _Rb_tree_node_base*
_ZSt18_Rb_tree_decrementPSt18_Rb_tree_node_base(_Rb_tree_node_base* x) {
    return _Rb_tree_decrement_impl(x);
}

extern "C" _Rb_tree_node_base*
_ZSt18_Rb_tree_decrementPKSt18_Rb_tree_node_base(_Rb_tree_node_base* x) {
    return _Rb_tree_decrement_impl(x);
}

// ── Rb_tree: insert and rebalance ────────────────────────────────────────────

extern "C" void
_ZSt29_Rb_tree_insert_and_rebalancebPSt18_Rb_tree_node_baseS0_RS_(
    int insert_left,
    _Rb_tree_node_base* x,
    _Rb_tree_node_base* p,
    _Rb_tree_node_base& header)
{
    _Rb_tree_node_base*& root = header._M_parent;

    x->_M_parent = p;
    x->_M_left = 0;
    x->_M_right = 0;
    x->_M_color = 0;

    if (insert_left) {
        p->_M_left = x;
        if (p == &header) { header._M_parent = x; header._M_right = x; }
        else if (p == header._M_left) header._M_left = x;
    } else {
        p->_M_right = x;
        if (p == header._M_right) header._M_right = x;
    }

    while (x != root && x->_M_parent->_M_color == 0) {
        _Rb_tree_node_base* xpp = x->_M_parent->_M_parent;
        if (x->_M_parent == xpp->_M_left) {
            _Rb_tree_node_base* y = xpp->_M_right;
            if (y && y->_M_color == 0) {
                x->_M_parent->_M_color = 1;
                y->_M_color = 1;
                xpp->_M_color = 0;
                x = xpp;
            } else {
                if (x == x->_M_parent->_M_right) {
                    x = x->_M_parent;
                    _Rb_tree_rotate_left(x, root);
                }
                x->_M_parent->_M_color = 1;
                xpp->_M_color = 0;
                _Rb_tree_rotate_right(xpp, root);
            }
        } else {
            _Rb_tree_node_base* y = xpp->_M_left;
            if (y && y->_M_color == 0) {
                x->_M_parent->_M_color = 1;
                y->_M_color = 1;
                xpp->_M_color = 0;
                x = xpp;
            } else {
                if (x == x->_M_parent->_M_left) {
                    x = x->_M_parent;
                    _Rb_tree_rotate_right(x, root);
                }
                x->_M_parent->_M_color = 1;
                xpp->_M_color = 0;
                _Rb_tree_rotate_left(xpp, root);
            }
        }
    }
    root->_M_color = 1;
}

// ── Rb_tree: rebalance for erase ─────────────────────────────────────────────

static void
_Rb_tree_transplant(_Rb_tree_node_base* u, _Rb_tree_node_base* v,
                    _Rb_tree_node_base*& root) {
    if (!u->_M_parent) root = v;
    else if (u == u->_M_parent->_M_left) u->_M_parent->_M_left = v;
    else u->_M_parent->_M_right = v;
    if (v) v->_M_parent = u->_M_parent;
}

extern "C" void
_ZSt28_Rb_tree_rebalance_for_erasePSt18_Rb_tree_node_baseRS_(
    _Rb_tree_node_base* z, _Rb_tree_node_base& header)
{
    _Rb_tree_node_base*& root = header._M_parent;
    _Rb_tree_node_base* y = z;
    _Rb_tree_node_base* x = 0;
    _Rb_tree_node_base* x_parent = 0;
    int y_orig_color = y->_M_color;

    if (!y->_M_left) {
        x = y->_M_right;
        _Rb_tree_transplant(y, y->_M_right, root);
    } else if (!y->_M_right) {
        x = y->_M_left;
        _Rb_tree_transplant(y, y->_M_left, root);
    } else {
        y = y->_M_right;
        while (y->_M_left) y = y->_M_left;
        y_orig_color = y->_M_color;
        x = y->_M_right;
        if (y->_M_parent == z) {
            x_parent = y;
        } else {
            x_parent = y->_M_parent;
            _Rb_tree_transplant(y, y->_M_right, root);
            y->_M_right = z->_M_right;
            y->_M_right->_M_parent = y;
        }
        _Rb_tree_transplant(z, y, root);
        y->_M_left = z->_M_left;
        y->_M_left->_M_parent = y;
        y->_M_color = z->_M_color;
    }

    if (y_orig_color == 1) {
        while (x != root && (!x || x->_M_color == 1)) {
            if (x == x_parent->_M_left) {
                _Rb_tree_node_base* w = x_parent->_M_right;
                if (w && w->_M_color == 0) {
                    w->_M_color = 1;
                    x_parent->_M_color = 0;
                    _Rb_tree_rotate_left(x_parent, root);
                    w = x_parent->_M_right;
                }
                if ((!w->_M_left || w->_M_left->_M_color == 1) &&
                    (!w->_M_right || w->_M_right->_M_color == 1)) {
                    if (w) w->_M_color = 0;
                    x = x_parent;
                    x_parent = x_parent->_M_parent;
                } else {
                    if (!w->_M_right || w->_M_right->_M_color == 1) {
                        if (w->_M_left) w->_M_left->_M_color = 1;
                        if (w) w->_M_color = 0;
                        _Rb_tree_rotate_right(w, root);
                        w = x_parent->_M_right;
                    }
                    if (w) w->_M_color = x_parent->_M_color;
                    x_parent->_M_color = 1;
                    if (w && w->_M_right) w->_M_right->_M_color = 1;
                    _Rb_tree_rotate_left(x_parent, root);
                    break;
                }
            } else {
                _Rb_tree_node_base* w = x_parent->_M_left;
                if (w && w->_M_color == 0) {
                    w->_M_color = 1;
                    x_parent->_M_color = 0;
                    _Rb_tree_rotate_right(x_parent, root);
                    w = x_parent->_M_left;
                }
                if ((!w->_M_right || w->_M_right->_M_color == 1) &&
                    (!w->_M_left || w->_M_left->_M_color == 1)) {
                    if (w) w->_M_color = 0;
                    x = x_parent;
                    x_parent = x_parent->_M_parent;
                } else {
                    if (!w->_M_left || w->_M_left->_M_color == 1) {
                        if (w->_M_right) w->_M_right->_M_color = 1;
                        if (w) w->_M_color = 0;
                        _Rb_tree_rotate_left(w, root);
                        w = x_parent->_M_left;
                    }
                    if (w) w->_M_color = x_parent->_M_color;
                    x_parent->_M_color = 1;
                    if (w && w->_M_left) w->_M_left->_M_color = 1;
                    _Rb_tree_rotate_right(x_parent, root);
                    break;
                }
            }
        }
        if (x) x->_M_color = 1;
    }

    if (z == header._M_left) {
        _Rb_tree_node_base* h = z;
        while (h->_M_left) h = h->_M_left;
        header._M_left = h;
    }
    if (z == header._M_right) {
        _Rb_tree_node_base* h = z;
        while (h->_M_right) h = h->_M_right;
        header._M_right = h;
    }
}

// ── C++ ABI stubs ───────────────────────────────────────────────────────────
// These are normally provided by libsupc++ / compiler-rt.
// Without them the linker cannot resolve the C++ support symbols.

extern "C" {

void __cxa_pure_virtual(void) { __builtin_trap(); }

// Guard variables (static local initialization)
int __cxa_guard_acquire(int* guard) {
    // Simple: if byte 0 is 0, we're first; set it to 1.
    // Since we're single-threaded, no atomic needed.
    char& g = *reinterpret_cast<char*>(guard);
    if (g == 0) { g = 1; return 1; }  // first call, needs init
    return 0;  // already initialized
}
void __cxa_guard_release(int* guard) {
    char& g = *reinterpret_cast<char*>(guard);
    g = 2;  // mark as fully initialized
}
void __cxa_guard_abort(int* guard) {
    (void)guard;  // abort initialization: do nothing, will retry
}

// atexit handler for static destructors
int __cxa_atexit(void (*func)(void*), void* arg, void* dso_handle) {
    // We ignore destructor registration (kernel never shuts down userland properly)
    (void)func; (void)arg; (void)dso_handle;
    return 0;
}

// Dynamic shared object handle
// This weak symbol is referenced by objects compiled with -fPIC.
void* __dso_handle = (void*)&__dso_handle;

// Thread-safe static local initialization (single-threaded stub)
// __cxa_thread_atexit is called when std::thread_local destructors run.
void __cxa_thread_atexit(void (*func)(void*), void* obj, void* dso) {
    (void)func; (void)obj; (void)dso;
}

// One-time construction (for std::once_flag / pthread_once equivalent)
void __cxa_call_once(void* flag, void (*func)(void*), void* arg) {
    (void)flag; func(arg);
}

} // extern "C"

// ── Qt internal platform stubs (missing platform-specific implementations) ──
// These should never be called at runtime in our test app; they exist only to
// satisfy the linker. Compiled with -fno-rtti -fno-exceptions.
//
// NOTE: QElapsedTimer, QDeadlineTimer, qt_readlink are now provided by the
// real Qt6 static libraries (libQt6Core.a). We must NOT define them here or
// the object file definitions shadow the archive definitions and the real
// implementations are never linked.

// QThreadData::current(bool create) — normally in qthread_unix.cpp
extern "C" void* _ZN11QThreadData7currentEb(void* this_, int create) {
    (void)this_; (void)create;
    return 0;
}

// QLockFilePrivate::tryLock_sys() — normally in qlockfile_unix.cpp
extern "C" int _ZN16QLockFilePrivate11tryLock_sysEv(void* this_) {
    (void)this_;
    return -1;  // lock failed
}

// QLockFilePrivate::removeStaleLock()
extern "C" void _ZN16QLockFilePrivate15removeStaleLockEv(void* this_) {
    (void)this_;
}

// QLockFilePrivate::isProcessRunning(long long pid, QString const& name)
extern "C" int _ZN16QLockFilePrivate16isProcessRunningExRK7QString(void* this_, long long pid, void* name) {
    (void)this_; (void)pid; (void)name;
    return 0;  // false
}

// QTzTimeZonePrivate::QTzTimeZonePrivate() — normally in qtztimezoneprivate.cpp
extern "C" void _ZN18QTzTimeZonePrivateC1Ev(void* this_) {
    (void)this_;
}

// QThread::msleep(unsigned long) — normally in qthread_unix.cpp
extern "C" void _ZN7QThread6msleepEm(void* this_, unsigned long msecs) {
    (void)this_; (void)msecs;
}

// QWaitCondition::QWaitCondition() — normally in qwaitcondition_unix.cpp
extern "C" void _ZN14QWaitConditionC1Ev(void* this_) {
    (void)this_;
}
extern "C" void _ZN14QWaitConditionC2Ev(void* this_) {
    (void)this_;
}

// QWaitCondition::~QWaitCondition() — normally in qwaitcondition_unix.cpp
extern "C" void _ZN14QWaitConditionD1Ev(void* this_) {
    (void)this_;
}
extern "C" void _ZN14QWaitConditionD2Ev(void* this_) {
    (void)this_;
}
extern "C" void _ZN14QWaitConditionD0Ev(void* this_) {
    (void)this_;
}

// QWaitCondition::wait(QMutex*, QDeadlineTimer) — normally in qwaitcondition_unix.cpp
extern "C" int _ZN14QWaitCondition4waitEP6QMutex14QDeadlineTimer(void* this_, void* mutex, void* timer) {
    (void)this_; (void)mutex; (void)timer;
    return 1;  // always succeeds
}

// QWaitCondition::wakeOne() — normally in qwaitcondition_unix.cpp
extern "C" void _ZN14QWaitCondition7wakeOneEv(void* this_) {
    (void)this_;
}

// QWaitCondition::wakeAll() — normally in qwaitcondition_unix.cpp
extern "C" void _ZN14QWaitCondition7wakeAllEv(void* this_) {
    (void)this_;
}

// QThread::start(QThread::Priority) — normally in qthread_unix.cpp
extern "C" void _ZN7QThread5startENS_8PriorityE(void* this_, int priority) {
    (void)this_; (void)priority;
}

// QThread::wait(QDeadlineTimer) — normally in qthread_unix.cpp
extern "C" int _ZN7QThread4waitE14QDeadlineTimer(void* this_, void* timer) {
    (void)this_; (void)timer;
    return 1;  // always succeeds
}

// QThread::idealThreadCount() — static, normally in qthread_unix.cpp
extern "C" int _ZN7QThread16idealThreadCountEv() {
    return 1;
}

// QFSFileEngine::id() const — normally in qfsfileengine_unix.cpp
extern "C" unsigned int _ZNK13QFSFileEngine2idEv(void* this_) {
    (void)this_;
    return 0;
}

// QFSFileEngine::owner(FileOwner) const — normally in qfsfileengine_unix.cpp
extern "C" void _ZNK13QFSFileEngine5ownerEN19QAbstractFileEngine9FileOwnerE(void* this_, int owner) {
    (void)this_; (void)owner;
}

// QFSFileEngine::ownerId(FileOwner) const — normally in qfsfileengine_unix.cpp
extern "C" unsigned int _ZNK13QFSFileEngine7ownerIdEN19QAbstractFileEngine9FileOwnerE(void* this_, int owner) {
    (void)this_; (void)owner;
    return 0;
}

// QFSFileEngine::setFileTime(QDateTime const&, FileTime) — normally in qfsfileengine_unix.cpp
extern "C" int _ZN13QFSFileEngine11setFileTimeERK9QDateTimeN19QAbstractFileEngine8FileTimeE(void* this_, void* dt, int fileTime) {
    (void)this_; (void)dt; (void)fileTime;
    return 0;  // false (unsupported)
}

// QFSFileEngine::cloneTo(QAbstractFileEngine*) — normally in qfsfileengine_unix.cpp
extern "C" int _ZN13QFSFileEngine7cloneToEP19QAbstractFileEngine(void* this_, void* target) {
    (void)this_; (void)target;
    return 0;  // false (unsupported)
}

// operator new(unsigned long, std::nothrow_t const&) — nothrow placement new (64-bit)
extern "C" void* _ZnwmRKSt9nothrow_t(unsigned long sz, void* nt) {
    (void)nt;
    return operator new(sz);
}

// operator new[](unsigned long, std::nothrow_t const&) — nothrow array placement new (64-bit)
extern "C" void* _ZnamRKSt9nothrow_t(unsigned long sz, void* nt) {
    (void)nt;
    return operator new[](sz);
}

// QTzTimeZonePrivate::QTzTimeZonePrivate(QByteArray const&) — additional ctor
extern "C" void _ZN18QTzTimeZonePrivateC2ERK10QByteArray(void* this_, void* id) {
    (void)this_; (void)id;
}

// QLockFile::unlock() — normally in qlockfile_unix.cpp
extern "C" void _ZN9QLockFile6unlockEv(void* this_) {
    (void)this_;
}

// ── std::__detail::_List_node_base ──────────────────────────────────────────
// Normally in libstdc++ (src/c++98/list.cc or similar)

extern "C" void _ZNSt8__detail15_List_node_base7_M_hookEPS0_(
    void* this_, void* next) {
    (void)this_; (void)next;
}

extern "C" void _ZNSt8__detail15_List_node_base9_M_unhookEv(
    void* this_) {
    (void)this_;
}

// ── __cxxabiv1 vtable data (RTTI support) ────────────────────────────────────
// These are normally provided by libsupc++. Since we compile with -fno-rtti,
// these vtables are never actually dispatched, but the linker still needs the
// data symbols to satisfy references from precompiled Qt6 object files.
//
// vtable layout (Itanium C++ ABI):
//   [-2]: offset_to_top (always 0 for non-virtual bases)
//   [-1]: typeinfo pointer for the class itself
//   [0..n]: virtual function pointers
//
// Size of each vtable = 11 pointers (88 bytes on x86_64).

// Typeinfo name strings (__cxxabiv1 type_info objects need these)
extern "C" const char _ZTSN10__cxxabiv117__class_type_infoE[] = "N10__cxxabiv117__class_type_infoE";
extern "C" const char _ZTSN10__cxxabiv120__si_class_type_infoE[] = "N10__cxxabiv120__si_class_type_infoE";
extern "C" const char _ZTSN10__cxxabiv121__vmi_class_type_infoE[] = "N10__cxxabiv121__vmi_class_type_infoE";

// Typeinfo for std::type_info (needed by __class_type_info typeinfo)
extern "C" const char _ZTSSt9type_info[] = "St9type_info";

// Forward declarations of vtables (needed for cross-references)
extern "C" void* _ZTVN10__cxxabiv117__class_type_infoE[11];
extern "C" void* _ZTVN10__cxxabiv120__si_class_type_infoE[11];
extern "C" void* _ZTVN10__cxxabiv121__vmi_class_type_infoE[11];

// typeinfo for std::type_info (root of all type_info classes)
void* _ZTISt9type_info[2] = {
    (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
    (void*)_ZTSSt9type_info
};

// Typeinfo objects for the __cxxabiv1 type_info classes themselves
void* _ZTIN10__cxxabiv117__class_type_infoE[2] = {
    (void*)((char*)_ZTVN10__cxxabiv117__class_type_infoE + 16),
    (void*)_ZTSN10__cxxabiv117__class_type_infoE
};

void* _ZTIN10__cxxabiv120__si_class_type_infoE[2] = {
    (void*)((char*)_ZTVN10__cxxabiv120__si_class_type_infoE + 16),
    (void*)_ZTSN10__cxxabiv120__si_class_type_infoE
};

void* _ZTIN10__cxxabiv121__vmi_class_type_infoE[2] = {
    (void*)((char*)_ZTVN10__cxxabiv121__vmi_class_type_infoE + 16),
    (void*)_ZTSN10__cxxabiv121__vmi_class_type_infoE
};

// Vtables for __cxxabiv1 type_info classes (11 pointers each = 88 bytes)
// Entries are zero-initialized since RTTI is disabled and these are never called.
void* _ZTVN10__cxxabiv117__class_type_infoE[11] = {
    (void*)0,                                // [-2]: offset_to_top
    (void*)_ZTIN10__cxxabiv117__class_type_infoE, // [-1]: typeinfo ptr
    (void*)0, (void*)0, (void*)0, (void*)0,  // [0-3]: virtual functions (type_info)
    (void*)0, (void*)0, (void*)0, (void*)0,  // [4-7]: virtual functions (type_info)
    (void*)0                                 // [8]: virtual function (__class_type_info)
};

void* _ZTVN10__cxxabiv120__si_class_type_infoE[11] = {
    (void*)0,                                     // [-2]: offset_to_top
    (void*)_ZTIN10__cxxabiv120__si_class_type_infoE, // [-1]: typeinfo ptr
    (void*)0, (void*)0, (void*)0, (void*)0,       // [0-3]: virtual functions
    (void*)0, (void*)0, (void*)0, (void*)0,       // [4-7]: virtual functions
    (void*)0                                      // [8]: virtual function
};

void* _ZTVN10__cxxabiv121__vmi_class_type_infoE[11] = {
    (void*)0,                                     // [-2]: offset_to_top
    (void*)_ZTIN10__cxxabiv121__vmi_class_type_infoE, // [-1]: typeinfo ptr
    (void*)0, (void*)0, (void*)0, (void*)0,       // [0-3]: virtual functions
    (void*)0, (void*)0, (void*)0, (void*)0,       // [4-7]: virtual functions
    (void*)0                                      // [8]: virtual function
};
