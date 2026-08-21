use ::core::arch::asm;

// Linker symbols (not C): `rust_main` is the safe kernel crate's entry point
// (`#[unsafe(no_mangle)] pub extern "C" fn rust_main` in alloy-kernel-rust);
// `kernel_stack_top` is provided by the syscall_entry asm, which survives the
// C removal. All former C-ABI boot calls below go straight to their Rust
// implementations in this crate instead of through legacy symbol names.
extern "C" {
    fn rust_main();
    static mut kernel_stack_top: u64;
}
pub const MULTIBOOT2_BOOTLOADER_MAGIC: ::core::ffi::c_int = 0x36d76289;
unsafe extern "C" fn arch_halt() {
    asm!("cli; hlt\n", options(preserves_flags, att_syntax));
}
#[no_mangle]
pub unsafe extern "C" fn kernel_main(magic: u32, multiboot_addr: u32) {
    use crate::drivers::initrd::Initrd;
    use crate::drivers::serial::Serial;
    use crate::drivers::vesa::Vesa;
    use crate::mem::{paging, pmm, vmm};

    Serial::init();
    if magic != MULTIBOOT2_BOOTLOADER_MAGIC as u32 {
        loop { arch_halt(); }
    }
    crate::arch::x86_64::gdt_init();
    crate::arch::x86_64::idt_init();
    crate::arch::x86_64::syscall_init();
    crate::arch::x86_64::tss_update_rsp0(kernel_stack_top);
    pmm::pmm_init(multiboot_addr);
    paging::paging_init();
    paging::paging_enable();
    vmm::vmm_init();
    Vesa::init(multiboot_addr);
    Initrd::init(multiboot_addr);
    rust_main();
    loop { arch_halt(); }
}

#[no_mangle]
pub unsafe extern "C" fn rust_dispatcher(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    crate::api::callback::invoke_syscall_dispatcher(eax, ebx, ecx, edx)
}
