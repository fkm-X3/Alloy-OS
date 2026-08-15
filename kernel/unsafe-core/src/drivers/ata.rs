//! Safe ATA PIO driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/ata.rs`. Identifies up to four drives
//! (two buses, master/slave) and performs 28-bit / 48-bit LBA PIO transfers.
//! Buffers are slices, so bounds are enforced before any port I/O.
//!
//! No C-ABI entry points need to survive here: the ported AHCI driver that
//! used to share these registers is migrated to [`crate::drivers::ahci`] and
//! nothing else in the tree references the old `ata_*` symbols.

use crate::drivers::serial::Serial;
use crate::raw::asm::x86_64::{inb, inw, outb, outw};

/// Drive selector on a bus.
pub const ATA_DRIVE_MASTER: u8 = 0;
pub const ATA_DRIVE_SLAVE: u8 = 1;

// ATA commands.
const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
const ATA_CMD_IDENTIFY: u8 = 0xec;
const ATA_CMD_FLUSH_CACHE: u8 = 0xe7;

// Status register flags.
const ATA_STATUS_ERR: u8 = 0x01;
const ATA_STATUS_DRQ: u8 = 0x08;
const ATA_STATUS_BSY: u8 = 0x80;

// I/O bases: primary/secondary bus, command block and control block.
const ATA_PRIMARY_IO: u16 = 0x1f0;
const ATA_PRIMARY_CTRL: u16 = 0x3f6;
const ATA_SECONDARY_IO: u16 = 0x170;
const ATA_SECONDARY_CTRL: u16 = 0x376;

const ATA_IO_BASES: [u16; 2] = [ATA_PRIMARY_IO, ATA_SECONDARY_IO];
const ATA_CTRL_BASES: [u16; 2] = [ATA_PRIMARY_CTRL, ATA_SECONDARY_CTRL];

/// Words-per-sector for PIO transfers.
const WORDS_PER_SECTOR: usize = 256;

/// Identified geometry and identity of one ATA drive.
#[derive(Debug, Clone, Copy)]
pub struct AtaDriveInfo {
    pub present: bool,
    pub is_lba48: bool,
    pub num_sectors: u64,
    pub model: [u8; 41],
}

impl Default for AtaDriveInfo {
    fn default() -> Self {
        AtaDriveInfo {
            present: false,
            is_lba48: false,
            num_sectors: 0,
            model: [0; 41],
        }
    }
}

static mut ATA_DRIVES: [[AtaDriveInfo; 2]; 2] =
    [[AtaDriveInfo { present: false, is_lba48: false, num_sectors: 0, model: [0; 41] }; 2]; 2];
static mut ATA_INITIALIZED: bool = false;

/// Safe ATA PIO facade.
pub struct Ata;

impl Ata {
    /// Identify every drive on both buses. Idempotent: subsequent calls are
    /// no-ops. Returns true once the scan has completed.
    pub fn init() -> bool {
        unsafe {
            if ATA_INITIALIZED {
                return true;
            }
        }
        Serial::write_str("[ATA] Initializing ATA PIO driver...\n");
        unsafe {
            ATA_DRIVES = [[AtaDriveInfo::default(); 2]; 2];
        }
        for bus in 0..2u8 {
            Self::identify(bus, ATA_DRIVE_MASTER);
            Self::identify(bus, ATA_DRIVE_SLAVE);
        }
        unsafe {
            ATA_INITIALIZED = true;
        }
        Serial::write_str("[ATA] ATA PIO driver initialized\n");
        true
    }

    /// True when the drive at (bus, drive) identified successfully.
    pub fn drive_present(bus: u8, drive: u8) -> bool {
        if bus > 1 || drive > 1 {
            return false;
        }
        unsafe { ATA_DRIVES[bus as usize][drive as usize].present }
    }

    /// Copy the identified geometry for (bus, drive).
    pub fn drive_info(bus: u8, drive: u8) -> Option<AtaDriveInfo> {
        if bus > 1 || drive > 1 {
            return None;
        }
        Some(unsafe { ATA_DRIVES[bus as usize][drive as usize] })
    }

    /// Read `count` sectors (512 B each) from `lba` into `buf`.
    pub fn read_sectors(bus: u8, drive: u8, lba: u64, count: u8, buf: &mut [u8]) -> bool {
        if !Self::transfer_checks(bus, drive, count, buf.len()) {
            return false;
        }
        let io = ATA_IO_BASES[bus as usize];
        if !busy_wait(io, 5) {
            return false;
        }
        let (_, words, _) = unsafe { buf.align_to_mut::<u16>() };
        if words.len() < (count as usize) * WORDS_PER_SECTOR {
            return false;
        }
        if Self::is_lba48(bus, drive) {
            pio_read_lba48(io, drive, lba, count, words)
        } else {
            pio_read_lba28(io, drive, lba as u32, count, words)
        }
    }

