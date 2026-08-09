use ::core::arch::asm;
extern "C" {
    fn timer_get_uptime_ms_ffi() -> uint64_t;
    fn timer_handler();
    fn rust_handle_page_fault(addr: uintptr_t, err_code: uint32_t);
    static mut _exception_vectors: uint8_t;
}
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
#[no_mangle]
pub unsafe extern "C" fn exception_handler_el1() {
    loop {
        asm!("wfi\n", options(preserves_flags));
    }
}
#[no_mangle]
pub unsafe extern "C" fn irq_handler_el1() {
    timer_handler();
}
#[no_mangle]
pub unsafe extern "C" fn page_fault_handler(mut far: uint64_t, mut esr: uint64_t) {
    let mut err_code: uint32_t = (esr & 0xff as uint64_t) as uint32_t;
    rust_handle_page_fault(far as uintptr_t, err_code);
}
#[no_mangle]
pub unsafe extern "C" fn svc_handler(
    mut num: uint64_t,
    mut arg0: uint64_t,
    mut arg1: uint64_t,
    mut arg2: uint64_t,
    mut arg3: uint64_t,
    mut arg4: uint64_t,
) -> uint32_t {
    extern "C" {
        #[link_name = "syscall_dispatcher"]
        fn syscall_dispatcher_0(
            syscall_no: uint32_t,
            arg0_0: uint32_t,
            arg1_0: uint32_t,
            arg2_0: uint32_t,
            arg3_0: uint32_t,
            arg4_0: uint32_t,
            frame: *mut uint32_t,
        ) -> uint32_t;
    }
    return syscall_dispatcher_0(
        num as uint32_t,
        arg0 as uint32_t,
        arg1 as uint32_t,
        arg2 as uint32_t,
        arg3 as uint32_t,
        arg4 as uint32_t,
        ::core::ptr::null_mut::<uint32_t>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn init_idt() {
    asm!(
        "msr vbar_el1, {0}\n", inlateout(reg) & raw mut _exception_vectors => _,
        options(preserves_flags)
    );
    asm!("isb\n", options(preserves_flags));
    asm!("msr daifclr, #0b0010\n", options(preserves_flags));
}
#[no_mangle]
pub unsafe extern "C" fn get_system_uptime_ms() -> uint64_t {
    return timer_get_uptime_ms_ffi();
}
