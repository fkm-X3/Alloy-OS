use ::core::arch::asm;
extern "C" {
    fn pmm_init(multiboot_addr: u32);
    fn paging_init();
    fn paging_enable();
    fn vmm_init();
    fn init_serial();
    fn vesa_init_from_multiboot(multiboot_addr: u32);
    fn initrd_init(multiboot_addr: u32);
    fn rust_main();
    static mut kernel_stack_top: u64;
}
pub const MULTIBOOT2_BOOTLOADER_MAGIC: ::core::ffi::c_int = 0x36d76289;
unsafe extern "C" fn arch_halt() {
    asm!("cli; hlt\n", options(preserves_flags, att_syntax));
}
#[no_mangle]
pub unsafe extern "C" fn kernel_main(magic: u32, multiboot_addr: u32) {
    init_serial();
    if magic != MULTIBOOT2_BOOTLOADER_MAGIC as u32 {
        loop { arch_halt(); }
    }
    // Safe arch init — these were moved from the ported C2Rust modules
    // to the safe arch layer in Session 3.6.
    crate::arch::x86_64::gdt_init();
    crate::arch::x86_64::idt_init();
    crate::arch::x86_64::syscall_init();
    crate::arch::x86_64::tss_update_rsp0(kernel_stack_top);
    pmm_init(multiboot_addr);
    paging_init();
    paging_enable();
    vmm_init();
    vesa_init_from_multiboot(multiboot_addr);
    initrd_init(multiboot_addr);
    rust_main();
    loop { arch_halt(); }
}
