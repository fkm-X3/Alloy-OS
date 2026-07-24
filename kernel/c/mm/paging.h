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
#define PAGE_COW        0x200  // Bit 9: Copy-on-write (available bit)

#ifdef ARCH_X86_64
/* x86_64 4-level paging: 512 entries/table, 8 bytes/entry */
typedef uint64_t page_dir_entry_t;
typedef uint64_t page_table_entry_t;
#define PAGE_TABLE_ENTRIES 512
#else
/* x86 2-level paging: 1024 entries/table, 4 bytes/entry */
typedef uint32_t page_dir_entry_t;
typedef uint32_t page_table_entry_t;
#define PAGE_TABLE_ENTRIES 1024
#endif

struct page_directory {
    page_dir_entry_t entries[PAGE_TABLE_ENTRIES];
} __attribute__((aligned(4096)));

struct page_table {
    page_table_entry_t entries[PAGE_TABLE_ENTRIES];
} __attribute__((aligned(4096)));

typedef struct {
    struct page_directory* kernel_directory;
    struct page_table* kernel_tables[PAGE_TABLE_ENTRIES];
} Paging;

extern Paging g_paging;

#ifdef __cplusplus
extern "C" {
#endif

void paging_init();
void paging_enable();
bool paging_map_page(uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags);
void paging_unmap_page(uintptr_t virt_addr);
uintptr_t paging_get_physical_address(uintptr_t virt_addr);
uintptr_t paging_create_directory_phys();
void paging_destroy_directory(uintptr_t pd_phys);
bool paging_switch_to_directory(uintptr_t pd_phys);
uintptr_t paging_get_kernel_directory_phys();
uintptr_t paging_clone_directory(uintptr_t pd_phys);
uintptr_t paging_fork_directory(uintptr_t pd_phys);
uint8_t paging_handle_cow_fault(uintptr_t fault_addr);
bool paging_map_page_in_pd(uintptr_t pd_phys, uintptr_t virt_addr, uintptr_t phys_addr, uint32_t flags);
bool paging_demand_map_kernel_page(uint64_t fault_addr, uint64_t user_cr3);
void* paging_temp_map_frame(uintptr_t phys_addr);
void paging_temp_unmap_frame(void);

#ifdef __cplusplus
}
#endif

#endif // ALLOY_PAGING_H