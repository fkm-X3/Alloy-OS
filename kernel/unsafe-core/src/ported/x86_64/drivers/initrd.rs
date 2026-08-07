extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
}
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uintptr_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct initrd_module {
    pub start: uintptr_t,
    pub end: uintptr_t,
    pub size: uintptr_t,
    pub cmdline: [::core::ffi::c_char; 64],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiboot_tag {
    pub type_0: uint32_t,
    pub size: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct multiboot_tag_module {
    pub type_0: uint32_t,
    pub size: uint32_t,
    pub mod_start: uint32_t,
    pub mod_end: uint32_t,
    pub cmdline: [::core::ffi::c_char; 0],
}
pub const MAX_INITRD_MODULES: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_END: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_MODULE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static mut g_modules: [initrd_module; 16] = [initrd_module {
    start: 0,
    end: 0,
    size: 0,
    cmdline: [0; 64],
}; 16];
static mut g_module_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn initrd_init(mut multiboot_addr: uint32_t) {
    serial_print(
        b"[INITRD] Scanning multiboot modules...\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    g_module_count = 0 as ::core::ffi::c_int;
    if multiboot_addr == 0 as uint32_t {
        serial_print(b"[INITRD] No multiboot info\n\0" as *const u8 as *const ::core::ffi::c_char);
        return;
    }
    let mut tag: *mut multiboot_tag =
        multiboot_addr.wrapping_add(8 as uint32_t) as *mut multiboot_tag;
    while (*tag).type_0 != MULTIBOOT_TAG_TYPE_END as uint32_t {
        if (*tag).type_0 == MULTIBOOT_TAG_TYPE_MODULE as uint32_t {
            if g_module_count >= MAX_INITRD_MODULES {
                serial_print(
                    b"[INITRD] Too many modules\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
                break;
            } else {
                let mut mod_0: *mut multiboot_tag_module = tag as *mut multiboot_tag_module;
                let mut m: *mut initrd_module = (&raw mut g_modules as *mut initrd_module)
                    .offset(g_module_count as isize)
                    as *mut initrd_module;
                (*m).start = (*mod_0).mod_start as uintptr_t;
                (*m).end = (*mod_0).mod_end as uintptr_t;
                (*m).size = (*mod_0).mod_end.wrapping_sub((*mod_0).mod_start) as uintptr_t;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while *(&raw mut (*mod_0).cmdline as *mut ::core::ffi::c_char).offset(i as isize)
                    as ::core::ffi::c_int
                    != 0
                    && i < 63 as ::core::ffi::c_int
                {
                    (*m).cmdline[i as usize] =
                        *(&raw mut (*mod_0).cmdline as *mut ::core::ffi::c_char).offset(i as isize);
                    i += 1;
                }
                (*m).cmdline[i as usize] = '\0' as i32 as ::core::ffi::c_char;
                serial_print(b"[INITRD] Module \0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex(g_module_count as uint32_t);
                serial_print(b": start=0x\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*m).start as uint32_t);
                serial_print(b" end=0x\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*m).end as uint32_t);
                serial_print(b" size=\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*m).size as uint32_t);
                serial_print(b" cmdline=\"\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print(&raw mut (*m).cmdline as *mut ::core::ffi::c_char);
                serial_print(b"\"\n\0" as *const u8 as *const ::core::ffi::c_char);
                g_module_count += 1;
            }
        }
        tag = (tag as *mut uint8_t).offset(
            ((*tag).size.wrapping_add(7 as uint32_t) & !(7 as ::core::ffi::c_int) as uint32_t)
                as isize,
        ) as *mut multiboot_tag;
    }
    serial_print(b"[INITRD] Found \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(g_module_count as uint32_t);
    serial_print(b" module(s)\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn initrd_module_count() -> ::core::ffi::c_int {
    return g_module_count;
}
#[no_mangle]
pub unsafe extern "C" fn initrd_get_module(
    mut index: ::core::ffi::c_int,
    mut mod_0: *mut initrd_module,
) -> ::core::ffi::c_int {
    if index < 0 as ::core::ffi::c_int || index >= g_module_count || mod_0.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *mod_0 = g_modules[index as usize];
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn initrd_module_start_ffi(mut index: ::core::ffi::c_int) -> uintptr_t {
    if index < 0 as ::core::ffi::c_int || index >= g_module_count {
        return 0 as uintptr_t;
    }
    return g_modules[index as usize].start;
}
#[no_mangle]
pub unsafe extern "C" fn initrd_module_end_ffi(mut index: ::core::ffi::c_int) -> uintptr_t {
    if index < 0 as ::core::ffi::c_int || index >= g_module_count {
        return 0 as uintptr_t;
    }
    return g_modules[index as usize].end;
}
#[no_mangle]
pub unsafe extern "C" fn initrd_module_size_ffi(mut index: ::core::ffi::c_int) -> uintptr_t {
    if index < 0 as ::core::ffi::c_int || index >= g_module_count {
        return 0 as uintptr_t;
    }
    return g_modules[index as usize].size;
}
#[no_mangle]
pub unsafe extern "C" fn initrd_module_cmdline_ffi(
    mut index: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
    mut max_len: uint32_t,
) {
    if index < 0 as ::core::ffi::c_int
        || index >= g_module_count
        || buf.is_null()
        || max_len == 0 as uint32_t
    {
        if !buf.is_null() && max_len > 0 as uint32_t {
            *buf.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
        }
        return;
    }
    let mut i: uint32_t = 0 as uint32_t;
    while g_modules[index as usize].cmdline[i as usize] as ::core::ffi::c_int != 0
        && i < max_len.wrapping_sub(1 as uint32_t)
    {
        *buf.offset(i as isize) = g_modules[index as usize].cmdline[i as usize];
        i = i.wrapping_add(1);
    }
    *buf.offset(i as isize) = '\0' as i32 as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn initrd_has_modules_ffi() -> ::core::ffi::c_int {
    return (g_module_count > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
