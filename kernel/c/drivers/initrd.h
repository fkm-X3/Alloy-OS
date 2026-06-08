#ifndef ALLOY_INITRD_H
#define ALLOY_INITRD_H

#include "boot/types.h"

#define MAX_INITRD_MODULES 16

struct initrd_module {
    uint32_t start;
    uint32_t end;
    uint32_t size;
    char     cmdline[64];
};

#ifdef __cplusplus
extern "C" {
#endif

void initrd_init(uint32_t multiboot_addr);
int  initrd_module_count(void);
int  initrd_get_module(int index, struct initrd_module* mod);
uint32_t initrd_module_start_ffi(int index);
uint32_t initrd_module_end_ffi(int index);
uint32_t initrd_module_size_ffi(int index);
void     initrd_module_cmdline_ffi(int index, char* buf, uint32_t max_len);
int      initrd_has_modules_ffi(void);

#ifdef __cplusplus
}
#endif

#endif
