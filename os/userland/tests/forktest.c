// forktest.c -- x86_64 COW fork smoke test (syscall 20)
// Parent forks; the child writes a shared stack page (COW split), both print
// their views of the variable.  Expected serial:
//   forktest[PARENT] forked pid=N
//   forktest[CHILD]  shared before write = 42
//   forktest[CHILD]  shared after write  = 43
//   forktest[PARENT] shared still 42 (COW isolated)
//
// If COW isolation is broken the parent would print 43.

#include <stdint.h>

#define SYS_EXIT   0
#define SYS_WRITE  6
#define SYS_FORK   20

#ifdef __x86_64__
#include "alloy_syscall_x86_64.h"

static void say(const char *s) {
    int len = 0;
    while (s[len]) len++;
    syscall_x86_64(SYS_WRITE, 1, (uintptr_t)s, len, 0, 0);
}

static void say_int(uint32_t v) {
    char buf[12];
    int i = 11;
    buf[i] = 0;
    if (v == 0) buf[--i] = '0';
    while (v) {
        buf[--i] = (char)('0' + (v % 10));
        v /= 10;
    }
    say(&buf[i]);
}

int main(void) {
    volatile uint32_t shared = 42;
    long pid = syscall_x86_64(SYS_FORK, 0, 0, 0, 0, 0);
    if (pid == 0) {
        say("forktest[CHILD] shared before write = ");
        say_int(shared);
        say("\n");
        shared = 43;  // write to shared COW page -> COW split
        say("forktest[CHILD] shared after write  = ");
        say_int(shared);
        say("\n");
        syscall_x86_64(SYS_EXIT, 0, 0, 0, 0, 0);
    }
    say("forktest[PARENT] forked pid=");
    say_int((uint32_t)pid);
    say("\n");
    say("forktest[PARENT] shared still ");
    say_int(shared);
    say(" (COW isolated)\n");
    return 0;
}
#else
int main(void) {
    return 0;
}
#endif
