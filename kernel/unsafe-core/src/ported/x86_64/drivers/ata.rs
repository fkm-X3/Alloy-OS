use ::core::arch::asm;
extern "C" {
    fn serial_print(str: *const ::core::ffi::c_char);
    fn serial_print_hex(value: uint32_t);
}
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ata_drive_info {
    pub present: uint8_t,
    pub is_lba48: uint8_t,
    pub signature: uint16_t,
    pub capabilities: uint16_t,
    pub command_sets: uint32_t,
    pub num_sectors: uint64_t,
    pub model: [::core::ffi::c_char; 41],
}
pub const ATA_DRIVE_MASTER: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ATA_DRIVE_SLAVE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ATA_CMD_READ_PIO: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const ATA_CMD_READ_PIO_EXT: ::core::ffi::c_int = 0x24 as ::core::ffi::c_int;
pub const ATA_CMD_WRITE_PIO: ::core::ffi::c_int = 0x30 as ::core::ffi::c_int;
pub const ATA_CMD_WRITE_PIO_EXT: ::core::ffi::c_int = 0x34 as ::core::ffi::c_int;
pub const ATA_CMD_IDENTIFY: ::core::ffi::c_int = 0xec as ::core::ffi::c_int;
pub const ATA_CMD_FLUSH_CACHE: ::core::ffi::c_int = 0xe7 as ::core::ffi::c_int;
pub const ATA_STATUS_ERR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ATA_STATUS_DRQ: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const ATA_STATUS_BSY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ATA_PRIMARY_IO: ::core::ffi::c_int = 0x1f0 as ::core::ffi::c_int;
pub const ATA_PRIMARY_CTRL: ::core::ffi::c_int = 0x3f6 as ::core::ffi::c_int;
pub const ATA_SECONDARY_IO: ::core::ffi::c_int = 0x170 as ::core::ffi::c_int;
pub const ATA_SECONDARY_CTRL: ::core::ffi::c_int = 0x376 as ::core::ffi::c_int;
static mut ata_io_bases: [uint16_t; 2] = [ATA_PRIMARY_IO as uint16_t, ATA_SECONDARY_IO as uint16_t];
static mut ata_ctrl_bases: [uint16_t; 2] =
    [ATA_PRIMARY_CTRL as uint16_t, ATA_SECONDARY_CTRL as uint16_t];
