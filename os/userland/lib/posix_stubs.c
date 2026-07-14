// POSIX/libc stubs for freestanding Alloy OS userland.
// Provides function implementations that Qt6Core expects at link time.
// This file is compiled standalone (no system headers except freestanding ones).

#include "alloy_syscall.h"
#ifdef __x86_64__
#include "alloy_syscall_x86_64.h"
#define SYSCALL_FN syscall_x86_64
#elif defined(__aarch64__)
#include "alloy_syscall_aarch64.h"
#define SYSCALL_FN syscall_aarch64
#else
#define SYSCALL_FN syscall
#endif

#include <stddef.h>
#include <stdint.h>
#include <stdarg.h>

// Type definitions used by the stubs (not provided by freestanding headers)
typedef long time_t;
typedef long suseconds_t;
typedef long off_t;
typedef long ssize_t;
typedef unsigned int uid_t;
typedef unsigned int gid_t;
typedef unsigned int pid_t;
typedef unsigned int mode_t;
typedef unsigned long nfds_t;
typedef unsigned int socklen_t;
typedef int clockid_t;

struct timespec { time_t tv_sec; long tv_nsec; };
struct timeval { time_t tv_sec; suseconds_t tv_usec; };

struct tm {
    int tm_sec, tm_min, tm_hour, tm_mday, tm_mon, tm_year;
    int tm_wday, tm_yday, tm_isdst;
};

struct stat { int st_mode; off_t st_size; };

typedef struct { int fd; } DIR;
struct dirent { long d_ino; char d_name[256]; };

struct sockaddr { unsigned short sa_family; char sa_data[14]; };
struct pollfd { int fd; short events; short revents; };

// pthread types (single-threaded stubs)
typedef unsigned long pthread_t;
typedef unsigned long pthread_mutex_t;
typedef unsigned long pthread_mutexattr_t;
typedef unsigned long pthread_cond_t;
typedef unsigned long pthread_condattr_t;
typedef unsigned long pthread_key_t;
typedef unsigned long pthread_once_t;
typedef unsigned long pthread_attr_t;
typedef unsigned long pthread_rwlock_t;
typedef unsigned long pthread_rwlockattr_t;
typedef unsigned long pthread_spinlock_t;
typedef unsigned long pthread_barrier_t;
typedef unsigned long pthread_barrierattr_t;

#define PTHREAD_MUTEX_INITIALIZER 0
#define PTHREAD_ONCE_INIT 0
#define PTHREAD_COND_INITIALIZER 0

// ── Threads / pthread stubs ──────────────────────────────────────────────────

