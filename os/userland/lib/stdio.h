#ifndef _STDIO_H
#define _STDIO_H

#ifdef __cplusplus
extern "C" {
#endif

int puts(const char *s);
int write(int fd, const void *buf, int len);

#ifdef __cplusplus
}
#endif

#endif