static mut g_drives: [[ata_drive_info; 2]; 2] = [[ata_drive_info {
    present: 0,
    is_lba48: 0,
    signature: 0,
    capabilities: 0,
    command_sets: 0,
    num_sectors: 0,
    model: [0; 41],
}; 2]; 2];
static mut g_ata_initialized: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn outb(mut port: uint16_t, mut value: uint8_t) {
    asm!(
        "outb %al, %dx\n", inlateout("al") value => _, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
}
#[inline]
unsafe extern "C" fn inb(mut port: uint16_t) -> uint8_t {
    let mut ret: uint8_t = 0;
    asm!(
        "inb %dx, %al\n", lateout("al") ret, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return ret;
}
#[inline]
unsafe extern "C" fn outw(mut port: uint16_t, mut value: uint16_t) {
    asm!(
        "outw %ax, %dx\n", inlateout("ax") value => _, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
}
#[inline]
unsafe extern "C" fn inw(mut port: uint16_t) -> uint16_t {
    let mut ret: uint16_t = 0;
    asm!(
        "inw %dx, %ax\n", lateout("ax") ret, inlateout("dx") port => _,
        options(preserves_flags, att_syntax)
    );
    return ret;
}
unsafe extern "C" fn ata_wait(mut io_base: uint16_t) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        inb(
            ata_ctrl_bases[(if io_base as ::core::ffi::c_int == ATA_PRIMARY_IO {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as usize],
        );
        i += 1;
    }
}
unsafe extern "C" fn ata_busy_wait(
    mut io_base: uint16_t,
    mut timeout_ms: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < timeout_ms * 1000 as ::core::ffi::c_int {
        let mut status: uint8_t =
            inb((io_base as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t);
        if status as ::core::ffi::c_int & ATA_STATUS_BSY == 0 {
            return 1 as ::core::ffi::c_int;
        }
        ata_wait(io_base);
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ata_drq_wait(
    mut io_base: uint16_t,
    mut timeout_ms: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < timeout_ms * 1000 as ::core::ffi::c_int {
        let mut status: uint8_t =
            inb((io_base as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t);
        if status as ::core::ffi::c_int & ATA_STATUS_ERR != 0 {
            return -(1 as ::core::ffi::c_int);
        }
        if status as ::core::ffi::c_int & ATA_STATUS_DRQ != 0 {
            return 1 as ::core::ffi::c_int;
        }
        if status as ::core::ffi::c_int & ATA_STATUS_BSY == 0 {
            return 0 as ::core::ffi::c_int;
        }
        ata_wait(io_base);
        i += 1;
    }
    return -(2 as ::core::ffi::c_int);
}
unsafe extern "C" fn ata_soft_reset(mut bus: uint8_t) {
    let mut ctrl: uint16_t = ata_ctrl_bases[bus as usize];
    outb(ctrl, 0x4 as uint8_t);
    ata_wait(ata_io_bases[bus as usize]);
    outb(ctrl, 0 as uint8_t);
    ata_wait(ata_io_bases[bus as usize]);
}
unsafe extern "C" fn ata_words_to_sectors(mut info: *mut ata_drive_info, mut buf: *mut uint16_t) {
    if (*info).command_sets & ((1 as ::core::ffi::c_int) << 26 as ::core::ffi::c_int) as uint32_t
        != 0
    {
        let mut lo: uint64_t = *buf.offset(100 as ::core::ffi::c_int as isize) as uint32_t
            as uint64_t
            | (*buf.offset(101 as ::core::ffi::c_int as isize) as uint32_t as uint64_t)
                << 16 as ::core::ffi::c_int;
        let mut hi: uint64_t = *buf.offset(102 as ::core::ffi::c_int as isize) as uint32_t
            as uint64_t
            | (*buf.offset(103 as ::core::ffi::c_int as isize) as uint32_t as uint64_t)
                << 16 as ::core::ffi::c_int;
        (*info).num_sectors = lo | hi << 32 as ::core::ffi::c_int;
    } else {
        (*info).num_sectors = (*buf.offset(60 as ::core::ffi::c_int as isize) as uint32_t
            | (*buf.offset(61 as ::core::ffi::c_int as isize) as uint32_t)
                << 16 as ::core::ffi::c_int) as uint64_t;
    };
}
unsafe extern "C" fn ata_identify(mut bus: uint8_t, mut drive: uint8_t) -> ::core::ffi::c_int {
    let mut io: uint16_t = ata_io_bases[bus as usize];
    ata_soft_reset(bus);
    if ata_busy_wait(io, 1 as ::core::ffi::c_int) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    outb(
        (io as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16_t,
        (if drive as ::core::ffi::c_int == ATA_DRIVE_MASTER {
            0xa0 as ::core::ffi::c_int
        } else {
            0xb0 as ::core::ffi::c_int
        }) as uint8_t,
    );
    ata_wait(io);
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        0 as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        0 as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        0 as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        0 as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
        ATA_CMD_IDENTIFY as uint8_t,
    );
    ata_wait(io);
    let mut status: uint8_t = inb((io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t);
    if status as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if ata_busy_wait(io, 1 as ::core::ffi::c_int) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if inb((io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && inb((io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t)
            as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if ata_drq_wait(io, 1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut info: *mut ata_drive_info = (&raw mut *(&raw mut g_drives as *mut [ata_drive_info; 2])
        .offset(bus as isize) as *mut ata_drive_info)
        .offset(drive as isize) as *mut ata_drive_info;
    (*info).present = 1 as uint8_t;
    let mut buf: [uint16_t; 256] = [0; 256];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        buf[i as usize] = inw(io);
        i += 1;
    }
    (*info).signature = buf[0 as ::core::ffi::c_int as usize];
    (*info).capabilities = buf[49 as ::core::ffi::c_int as usize];
    (*info).command_sets = (buf[83 as ::core::ffi::c_int as usize] as uint32_t)
        << 16 as ::core::ffi::c_int
        | buf[82 as ::core::ffi::c_int as usize] as uint32_t;
    (*info).is_lba48 = (if (*info).command_sets
        & ((1 as ::core::ffi::c_int) << 26 as ::core::ffi::c_int) as uint32_t
        != 0
    {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as uint8_t;
    ata_words_to_sectors(info, &raw mut buf as *mut uint16_t);
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 20 as ::core::ffi::c_int {
        let mut w: uint16_t = buf[(27 as ::core::ffi::c_int + i_0) as usize];
        (*info).model[(i_0 * 2 as ::core::ffi::c_int) as usize] =
            (w as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as ::core::ffi::c_char;
        (*info).model[(i_0 * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
            (w as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as ::core::ffi::c_char;
        i_0 += 1;
    }
    (*info).model[40 as ::core::ffi::c_int as usize] = '\0' as i32 as ::core::ffi::c_char;
    let mut i_1: ::core::ffi::c_int = 39 as ::core::ffi::c_int;
    while i_1 >= 0 as ::core::ffi::c_int
        && (*info).model[i_1 as usize] as ::core::ffi::c_int == ' ' as i32
    {
        (*info).model[i_1 as usize] = '\0' as i32 as ::core::ffi::c_char;
        i_1 -= 1;
    }
    serial_print(b"[ATA] Drive \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(bus as uint32_t);
    serial_print(b":\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex(drive as uint32_t);
    serial_print(b": \0" as *const u8 as *const ::core::ffi::c_char);
    serial_print(&raw mut (*info).model as *mut ::core::ffi::c_char);
    serial_print(b" (\0" as *const u8 as *const ::core::ffi::c_char);
    serial_print_hex((*info).num_sectors.wrapping_div(2048 as uint64_t) as uint32_t);
    serial_print(b" MB LBA\0" as *const u8 as *const ::core::ffi::c_char);
    if (*info).is_lba48 != 0 {
        serial_print(b"48\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        serial_print(b"28\0" as *const u8 as *const ::core::ffi::c_char);
    }
    serial_print(b")\n\0" as *const u8 as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ata_init() -> ::core::ffi::c_int {
    if g_ata_initialized != 0 {
        return 1 as ::core::ffi::c_int;
    }
    serial_print(
        b"[ATA] Initializing ATA PIO driver...\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 2 as ::core::ffi::c_int {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < 2 as ::core::ffi::c_int {
            g_drives[i as usize][j as usize].present = 0 as uint8_t;
            g_drives[i as usize][j as usize].is_lba48 = 0 as uint8_t;
            g_drives[i as usize][j as usize].num_sectors = 0 as uint64_t;
            g_drives[i as usize][j as usize].model[0 as ::core::ffi::c_int as usize] =
                '\0' as i32 as ::core::ffi::c_char;
            j += 1;
        }
        i += 1;
    }
    let mut bus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while bus < 2 as ::core::ffi::c_int {
        ata_identify(bus as uint8_t, ATA_DRIVE_MASTER as uint8_t);
        ata_identify(bus as uint8_t, ATA_DRIVE_SLAVE as uint8_t);
        bus += 1;
    }
    g_ata_initialized = 1 as ::core::ffi::c_int;
    serial_print(
        b"[ATA] ATA PIO driver initialized\n\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ata_drive_present(
    mut bus: uint8_t,
    mut drive: uint8_t,
) -> ::core::ffi::c_int {
    if bus as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || drive as ::core::ffi::c_int > 1 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return g_drives[bus as usize][drive as usize].present as ::core::ffi::c_int;
}
unsafe extern "C" fn ata_pio_read_lba28(
    mut io: uint16_t,
    mut drive: uint8_t,
    mut lba: uint32_t,
    mut count: uint8_t,
    mut buffer: *mut uint16_t,
) -> ::core::ffi::c_int {
    outb(
        (io as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16_t,
        ((0xe0 as ::core::ffi::c_int | (drive as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
            as uint32_t
            | lba >> 24 as ::core::ffi::c_int & 0xf as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        count,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        (lba & 0xff as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        (lba >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        (lba >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
        ATA_CMD_READ_PIO as uint8_t,
    );
    let mut s: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while s < count as ::core::ffi::c_int {
        if ata_drq_wait(io, 5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            *buffer.offset((s * 256 as ::core::ffi::c_int + i) as isize) = inw(io);
            i += 1;
        }
        s += 1;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn ata_pio_read_lba48(
    mut io: uint16_t,
    mut drive: uint8_t,
    mut lba: uint64_t,
    mut count: uint16_t,
    mut buffer: *mut uint16_t,
) -> ::core::ffi::c_int {
    outb(
        (io as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16_t,
        (0x40 as ::core::ffi::c_int | (drive as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
            as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        (lba >> 24 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        (lba >> 32 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        (lba >> 40 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        (count as ::core::ffi::c_int >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        (lba & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        (lba >> 8 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        (lba >> 16 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        (count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
        ATA_CMD_READ_PIO_EXT as uint8_t,
    );
    let mut s: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while s < count as ::core::ffi::c_int {
        if ata_drq_wait(io, 5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            *buffer.offset((s * 256 as ::core::ffi::c_int + i) as isize) = inw(io);
            i += 1;
        }
        s += 1;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ata_read_sectors(
    mut bus: uint8_t,
    mut drive: uint8_t,
    mut lba: uint64_t,
    mut count: uint8_t,
    mut buffer: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if bus as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || drive as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || g_drives[bus as usize][drive as usize].present == 0
        || count as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut io: uint16_t = ata_io_bases[bus as usize];
    if ata_busy_wait(io, 5 as ::core::ffi::c_int) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut result: ::core::ffi::c_int = 0;
    if g_drives[bus as usize][drive as usize].is_lba48 != 0 {
        result = ata_pio_read_lba48(io, drive, lba, count as uint16_t, buffer as *mut uint16_t);
    } else {
        result = ata_pio_read_lba28(io, drive, lba as uint32_t, count, buffer as *mut uint16_t);
    }
    return result;
}
unsafe extern "C" fn ata_pio_write_lba28(
    mut io: uint16_t,
    mut drive: uint8_t,
    mut lba: uint32_t,
    mut count: uint8_t,
    mut buffer: *const uint16_t,
) -> ::core::ffi::c_int {
    outb(
        (io as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16_t,
        ((0xe0 as ::core::ffi::c_int | (drive as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
            as uint32_t
            | lba >> 24 as ::core::ffi::c_int & 0xf as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        count,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        (lba & 0xff as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        (lba >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        (lba >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
        ATA_CMD_WRITE_PIO as uint8_t,
    );
    let mut s: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while s < count as ::core::ffi::c_int {
        if ata_drq_wait(io, 5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            outw(
                io,
                *buffer.offset((s * 256 as ::core::ffi::c_int + i) as isize),
            );
            i += 1;
        }
        s += 1;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn ata_pio_write_lba48(
    mut io: uint16_t,
    mut drive: uint8_t,
    mut lba: uint64_t,
    mut count: uint16_t,
    mut buffer: *const uint16_t,
) -> ::core::ffi::c_int {
    outb(
        (io as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16_t,
        (0x40 as ::core::ffi::c_int | (drive as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
            as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        (lba >> 40 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        (lba >> 32 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        (lba >> 24 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        (count as ::core::ffi::c_int >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16_t,
        (lba & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16_t,
        (lba >> 8 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16_t,
        (lba >> 16 as ::core::ffi::c_int & 0xff as uint64_t) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t,
        (count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
    );
    outb(
        (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
        ATA_CMD_WRITE_PIO_EXT as uint8_t,
    );
    let mut s: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while s < count as ::core::ffi::c_int {
        if ata_drq_wait(io, 5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            outw(
                io,
                *buffer.offset((s * 256 as ::core::ffi::c_int + i) as isize),
            );
            i += 1;
        }
        s += 1;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ata_write_sectors(
    mut bus: uint8_t,
    mut drive: uint8_t,
    mut lba: uint64_t,
    mut count: uint8_t,
    mut buffer: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if bus as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || drive as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || g_drives[bus as usize][drive as usize].present == 0
        || count as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut io: uint16_t = ata_io_bases[bus as usize];
    if ata_busy_wait(io, 5 as ::core::ffi::c_int) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut result: ::core::ffi::c_int = 0;
    if g_drives[bus as usize][drive as usize].is_lba48 != 0 {
        result = ata_pio_write_lba48(io, drive, lba, count as uint16_t, buffer as *const uint16_t);
    } else {
        result = ata_pio_write_lba28(io, drive, lba as uint32_t, count, buffer as *const uint16_t);
    }
    if result != 0 {
        outb(
            (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
            ATA_CMD_FLUSH_CACHE as uint8_t,
        );
        ata_busy_wait(io, 5 as ::core::ffi::c_int);
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ata_get_info(
    mut bus: uint8_t,
    mut drive: uint8_t,
    mut info: *mut ata_drive_info,
) {
    if bus as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || drive as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || info.is_null()
    {
        return;
    }
    *info = g_drives[bus as usize][drive as usize];
}
#[no_mangle]
pub unsafe extern "C" fn ata_flush_cache(mut bus: uint8_t, mut drive: uint8_t) {
    if bus as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || drive as ::core::ffi::c_int > 1 as ::core::ffi::c_int
    {
        return;
    }
    let mut io: uint16_t = ata_io_bases[bus as usize];
    outb(
        (io as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16_t,
        ATA_CMD_FLUSH_CACHE as uint8_t,
    );
    ata_busy_wait(io, 5 as ::core::ffi::c_int);
}