int pthread_mutex_init(pthread_mutex_t *mutex, const pthread_mutexattr_t *attr) {
    (void)mutex; (void)attr; return 0;
}
int pthread_mutex_destroy(pthread_mutex_t *mutex) { (void)mutex; return 0; }
int pthread_mutex_lock(pthread_mutex_t *mutex) { (void)mutex; return 0; }
int pthread_mutex_trylock(pthread_mutex_t *mutex) { (void)mutex; return 0; }
int pthread_mutex_unlock(pthread_mutex_t *mutex) { (void)mutex; return 0; }
int pthread_cond_init(pthread_cond_t *cond, const pthread_condattr_t *attr) {
    (void)cond; (void)attr; return 0;
}
int pthread_cond_destroy(pthread_cond_t *cond) { (void)cond; return 0; }
int pthread_cond_wait(pthread_cond_t *cond, pthread_mutex_t *mutex) {
    (void)cond; (void)mutex; return 0;
}
int pthread_cond_timedwait(pthread_cond_t *cond, pthread_mutex_t *mutex,
                           const struct timespec *abstime) {
    (void)cond; (void)mutex; (void)abstime; return 0;
}
int pthread_cond_signal(pthread_cond_t *cond) { (void)cond; return 0; }
int pthread_cond_broadcast(pthread_cond_t *cond) { (void)cond; return 0; }
int pthread_cond_clockwait(pthread_cond_t *cond, pthread_mutex_t *mutex,
                           clockid_t clock_id, const struct timespec *abstime) {
    (void)cond; (void)mutex; (void)clock_id; (void)abstime; return 0;
}
int pthread_create(pthread_t *thread, const pthread_attr_t *attr,
                   void *(*start)(void*), void *arg) {
    (void)thread; (void)attr; (void)start; (void)arg; return -1;
}
int pthread_join(pthread_t thread, void **retval) {
    (void)thread; (void)retval; return -1;
}
void pthread_exit(void *retval) { (void)retval; __builtin_unreachable(); }
pthread_t pthread_self(void) { return 0; }
int pthread_equal(pthread_t a, pthread_t b) { return a == b; }
int pthread_detach(pthread_t thread) { (void)thread; return 0; }
int pthread_cancel(pthread_t thread) { (void)thread; return 0; }
int pthread_once(pthread_once_t *once, void (*init)(void)) {
    if (*once == 0) { *once = 1; init(); } return 0;
}
int pthread_key_create(pthread_key_t *key, void (*destructor)(void*)) {
    (void)key; (void)destructor; return 0;
}
int pthread_key_delete(pthread_key_t key) { (void)key; return 0; }
void *pthread_getspecific(pthread_key_t key) { (void)key; return 0; }
int pthread_setspecific(pthread_key_t key, const void *value) {
    (void)key; (void)value; return 0;
}
int pthread_rwlock_init(pthread_rwlock_t *rwlock, const pthread_rwlockattr_t *attr) {
    (void)rwlock; (void)attr; return 0;
}
int pthread_rwlock_destroy(pthread_rwlock_t *rwlock) { (void)rwlock; return 0; }
int pthread_rwlock_rdlock(pthread_rwlock_t *rwlock) { (void)rwlock; return 0; }
int pthread_rwlock_wrlock(pthread_rwlock_t *rwlock) { (void)rwlock; return 0; }
int pthread_rwlock_unlock(pthread_rwlock_t *rwlock) { (void)rwlock; return 0; }
int pthread_rwlock_tryrdlock(pthread_rwlock_t *rwlock) { (void)rwlock; return 0; }
int pthread_rwlock_trywrlock(pthread_rwlock_t *rwlock) { (void)rwlock; return 0; }
int pthread_spin_init(pthread_spinlock_t *lock, int pshared) {
    (void)lock; (void)pshared; return 0;
}
int pthread_spin_destroy(pthread_spinlock_t *lock) { (void)lock; return 0; }
int pthread_spin_lock(pthread_spinlock_t *lock) { (void)lock; return 0; }
int pthread_spin_trylock(pthread_spinlock_t *lock) { (void)lock; return 0; }
int pthread_spin_unlock(pthread_spinlock_t *lock) { (void)lock; return 0; }
int pthread_setname_np(pthread_t thread, const char *name) {
    (void)thread; (void)name; return 0;
}
int pthread_getname_np(pthread_t thread, char *name, size_t len) {
    (void)thread; (void)name; (void)len; return 0;
}
int pthread_mutexattr_init(pthread_mutexattr_t *attr) { (void)attr; return 0; }
int pthread_mutexattr_destroy(pthread_mutexattr_t *attr) { (void)attr; return 0; }
int pthread_mutexattr_settype(pthread_mutexattr_t *attr, int type) {
    (void)attr; (void)type; return 0;
}
int pthread_attr_init(pthread_attr_t *attr) { (void)attr; return 0; }
int pthread_attr_destroy(pthread_attr_t *attr) { (void)attr; return 0; }
int pthread_barrier_init(pthread_barrier_t *barrier,
                         const pthread_barrierattr_t *attr, unsigned count) {
    (void)barrier; (void)attr; (void)count; return 0;
}
int pthread_barrier_destroy(pthread_barrier_t *barrier) { (void)barrier; return 0; }
int pthread_barrier_wait(pthread_barrier_t *barrier) { (void)barrier; return 0; }

// ── Environment ──────────────────────────────────────────────────────────────

char *getenv(const char *name) { (void)name; return 0; }
int putenv(char *string) { (void)string; return -1; }
int setenv(const char *name, const char *value, int overwrite) {
    (void)name; (void)value; (void)overwrite; return -1;
}
int unsetenv(const char *name) { (void)name; return -1; }

// sbrk and _exit are in stdlib.c
void *sbrk(int incr);
void _exit(int status);

// ── Process ──────────────────────────────────────────────────────────────────

pid_t getpid(void) { return (pid_t)SYSCALL_FN(SYS_GETPID, 0, 0, 0, 0, 0); }
void abort(void) { __builtin_trap(); }
void exit(int status) { _exit(status); }
int atexit(void (*func)(void)) { (void)func; return 0; }

// ── Time ─────────────────────────────────────────────────────────────────────

int clock_gettime(clockid_t clk_id, struct timespec *tp) {
    (void)clk_id;
    if (!tp) return -1;
    // Use SYS_GETTIMEOFDAY to get real uptime
    unsigned long tv_buf[2] = {0, 0};
    long ret = SYSCALL_FN(SYS_GETTIMEOFDAY, (long)tv_buf, 0, 0, 0, 0);
    if (ret == 0 || ret == (long)tv_buf) {
        tp->tv_sec = (time_t)tv_buf[0];
        tp->tv_nsec = (long)tv_buf[1] * 1000; // usec -> nsec
    } else {
        tp->tv_sec = 0;
        tp->tv_nsec = 0;
    }
    return 0;
}
int gettimeofday(struct timeval *tv, void *tz) {
    (void)tz;
    if (!tv) return -1;
    unsigned long tv_buf[2] = {0, 0};
    long ret = SYSCALL_FN(SYS_GETTIMEOFDAY, (long)tv_buf, 0, 0, 0, 0);
    if (ret == 0 || ret == (long)tv_buf) {
        tv->tv_sec = (time_t)tv_buf[0];
        tv->tv_usec = (suseconds_t)tv_buf[1];
    } else {
        tv->tv_sec = 0;
        tv->tv_usec = 0;
    }
    return 0;
}
time_t time(time_t *t) { if (t) *t = 0; return 0; }
int nanosleep(const struct timespec *req, struct timespec *rem) {
    (void)req; (void)rem; SYSCALL_FN(SYS_YIELD, 0, 0, 0, 0, 0); return 0;
}
unsigned sleep(unsigned seconds) {
    (void)seconds; SYSCALL_FN(SYS_YIELD, 0, 0, 0, 0, 0); return 0;
}
struct tm *localtime_r(const time_t *timer, struct tm *buf) {
    (void)timer;
    if (buf) { buf->tm_sec=0; buf->tm_min=0; buf->tm_hour=0;
               buf->tm_mday=1; buf->tm_mon=0; buf->tm_year=0;
               buf->tm_wday=0; buf->tm_yday=0; buf->tm_isdst=0; }
    return buf;
}
struct tm *localtime(const time_t *timer) {
    static struct tm buf; return localtime_r(timer, &buf);
}
time_t mktime(struct tm *tm) { (void)tm; return 0; }
char *asctime(const struct tm *tm) { (void)tm; return 0; }
char *ctime(const time_t *t) { (void)t; return 0; }
double difftime(time_t a, time_t b) { (void)a; (void)b; return 0; }