    /// Write `count` sectors (512 B each) from `buf` to `lba`.
    pub fn write_sectors(bus: u8, drive: u8, lba: u64, count: u8, buf: &[u8]) -> bool {
        if !Self::transfer_checks(bus, drive, count, buf.len()) {
            return false;
        }
        let io = ATA_IO_BASES[bus as usize];
        if !busy_wait(io, 5) {
            return false;
        }
        let (_, words, _) = unsafe { buf.align_to::<u16>() };
        if words.len() < (count as usize) * WORDS_PER_SECTOR {
            return false;
        }
        let result = if Self::is_lba48(bus, drive) {
            pio_write_lba48(io, drive, lba, count, words)
        } else {
            pio_write_lba28(io, drive, lba as u32, count, words)
        };
        if result {
            outb(io + 7, ATA_CMD_FLUSH_CACHE);
            busy_wait(io, 5);
        }
        result
    }

    fn is_lba48(bus: u8, drive: u8) -> bool {
        unsafe { ATA_DRIVES[bus as usize][drive as usize].is_lba48 }
    }

    fn transfer_checks(bus: u8, drive: u8, count: u8, buf_len: usize) -> bool {
        if bus > 1
            || drive > 1
            || count == 0
            || !Self::drive_present(bus, drive)
            || (count as usize) * 512 > buf_len
        {
            return false;
        }
        true
    }

    fn identify(bus: u8, drive: u8) -> bool {
        let io = ATA_IO_BASES[bus as usize];
        soft_reset(bus);
        if !busy_wait(io, 1) {
            return false;
        }
        outb(
            io + 6,
            if drive == ATA_DRIVE_MASTER { 0xa0 } else { 0xb0 },
        );
        delay();
        outb(io + 2, 0);
        outb(io + 3, 0);
        outb(io + 4, 0);
        outb(io + 5, 0);
        outb(io + 7, ATA_CMD_IDENTIFY);
        delay();
        let status = inb(io + 7);
        if status == 0 {
            return false;
        }
        if !busy_wait(io, 1) {
            return false;
        }
        if inb(io + 4) != 0 && inb(io + 5) != 0 {
            return false;
        }
        if drq_wait(io, 1) < 0 {
            return false;
        }

        let mut words = [0u16; 256];
        for w in words.iter_mut() {
            *w = inw(io);
        }

        let mut info = AtaDriveInfo::default();
        info.present = true;
        info.is_lba48 = words[83] & (1 << 10) != 0;
        info.num_sectors = sectors_from_identify(&words);
        for i in 0..20usize {
            let w = words[27 + i];
            info.model[i * 2] = (w >> 8) as u8;
            info.model[i * 2 + 1] = (w & 0xff) as u8;
        }
        info.model[40] = 0;
        let mut end = 39;
        while end > 0 && info.model[end] == b' ' {
            info.model[end] = 0;
            end -= 1;
        }
        if end == 0 && info.model[0] == b' ' {
            info.model[0] = 0;
        }

        unsafe {
            ATA_DRIVES[bus as usize][drive as usize] = info;
        }

        Serial::write_str("[ATA] Drive ");
        Serial::write_hex(bus as u32);
        Serial::write_str(":");
        Serial::write_hex(drive as u32);
        Serial::write_str(": ");
        print_cstr(&info.model);
        Serial::write_str(" (");
        Serial::write_hex((info.num_sectors / 2048) as u32);
        Serial::write_str(" MB LBA");
        Serial::write_str(if info.is_lba48 { "48" } else { "28" });
        Serial::write_str(")\n");
        true
    }
}

/// Extract the sector count from an IDENTIFY DEVICE payload.
fn sectors_from_identify(words: &[u16; 256]) -> u64 {
    let supports_lba48 = words[83] & (1 << 10) != 0;
    if supports_lba48 {
        let lo = words[100] as u64 | ((words[101] as u64) << 16);
        let hi = words[102] as u64 | ((words[103] as u64) << 16);
        lo | (hi << 32)
    } else {
        words[60] as u64 | ((words[61] as u64) << 16)
    }
}

/// Wait a short fixed time by polling the alt-status register.
fn delay() {
    for _ in 0..4 {
        inb(ATA_CTRL_BASES[0]);
        inb(ATA_CTRL_BASES[1]);
    }
}

/// Poll the status register until BSY clears or `timeout_ms` elapses.
fn busy_wait(io_base: u16, timeout_ms: u32) -> bool {
    for _ in 0..(timeout_ms * 1000) {
        let status = inb(io_base + 7);
        if status & ATA_STATUS_BSY == 0 {
            return true;
        }
        delay();
    }
    false
}

