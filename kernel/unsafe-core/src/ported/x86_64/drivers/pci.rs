use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pci_device {
    pub bus: uint8_t,
    pub slot: uint8_t,
    pub func: uint8_t,
    pub vendor_id: uint16_t,
    pub device_id: uint16_t,
    pub revision_id: uint8_t,
    pub class_code: uint8_t,
    pub subclass: uint8_t,
    pub prog_if: uint8_t,
    pub header_type: uint8_t,
    pub bars: [uint32_t; 6],
}
pub const PCI_CONFIG_ADDRESS: ::core::ffi::c_int = 0xcf8 as ::core::ffi::c_int;
pub const PCI_CONFIG_DATA: ::core::ffi::c_int = 0xcfc as ::core::ffi::c_int;
pub const PCI_VENDOR_ID: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PCI_DEVICE_ID: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PCI_SECONDARY_BUS: ::core::ffi::c_int = 0x19 as ::core::ffi::c_int;
pub const PCI_HEADER_TYPE_BRIDGE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MAX_PCI_DEVICES: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
static mut g_devices: [pci_device; 256] = [pci_device {
    bus: 0,
    slot: 0,
    func: 0,
    vendor_id: 0,
    device_id: 0,
    revision_id: 0,
    class_code: 0,
    subclass: 0,
    prog_if: 0,
    header_type: 0,
    bars: [0; 6],
}; 256];
static mut g_device_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn outl(mut port: uint16_t, mut value: uint32_t) {
    asm!(
        "outl %eax, %dx\n", inlateout("dx") port => _, inlateout("eax") value => _,
        options(preserves_flags, att_syntax)
    );
}
#[inline]
unsafe extern "C" fn inl(mut port: uint16_t) -> uint32_t {
    let mut ret: uint32_t = 0;
    asm!(
        "inl %dx, %eax\n", lateout("eax") ret, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return ret;
}
unsafe extern "C" fn pci_make_addr(
    mut bus: uint8_t,
    mut slot: uint8_t,
    mut func: uint8_t,
    mut offset: uint8_t,
) -> uint32_t {
    return 0x80000000 as uint32_t
        | ((bus as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32_t
        | ((slot as ::core::ffi::c_int) << 11 as ::core::ffi::c_int) as uint32_t
        | ((func as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32_t
        | (offset as ::core::ffi::c_int & 0xfc as ::core::ffi::c_int) as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn pci_config_read_dword(
    mut bus: uint8_t,
    mut slot: uint8_t,
    mut func: uint8_t,
    mut offset: uint8_t,
) -> uint32_t {
    let mut addr: uint32_t = pci_make_addr(bus, slot, func, offset);
    outl(PCI_CONFIG_ADDRESS as uint16_t, addr);
    return inl(PCI_CONFIG_DATA as uint16_t);
}
#[no_mangle]
pub unsafe extern "C" fn pci_config_write_dword(
    mut bus: uint8_t,
    mut slot: uint8_t,
    mut func: uint8_t,
    mut offset: uint8_t,
    mut value: uint32_t,
) {
    let mut addr: uint32_t = pci_make_addr(bus, slot, func, offset);
    outl(PCI_CONFIG_ADDRESS as uint16_t, addr);
    outl(PCI_CONFIG_DATA as uint16_t, value);
}
#[no_mangle]
pub unsafe extern "C" fn pci_config_read_word(
    mut bus: uint8_t,
    mut slot: uint8_t,
    mut func: uint8_t,
    mut offset: uint8_t,
) -> uint16_t {
    let mut dword: uint32_t = pci_config_read_dword(bus, slot, func, offset);
    if offset as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 {
        return (dword >> 16 as ::core::ffi::c_int) as uint16_t;
    }
    return (dword & 0xffff as uint32_t) as uint16_t;
}
unsafe extern "C" fn pci_read_device(mut bus: uint8_t, mut slot: uint8_t, mut func: uint8_t) {
    let mut vendor: uint16_t = pci_config_read_word(bus, slot, func, PCI_VENDOR_ID as uint8_t);
    if vendor as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        return;
    }
    if g_device_count >= MAX_PCI_DEVICES {
        return;
    }
    let mut dev: *mut pci_device =
        (&raw mut g_devices as *mut pci_device).offset(g_device_count as isize) as *mut pci_device;
    (*dev).bus = bus;
    (*dev).slot = slot;
    (*dev).func = func;
    (*dev).vendor_id = vendor;
    (*dev).device_id = pci_config_read_word(bus, slot, func, PCI_DEVICE_ID as uint8_t);
    let mut class_reg: uint32_t = pci_config_read_dword(bus, slot, func, 0x8 as uint8_t);
    (*dev).revision_id = (class_reg & 0xff as uint32_t) as uint8_t;
    (*dev).prog_if = (class_reg >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t;
    (*dev).subclass = (class_reg >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t;
    (*dev).class_code = (class_reg >> 24 as ::core::ffi::c_int) as uint8_t;
    let mut header: uint32_t = pci_config_read_dword(bus, slot, func, 0xc as uint8_t);
    (*dev).header_type = (header >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6 as ::core::ffi::c_int {
        (*dev).bars[i as usize] = pci_config_read_dword(
            bus,
            slot,
            func,
            (0x10 as ::core::ffi::c_int + i * 4 as ::core::ffi::c_int) as uint8_t,
        );
        i += 1;
    }
    g_device_count += 1;
}
unsafe extern "C" fn pci_scan_bus(mut bus: uint8_t) {
    let mut slot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while slot < 32 as ::core::ffi::c_int {
        let mut vendor: uint16_t =
            pci_config_read_word(bus, slot as uint8_t, 0 as uint8_t, PCI_VENDOR_ID as uint8_t);
        if !(vendor as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int) {
            pci_read_device(bus, slot as uint8_t, 0 as uint8_t);
            let mut header: uint32_t =
                pci_config_read_dword(bus, slot as uint8_t, 0 as uint8_t, 0xc as uint8_t);
            let mut header_type: uint8_t =
                (header >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t;
            if header_type as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                let mut func: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while func < 8 as ::core::ffi::c_int {
                    vendor = pci_config_read_word(
                        bus,
                        slot as uint8_t,
                        func as uint8_t,
                        PCI_VENDOR_ID as uint8_t,
                    );
                    if vendor as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
                        pci_read_device(bus, slot as uint8_t, func as uint8_t);
                    }
                    func += 1;
                }
            }
            if header_type as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int
                == PCI_HEADER_TYPE_BRIDGE
            {
                let mut bus_reg: uint32_t = pci_config_read_dword(
                    bus,
                    slot as uint8_t,
                    0 as uint8_t,
                    PCI_SECONDARY_BUS as uint8_t,
                );
                let mut secondary_bus: uint8_t =
                    (bus_reg >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t;
                if secondary_bus as ::core::ffi::c_int != bus as ::core::ffi::c_int {
                    pci_scan_bus(secondary_bus);
                }
            }
        }
        slot += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pci_init() {
    serial_print(b"[PCI] Scanning PCI bus...\n\0" as *const u8 as *const ::core::ffi::c_char);
    g_device_count = 0 as ::core::ffi::c_int;
    let mut vendor: uint16_t = pci_config_read_word(
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        PCI_VENDOR_ID as uint8_t,
    );
    if vendor as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        serial_print(
            b"[PCI] No PCI host controller found\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return;
    }
    pci_scan_bus(0 as uint8_t);
    serial_print(b"[PCI] Found \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(g_device_count as uint32_t);
    serial_print(b" devices\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < g_device_count {
        let mut dev: *mut pci_device =
            (&raw mut g_devices as *mut pci_device).offset(i as isize) as *mut pci_device;
        serial_print(b"  \0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).bus as uint32_t);
        serial_print(b":\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).slot as uint32_t);
        serial_print(b".\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).func as uint32_t);
        serial_print(b" [\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).class_code as uint32_t);
        serial_print(b".\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).subclass as uint32_t);
        serial_print(b".\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).prog_if as uint32_t);
        serial_print(b"] vendor=\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).vendor_id as uint32_t);
        serial_print(b" device=\0" as *const u8 as *const ::core::ffi::c_char);
        serial_print_hex((*dev).device_id as uint32_t);
        serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pci_device_count() -> ::core::ffi::c_int {
    return g_device_count;
}
#[no_mangle]
pub unsafe extern "C" fn pci_get_device(
    mut index: ::core::ffi::c_int,
    mut dev: *mut pci_device,
) -> ::core::ffi::c_int {
    if index < 0 as ::core::ffi::c_int || index >= g_device_count || dev.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *dev = g_devices[index as usize];
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn pci_find_devices(
    mut class_code: uint8_t,
    mut subclass: uint8_t,
    mut prog_if: uint8_t,
    mut out: *mut pci_device,
    mut max_count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < g_device_count && count < max_count {
        if g_devices[i as usize].class_code as ::core::ffi::c_int
            == class_code as ::core::ffi::c_int
            && g_devices[i as usize].subclass as ::core::ffi::c_int
                == subclass as ::core::ffi::c_int
            && (prog_if as ::core::ffi::c_int == 0xff as ::core::ffi::c_int
                || g_devices[i as usize].prog_if as ::core::ffi::c_int
                    == prog_if as ::core::ffi::c_int)
        {
            let fresh0 = count;
            count = count + 1;
            *out.offset(fresh0 as isize) = g_devices[i as usize];
        }
        i += 1;
    }
    return count;
}