// ── Locale ───────────────────────────────────────────────────────────────────

struct lconv {
    char *decimal_point, *thousands_sep, *grouping;
    char *mon_decimal_point, *mon_thousands_sep, *mon_grouping;
    char *positive_sign, *negative_sign, *currency_symbol, *int_curr_symbol;
    char frac_digits, p_cs_precedes, n_cs_precedes, p_sep_by_space, n_sep_by_space;
    char p_sign_posn, n_sign_posn, int_frac_digits;
    char int_p_cs_precedes, int_n_cs_precedes, int_p_sep_by_space, int_n_sep_by_space;
    char int_p_sign_posn, int_n_sign_posn;
};

static struct lconv default_lconv = {
    ".","","","","","","","","","",
    0,0,0,0,0,0,0,0,0,0,0,0,0,0
};

struct lconv *localeconv(void) { return &default_lconv; }
char *setlocale(int category, const char *locale) {
    (void)category; (void)locale; return "C";
}

// ── Math stubs ───────────────────────────────────────────────────────────────

double fabs(double x) { return x < 0 ? -x : x; }
float fabsf(float x) { return x < 0 ? -x : x; }
double sqrt(double x) { (void)x; return x; }
double sin(double x) { (void)x; return 0; }
double cos(double x) { (void)x; return 1; }
double floor(double x) { return (double)(long long)x; }
double ceil(double x) { long long i=(long long)x; return x>i?(double)(i+1):(double)i; }
double pow(double x, double y) { (void)y; return x; }
double fmod(double x, double y) { (void)y; return x; }
double log(double x) { (void)x; return 0; }
double log2(double x) { (void)x; return 0; }
double log10(double x) { (void)x; return 0; }
double exp(double x) { (void)x; return 1; }
double atan2(double y, double x) { (void)y; (void)x; return 0; }
double tan(double x) { (void)x; return 0; }
double atan(double x) { (void)x; return 0; }
double acos(double x) { (void)x; return 0; }
double asin(double x) { (void)x; return 0; }
double cosh(double x) { (void)x; return 1; }
double sinh(double x) { (void)x; return 0; }
double tanh(double x) { (void)x; return 0; }
double modf(double x, double *iptr) { *iptr=(long long)x; return x-*iptr; }
double frexp(double x, int *exp) { *exp=0; return x; }
double ldexp(double x, int exp) { (void)exp; return x; }

// ── Additional math stubs ─────────────────────────────────────────────────────

float powf(float x, float y) { (void)y; return x; }
float sinf(float x) { (void)x; return 0; }
float acosf(float x) { (void)x; return 0; }
float lroundf(float x) { return (float)(long long)(x + 0.5f); }
double round(double x) { return (double)(long long)(x + 0.5); }
void sincos(double x, double *sinx, double *cosx) { (void)x; *sinx = 0; *cosx = 1; }
void tzset(void) {}

// ── Integer math ─────────────────────────────────────────────────────────────

int abs(int x) { return x < 0 ? -x : x; }
long labs(long x) { return x < 0 ? -x : x; }
long long llabs(long long x) { return x < 0 ? -x : x; }
int rand(void) { return 1; }
void srand(unsigned seed) { (void)seed; }
long random(void) { return 1; }
void srandom(unsigned seed) { (void)seed; }
int atoi(const char *s) { int n=0, sign=1; if(*s=='-'){sign=-1;s++;}else if(*s=='+')s++; while(*s>='0'&&*s<='9')n=n*10+(*s++-'0'); return sign*n; }
long atol(const char *s) { return (long)atoi(s); }
long long atoll(const char *s) { return (long long)atoi(s); }
long strtol(const char *s, char **end, int base) {
    (void)base; long n=0; int sign=1;
    if(*s=='-'){sign=-1;s++;}else if(*s=='+')s++;
    while(*s>='0'&&*s<='9')n=n*10+(*s++-'0');
    if(end)*end=(char*)s; return sign*n;
}
unsigned long strtoul(const char *s, char **end, int base) {
    (void)base; unsigned long n=0;
    while(*s>='0'&&*s<='9')n=n*10+(*s++-'0');
    if(end)*end=(char*)s; return n;
}
double strtod(const char *nptr, char **endptr) { (void)nptr; (void)endptr; return 0.0; }
float strtof(const char *nptr, char **endptr) { (void)nptr; (void)endptr; return 0.0f; }

