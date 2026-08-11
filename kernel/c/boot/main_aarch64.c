#include "types.h"
#include "../mm/pmm.h"
#include "../mm/vmm.h"
#include "../drivers/serial.h"
#include "../drivers/timer.h"
#include "../drivers/pl110.h"
#include "../mm/paging.h"

void syscall_init();
void init_gdt();
void init_idt();
void rust_main();

#define PL110_FRAMEBUFFER 0x47D00000
#define SCREEN_WIDTH  1024
#define SCREEN_HEIGHT 768

static void arch_halt() {
    asm volatile("wfi");
}

void kernel_main(uint32_t magic, uint32_t multiboot_addr) {
    (void)magic;
    (void)multiboot_addr;

    init_serial();

    serial_print("[Boot] Alloy OS booting on aarch64\n");
    serial_print("[Boot] Architecture: aarch64 (ARM64)\n");

    init_gdt();
    init_idt();
    syscall_init();
    pmm_init(0);
    paging_init();
    paging_enable();
    vmm_init();
    timer_init_ffi(1000);
    pl110_init(PL110_FRAMEBUFFER, SCREEN_WIDTH, SCREEN_HEIGHT);

    serial_print("[Boot] Calling Rust kernel main...\n");
    rust_main();

    while(1) { arch_halt(); }
}
