use ::core::arch::asm;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cpu_info {
    pub vendor: [::core::ffi::c_char; 16],
    pub features: uint32_t,
    pub family: uint32_t,
    pub model: uint32_t,
    pub stepping: uint32_t,
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_vendor(mut vendor: *mut ::core::ffi::c_char) {
    let mut midr: uint64_t = 0;
    asm!("mrs {0}, midr_el1\n", lateout(reg) midr, options(preserves_flags));
    let mut implementer: uint32_t =
        (midr >> 16 as ::core::ffi::c_int & 0xff as uint64_t) as uint32_t;
    match implementer {
        65 => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"ARM Limited\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                12 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
        66 => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"Broadcom\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
        67 => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"Cavium\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                7 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
        78 => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"NVIDIA\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                7 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
        81 => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"Qualcomm\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
        83 => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"Samsung\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                8 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
        _ => {
            crate::raw::string::memcpy(
                vendor as *mut ::core::ffi::c_void,
                b"Unknown\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                8 as ::core::ffi::c_int as ::core::ffi::c_ulong as crate::raw::string::size_t,
            );
        }
    }
    *vendor.offset(12 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_features() -> uint32_t {
    let mut isar0: uint64_t = 0;
    asm!("mrs {0}, id_aa64isar0_el1\n", lateout(reg) isar0, options(preserves_flags));
    return isar0 as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn cpu_get_model_info(
    mut family: *mut uint32_t,
    mut model: *mut uint32_t,
    mut stepping: *mut uint32_t,
) {
    let mut midr: uint64_t = 0;
    asm!("mrs {0}, midr_el1\n", lateout(reg) midr, options(preserves_flags));
    *family = (midr >> 20 as ::core::ffi::c_int & 0xf as uint64_t) as uint32_t;
    *model = (midr >> 4 as ::core::ffi::c_int & 0xfff as uint64_t) as uint32_t;
    *stepping = (midr & 0xf as uint64_t) as uint32_t;
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
