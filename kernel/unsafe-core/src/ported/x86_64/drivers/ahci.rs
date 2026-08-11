use ::c2rust_bitfields::BitfieldStruct;
extern "C" {
    fn pci_find_devices(
        class_code: uint8_t,
        subclass: uint8_t,
        prog_if: uint8_t,
        out: *mut pci_device,
        max_count: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn pci_config_read_word(
        bus: uint8_t,
        slot: uint8_t,
        func: uint8_t,
        offset: uint8_t,
    ) -> uint16_t;
    fn pci_config_write_dword(
        bus: uint8_t,
        slot: uint8_t,
        func: uint8_t,
        offset: uint8_t,
        value: uint32_t,
    );
    fn pmm_alloc_frame() -> *mut ::core::ffi::c_void;
    fn pmm_free_frame(addr: *mut ::core::ffi::c_void);
    fn vmm_alloc_region(size: uintptr_t, flags: uint32_t) -> *mut ::core::ffi::c_void;
    fn vmm_map(
        virt_addr: *mut ::core::ffi::c_void,
        phys_addr: *mut ::core::ffi::c_void,
        flags: uint32_t,
    ) -> bool_0;
    fn vmm_unmap(virt_addr: *mut ::core::ffi::c_void);
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type bool_0 = bool;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ahci_drive_info {
    pub present: uint8_t,
    pub port_num: uint8_t,
    pub num_sectors: uint64_t,
    pub serial: [::core::ffi::c_char; 21],
    pub firmware: [::core::ffi::c_char; 9],
    pub model: [::core::ffi::c_char; 41],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct drive {
    pub port: ::core::ffi::c_int,
    pub present: ::core::ffi::c_int,
    pub sectors: uint64_t,
    pub model: [::core::ffi::c_char; 41],
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C, packed)]
pub struct prdt_entry {
    pub dba: uint32_t,
    pub dbau: uint32_t,
    pub rsv: uint32_t,
    #[bitfield(name = "dbc", ty = "uint32_t", bits = "0..=21")]
    #[bitfield(name = "rsv2", ty = "uint32_t", bits = "22..=30")]
    #[bitfield(name = "i", ty = "uint32_t", bits = "31..=31")]
    pub dbc_rsv2_i: [u8; 4],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct hba_cmd_tbl {
    pub cfis: [uint8_t; 64],
    pub acmd: [uint8_t; 16],
    pub rsv: [uint8_t; 48],
    pub prdt: [prdt_entry; 1],
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C, packed)]
pub struct hba_cmd_hdr {
    #[bitfield(name = "cfl", ty = "uint16_t", bits = "0..=4")]
    #[bitfield(name = "a", ty = "uint16_t", bits = "5..=5")]
    #[bitfield(name = "w", ty = "uint16_t", bits = "6..=6")]
    #[bitfield(name = "p", ty = "uint16_t", bits = "7..=7")]
    #[bitfield(name = "r", ty = "uint16_t", bits = "8..=8")]
    #[bitfield(name = "b", ty = "uint16_t", bits = "9..=9")]
    #[bitfield(name = "c", ty = "uint16_t", bits = "10..=10")]
    #[bitfield(name = "rsv", ty = "uint16_t", bits = "11..=11")]
    #[bitfield(name = "rsv2", ty = "uint16_t", bits = "12..=15")]
    pub cfl_a_w_p_r_b_c_rsv_rsv2: [u8; 2],
    pub prdtl: uint16_t,
    pub prdbc: uint32_t,
    pub ctba: uint32_t,
    pub ctbau: uint32_t,
    pub rsv3: [uint32_t; 4],
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C, packed)]
pub struct h2d_fis {
    pub type_0: uint8_t,
    #[bitfield(name = "pmport", ty = "uint8_t", bits = "0..=3")]
    #[bitfield(name = "rsv0", ty = "uint8_t", bits = "4..=6")]
    #[bitfield(name = "c", ty = "uint8_t", bits = "7..=7")]
    pub pmport_rsv0_c: [u8; 1],
    pub cmd: uint8_t,
    pub feat_lo: uint8_t,
    pub lba0: uint8_t,
    pub lba1: uint8_t,
    pub lba2: uint8_t,
    pub dev: uint8_t,
    pub lba3: uint8_t,
    pub lba4: uint8_t,
    pub lba5: uint8_t,
    pub feat_hi: uint8_t,
    pub rsv1: [uint8_t; 4],
}
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const PCI_COMMAND: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PCI_CLASS_MASS_STORAGE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PCI_SUBCLASS_SATA: ::core::ffi::c_int = 0x6 as ::core::ffi::c_int;
pub const PAGE_PRESENT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PAGE_WRITE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const HBA_PI: ::core::ffi::c_int = 0xc as ::core::ffi::c_int;
pub const HBA_GHC: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const HBA_GHC_AE: ::core::ffi::c_uint = (1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int;
pub const HBA_CAP: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const HBA_CAP_NP: ::core::ffi::c_int = 0x1f as ::core::ffi::c_int;
pub const PORT_CMD_ST: ::core::ffi::c_uint = (1 as ::core::ffi::c_uint) << 0 as ::core::ffi::c_int;
pub const PORT_CMD_FRE: ::core::ffi::c_uint = (1 as ::core::ffi::c_uint) << 4 as ::core::ffi::c_int;
pub const PORT_CMD_SPIN_UP: ::core::ffi::c_uint =
    (1 as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int;
pub const PORT_CMD_POWER_ON: ::core::ffi::c_uint =
    (1 as ::core::ffi::c_uint) << 2 as ::core::ffi::c_int;
pub const PORT_TFD_BSY: ::core::ffi::c_uint = (1 as ::core::ffi::c_uint) << 7 as ::core::ffi::c_int;
pub const PORT_TFD_DRQ: ::core::ffi::c_uint = (1 as ::core::ffi::c_uint) << 3 as ::core::ffi::c_int;
pub const PORT_SSTS_DET: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PORT_SIG_ATA: ::core::ffi::c_int = 0x101 as ::core::ffi::c_int;
pub const REG_CMD: ::core::ffi::c_int = 0x18 as ::core::ffi::c_int;
pub const REG_TFD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const REG_SIG: ::core::ffi::c_int = 0x24 as ::core::ffi::c_int;
pub const REG_SSTS: ::core::ffi::c_int = 0x28 as ::core::ffi::c_int;
pub const REG_SERR: ::core::ffi::c_int = 0x30 as ::core::ffi::c_int;
pub const REG_CI: ::core::ffi::c_int = 0x38 as ::core::ffi::c_int;
pub const REG_CLB: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const REG_CLBU: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const REG_FB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const REG_FBU: ::core::ffi::c_int = 0xc as ::core::ffi::c_int;
pub const CMD_IDENTIFY: ::core::ffi::c_int = 0xec as ::core::ffi::c_int;
pub const CMD_READ_DMA_EXT: ::core::ffi::c_int = 0x25 as ::core::ffi::c_int;
pub const CMD_WRITE_DMA_EXT: ::core::ffi::c_int = 0x35 as ::core::ffi::c_int;
pub const H2D_FIS_TYPE: ::core::ffi::c_int = 0x27 as ::core::ffi::c_int;
static mut g_abar: *mut uint8_t = ::core::ptr::null::<uint8_t>() as *mut uint8_t;
static mut g_initialized: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut g_drives: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_devs: [drive; 32] = [drive {
    port: 0,
    present: 0,
    sectors: 0,
    model: [0; 41],
}; 32];
#[inline]
unsafe extern "C" fn mmio_r32(mut o: uint32_t) -> uint32_t {
    return *(g_abar.offset(o as isize) as *mut uint32_t);
}
#[inline]
unsafe extern "C" fn mmio_w32(mut o: uint32_t, mut v: uint32_t) {
    ::core::ptr::write_volatile((g_abar.offset(o as isize) as *mut uint32_t), v);
}
#[inline]
unsafe extern "C" fn port_r(mut p: ::core::ffi::c_int, mut o: uint32_t) -> uint32_t {
    return mmio_r32(
        ((0x100 as ::core::ffi::c_int + p * 0x80 as ::core::ffi::c_int) as uint32_t)
            .wrapping_add(o),
    );
}
#[inline]
unsafe extern "C" fn port_w(mut p: ::core::ffi::c_int, mut o: uint32_t, mut v: uint32_t) {
    mmio_w32(
        ((0x100 as ::core::ffi::c_int + p * 0x80 as ::core::ffi::c_int) as uint32_t)
            .wrapping_add(o),
        v,
    );
}
unsafe extern "C" fn alloc_page(
    mut phys: *mut uint32_t,
    mut virt: *mut *mut uint8_t,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_void = pmm_alloc_frame();
    if p.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *phys = p as uint32_t;
    let mut v: *mut ::core::ffi::c_void = vmm_alloc_region(
        4096 as ::core::ffi::c_int as uintptr_t,
        (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
    );
    if v.is_null() {
        pmm_free_frame(p);
        return 0 as ::core::ffi::c_int;
    }
    let mut va: uint32_t = v as uint32_t;
    vmm_unmap(va as *mut ::core::ffi::c_void);
    if !vmm_map(
        va as *mut ::core::ffi::c_void,
        *phys as *mut ::core::ffi::c_void,
        (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
    ) {
        pmm_free_frame(p);
        return 0 as ::core::ffi::c_int;
    }
    *virt = va as *mut uint8_t;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn spin_ready(
    mut p: ::core::ffi::c_int,
    mut ms: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ms * 10000 as ::core::ffi::c_int {
        let mut tfd: uint32_t = port_r(p, REG_TFD as uint32_t);
        if tfd & PORT_TFD_BSY as uint32_t == 0 && tfd & PORT_TFD_DRQ as uint32_t == 0 {
            return 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn send_cmd(
    mut d: *mut drive,
    mut write: ::core::ffi::c_int,
    mut lba: uint64_t,
    mut count: uint8_t,
    mut data_phys: uint32_t,
) -> ::core::ffi::c_int {
    if spin_ready((*d).port, 10 as ::core::ffi::c_int) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut clb_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut clb_p: uint32_t = 0 as uint32_t;
    let mut ct_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut ct_p: uint32_t = 0 as uint32_t;
    if alloc_page(&raw mut clb_p, &raw mut clb_v) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if alloc_page(&raw mut ct_p, &raw mut ct_v) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut hdr: *mut hba_cmd_hdr = clb_v as *mut hba_cmd_hdr;
    let mut tbl: *mut hba_cmd_tbl = ct_v as *mut hba_cmd_tbl;
    let mut fis: *mut h2d_fis = &raw mut (*tbl).cfis as *mut uint8_t as *mut h2d_fis;
    (*fis).type_0 = H2D_FIS_TYPE as uint8_t;
    (*fis).set_c(1 as uint8_t as uint8_t);
    (*fis).cmd = (if write != 0 {
        CMD_WRITE_DMA_EXT
    } else {
        CMD_READ_DMA_EXT
    }) as uint8_t;
    (*fis).lba0 = lba as uint8_t;
    (*fis).lba1 = (lba >> 8 as ::core::ffi::c_int) as uint8_t;
    (*fis).lba2 = (lba >> 16 as ::core::ffi::c_int) as uint8_t;
    (*fis).dev = 0x40 as uint8_t;
    (*fis).lba3 = (lba >> 24 as ::core::ffi::c_int) as uint8_t;
    (*fis).lba4 = (lba >> 32 as ::core::ffi::c_int) as uint8_t;
    (*fis).lba5 = (lba >> 40 as ::core::ffi::c_int) as uint8_t;
    (*fis).feat_lo = 0 as uint8_t;
    (*fis).feat_hi = 0 as uint8_t;
    (*tbl).cfis[12 as ::core::ffi::c_int as usize] = count;
    (*hdr).set_cfl(
        (::core::mem::size_of::<h2d_fis>() as usize).wrapping_div(4 as usize) as uint16_t
            as uint16_t,
    );
    (*hdr).set_w(write as uint16_t as uint16_t);
    (*hdr).prdtl = 1 as uint16_t;
    (*hdr).prdbc = (count as ::core::ffi::c_int * 512 as ::core::ffi::c_int) as uint32_t;
    (*hdr).ctba = ct_p;
    (*(&raw mut (*tbl).prdt as *mut prdt_entry).offset(0 as ::core::ffi::c_int as isize)).dba =
        data_phys;
    let ref mut fresh2 =
        *(&raw mut (*tbl).prdt as *mut prdt_entry).offset(0 as ::core::ffi::c_int as isize);
    (*fresh2).set_dbc(
        (count as ::core::ffi::c_int * 512 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
            as uint32_t as uint32_t,
    );
    let ref mut fresh3 =
        *(&raw mut (*tbl).prdt as *mut prdt_entry).offset(0 as ::core::ffi::c_int as isize);
    (*fresh3).set_i(1 as uint32_t as uint32_t);
    port_w((*d).port, REG_CLB as uint32_t, clb_p);
    port_w((*d).port, REG_CLBU as uint32_t, 0 as uint32_t);
    port_w((*d).port, REG_FB as uint32_t, 0 as uint32_t);
    port_w((*d).port, REG_FBU as uint32_t, 0 as uint32_t);
    port_w((*d).port, REG_CI as uint32_t, 1 as uint32_t);
    let mut ok: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 10000000 as ::core::ffi::c_int {
        if port_r((*d).port, REG_CI as uint32_t) & 1 as uint32_t == 0 {
            ok = 1 as ::core::ffi::c_int;
            break;
        } else {
            i += 1;
        }
    }
    pmm_free_frame(clb_p as *mut ::core::ffi::c_void);
    pmm_free_frame(ct_p as *mut ::core::ffi::c_void);
    return ok;
}
#[no_mangle]
pub unsafe extern "C" fn ahci_init() -> ::core::ffi::c_int {
    if g_initialized != 0 {
        return 1 as ::core::ffi::c_int;
    }
    serial_print(b"[AHCI] Scanning SATA...\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut devs: [pci_device; 8] = [pci_device {
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
    }; 8];
    let mut n: ::core::ffi::c_int = pci_find_devices(
        PCI_CLASS_MASS_STORAGE as uint8_t,
        PCI_SUBCLASS_SATA as uint8_t,
        0xff as uint8_t,
        &raw mut devs as *mut pci_device,
        8 as ::core::ffi::c_int,
    );
    if n == 0 {
        serial_print(b"[AHCI] No SATA host\n\0" as *const u8 as *const ::core::ffi::c_char);
        g_initialized = 1 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    }
    let mut c: *mut pci_device = (&raw mut devs as *mut pci_device)
        .offset(0 as ::core::ffi::c_int as isize)
        as *mut pci_device;
    let mut cmd: uint16_t =
        pci_config_read_word((*c).bus, (*c).slot, (*c).func, PCI_COMMAND as uint8_t);
    cmd = (cmd as ::core::ffi::c_int | 7 as ::core::ffi::c_int) as uint16_t;
    pci_config_write_dword(
        (*c).bus,
        (*c).slot,
        (*c).func,
        PCI_COMMAND as uint8_t,
        cmd as uint32_t,
    );
    let mut abar: uint32_t =
        (*c).bars[5 as ::core::ffi::c_int as usize] & !(1 as ::core::ffi::c_int) as uint32_t;
    if abar == 0 {
        serial_print(b"[AHCI] No ABAR\n\0" as *const u8 as *const ::core::ffi::c_char);
        g_initialized = 1 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    }
    serial_print(b"[AHCI] ABAR=0x\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(abar);
    serial_print(b"\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut v: *mut ::core::ffi::c_void = vmm_alloc_region(
        8192 as ::core::ffi::c_int as uintptr_t,
        (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
    );
    if v.is_null() {
        g_initialized = 1 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    }
    g_abar = v as *mut uint8_t;
    let mut va: uint32_t = g_abar as uint32_t;
    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while off < 8192 as ::core::ffi::c_int {
        vmm_unmap(va.wrapping_add(off as uint32_t) as *mut ::core::ffi::c_void);
        vmm_map(
            va.wrapping_add(off as uint32_t) as *mut ::core::ffi::c_void,
            abar.wrapping_add(off as uint32_t) as *mut ::core::ffi::c_void,
            (PAGE_PRESENT | PAGE_WRITE) as uint32_t,
        );
        off += 4096 as ::core::ffi::c_int;
    }
    mmio_w32(HBA_GHC as uint32_t, HBA_GHC_AE as uint32_t);
    let mut pi: uint32_t = mmio_r32(HBA_PI as uint32_t);
    let mut ports: ::core::ffi::c_int =
        (mmio_r32(HBA_CAP as uint32_t) & HBA_CAP_NP as uint32_t) as ::core::ffi::c_int;
    g_drives = 0 as ::core::ffi::c_int;
    let mut p: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while p < ports && p < 32 as ::core::ffi::c_int {
        if !(pi & ((1 as ::core::ffi::c_int) << p) as uint32_t == 0) {
            port_w(
                p,
                REG_CMD as uint32_t,
                port_r(p, REG_CMD as uint32_t)
                    | PORT_CMD_SPIN_UP as uint32_t
                    | PORT_CMD_POWER_ON as uint32_t,
            );
            let mut ssts: uint32_t = port_r(p, REG_SSTS as uint32_t);
            if !(ssts & 0xf as uint32_t != PORT_SSTS_DET as uint32_t) {
                if !(port_r(p, REG_SIG as uint32_t) != PORT_SIG_ATA as uint32_t) {
                    let mut d: *mut drive =
                        (&raw mut g_devs as *mut drive).offset(g_drives as isize) as *mut drive;
                    (*d).port = p;
                    let mut clb_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                    let mut ct_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                    let mut id_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                    let mut clb_p: uint32_t = 0;
                    let mut ct_p: uint32_t = 0;
                    let mut id_p: uint32_t = 0;
                    if !(alloc_page(&raw mut clb_p, &raw mut clb_v) == 0) {
                        if alloc_page(&raw mut ct_p, &raw mut ct_v) == 0 {
                            pmm_free_frame(clb_p as *mut ::core::ffi::c_void);
                        } else if alloc_page(&raw mut id_p, &raw mut id_v) == 0 {
                            pmm_free_frame(clb_p as *mut ::core::ffi::c_void);
                            pmm_free_frame(ct_p as *mut ::core::ffi::c_void);
                        } else {
                            let mut hdr: *mut hba_cmd_hdr = clb_v as *mut hba_cmd_hdr;
                            let mut tbl: *mut hba_cmd_tbl = ct_v as *mut hba_cmd_tbl;
                            let mut fis: *mut h2d_fis =
                                &raw mut (*tbl).cfis as *mut uint8_t as *mut h2d_fis;
                            (*fis).type_0 = H2D_FIS_TYPE as uint8_t;
                            (*fis).set_c(1 as uint8_t as uint8_t);
                            (*fis).cmd = CMD_IDENTIFY as uint8_t;
                            (*hdr).set_cfl(
                                (::core::mem::size_of::<h2d_fis>() as usize)
                                    .wrapping_div(4 as usize)
                                    as uint16_t as uint16_t,
                            );
                            (*hdr).set_w(0 as uint16_t as uint16_t);
                            (*hdr).prdtl = 1 as uint16_t;
                            (*hdr).ctba = ct_p;
                            (*(&raw mut (*tbl).prdt as *mut prdt_entry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .dba = id_p;
                            let ref mut fresh0 = *(&raw mut (*tbl).prdt as *mut prdt_entry)
                                .offset(0 as ::core::ffi::c_int as isize);
                            (*fresh0).set_dbc(
                                (512 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint32_t
                                    as uint32_t,
                            );
                            let ref mut fresh1 = *(&raw mut (*tbl).prdt as *mut prdt_entry)
                                .offset(0 as ::core::ffi::c_int as isize);
                            (*fresh1).set_i(1 as uint32_t as uint32_t);
                            port_w(p, REG_CLB as uint32_t, clb_p);
                            port_w(p, REG_CLBU as uint32_t, 0 as uint32_t);
                            port_w(p, REG_FB as uint32_t, 0 as uint32_t);
                            port_w(p, REG_FBU as uint32_t, 0 as uint32_t);
                            port_w(p, REG_SERR as uint32_t, !(0 as uint32_t));
                            port_w(
                                p,
                                REG_CMD as uint32_t,
                                port_r(p, REG_CMD as uint32_t)
                                    | PORT_CMD_FRE as uint32_t
                                    | PORT_CMD_ST as uint32_t,
                            );
                            if spin_ready(p, 5 as ::core::ffi::c_int) != 0 {
                                port_w(p, REG_CI as uint32_t, 1 as uint32_t);
                                let mut ok: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while i < 5000000 as ::core::ffi::c_int {
                                    if port_r(p, REG_CI as uint32_t) & 1 as uint32_t == 0 {
                                        ok = 1 as ::core::ffi::c_int;
                                        break;
                                    } else {
                                        i += 1;
                                    }
                                }
                                if ok != 0 {
                                    let mut id: *mut uint16_t = id_v as *mut uint16_t;
                                    (*d).present = 1 as ::core::ffi::c_int;
                                    if *id.offset(83 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_int) << 10 as ::core::ffi::c_int
                                        != 0
                                    {
                                        let mut lo: uint64_t = *id
                                            .offset(100 as ::core::ffi::c_int as isize)
                                            as uint32_t
                                            as uint64_t
                                            | (*id.offset(101 as ::core::ffi::c_int as isize)
                                                as uint32_t
                                                as uint64_t)
                                                << 16 as ::core::ffi::c_int;
                                        let mut hi: uint64_t = *id
                                            .offset(102 as ::core::ffi::c_int as isize)
                                            as uint32_t
                                            as uint64_t
                                            | (*id.offset(103 as ::core::ffi::c_int as isize)
                                                as uint32_t
                                                as uint64_t)
                                                << 16 as ::core::ffi::c_int;
                                        (*d).sectors = lo | hi << 32 as ::core::ffi::c_int;
                                    } else {
                                        (*d).sectors = (*id
                                            .offset(60 as ::core::ffi::c_int as isize)
                                            as uint32_t
                                            | (*id.offset(61 as ::core::ffi::c_int as isize)
                                                as uint32_t)
                                                << 16 as ::core::ffi::c_int)
                                            as uint64_t;
                                    }
                                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while i_0 < 20 as ::core::ffi::c_int {
                                        let mut w: uint16_t =
                                            *id.offset((27 as ::core::ffi::c_int + i_0) as isize);
                                        (*d).model[(i_0 * 2 as ::core::ffi::c_int) as usize] =
                                            (w as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                                as ::core::ffi::c_char;
                                        (*d).model[(i_0 * 2 as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int)
                                            as usize] = (w as ::core::ffi::c_int
                                            & 0xff as ::core::ffi::c_int)
                                            as ::core::ffi::c_char;
                                        i_0 += 1;
                                    }
                                    (*d).model[40 as ::core::ffi::c_int as usize] =
                                        0 as ::core::ffi::c_char;
                                    let mut i_1: ::core::ffi::c_int = 39 as ::core::ffi::c_int;
                                    while i_1 >= 0 as ::core::ffi::c_int
                                        && (*d).model[i_1 as usize] as ::core::ffi::c_int
                                            == ' ' as i32
                                    {
                                        (*d).model[i_1 as usize] = 0 as ::core::ffi::c_char;
                                        i_1 -= 1;
                                    }
                                    serial_print(
                                        b"[AHCI] Port \0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    serial_print_hex(p as uint32_t);
                                    serial_print(
                                        b": \0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    serial_print(&raw mut (*d).model as *mut ::core::ffi::c_char);
                                    serial_print(
                                        b" (\0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    serial_print_hex(
                                        (*d).sectors.wrapping_div(2048 as uint64_t) as uint32_t
                                    );
                                    serial_print(
                                        b" MB)\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    g_drives += 1;
                                }
                            }
                            pmm_free_frame(clb_p as *mut ::core::ffi::c_void);
                            pmm_free_frame(ct_p as *mut ::core::ffi::c_void);
                            pmm_free_frame(id_p as *mut ::core::ffi::c_void);
                        }
                    }
                }
            }
        }
        p += 1;
    }
    g_initialized = 1 as ::core::ffi::c_int;
    serial_print(b"[AHCI] \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(g_drives as uint32_t);
    serial_print(b" SATA drive(s)\n\0" as *const u8 as *const ::core::ffi::c_char);
    return (g_drives > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ahci_drive_count() -> ::core::ffi::c_int {
    return g_drives;
}
#[no_mangle]
pub unsafe extern "C" fn ahci_get_drive(
    mut idx: ::core::ffi::c_int,
    mut info: *mut ahci_drive_info,
) -> ::core::ffi::c_int {
    if idx < 0 as ::core::ffi::c_int || idx >= g_drives || info.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*info).present = g_devs[idx as usize].present as uint8_t;
    (*info).port_num = g_devs[idx as usize].port as uint8_t;
    (*info).num_sectors = g_devs[idx as usize].sectors;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 41 as ::core::ffi::c_int {
        (*info).model[i as usize] = g_devs[idx as usize].model[i as usize];
        i += 1;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ahci_read_sectors(
    mut idx: ::core::ffi::c_int,
    mut lba: uint64_t,
    mut count: uint8_t,
    mut buf: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if idx < 0 as ::core::ffi::c_int || idx >= g_drives {
        return 0 as ::core::ffi::c_int;
    }
    let mut dma_p: uint32_t = 0;
    let mut dma_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    if alloc_page(&raw mut dma_p, &raw mut dma_v) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut total: uint32_t = (count as ::core::ffi::c_int * 512 as ::core::ffi::c_int) as uint32_t;
    if send_cmd(
        (&raw mut g_devs as *mut drive).offset(idx as isize) as *mut drive,
        0 as ::core::ffi::c_int,
        lba,
        count,
        dma_p,
    ) == 0
    {
        pmm_free_frame(dma_p as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    let mut i: uint32_t = 0 as uint32_t;
    while i < total {
        *(buf as *mut uint8_t).offset(i as isize) = *dma_v.offset(i as isize);
        i = i.wrapping_add(1);
    }
    pmm_free_frame(dma_p as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ahci_write_sectors(
    mut idx: ::core::ffi::c_int,
    mut lba: uint64_t,
    mut count: uint8_t,
    mut buf: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if idx < 0 as ::core::ffi::c_int || idx >= g_drives {
        return 0 as ::core::ffi::c_int;
    }
    let mut dma_p: uint32_t = 0;
    let mut dma_v: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    if alloc_page(&raw mut dma_p, &raw mut dma_v) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut total: uint32_t = (count as ::core::ffi::c_int * 512 as ::core::ffi::c_int) as uint32_t;
    let mut i: uint32_t = 0 as uint32_t;
    while i < total {
        *dma_v.offset(i as isize) = *(buf as *const uint8_t).offset(i as isize);
        i = i.wrapping_add(1);
    }
    let mut r: ::core::ffi::c_int = send_cmd(
        (&raw mut g_devs as *mut drive).offset(idx as isize) as *mut drive,
        1 as ::core::ffi::c_int,
        lba,
        count,
        dma_p,
    );
    pmm_free_frame(dma_p as *mut ::core::ffi::c_void);
    return r;
}
