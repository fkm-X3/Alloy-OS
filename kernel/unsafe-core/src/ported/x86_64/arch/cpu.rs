use ::core::arch::asm;
pub type uint32_t = u32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cpu_info {
    pub vendor: [::core::ffi::c_char; 16],
    pub features: uint32_t,
    pub family: uint32_t,
    pub model: uint32_t,
    pub stepping: uint32_t,
}
#[inline]
unsafe extern "C" fn cpuid(
    mut code: uint32_t,
    mut eax: *mut uint32_t,
    mut ebx: *mut uint32_t,
    mut ecx: *mut uint32_t,
    mut edx: *mut uint32_t,
) {
    asm!(
        "cpuid\n", "mov {restmp0:x}, %bx\n", restmp0 = lateout(reg) * ebx,
        inlateout("ax") code => * eax, lateout("cx") * ecx, lateout("dx") * edx,
        options(preserves_flags, att_syntax)
    );
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_vendor(mut vendor: *mut ::core::ffi::c_char) {
    let mut eax: uint32_t = 0;
    let mut ebx: uint32_t = 0;
    let mut ecx: uint32_t = 0;
    let mut edx: uint32_t = 0;
    cpuid(
        0 as uint32_t,
        &raw mut eax,
        &raw mut ebx,
        &raw mut ecx,
        &raw mut edx,
    );
    *(vendor.offset(0 as ::core::ffi::c_int as isize) as *mut uint32_t) = ebx;
    *(vendor.offset(4 as ::core::ffi::c_int as isize) as *mut uint32_t) = edx;
    *(vendor.offset(8 as ::core::ffi::c_int as isize) as *mut uint32_t) = ecx;
    *vendor.offset(12 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_features() -> uint32_t {
    let mut eax: uint32_t = 0;
    let mut ebx: uint32_t = 0;
    let mut ecx: uint32_t = 0;
    let mut edx: uint32_t = 0;
    cpuid(
        1 as uint32_t,
        &raw mut eax,
        &raw mut ebx,
        &raw mut ecx,
        &raw mut edx,
    );
    return edx;
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_model_info(
    mut family: *mut uint32_t,
    mut model: *mut uint32_t,
    mut stepping: *mut uint32_t,
) {
    let mut eax: uint32_t = 0;
    let mut ebx: uint32_t = 0;
    let mut ecx: uint32_t = 0;
    let mut edx: uint32_t = 0;
    cpuid(
        1 as uint32_t,
        &raw mut eax,
        &raw mut ebx,
        &raw mut ecx,
        &raw mut edx,
    );
    *stepping = eax & 0xf as uint32_t;
    *model = eax >> 4 as ::core::ffi::c_int & 0xf as uint32_t;
    *family = eax >> 8 as ::core::ffi::c_int & 0xf as uint32_t;
    let mut ext_model: uint32_t = eax >> 16 as ::core::ffi::c_int & 0xf as uint32_t;
    let mut ext_family: uint32_t = eax >> 20 as ::core::ffi::c_int & 0xff as uint32_t;
    if *family == 0xf as uint32_t {
        *family = (*family).wrapping_add(ext_family);
    }
    if *family == 0x6 as uint32_t || *family == 0xf as uint32_t {
        *model = (*model).wrapping_add(ext_model << 4 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn cpu_detect(mut info: *mut cpu_info) {
    if info.is_null() {
        return;
    }
    cpu_get_vendor(&raw mut (*info).vendor as *mut ::core::ffi::c_char);
    (*info).features = cpu_get_features();
    cpu_get_model_info(
        &raw mut (*info).family,
        &raw mut (*info).model,
        &raw mut (*info).stepping,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_vendor_ffi(mut vendor: *mut ::core::ffi::c_char) {
    cpu_get_vendor(vendor);
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_features_ffi() -> uint32_t {
    return cpu_get_features();
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_model_info_ffi(
    mut family: *mut uint32_t,
    mut model: *mut uint32_t,
    mut stepping: *mut uint32_t,
) {
    cpu_get_model_info(family, model, stepping);
}