// ── Sorting ──────────────────────────────────────────────────────────────────

void qsort(void *base, size_t nmemb, size_t size,
           int (*compar)(const void *, const void *)) {
    (void)base; (void)nmemb; (void)size; (void)compar;
}
void *bsearch(const void *key, const void *base, size_t nmemb,
              size_t size, int (*compar)(const void *, const void *)) {
    (void)key; (void)base; (void)nmemb; (void)size; (void)compar; return 0;
}

// ── Allocation ───────────────────────────────────────────────────────────────

void *malloc(size_t size) {
    if (size == 0) size = 1;
    void *p = sbrk((int)size);
    return (p == (void*)-1) ? 0 : p;
}
void *realloc(void *ptr, size_t size) { (void)ptr; return malloc(size); }
void *calloc(size_t nmemb, size_t size) {
    size_t total = nmemb * size;
    void *p = malloc(total);
    if (p) { unsigned char *cp = (unsigned char*)p; for (size_t i = 0; i < total; i++) cp[i] = 0; }
    return p;
}
void free(void *ptr) { (void)ptr; }

// ── Signals ──────────────────────────────────────────────────────────────────

typedef void (*sighandler_t)(int);
#define SIG_DFL ((sighandler_t)0)

int sigaction(int signum, const void *act, void *oldact) {
    (void)signum; (void)act; (void)oldact; return 0;
}
sighandler_t signal(int signum, sighandler_t handler) {
    (void)signum; (void)handler; return SIG_DFL;
}
int raise(int sig) { (void)sig; return 0; }
int kill(pid_t pid, int sig) { (void)pid; (void)sig; return 0; }

// ── Errno ────────────────────────────────────────────────────────────────────

int *__errno_location(void) { static int errno_val = 0; return &errno_val; }

// ── Sysconf ──────────────────────────────────────────────────────────────────

long sysconf(int name) { (void)name; return -1; }

// ── Socket stubs ─────────────────────────────────────────────────────────────

#define POLLIN  1
#define POLLOUT 2
#define POLLERR 8
#define POLLHUP 16
#define POLLNVAL 32

int poll(struct pollfd *fds, nfds_t nfds, int timeout) {
    (void)timeout;
    int ready = 0;
    for (nfds_t i = 0; i < nfds; i++) {
        fds[i].revents = 0;
        if (fds[i].fd < 0) { fds[i].revents = POLLNVAL; ready++; continue; }
        fds[i].revents = fds[i].events & (POLLIN | POLLOUT);
        if (fds[i].revents) ready++;
    }
    return ready;
}

int socket(int domain, int type, int protocol) {
    (void)domain; (void)type; (void)protocol; return -1;
}
int bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    (void)sockfd; (void)addr; (void)addrlen; return -1;
}
int listen(int sockfd, int backlog) { (void)sockfd; (void)backlog; return -1; }
int accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen) {
    (void)sockfd; (void)addr; (void)addrlen; return -1;
}
int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    (void)sockfd; (void)addr; (void)addrlen; return -1;
}
ssize_t send(int sockfd, const void *buf, size_t len, int flags) {
    (void)sockfd; (void)buf; (void)len; (void)flags; return -1;
}
ssize_t recv(int sockfd, void *buf, size_t len, int flags) {
    (void)sockfd; (void)buf; (void)len; (void)flags; return -1;
}
int setsockopt(int sockfd, int level, int optname,
               const void *optval, socklen_t optlen) {
    (void)sockfd; (void)level; (void)optname; (void)optval; (void)optlen; return -1;
}
int getsockopt(int sockfd, int level, int optname, void *optval, socklen_t *optlen) {
    (void)sockfd; (void)level; (void)optname; (void)optval; (void)optlen; return -1;
}
int shutdown(int sockfd, int how) { (void)sockfd; (void)how; return -1; }

// ── Misc ─────────────────────────────────────────────────────────────────────

int isatty(int fd) { (void)fd; return 0; }
char *getcwd(char *buf, size_t size) { (void)buf; (void)size; return 0; }
int chdir(const char *path) { (void)path; return -1; }
int gethostname(char *name, size_t len) { (void)name; (void)len; return -1; }
uid_t getuid(void) { return 0; }
uid_t geteuid(void) { return 0; }
gid_t getgid(void) { return 0; }
gid_t getegid(void) { return 0; }
int setuid(uid_t uid) { (void)uid; return -1; }
int setgid(gid_t gid) { (void)gid; return -1; }
pid_t getppid(void) { return 1; }
int fsync(int fd) { (void)fd; return 0; }
long fpathconf(int fd, int name) { (void)fd; (void)name; return -1; }
long pathconf(const char *path, int name) { (void)path; (void)name; return -1; }

