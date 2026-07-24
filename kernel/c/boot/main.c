#include "multiboot2.h"
#include "types.h"
#include "../mm/pmm.h"
#include "../mm/paging.h"
#include "../mm/vmm.h"
#include "../drivers/serial.h"
#include "../drivers/vesa.h"
#include "../drivers/timer.h"

void init_gdt();
void init_idt();
void syscall_init();
void tss_update_rsp0(uint64_t rsp0);
void rust_main();

extern uint64_t kernel_stack_top;

static void arch_halt() {
    asm volatile("cli; hlt");
}

void kernel_main(uint32_t magic, uint32_t multiboot_addr) {
    init_serial();

    if (magic != MULTIBOOT2_BOOTLOADER_MAGIC) {
        while(1) { arch_halt(); }
    }

    init_gdt();
    init_idt();
    syscall_init();
    tss_update_rsp0(kernel_stack_top);
    pmm_init(multiboot_addr);
    paging_init();
    paging_enable();
    vmm_init();
    vesa_init_from_multiboot(multiboot_addr);
    rust_main();

    while(1) { arch_halt(); }
}
