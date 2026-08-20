use ::core::arch::asm;
extern "C" {
    fn pmm_init(multiboot_addr: u32);
    fn vmm_init();
    fn init_serial();
    fn serial_print(str: *const ::core::ffi::c_char);
    fn timer_init_ffi(frequency: u32);
    fn pl110_init(fb_addr: u64, width: u16, height: u16);
    fn paging_init();
    fn paging_enable();
    fn rust_main();
}
pub const PL110_FRAMEBUFFER: u64 = 0x47D0_0000;
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;
unsafe extern "C" fn arch_halt() {
    asm!("wfi\n", options(preserves_flags));
}
#[no_mangle]
pub unsafe extern "C" fn kernel_main(magic: u32, _multiboot_addr: u32) {
    init_serial();
    serial_print(b"[Boot] Alloy OS booting on aarch64\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"[Boot] Architecture: aarch64 (ARM64)\n\0" as *const u8 as *const ::core::ffi::c_char);
    crate::arch::aarch64::gdt_init();
    crate::arch::aarch64::idt_init();
    crate::arch::aarch64::syscall_init();
    pmm_init(0);
    paging_init();
    paging_enable();
    vmm_init();
    timer_init_ffi(1000);
    pl110_init(PL110_FRAMEBUFFER, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16);
    serial_print(b"[Boot] Calling Rust kernel main...\n\0" as *const u8 as *const ::core::ffi::c_char);
    rust_main();
    loop { arch_halt(); }
}