/// Poll the status register for DRQ (1), ERR (-1), or done-without-data (0).
fn drq_wait(io_base: u16, timeout_ms: u32) -> i32 {
    for _ in 0..(timeout_ms * 1000) {
        let status = inb(io_base + 7);
        if status & ATA_STATUS_ERR != 0 {
            return -1;
        }
        if status & ATA_STATUS_DRQ != 0 {
            return 1;
        }
        if status & ATA_STATUS_BSY == 0 {
            return 0;
        }
        delay();
    }
    -2
}

/// Pulse a device reset on the control register.
fn soft_reset(bus: u8) {
    let ctrl = ATA_CTRL_BASES[bus as usize];
    outb(ctrl, 0x04);
    delay();
    outb(ctrl, 0);
    delay();
}

/// 28-bit PIO read: write the command block, then drain 256 words per sector.
fn pio_read_lba28(io: u16, drive: u8, lba: u32, count: u8, buffer: &mut [u16]) -> bool {
    outb(io + 6, (0xe0 | (drive << 4) | ((lba >> 24) & 0x0f) as u8) as u8);
    outb(io + 2, count);
    outb(io + 3, (lba & 0xff) as u8);
    outb(io + 4, ((lba >> 8) & 0xff) as u8);
    outb(io + 5, ((lba >> 16) & 0xff) as u8);
    outb(io + 7, ATA_CMD_READ_PIO);
    for s in 0..count as usize {
        if drq_wait(io, 5) < 0 {
            return false;
        }
        for i in 0..WORDS_PER_SECTOR {
            buffer[s * WORDS_PER_SECTOR + i] = inw(io);
        }
    }
    true
}

/// 48-bit PIO read: two writes per register field, then drain words.
fn pio_read_lba48(io: u16, drive: u8, lba: u64, count: u8, buffer: &mut [u16]) -> bool {
    outb(io + 6, 0x40 | (drive << 4));
    outb(io + 5, ((lba >> 24) & 0xff) as u8);
    outb(io + 4, ((lba >> 32) & 0xff) as u8);
    outb(io + 3, ((lba >> 40) & 0xff) as u8);
    outb(io + 2, ((count as u16) >> 8) as u8);
    outb(io + 3, (lba & 0xff) as u8);
    outb(io + 4, ((lba >> 8) & 0xff) as u8);
    outb(io + 5, ((lba >> 16) & 0xff) as u8);
    outb(io + 2, (count & 0xff) as u8);
    outb(io + 7, ATA_CMD_READ_PIO_EXT);
    for s in 0..count as usize {
        if drq_wait(io, 5) < 0 {
            return false;
        }
        for i in 0..WORDS_PER_SECTOR {
            buffer[s * WORDS_PER_SECTOR + i] = inw(io);
        }
    }
    true
}

/// 28-bit PIO write: write the command block, then push 256 words per sector.
fn pio_write_lba28(io: u16, drive: u8, lba: u32, count: u8, buffer: &[u16]) -> bool {
    outb(io + 6, (0xe0 | (drive << 4) | ((lba >> 24) & 0x0f) as u8) as u8);
    outb(io + 2, count);
    outb(io + 3, (lba & 0xff) as u8);
    outb(io + 4, ((lba >> 8) & 0xff) as u8);
    outb(io + 5, ((lba >> 16) & 0xff) as u8);
    outb(io + 7, ATA_CMD_WRITE_PIO);
    for s in 0..count as usize {
        if drq_wait(io, 5) < 0 {
            return false;
        }
        for i in 0..WORDS_PER_SECTOR {
            outw(io, buffer[s * WORDS_PER_SECTOR + i]);
        }
    }
    true
}

/// 48-bit PIO write: two writes per register field, then push words.
fn pio_write_lba48(io: u16, drive: u8, lba: u64, count: u8, buffer: &[u16]) -> bool {
    outb(io + 6, 0x40 | (drive << 4));
    outb(io + 5, ((lba >> 40) & 0xff) as u8);
    outb(io + 4, ((lba >> 32) & 0xff) as u8);
    outb(io + 3, ((lba >> 24) & 0xff) as u8);
    outb(io + 2, ((count as u16) >> 8) as u8);
    outb(io + 3, (lba & 0xff) as u8);
    outb(io + 4, ((lba >> 8) & 0xff) as u8);
    outb(io + 5, ((lba >> 16) & 0xff) as u8);
    outb(io + 2, (count & 0xff) as u8);
    outb(io + 7, ATA_CMD_WRITE_PIO_EXT);
    for s in 0..count as usize {
        if drq_wait(io, 5) < 0 {
            return false;
        }
        for i in 0..WORDS_PER_SECTOR {
            outw(io, buffer[s * WORDS_PER_SECTOR + i]);
        }
    }
    true
}

/// Write a NUL-terminated byte string (used for the IDENTIFY model field).
fn print_cstr(bytes: &[u8]) {
    for &b in bytes {
        if b == 0 {
            break;
        }
        Serial::write_byte(b);
    }
}
