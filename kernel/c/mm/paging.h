#ifndef ALLOY_PAGING_H
#define ALLOY_PAGING_H

#include "../boot/types.h"

#define PAGE_PRESENT    0x001
#define PAGE_WRITE      0x002
#define PAGE_USER       0x004
#define PAGE_WRITETHROUGH 0x008
#define PAGE_CACHE_DISABLE 0x010
#define PAGE_ACCESSED   0x020
#define PAGE_DIRTY      0x040
#define PAGE_SIZE_FLAG  0x080
#define PAGE_GLOBAL     0x100

typedef uint32_t page_dir_entry_t;
typedef uint32_t page_table_entry_t;

struct page_directory {
    page_dir_entry_t entries[1024];
} __attribute__((aligned(4096)));

struct page_table {
    page_table_entry_t entries[1024];
} __attribute__((aligned(4096)));

typedef struct {
    struct page_directory* kernel_directory;
    struct page_table* kernel_tables[1024];
} Paging;

extern Paging g_paging;

#ifdef __cplusplus
extern "C" {
#endif

void paging_init();
void paging_enable();
bool paging_map_page(uint32_t virt_addr, uint32_t phys_addr, uint32_t flags);
void paging_unmap_page(uint32_t virt_addr);
uint32_t paging_get_physical_address(uint32_t virt_addr);
uint32_t paging_create_directory_phys();
void paging_destroy_directory(uint32_t pd_phys);
bool paging_switch_to_directory(uint32_t pd_phys);
uint32_t paging_get_kernel_directory_phys();

#ifdef __cplusplus
}
#endif

#endif // ALLOY_PAGING_H