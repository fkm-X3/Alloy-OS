use ::core::arch::asm;
extern "C" {
    fn pmm_init(multiboot_addr: uint32_t);
    fn paging_init();
    fn paging_enable();
    fn vmm_init();
    fn init_serial();
    fn vesa_init_from_multiboot(multiboot_addr: uint32_t);
    fn init_gdt();
    fn init_idt();
    fn syscall_init();
    fn tss_update_rsp0(rsp0: uint64_t);
    fn rust_main();
    static mut kernel_stack_top: uint64_t;
}
pub type uint32_t = u32;
pub type uint64_t = u64;
pub const MULTIBOOT2_BOOTLOADER_MAGIC: ::core::ffi::c_int = 0x36d76289 as ::core::ffi::c_int;
unsafe extern "C" fn arch_halt() {
    asm!("cli; hlt\n", options(preserves_flags, att_syntax));
}
#[no_mangle]
pub unsafe extern "C" fn kernel_main(mut magic: uint32_t, mut multiboot_addr: uint32_t) {
    init_serial();
    if magic != MULTIBOOT2_BOOTLOADER_MAGIC as uint32_t {
        loop {
            arch_halt();
        }
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
    loop {
        arch_halt();
    }
}
