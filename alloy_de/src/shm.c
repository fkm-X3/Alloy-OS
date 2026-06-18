#include "shm.h"
#include "alloy_syscall.h"

int alloy_shm_alloc(unsigned int width, unsigned int height,
                    unsigned int bpp) {
    return syscall(SYS_ALLOC_SHM, (int)width, (int)height, (int)bpp, 0, 0);
}

void *alloy_shm_user_vaddr(int fd) {
    return (void*)syscall(SYS_SHM_USER_VADDR, fd, 0, 0, 0, 0);
}
