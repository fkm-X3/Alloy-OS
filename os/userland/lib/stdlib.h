#ifndef _STDLIB_H
#define _STDLIB_H

#ifdef __cplusplus
extern "C" {
#endif

void _exit(int status);
void *brk(void *addr);
void *sbrk(int incr);

#ifdef __cplusplus
}
#endif

#endif
