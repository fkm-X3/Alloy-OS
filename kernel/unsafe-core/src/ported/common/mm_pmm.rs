extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
    static mut _kernel_start: uint32_t;
    static mut _kernel_end: uint32_t;
}
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type int32_t = i32;
pub type bool_0 = bool;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PhysicalMemoryManager {
    pub bitmap: *mut uint32_t,
    pub total_frames: uint32_t,
    pub used_frames: uint32_t,
    pub total_memory: uint64_t,
    pub available_memory: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiboot_tag {
    pub type_0: uint32_t,
    pub size: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiboot_mmap_entry {
    pub addr: uint64_t,
    pub len: uint64_t,
    pub type_0: uint32_t,
    pub zero: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiboot_tag_mmap {
    pub type_0: uint32_t,
    pub size: uint32_t,
    pub entry_size: uint32_t,
    pub entry_version: uint32_t,
    pub entries: [multiboot_mmap_entry; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiboot_tag_basic_meminfo {
    pub type_0: uint32_t,
    pub size: uint32_t,
    pub mem_lower: uint32_t,
    pub mem_upper: uint32_t,
}
pub const PAGE_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_END: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_BASIC_MEMINFO: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const MULTIBOOT_TAG_TYPE_MMAP: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_pmm: PhysicalMemoryManager = PhysicalMemoryManager {
    bitmap: ::core::ptr::null::<uint32_t>() as *mut uint32_t,
    total_frames: 0,
    used_frames: 0,
    total_memory: 0,
    available_memory: 0,
};
pub const MAX_PHYSICAL_FRAMES: ::core::ffi::c_int =
    1024 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int;
static mut frame_refcounts: [uint32_t; 1048576] = [0; 1048576];
pub const MULTIBOOT_MEMORY_AVAILABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut frame_bitmap: [uint32_t; 32768] = [0; 32768];
unsafe extern "C" fn set_frame(mut frame_number: uint32_t) {
    let mut index: uint32_t = frame_number.wrapping_div(32 as uint32_t);
    let mut bit: uint32_t = frame_number.wrapping_rem(32 as uint32_t);
    *g_pmm.bitmap.offset(index as isize) |= ((1 as ::core::ffi::c_int) << bit) as uint32_t;
}
unsafe extern "C" fn clear_frame(mut frame_number: uint32_t) {
    let mut index: uint32_t = frame_number.wrapping_div(32 as uint32_t);
    let mut bit: uint32_t = frame_number.wrapping_rem(32 as uint32_t);
    *g_pmm.bitmap.offset(index as isize) &= !((1 as ::core::ffi::c_int) << bit) as uint32_t;
}
unsafe extern "C" fn test_frame(mut frame_number: uint32_t) -> bool_0 {
    let mut index: uint32_t = frame_number.wrapping_div(32 as uint32_t);
    let mut bit: uint32_t = frame_number.wrapping_rem(32 as uint32_t);
    return *g_pmm.bitmap.offset(index as isize) & ((1 as ::core::ffi::c_int) << bit) as uint32_t
        != 0 as uint32_t;
}
unsafe extern "C" fn find_free_frame() -> int32_t {
    let mut i: uint32_t = 0 as uint32_t;
    while i < g_pmm.total_frames.wrapping_div(32 as uint32_t) {
        if *g_pmm.bitmap.offset(i as isize) != 0xffffffff as uint32_t {
            let mut bit: uint32_t = 0 as uint32_t;
            while bit < 32 as uint32_t {
                if *g_pmm.bitmap.offset(i as isize) & ((1 as ::core::ffi::c_int) << bit) as uint32_t
                    == 0 as uint32_t
                {
                    return i.wrapping_mul(32 as uint32_t).wrapping_add(bit) as int32_t;
                }
                bit = bit.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    return -(1 as int32_t);
}
#[no_mangle]
pub unsafe extern "C" fn pmm_init(mut multiboot_addr: uint32_t) {
    serial_print(
        b"PMM: Initializing physical memory manager...\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    g_pmm.bitmap = &raw mut frame_bitmap as *mut uint32_t;
    g_pmm.total_frames = 0 as uint32_t;
    g_pmm.used_frames = 0 as uint32_t;
    g_pmm.total_memory = 0 as uint64_t;
    g_pmm.available_memory = 0 as uint64_t;
    let mut i: uint32_t = 0 as uint32_t;
    while (i as usize)
        < (::core::mem::size_of::<[uint32_t; 32768]>() as usize)
            .wrapping_div(::core::mem::size_of::<uint32_t>() as usize)
    {
        *g_pmm.bitmap.offset(i as isize) = 0xffffffff as ::core::ffi::c_uint as uint32_t;
        i = i.wrapping_add(1);
    }
    crate::raw::string::memset(
        &raw mut frame_refcounts as *mut uint32_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint32_t; 1048576]>() as crate::raw::string::size_t,
    );
    if multiboot_addr == 0 as uint32_t {
        serial_print(
            b"PMM: No multiboot info (aarch64), using default memory layout\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        g_pmm.total_memory = (128 as ::core::ffi::c_int
            * 1024 as ::core::ffi::c_int
            * 1024 as ::core::ffi::c_int) as uint64_t;
        g_pmm.available_memory = (128 as ::core::ffi::c_int
            * 1024 as ::core::ffi::c_int
            * 1024 as ::core::ffi::c_int) as uint64_t;
        let mut ram_start_frame: uint32_t = (&raw mut _kernel_end as uint32_t)
            .wrapping_add(PAGE_SIZE as uint32_t)
            .wrapping_sub(1 as uint32_t)
            .wrapping_div(PAGE_SIZE as uint32_t);
        let mut ram_end_frame: uint32_t =
            (0x48000000 as ::core::ffi::c_int / PAGE_SIZE) as uint32_t;
        g_pmm.total_frames = ram_end_frame;
        let mut i_0: uint32_t = ram_start_frame;
        while i_0 < ram_end_frame {
            clear_frame(i_0);
            i_0 = i_0.wrapping_add(1);
        }
    } else {
        let mut tag: *mut multiboot_tag =
            multiboot_addr.wrapping_add(8 as uint32_t) as *mut multiboot_tag;
        while (*tag).type_0 != MULTIBOOT_TAG_TYPE_END as uint32_t {
            if (*tag).type_0 == MULTIBOOT_TAG_TYPE_BASIC_MEMINFO as uint32_t {
                let mut meminfo: *mut multiboot_tag_basic_meminfo =
                    tag as *mut multiboot_tag_basic_meminfo;
                serial_print(
                    b"PMM: Basic memory info:\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
                serial_print(b"  Lower memory: \0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*meminfo).mem_lower);
                serial_print(b" KB\n\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print(b"  Upper memory: \0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*meminfo).mem_upper);
                serial_print(b" KB\n\0" as *const u8 as *const ::core::ffi::c_char);
            } else if (*tag).type_0 == MULTIBOOT_TAG_TYPE_MMAP as uint32_t {
                let mut mmap: *mut multiboot_tag_mmap = tag as *mut multiboot_tag_mmap;
                serial_print(b"PMM: Memory map:\n\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print(b"  entry_size=\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*mmap).entry_size);
                serial_print(b", tag->size=\0" as *const u8 as *const ::core::ffi::c_char);
                serial_print_hex((*tag).size);
                serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
                let mut tag_end: *mut uint8_t = (tag as *mut uint8_t).offset((*tag).size as isize);
                let mut entry_ptr: *mut uint8_t =
                    &raw mut (*mmap).entries as *mut multiboot_mmap_entry as *mut uint8_t;
                while entry_ptr
                    .offset(::core::mem::size_of::<multiboot_mmap_entry>() as usize as isize)
                    <= tag_end
                {
                    let mut entry: *mut multiboot_mmap_entry =
                        entry_ptr as *mut multiboot_mmap_entry;
                    serial_print(b"  Region: addr=0x\0" as *const u8 as *const ::core::ffi::c_char);
                    serial_print_hex((*entry).addr as uint32_t);
                    serial_print(b", len=0x\0" as *const u8 as *const ::core::ffi::c_char);
                    serial_print_hex((*entry).len as uint32_t);
                    serial_print(b", type=\0" as *const u8 as *const ::core::ffi::c_char);
                    serial_print_hex((*entry).type_0);
                    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
                    g_pmm.total_memory = g_pmm.total_memory.wrapping_add((*entry).len);
                    if (*entry).type_0 == MULTIBOOT_MEMORY_AVAILABLE as uint32_t {
                        g_pmm.available_memory = g_pmm.available_memory.wrapping_add((*entry).len);
                        let mut base: uint64_t = (*entry).addr;
                        let mut length: uint64_t = (*entry).len;
                        if base.wrapping_rem(PAGE_SIZE as uint64_t) != 0 as uint64_t {
                            let mut offset: uint64_t = (PAGE_SIZE as uint64_t)
                                .wrapping_sub(base.wrapping_rem(PAGE_SIZE as uint64_t));
                            base = base.wrapping_add(offset);
                            if length > offset {
                                length = length.wrapping_sub(offset);
                            } else {
                                length = 0 as uint64_t;
                            }
                        }
                        length = length
                            .wrapping_div(PAGE_SIZE as uint64_t)
                            .wrapping_mul(PAGE_SIZE as uint64_t);
                        let mut start_frame: uint32_t =
                            base.wrapping_div(PAGE_SIZE as uint64_t) as uint32_t;
                        let mut num_frames: uint32_t =
                            length.wrapping_div(PAGE_SIZE as uint64_t) as uint32_t;
                        let mut end_frame: uint32_t = start_frame.wrapping_add(num_frames);
                        let mut start_idx: uint32_t = start_frame.wrapping_div(32 as uint32_t);
                        let mut end_idx: uint32_t = end_frame
                            .wrapping_sub(1 as uint32_t)
                            .wrapping_div(32 as uint32_t);
                        let mut start_bit: uint32_t = start_frame & 31 as uint32_t;
                        let mut end_bit: uint32_t =
                            end_frame.wrapping_sub(1 as uint32_t) & 31 as uint32_t;
                        let mut max_idx: uint32_t = (::core::mem::size_of::<[uint32_t; 32768]>()
                            as usize)
                            .wrapping_div(::core::mem::size_of::<uint32_t>() as usize)
                            as uint32_t;
                        if end_idx >= max_idx {
                            end_idx = max_idx.wrapping_sub(1 as uint32_t);
                            end_bit = 31 as uint32_t;
                            end_frame = max_idx.wrapping_mul(32 as uint32_t);
                        }
                        if start_idx >= max_idx {
                            serial_print(
                                b"PMM: WARNING - start_idx out of bounds, skipping entry\n\0"
                                    as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        } else {
                            if start_idx == end_idx {
                                let mut mask: uint32_t = 0;
                                if end_bit == 31 as uint32_t {
                                    mask = ((0xffffffff as ::core::ffi::c_uint) << start_bit)
                                        as uint32_t;
                                } else {
                                    mask = ((0xffffffff as ::core::ffi::c_uint) << start_bit
                                        & !((0xffffffff as ::core::ffi::c_uint)
                                            << end_bit.wrapping_add(1 as uint32_t)))
                                        as uint32_t;
                                }
                                *g_pmm.bitmap.offset(start_idx as isize) &= !mask;
                            } else {
                                let ref mut fresh0 = *g_pmm.bitmap.offset(start_idx as isize);
                                *fresh0 = (*fresh0 as ::core::ffi::c_uint
                                    & !((0xffffffff as ::core::ffi::c_uint) << start_bit))
                                    as uint32_t;
                                let mut i_1: uint32_t = start_idx.wrapping_add(1 as uint32_t);
                                while i_1 < end_idx && i_1 < max_idx {
                                    *g_pmm.bitmap.offset(i_1 as isize) = 0 as uint32_t;
                                    i_1 = i_1.wrapping_add(1);
                                }
                                if end_idx < max_idx {
                                    if end_bit == 31 as uint32_t {
                                        *g_pmm.bitmap.offset(end_idx as isize) = 0 as uint32_t;
                                    } else {
                                        *g_pmm.bitmap.offset(end_idx as isize) &= !(((1
                                            as ::core::ffi::c_int)
                                            << end_bit.wrapping_add(1 as uint32_t))
                                            - 1 as ::core::ffi::c_int)
                                            as uint32_t;
                                    }
                                }
                            }
                            if end_frame > g_pmm.total_frames {
                                g_pmm.total_frames = end_frame;
                            }
                        }
                    }
                    entry_ptr = entry_ptr.offset((*mmap).entry_size as isize);
                }
                serial_print(
                    b"PMM: Memory map entries done\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            tag = (tag as *mut uint8_t).offset(
                ((*tag).size.wrapping_add(7 as uint32_t) & !(7 as ::core::ffi::c_int) as uint32_t)
                    as isize,
            ) as *mut multiboot_tag;
        }
    }
    let mut frame: uint32_t = 0 as uint32_t;
    while frame < 256 as uint32_t {
        set_frame(frame);
        g_pmm.used_frames = g_pmm.used_frames.wrapping_add(1);
        frame = frame.wrapping_add(1);
    }
    let mut kernel_start: uint32_t = &raw mut _kernel_start as uint32_t;
    let mut kernel_end: uint32_t = &raw mut _kernel_end as uint32_t;
    serial_print(b"  Kernel region start: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(kernel_start);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Kernel region end: 0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(kernel_end);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut kernel_start_frame: uint32_t = kernel_start.wrapping_div(PAGE_SIZE as uint32_t);
    let mut kernel_end_frame: uint32_t = kernel_end
        .wrapping_add(PAGE_SIZE as uint32_t)
        .wrapping_sub(1 as uint32_t)
        .wrapping_div(PAGE_SIZE as uint32_t);
    let mut frame_0: uint32_t = kernel_start_frame;
    while frame_0 < kernel_end_frame {
        if !test_frame(frame_0) {
            set_frame(frame_0);
            g_pmm.used_frames = g_pmm.used_frames.wrapping_add(1);
        }
        frame_0 = frame_0.wrapping_add(1);
    }
    serial_print(b"PMM: Initialization complete\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Total memory: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(
        g_pmm
            .total_memory
            .wrapping_div(1024 as uint64_t)
            .wrapping_div(1024 as uint64_t) as uint32_t,
    );
    serial_print(b" MB\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Available memory: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(
        g_pmm
            .available_memory
            .wrapping_div(1024 as uint64_t)
            .wrapping_div(1024 as uint64_t) as uint32_t,
    );
    serial_print(b" MB\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Total frames: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(g_pmm.total_frames);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(b"  Used frames: \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(g_pmm.used_frames);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn pmm_alloc_frame() -> *mut ::core::ffi::c_void {
    let mut frame: int32_t = find_free_frame();
    if frame == -(1 as int32_t) {
        serial_print(b"PMM: ERROR - Out of memory!\n\0" as *const u8 as *const ::core::ffi::c_char);
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    set_frame(frame as uint32_t);
    g_pmm.used_frames = g_pmm.used_frames.wrapping_add(1);
    if frame < MAX_PHYSICAL_FRAMES as int32_t {
        frame_refcounts[frame as usize] = 1 as uint32_t;
    }
    return (frame * PAGE_SIZE as int32_t) as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn pmm_free_frame(mut addr: *mut ::core::ffi::c_void) {
    let mut frame: uint32_t = (addr as uint32_t).wrapping_div(PAGE_SIZE as uint32_t);
    if frame >= g_pmm.total_frames {
        serial_print(
            b"PMM: ERROR - Invalid frame address\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return;
    }
    if !test_frame(frame) {
        serial_print(
            b"PMM: WARNING - Double free detected\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return;
    }
    clear_frame(frame);
    g_pmm.used_frames = g_pmm.used_frames.wrapping_sub(1);
}
#[no_mangle]
pub unsafe extern "C" fn pmm_get_total_memory() -> uint64_t {
    return g_pmm.total_memory;
}
#[no_mangle]
pub unsafe extern "C" fn pmm_get_available_memory() -> uint64_t {
    return g_pmm.available_memory;
}
#[no_mangle]
pub unsafe extern "C" fn pmm_get_total_frames() -> uint32_t {
    return g_pmm.total_frames;
}
#[no_mangle]
pub unsafe extern "C" fn pmm_get_used_frames() -> uint32_t {
    return g_pmm.used_frames;
}
#[no_mangle]
pub unsafe extern "C" fn pmm_refcount_inc(mut addr: *mut ::core::ffi::c_void) {
    let mut frame: uint32_t = (addr as uint32_t).wrapping_div(PAGE_SIZE as uint32_t);
    if frame < MAX_PHYSICAL_FRAMES as uint32_t {
        frame_refcounts[frame as usize] = frame_refcounts[frame as usize].wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn pmm_refcount_dec(mut addr: *mut ::core::ffi::c_void) {
    let mut frame: uint32_t = (addr as uint32_t).wrapping_div(PAGE_SIZE as uint32_t);
    if frame < MAX_PHYSICAL_FRAMES as uint32_t {
        if frame_refcounts[frame as usize] > 0 as uint32_t {
            frame_refcounts[frame as usize] = frame_refcounts[frame as usize].wrapping_sub(1);
        }
        if frame_refcounts[frame as usize] == 0 as uint32_t {
            pmm_free_frame(addr);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pmm_refcount_get(mut addr: *mut ::core::ffi::c_void) -> uint32_t {
    let mut frame: uint32_t = (addr as uint32_t).wrapping_div(PAGE_SIZE as uint32_t);
    if frame < MAX_PHYSICAL_FRAMES as uint32_t {
        return frame_refcounts[frame as usize];
    }
    return 0 as uint32_t;
}
