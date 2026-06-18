#ifndef _SHM_H
#define _SHM_H

int alloy_shm_alloc(unsigned int width, unsigned int height, unsigned int bpp);
void *alloy_shm_user_vaddr(int fd);

#endif
