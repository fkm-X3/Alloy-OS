use ::core::arch::asm;
extern "C" {
    fn gdt_flush(gdt_ptr: uint64_t);
    static mut kernel_stack_top: uint64_t;
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct gdt_entry {
    pub limit_low: uint16_t,
    pub base_low: uint16_t,
    pub base_middle: uint8_t,
    pub access: uint8_t,
    pub granularity: uint8_t,
    pub base_high: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct gdt_ptr {
    pub limit: uint16_t,
    pub base: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct tss(pub tss_Inner);
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct tss_Inner {
    pub reserved0: uint32_t,
    pub rsp0: uint64_t,
    pub rsp1: uint64_t,
    pub rsp2: uint64_t,
    pub reserved1: uint64_t,
    pub ist1: uint64_t,
    pub ist2: uint64_t,
    pub ist3: uint64_t,
    pub ist4: uint64_t,
    pub ist5: uint64_t,
    pub ist6: uint64_t,
    pub ist7: uint64_t,
    pub reserved2: uint64_t,
    pub reserved3: uint16_t,
    pub iopb_offset: uint16_t,
}
#[allow(dead_code, non_upper_case_globals)]
const tss_PADDING: usize = ::core::mem::size_of::<tss>() - ::core::mem::size_of::<tss_Inner>();
#[no_mangle]
pub static mut gdt: [gdt_entry; 7] = [gdt_entry {
    limit_low: 0,
    base_low: 0,
    base_middle: 0,
    access: 0,
    granularity: 0,
    base_high: 0,
}; 7];
#[no_mangle]
pub static mut gdtp: gdt_ptr = gdt_ptr { limit: 0, base: 0 };
#[no_mangle]
pub static mut kernel_tss: tss = tss(tss_Inner {
    reserved0: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    reserved1: 0,
    ist1: 0,
    ist2: 0,
    ist3: 0,
    ist4: 0,
    ist5: 0,
    ist6: 0,
    ist7: 0,
    reserved2: 0,
    reserved3: 0,
    iopb_offset: 0,
});
unsafe extern "C" fn gdt_set_gate(
    mut num: ::core::ffi::c_int,
    mut base: uint64_t,
    mut limit: uint64_t,
    mut access: uint8_t,
    mut gran: uint8_t,
) {
    gdt[num as usize].base_low = (base & 0xffff as uint64_t) as uint16_t;
    gdt[num as usize].base_middle =
        (base >> 16 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t;
    gdt[num as usize].base_high = (base >> 24 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t;
    gdt[num as usize].limit_low = (limit & 0xffff as uint64_t) as uint16_t;
    gdt[num as usize].granularity = (limit >> 16 as ::core::ffi::c_int & 0xf as uint64_t
        | (gran as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int) as uint64_t)
        as uint8_t;
    gdt[num as usize].access = access;
}
unsafe extern "C" fn tss_set_gate(
    mut num: ::core::ffi::c_int,
    mut base: uint64_t,
    mut limit: uint32_t,
) {
    gdt[num as usize].limit_low = (limit & 0xffff as uint32_t) as uint16_t;
    gdt[num as usize].base_low = (base & 0xffff as uint64_t) as uint16_t;
    gdt[num as usize].base_middle =
        (base >> 16 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t;
    gdt[num as usize].access = 0x89 as uint8_t;
    gdt[num as usize].granularity =
        (limit >> 16 as ::core::ffi::c_int & 0xf as uint32_t) as uint8_t;
    gdt[num as usize].base_high = (base >> 24 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t;
    let mut high: *mut gdt_entry = (&raw mut gdt as *mut gdt_entry)
        .offset((num + 1 as ::core::ffi::c_int) as isize)
        as *mut gdt_entry;
    (*high).limit_low = (base >> 32 as ::core::ffi::c_int & 0xffff as uint64_t) as uint16_t;
    (*high).base_low = (base >> 48 as ::core::ffi::c_int & 0xffff as uint64_t) as uint16_t;
    (*high).base_middle = 0 as uint8_t;
    (*high).access = 0 as uint8_t;
    (*high).granularity = 0 as uint8_t;
    (*high).base_high = 0 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn init_gdt() {
    gdtp.limit = (::core::mem::size_of::<gdt_entry>() as usize)
        .wrapping_mul(7 as usize)
        .wrapping_sub(1 as usize) as uint16_t;
    gdtp.base = &raw mut gdt as uint64_t;
    gdt_set_gate(
        0 as ::core::ffi::c_int,
        0 as uint64_t,
        0 as uint64_t,
        0 as uint8_t,
        0 as uint8_t,
    );
    gdt_set_gate(
        1 as ::core::ffi::c_int,
        0 as uint64_t,
        0 as uint64_t,
        0x9a as uint8_t,
        0x20 as uint8_t,
    );
    gdt_set_gate(
        2 as ::core::ffi::c_int,
        0 as uint64_t,
        0 as uint64_t,
        0x92 as uint8_t,
        0 as uint8_t,
    );
    gdt_set_gate(
        3 as ::core::ffi::c_int,
        0 as uint64_t,
        0 as uint64_t,
        0xf2 as uint8_t,
        0 as uint8_t,
    );
    gdt_set_gate(
        4 as ::core::ffi::c_int,
        0 as uint64_t,
        0 as uint64_t,
        0xfa as uint8_t,
        0x20 as uint8_t,
    );
    crate::raw::string::memset(
        &raw mut kernel_tss as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<tss>() as crate::raw::string::size_t,
    );
    kernel_tss.0.rsp0 = kernel_stack_top;
    kernel_tss.0.iopb_offset = ::core::mem::size_of::<tss>() as uint16_t;
    tss_set_gate(
        5 as ::core::ffi::c_int,
        &raw mut kernel_tss as uint64_t,
        (::core::mem::size_of::<tss>() as usize).wrapping_sub(1 as usize) as uint32_t,
    );
    gdt_flush(&raw mut gdtp as uint64_t);
    asm!(
        "ltr %ax\n", inlateout("ax") 0x28 as ::core::ffi::c_int as uint16_t => _,
        options(preserves_flags, att_syntax)
    );
}
#[no_mangle]
pub unsafe extern "C" fn tss_update_rsp0(mut rsp0: uint64_t) {
    kernel_tss.0.rsp0 = rsp0;
}
