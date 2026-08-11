use ::core::arch::asm;
extern "C" {
    fn pmm_init(multiboot_addr: uint32_t);
    fn vmm_init();
    fn init_serial();
    fn serial_print(str: *const ::core::ffi::c_char);
    fn timer_init_ffi(frequency: uint32_t);
    fn pl110_init(fb_addr: uint32_t, width: uint16_t, height: uint16_t);
    fn paging_init();
    fn paging_enable();
    fn syscall_init();
    fn init_gdt();
    fn init_idt();
    fn rust_main();
}
pub type uint16_t = u16;
pub type uint32_t = u32;
pub const PL110_FRAMEBUFFER: ::core::ffi::c_int = 0x47D00000 as ::core::ffi::c_int;
pub const SCREEN_WIDTH: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const SCREEN_HEIGHT: ::core::ffi::c_int = 768 as ::core::ffi::c_int;
unsafe extern "C" fn arch_halt() {
    asm!("wfi\n", options(preserves_flags));
}
#[no_mangle]
pub unsafe extern "C" fn kernel_main(mut magic: uint32_t, mut multiboot_addr: uint32_t) {
    init_serial();
    serial_print(
        b"[Boot] Alloy OS booting on aarch64\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    serial_print(
        b"[Boot] Architecture: aarch64 (ARM64)\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    init_gdt();
    init_idt();
    syscall_init();
    pmm_init(0 as uint32_t);
    paging_init();
    paging_enable();
    vmm_init();
    timer_init_ffi(1000 as uint32_t);
    pl110_init(
        PL110_FRAMEBUFFER as uint32_t,
        SCREEN_WIDTH as uint16_t,
        SCREEN_HEIGHT as uint16_t,
    );
    serial_print(
        b"[Boot] Calling Rust kernel main...\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    rust_main();
    loop {
        arch_halt();
    }
}