// ── File operations ──────────────────────────────────────────────────────────

int open(const char *path, int flags, ...) { (void)path; (void)flags; return -1; }
int close(int fd) { return (int)SYSCALL_FN(SYS_CLOSE, fd, 0, 0, 0, 0); }
ssize_t read(int fd, void *buf, size_t count) {
    if (!buf || count == 0) return -1;
    // Use SYS_SOCKET_READ for socket fds (Wayland connection)
    // For regular file reads, this may fail; callers should handle -1.
    return SYSCALL_FN(SYS_SOCKET_READ, fd, (long)(size_t)buf, (long)count, 0, 0);
}
ssize_t write(int fd, const void *buf, size_t count) {
    if (!buf) return -1;
    // Use SYS_SOCKET_WRITE for socket fds, SYS_WRITE for console fds
    // For simplicity, try socket write first; for fd=1 (stdout) this may
    // fail, so we special-case it.
    if (fd == 1 || fd == 2)
        return SYSCALL_FN(SYS_WRITE, fd, (long)(size_t)buf, (long)count, 0, 0);
    return SYSCALL_FN(SYS_SOCKET_WRITE, fd, (long)(size_t)buf, (long)count, 0, 0);
}

off_t lseek(int fd, off_t offset, int whence) {
    (void)fd; (void)offset; (void)whence; return -1;
}
int unlink(const char *path) { (void)path; return -1; }
int remove(const char *path) { (void)path; return -1; }
int rename(const char *oldn, const char *newn) { (void)oldn; (void)newn; return -1; }
int stat(const char *path, struct stat *buf) { (void)path; (void)buf; return -1; }
int fstat(int fd, struct stat *buf) { (void)fd; (void)buf; return -1; }
int lstat(const char *path, struct stat *buf) { (void)path; (void)buf; return -1; }
int access(const char *path, int mode) { (void)path; (void)mode; return 0; }
ssize_t readlink(const char *path, char *buf, size_t bufsiz) { (void)path; (void)buf; (void)bufsiz; return -1; }
int dup(int oldfd) { (void)oldfd; return -1; }
int dup2(int oldfd, int newfd) { (void)oldfd; (void)newfd; return -1; }
int pipe(int pipefd[2]) {
    if (!pipefd) return -1;
    return (int)SYSCALL_FN(SYS_PIPE, (long)pipefd, 0, 0, 0, 0);
}
int fcntl(int fd, int cmd, ...) { (void)fd; (void)cmd; return -1; }
int ioctl(int fd, unsigned long request, ...) { (void)fd; (void)request; return -1; }
int ftruncate(int fd, off_t length) { (void)fd; (void)length; return -1; }
int mkdir(const char *path, mode_t mode) { (void)path; (void)mode; return -1; }
int rmdir(const char *path) { (void)path; return -1; }
DIR *opendir(const char *path) { (void)path; return 0; }
struct dirent *readdir(DIR *dirp) { (void)dirp; return 0; }
int closedir(DIR *dirp) { (void)dirp; return 0; }

// ── Memory functions ──────────────────────────────────────────────────────────

void *memset(void *s, int c, size_t n) {
    unsigned char *p = (unsigned char *)s;
    for (size_t i = 0; i < n; i++) p[i] = (unsigned char)c;
    return s;
}

void *memcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dest;
    const unsigned char *s = (const unsigned char *)src;
    for (size_t i = 0; i < n; i++) d[i] = s[i];
    return dest;
}

void *memmove(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dest;
    const unsigned char *s = (const unsigned char *)src;
    if (d < s) { for (size_t i = 0; i < n; i++) d[i] = s[i]; }
    else { size_t i = n; while (i > 0) { i--; d[i] = s[i]; } }
    return dest;
}

int memcmp(const void *s1, const void *s2, size_t n) {
    const unsigned char *a = (const unsigned char *)s1;
    const unsigned char *b = (const unsigned char *)s2;
    for (size_t i = 0; i < n; i++) { if (a[i] != b[i]) return (int)a[i] - (int)b[i]; }
    return 0;
}

unsigned long __stack_chk_guard = 0;
void __stack_chk_fail(void) { __builtin_trap(); }
void *__memcpy_chk(void *dest, const void *src, size_t n, size_t destlen) {
    if (n > destlen) __builtin_trap();
    return memcpy(dest, src, n);
}
void *__memset_chk(void *s, int c, size_t n, size_t destlen) {
    if (n > destlen) __builtin_trap();
    return memset(s, c, n);
}

// ── String functions ─────────────────────────────────────────────────────────

