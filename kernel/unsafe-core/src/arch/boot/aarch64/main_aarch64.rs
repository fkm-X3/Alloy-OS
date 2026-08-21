use ::core::arch::asm;

// Linker symbols (not C): `rust_main` is the safe kernel crate's entry point
// (`#[unsafe(no_mangle)] pub extern "C" fn rust_main` in alloy-kernel-rust).
// All former C-ABI boot calls below go straight to their Rust implementations
// in this crate instead of through legacy symbol names.
extern "C" {
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
    use crate::drivers::pl110::Pl110;
    use crate::drivers::serial::Serial;
    use crate::drivers::timer::SystemTimer;
    use crate::mem::{paging_aarch64, pmm, vmm};

    Serial::init();
    Serial::write_str("[Boot] Alloy OS booting on aarch64\n");
    Serial::write_str("[Boot] Architecture: aarch64 (ARM64)\n");
    crate::arch::aarch64::gdt_init();
    crate::arch::aarch64::idt_init();
    crate::arch::aarch64::syscall_init();
    pmm::pmm_init(0);
    paging_aarch64::paging_init();
    paging_aarch64::paging_enable();
    vmm::vmm_init();
    SystemTimer::init(1000);
    Pl110::init(
        PL110_FRAMEBUFFER as u32,
        SCREEN_WIDTH as u16,
        SCREEN_HEIGHT as u16,
    );
    Serial::write_str("[Boot] Calling Rust kernel main...\n");
    rust_main();
    loop { arch_halt(); }
}