char *strcpy(char *dest, const char *src) {
    char *r = dest; while ((*dest++ = *src++)); return r;
}
char *strncpy(char *dest, const char *src, size_t n) {
    char *r = dest; while (n-- && (*dest++ = *src++)); while (n--) *dest++ = 0; return r;
}
char *strcat(char *dest, const char *src) {
    char *r = dest; while (*dest) dest++; while ((*dest++ = *src++)); return r;
}
char *strncat(char *dest, const char *src, size_t n) {
    char *r = dest; while (*dest) dest++; while (n-- && *src) *dest++ = *src++; *dest=0; return r;
}
int strcmp(const char *s1, const char *s2) {
    while (*s1 && *s1 == *s2) { s1++; s2++; } return (unsigned char)*s1 - (unsigned char)*s2;
}
int strncmp(const char *s1, const char *s2, size_t n) {
    while (n-- && *s1 && *s1 == *s2) { s1++; s2++; } return (int)(unsigned char)*s1 - (int)(unsigned char)*s2;
}
char *strchr(const char *s, int c) {
    while (*s) { if (*s == (char)c) return (char*)s; s++; } return 0;
}
char *strrchr(const char *s, int c) {
    const char *last=0; while (*s) { if (*s==(char)c) last=s; s++; } return (char*)last;
}
char *strstr(const char *haystack, const char *needle) {
    if (!*needle) return (char*)haystack;
    while (*haystack) { const char *h=haystack,*n=needle; while(*h&&*n&&*h==*n){h++;n++;} if(!*n)return(char*)haystack; haystack++; }
    return 0;
}
char *strdup(const char *s) {
    size_t len=0; while(s[len])len++; char *c=(char*)malloc(len+1); if(c){for(size_t i=0;i<=len;i++)c[i]=s[i];} return c;
}
char *strndup(const char *s, size_t n) {
    size_t len=0; while(len<n&&s[len])len++; char *c=(char*)malloc(len+1); if(c){for(size_t i=0;i<len;i++)c[i]=s[i];c[len]=0;} return c;
}
size_t strspn(const char *s, const char *accept) {
    size_t n=0; while(*s){const char*a=accept;int f=0;while(*a){if(*s==*a){f=1;break;}a++;}if(!f)break;n++;s++;} return n;
}
size_t strcspn(const char *s, const char *reject) {
    size_t n=0; while(*s){const char*r=reject;while(*r){if(*s==*r)return n;r++;}n++;s++;} return n;
}
char *strtok(char *s, const char *delim) {
    static char *last=0; if(!s) s=last; if(!s) return 0;
    while(*s&&strchr(delim,*s))s++; if(!*s){last=0;return 0;}
    char *start=s; while(*s&&!strchr(delim,*s))s++;
    if(*s){*s=0;last=s+1;}else last=0; return start;
}
char *strerror(int errnum) { (void)errnum; return "Unknown error"; }
size_t strlen(const char *s) { size_t n=0; while(*s)n++,s++; return n; }

// ── Wchar stubs ──────────────────────────────────────────────────────────────

#ifndef __cplusplus
typedef int wchar_t;
typedef int wint_t;
#endif
size_t wcslen(const wchar_t *s) { size_t n=0; while(*s)n++,s++; return n; }
wchar_t *wcscpy(wchar_t *d, const wchar_t *s) { wchar_t *r=d; while((*d++=*s++)); return r; }
int wcscmp(const wchar_t *s1, const wchar_t *s2) {
    while(*s1&&*s1==*s2){s1++;s2++;} return (int)(*s1-*s2);
}
int mbtowc(wchar_t *pwc, const char *s, size_t n) {
    if(!s)return 0; if(n==0)return -1; if(pwc)*pwc=(unsigned char)*s; return 1;
}
int wctomb(char *s, wchar_t wc) { if(!s)return 0; *s=(char)wc; return 1; }
size_t mbstowcs(wchar_t *dest, const char *src, size_t n) {
    size_t i; for(i=0;i<n&&src[i];i++)dest[i]=(unsigned char)src[i]; if(i<n)dest[i]=0; return i;
}
size_t wcstombs(char *dest, const wchar_t *src, size_t n) {
    size_t i; for(i=0;i<n&&src[i];i++)dest[i]=(char)src[i]; if(i<n)dest[i]=0; return i;
}
int mblen(const char *s, size_t n) { (void)n; if(!s)return 0; return 1; }
wint_t btowc(int c) { return (unsigned char)c; }
int wctob(wint_t c) { return (int)c; }

// ── ctype (isspace must be before functions that use it) ─────────────────────

int isspace(int c) { return c==' '||c=='\t'||c=='\n'||c=='\r'||c=='\f'||c=='\v'; }
int isalnum(int c) { return (c>='0'&&c<='9')||(c>='a'&&c<='z')||(c>='A'&&c<='Z'); }
int isalpha(int c) { return (c>='a'&&c<='z')||(c>='A'&&c<='Z'); }
int iscntrl(int c) { return (c>=0&&c<=31)||c==127; }
int isdigit(int c) { return c>='0'&&c<='9'; }
int isgraph(int c) { return c>=33&&c<=126; }
int islower(int c) { return c>='a'&&c<='z'; }
int isprint(int c) { return c>=32&&c<=126; }
int ispunct(int c) { return isprint(c)&&!isalnum(c)&&!isspace(c); }
int isupper(int c) { return c>='A'&&c<='Z'; }
int isxdigit(int c) { return isdigit(c)||(c>='a'&&c<='f')||(c>='A'&&c<='F'); }
int isblank(int c) { return c==' '||c=='\t'; }
int tolower(int c) { return (c>='A'&&c<='Z')?c+32:c; }
int toupper(int c) { return (c>='a'&&c<='z')?c-32:c; }

// ── Assert ───────────────────────────────────────────────────────────────────

void __assert_fail(const char *assertion, const char *file,
                   unsigned int line, const char *function) {
    (void)assertion; (void)file; (void)line; (void)function; __builtin_trap();
}

// ── Stdio formatting ─────────────────────────────────────────────────────────

int sprintf(char *str, const char *format, ...) { (void)str; (void)format; return 0; }
int snprintf(char *str, size_t size, const char *format, ...) {
    (void)str; (void)size; (void)format; return 0;
}
int printf(const char *format, ...) { (void)format; return 0; }

// ── FILE I/O stubs ──────────────────────────────────────────────────────────
struct _IO_FILE { int fd; };
struct _IO_FILE *stdin = 0;
struct _IO_FILE *stdout = 0;
struct _IO_FILE *stderr = 0;

int fprintf(struct _IO_FILE *stream, const char *format, ...) { (void)stream; (void)format; return 0; }
struct _IO_FILE *fopen(const char *path, const char *mode) { (void)path; (void)mode; return 0; }
int fclose(struct _IO_FILE *stream) { (void)stream; return 0; }
size_t fread(void *ptr, size_t size, size_t nmemb, struct _IO_FILE *stream) {
    (void)ptr; (void)size; (void)nmemb; (void)stream; return 0;
}
size_t fwrite(const void *ptr, size_t size, size_t nmemb, struct _IO_FILE *stream) {
    (void)ptr; (void)size; (void)nmemb; (void)stream; return 0;
}
int fseek(struct _IO_FILE *stream, long offset, int whence) {
    (void)stream; (void)offset; (void)whence; return 0;
}
long ftell(struct _IO_FILE *stream) { (void)stream; return 0; }
int fflush(struct _IO_FILE *stream) { (void)stream; return 0; }
int feof(struct _IO_FILE *stream) { (void)stream; return 1; }
int fputs(const char *s, struct _IO_FILE *stream) { (void)s; (void)stream; return 0; }
int fputc(int c, struct _IO_FILE *stream) { (void)c; (void)stream; return 0; }
char *fgets(char *s, int size, struct _IO_FILE *stream) {
    (void)s; (void)size; (void)stream; return 0;
}
int vfprintf(struct _IO_FILE *stream, const char *format, va_list ap) {
    (void)stream; (void)format; (void)ap; return 0;
}
int vprintf(const char *format, va_list ap) { (void)format; (void)ap; return 0; }
int puts(const char *s) { (void)s; return 0; }

// ── stdarg helper ───────────────────────────────────────────────────────────
int vasprintf(char **strp, const char *fmt, va_list ap) {
    (void)strp; (void)fmt; (void)ap; return 0;
}
int asprintf(char **strp, const char *fmt, ...) {
    (void)strp; (void)fmt; return 0;
}

// ── Open/read/write (64-bit variants) ─────────────────────────────────────────
int open64(const char *path, int flags, ...) { (void)path; (void)flags; return -1; }
off_t lseek64(int fd, off_t offset, int whence) { (void)fd; (void)offset; (void)whence; return -1; }
int stat64(const char *path, struct stat *buf) { (void)path; (void)buf; return -1; }
int fstat64(int fd, struct stat *buf) { (void)fd; (void)buf; return -1; }

// ── Memory mapping ─────────────────────────────────────────────────────────
void *mmap64(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    (void)fd; (void)offset; (void)prot;
    if (length == 0) return (void*)-1;
    // MAP_ANONYMOUS: allocate via SYS_MMAP
    int map_anon = (flags & 0x10) != 0;
    if (map_anon) {
        void *p = (void*)(long)SYSCALL_FN(SYS_MMAP, (long)addr, (long)length, (long)flags, 0, 0);
        return p;
    }
    return (void*)-1;
}
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return mmap64(addr, length, prot, flags, fd, offset);
}
int munmap(void *addr, size_t length) { (void)addr; (void)length; return 0; }

// ── System info ────────────────────────────────────────────────────────────
struct utsname { char sysname[65]; char nodename[65]; char release[65]; char version[65]; char machine[65]; };
int uname(struct utsname *buf) {
    if (buf) {
        __builtin_strcpy(buf->sysname, "AlloyOS");
        __builtin_strcpy(buf->nodename, "alloy");
        __builtin_strcpy(buf->release, "1.0");
        __builtin_strcpy(buf->version, "1.0");
        __builtin_strcpy(buf->machine, "x86_64");
    }
    return 0;
}

// ── TLS stubs ──────────────────────────────────────────────────────────────
void *__tls_get_addr(void *arr) { (void)arr; return 0; }

// ── Additional glibc internal stubs ────────────────────────────────────────
int __libc_single_threaded = 1;
void __ctype_b_loc(void) { }
void __ctype_tolower_loc(void) { }
void __ctype_toupper_loc(void) { }

// ── Checked string functions ───────────────────────────────────────────────
char *__strcpy_chk(char *dest, const char *src, size_t destlen) {
    (void)destlen; return __builtin_strcpy(dest, src);
}
char *__strncpy_chk(char *dest, const char *src, size_t n, size_t destlen) {
    (void)destlen; return __builtin_strncpy(dest, src, n);
}
int __sprintf_chk(char *str, int flag, size_t destlen, const char *format, ...) {
    (void)str; (void)flag; (void)destlen; (void)format; return 0;
}
int __vsnprintf_chk(char *str, size_t size, int flag, size_t destlen, const char *format, va_list ap) {
    (void)str; (void)size; (void)flag; (void)destlen; (void)format; (void)ap; return 0;
}
int __snprintf_chk(char *str, size_t size, int flag, size_t destlen, const char *format, ...) {
    (void)str; (void)size; (void)flag; (void)destlen; (void)format; return 0;
}
int __fprintf_chk(struct _IO_FILE *stream, int flag, const char *format, ...) {
    (void)stream; (void)flag; (void)format; return 0;
}
int __printf_chk(int flag, const char *format, ...) {
    (void)flag; (void)format; return 0;
}

// ── strcoll ─────────────────────────────────────────────────────────────────
int strcoll(const char *s1, const char *s2) { return __builtin_strcmp(s1, s2); }
char *strtok_r(char *str, const char *delim, char **saveptr) {
    return strtok(str, delim);
}

// ── Additional string/mem functions ─────────────────────────────────────────
void *__memmove_chk(void *dest, const void *src, size_t n, size_t destlen) {
    if (n > destlen) __builtin_trap();
    return memmove(dest, src, n);
}

// ── scanf stubs ─────────────────────────────────────────────────────────────
int sscanf(const char *str, const char *format, ...) { (void)str; (void)format; return 0; }
int vsscanf(const char *str, const char *format, va_list ap) { (void)str; (void)format; (void)ap; return 0; }
int fscanf(struct _IO_FILE *stream, const char *format, ...) { (void)stream; (void)format; return 0; }

// ── syscall wrapper ─────────────────────────────────────────────────────────
// Qt6 objects reference the libc syscall() function: long syscall(long, ...)
// This symbol is also provided in qt6_syscall_stub.c for the Qt6 build.
// Here we skip it because alloy_syscall.h already has a static inline "syscall".
// The Qt6 build uses a separate syscall stub file compiled without alloy_syscall.h.

// ── Compiler builtins ──────────────────────────────────────────────────────
unsigned int __popcountdi2(unsigned long long val) {
    unsigned int count = 0;
    while (val) { count += (unsigned int)(val & 1); val >>= 1; }
    return count;
}

// ── Harfbuzz stubs (from compiled bundled lib) ──────────────────────────────
size_t __fread_chk(void* ptr, size_t size, size_t nmemb, size_t bufsize, struct _IO_FILE* stream) {
    (void)ptr; (void)size; (void)nmemb; (void)bufsize; (void)stream; return 0;
}
int ferror(struct _IO_FILE* stream) { (void)stream; return 0; }
long __isoc23_strtol(const char* nptr, char** endptr, int base) {
    return strtol(nptr, endptr, base);
}
unsigned long __isoc23_strtoul(const char* nptr, char** endptr, int base) {
    return strtoul(nptr, endptr, base);
}
int __isoc23_sscanf(const char *str, const char *format, ...) { (void)str; (void)format; return 0; }

// ── setjmp/longjmp stubs (for libpng) ───────────────────────────────────────
// libpng uses setjmp/longjmp for error handling. We provide atomic-based
// implementations since we don't have the full setjmp machinery.

typedef struct { unsigned long __jmpbuf[8]; int __mask_was_saved; } __jmp_buf_struct;
typedef __jmp_buf_struct jmp_buf[1];
typedef __jmp_buf_struct sigjmp_buf[1];

int _setjmp(jmp_buf env) {
    (void)env;
    return 0;
}

void __longjmp_chk(jmp_buf env, int val) {
    (void)env; (void)val;
    __builtin_trap();
}

int setjmp(jmp_buf env) { return _setjmp(env); }
void longjmp(jmp_buf env, int val) { __longjmp_chk(env, val); }

// ── Additional C library stubs ──────────────────────────────────────────────
void* memchr(const void* s, int c, unsigned long n) {
    for (unsigned long i = 0; i < n; i++) {
        if (((const unsigned char*)s)[i] == (unsigned char)c)
            return (void*)((const unsigned char*)s + i);
    }
    return 0;
}

double hypot(double x, double y) {
    (void)x; (void)y;
    return 0.0;
}

// tzname — POSIX timezone name array
char* tzname[2] = { "UTC", "UTC" };
