//! Safe AHCI (SATA) driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/ahci.rs`. Locates a SATA mass-storage
//! controller on the PCI bus, maps its ABAR, probes every implemented port,
//! and issues DMA-EXT commands through a single-page command list / command
//! table per transfer.
//!
//! Transfers are split into 8-sector (4 KiB) commands so the single-page PRDT
//! bounce buffer always fits: the C2Rust driver wrote up to 255 sectors into
//! one page. No C-ABI entry points need to survive here; the boot path drives
//! [`crate::drivers::pci`] through [`Ahci::init`] directly.

use core::ffi::c_void;

use crate::drivers::pci::{Pci, PciDevice, PCI_CLASS_MASS_STORAGE, PCI_SUBCLASS_SATA};
use crate::drivers::serial::Serial;
use crate::mem::{map_page, PageFlags, VmRegion};
use crate::raw::ffi;

const PAGE_SIZE: usize = 4096;
const SECTOR_SIZE: u32 = 512;

/// Maximum sectors per command: the DMA bounce page is one 4 KiB frame.
const MAX_SECTORS_PER_CMD: u8 = 8;

// HBA registers (offsets from the ABAR).
const HBA_CAP: u32 = 0x00;
const HBA_GHC: u32 = 0x04;
const HBA_PI: u32 = 0x0c;
const HBA_GHC_AE: u32 = 1 << 31;
const HBA_CAP_NP_MASK: u32 = 0x1f;

// Port registers (offset = 0x100 + port * 0x80).
const PORT_BASE: u32 = 0x100;
const PORT_STRIDE: u32 = 0x80;
const REG_CMD: u32 = 0x18;
const REG_TFD: u32 = 0x20;
const REG_SIG: u32 = 0x24;
const REG_SSTS: u32 = 0x28;
const REG_SERR: u32 = 0x30;
const REG_CI: u32 = 0x38;
const REG_CLB: u32 = 0x00;
const REG_CLBU: u32 = 0x04;

const PORT_CMD_ST: u32 = 1 << 0;
const PORT_CMD_SPIN_UP: u32 = 1 << 1;
const PORT_CMD_POWER_ON: u32 = 1 << 2;
const PORT_TFD_BSY: u32 = 1 << 7;
const PORT_TFD_DRQ: u32 = 1 << 3;
const PORT_SSTS_DET: u32 = 3;
const PORT_SIG_ATA: u32 = 0x101;

const CMD_IDENTIFY: u8 = 0xec;
const CMD_READ_DMA_EXT: u8 = 0x25;
const CMD_WRITE_DMA_EXT: u8 = 0x35;
const H2D_FIS_TYPE: u8 = 0x27;

const PCI_COMMAND: u8 = 0x04;
const PCI_COMMAND_ENABLE: u16 = 0x07;

const MAX_PORTS: usize = 32;
const MAX_DRIVES: usize = 32;

/// Identified geometry and identity of one SATA drive.
#[derive(Debug, Clone, Copy)]
pub struct AhciDriveInfo {
    pub present: bool,
    pub port_num: u8,
    pub num_sectors: u64,
    pub model: [u8; 41],
    clb_va: usize,
}

impl Default for AhciDriveInfo {
    fn default() -> Self {
        AhciDriveInfo {
            present: false,
            port_num: 0,
            num_sectors: 0,
            model: [0; 41],
            clb_va: 0,
        }
    }
}

/// A single 4 KiB DMA page, owned by a mapped [`VmRegion`].
///
/// The physical address backing the region is handed to the controller; on
/// drop the region is unmapped and the frame returned to the PMM, so DMA
/// buffers never outlive the command they serve.
struct DmaPage {
    region: VmRegion,
}

impl DmaPage {
    fn alloc() -> Option<Self> {
        VmRegion::alloc(PAGE_SIZE, PageFlags::kernel_write()).map(|region| DmaPage { region })
    }

    fn va(&self) -> usize {
        self.region.addr()
    }

    fn phys(&self) -> u32 {
        unsafe { ffi::paging_get_physical_address(self.region.addr()) as u32 }
    }

    /// Forget the wrapper: the region stays mapped and its frame is never
    /// freed, so the page can live for the driver's lifetime.
    fn leak(self) -> (usize, u32) {
        let va = self.region.addr();
        let phys = self.phys();
        core::mem::forget(self);
        (va, phys)
    }
}

/// Command header: 32 bytes at the front of the command-list page.
#[repr(C)]
struct HbaCmdHdr {
    flags: u16,
    prdtl: u16,
    prdbc: u32,
    ctba: u32,
    ctbau: u32,
    rsv3: [u32; 4],
}

/// Command table: FIS area plus a single PRDT entry.
#[repr(C)]
struct HbaCmdTbl {
    cfis: [u8; 64],
    acmd: [u8; 16],
    rsv: [u8; 48],
    prdt: [PrdtEntry; 1],
}

/// Physical-region descriptor table entry (16 bytes).
#[repr(C, packed)]
struct PrdtEntry {
    dba: u32,
    dbau: u32,
    rsv: u32,
    flags: u32,
}

static mut G_ABAR_VA: usize = 0;
static mut G_INITIALIZED: bool = false;
static mut G_DRIVES: [AhciDriveInfo; MAX_DRIVES] = [AhciDriveInfo {
    present: false,
    port_num: 0,
    num_sectors: 0,
    model: [0; 41],
    clb_va: 0,
}; MAX_DRIVES];
static mut G_DRIVE_COUNT: usize = 0;

/// Safe AHCI facade.
pub struct Ahci;

impl Ahci {
    /// Scan the PCI bus for a SATA controller, map its ABAR, and probe every
    /// implemented port. Idempotent. Returns true when at least one drive was
    /// identified.
    pub fn init() -> bool {
        unsafe {
            if G_INITIALIZED {
                return G_DRIVE_COUNT > 0;
            }
        }
        Serial::write_str("[AHCI] Scanning SATA...\n");

        Pci::init();

        let mut hosts = [PciDevice::default(); 8];
        let n = Pci::find_devices(
            PCI_CLASS_MASS_STORAGE,
            PCI_SUBCLASS_SATA,
            0xff,
            &mut hosts,
        );
        if n == 0 {
            Serial::write_str("[AHCI] No SATA host\n");
            unsafe {
                G_INITIALIZED = true;
            }
            return false;
        }
        let host = hosts[0];
        let cmd = Pci::config_read_word(host.bus, host.slot, host.func, PCI_COMMAND);
        Pci::config_write_dword(
            host.bus,
            host.slot,
            host.func,
            PCI_COMMAND,
            (cmd | PCI_COMMAND_ENABLE) as u32,
        );

        let abar = host.bars[5] & !1;
        if abar == 0 {
            Serial::write_str("[AHCI] No ABAR\n");
            unsafe {
                G_INITIALIZED = true;
            }
            return false;
        }

        let region = match VmRegion::alloc(PAGE_SIZE * 2, PageFlags::kernel_write()) {
            Some(r) => r,
            None => {
                unsafe {
                    G_INITIALIZED = true;
                }
                return false;
            }
        };
        let abar_va = region.addr();
        for off in (0..PAGE_SIZE * 2).step_by(PAGE_SIZE) {
            unsafe {
                ffi::vmm_unmap((abar_va + off) as *mut c_void);
            }
            if !map_page(abar_va + off, (abar as usize) + off, PageFlags::kernel_write()) {
                unsafe {
                    G_INITIALIZED = true;
                }
                return false;
            }
        }
        // The ABAR mapping lives for the driver's lifetime.
        region.leak();
        unsafe {
            G_ABAR_VA = abar_va;
        }

        mmio_w32(HBA_GHC, HBA_GHC_AE);
        let pi = mmio_r32(HBA_PI);
        let ports = (mmio_r32(HBA_CAP) & HBA_CAP_NP_MASK) as usize;
        unsafe {
            G_DRIVE_COUNT = 0;
        }

        for p in 0..ports.min(MAX_PORTS) {
            if pi & (1 << p) == 0 {
                continue;
            }
            port_w(
                p as i32,
                REG_CMD,
                port_r(p as i32, REG_CMD) | PORT_CMD_SPIN_UP | PORT_CMD_POWER_ON,
            );
            let ssts = port_r(p as i32, REG_SSTS);
            let sig = port_r(p as i32, REG_SIG);
            if ssts & 0x0f != PORT_SSTS_DET {
                continue;
            }
            if sig != PORT_SIG_ATA {
                continue;
            }
            if let Some(info) = Self::identify(p as i32) {
                unsafe {
                    if G_DRIVE_COUNT >= MAX_DRIVES {
                        break;
                    }
                    G_DRIVES[G_DRIVE_COUNT] = info;
                    G_DRIVE_COUNT += 1;
                }
            }
        }

        unsafe {
            G_INITIALIZED = true;
        }
        Serial::write_str("[AHCI] ");
        Serial::write_hex(unsafe { G_DRIVE_COUNT } as u32);
        Serial::write_str(" SATA drive(s)\n");
        unsafe { G_DRIVE_COUNT > 0 }
    }

    /// Number of drives identified by [`Ahci::init`].
    pub fn drive_count() -> usize {
        unsafe { G_DRIVE_COUNT }
    }

    /// Copy out the geometry for drive `index`, if present.
    pub fn drive_info(index: usize) -> Option<AhciDriveInfo> {
        if index >= unsafe { G_DRIVE_COUNT } {
            return None;
        }
        Some(unsafe { G_DRIVES[index] })
    }

    /// Read `count` sectors (512 B each) from `lba` on drive `index` into
    /// `buf`, split into 8-sector DMA commands.
    pub fn read_sectors(index: usize, lba: u64, count: u8, buf: &mut [u8]) -> bool {
        if index >= Self::drive_count() || count == 0 || buf.len() < (count as usize) * 512 {
            return false;
        }
        let mut cur_lba = lba;
        let mut done = 0u64;
        let mut offset = 0usize;
        while done < count as u64 {
            let batch = core::cmp::min(count as u64 - done, MAX_SECTORS_PER_CMD as u64) as u8;
            let bytes = (batch as usize) * 512;
            let page = match DmaPage::alloc() {
                Some(p) => p,
                None => return false,
            };
            if !Self::send_cmd(index, false, cur_lba, batch, page.phys()) {
                return false;
            }
            let src = unsafe { core::slice::from_raw_parts(page.va() as *const u8, bytes) };
            buf[offset..offset + bytes].copy_from_slice(src);
            cur_lba += batch as u64;
            done += batch as u64;
            offset += bytes;
        }
        true
    }

    /// Write `count` sectors (512 B each) from `buf` to `lba` on drive
    /// `index`, split into 8-sector DMA commands.
    pub fn write_sectors(index: usize, lba: u64, count: u8, buf: &[u8]) -> bool {
        if index >= Self::drive_count() || count == 0 || buf.len() < (count as usize) * 512 {
            return false;
        }
        let mut cur_lba = lba;
        let mut done = 0u64;
        let mut offset = 0usize;
        while done < count as u64 {
            let batch = core::cmp::min(count as u64 - done, MAX_SECTORS_PER_CMD as u64) as u8;
            let bytes = (batch as usize) * 512;
            let page = match DmaPage::alloc() {
                Some(p) => p,
                None => return false,
            };
            let dst = unsafe { core::slice::from_raw_parts_mut(page.va() as *mut u8, bytes) };
            dst.copy_from_slice(&buf[offset..offset + bytes]);
            if !Self::send_cmd(index, true, cur_lba, batch, page.phys()) {
                return false;
            }
            cur_lba += batch as u64;
            done += batch as u64;
            offset += bytes;
        }
        true
    }

    /// Probe a port with an ATA signature: issue IDENTIFY and record geometry.
    fn identify(port: i32) -> Option<AhciDriveInfo> {
        let clb = DmaPage::alloc()?;
        let ct = DmaPage::alloc()?;
        let id = DmaPage::alloc()?;

        let hdr = unsafe { &mut *(clb.va() as *mut HbaCmdHdr) };
        hdr.flags = fis_cfl();
        hdr.prdtl = 1;
        hdr.prdbc = SECTOR_SIZE;
        hdr.ctba = ct.phys();
        hdr.ctbau = 0;
        hdr.rsv3 = [0; 4];

        let tbl = unsafe { &mut *(ct.va() as *mut HbaCmdTbl) };
        tbl.cfis = [0; 64];
        tbl.acmd = [0; 16];
        tbl.rsv = [0; 48];
        write_fis(&mut tbl.cfis, CMD_IDENTIFY, 0, 0);
        tbl.prdt[0].dba = id.phys();
        tbl.prdt[0].dbau = 0;
        tbl.prdt[0].rsv = 0;
        tbl.prdt[0].flags = (SECTOR_SIZE - 1) | (1 << 31);

        // Stop the port engine first: a prior boot stage may have left the
        // command list running, in which case the HBA ignores a new PxCLB
        // address and keeps executing the stale list. Cycling ST forces the
        // controller to unmap the old list and re-map ours on the next PxCMD
        // write.
        port_w(port, REG_CMD, port_r(port, REG_CMD) & !PORT_CMD_ST);
        port_w(port, REG_CLB, clb.phys());
        port_w(port, REG_CLBU, 0);
        port_w(port, REG_SERR, !0u32);
        port_w(port, REG_CMD, port_r(port, REG_CMD) | PORT_CMD_ST);
        if !spin_ready(port, 5) {
            return None;
        }
        port_w(port, REG_CI, 1);
        if !wait_command(port) {
            return None;
        }

        let words = unsafe { core::slice::from_raw_parts(id.va() as *const u16, 256) };
        let mut info = AhciDriveInfo::default();
        let (clb_va, _clb_phys) = clb.leak();
        info.present = true;
        info.port_num = port as u8;
        info.clb_va = clb_va;
        info.num_sectors = sectors_from_identify(words);
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

        Serial::write_str("[AHCI] Port ");
        Serial::write_hex(port as u32);
        Serial::write_str(": ");
        print_cstr(&info.model);
        Serial::write_str(" (");
        Serial::write_hex((info.num_sectors / 2048) as u32);
        Serial::write_str(" MB)\n");
        Some(info)
    }

    /// Build and fire a single DMA-EXT command for `data` on drive `index`.
    ///
    /// Reuses the drive's persistent command-list page (mapped at probe time),
    /// rewriting slot 0 in place with a fresh command-table page per call.
    fn send_cmd(index: usize, write: bool, lba: u64, count: u8, data_phys: u32) -> bool {
        let port = unsafe { G_DRIVES[index].port_num } as i32;
        let clb_va = unsafe { G_DRIVES[index].clb_va };
        if !spin_ready(port, 10) {
            return false;
        }
        let ct = match DmaPage::alloc() {
            Some(p) => p,
            None => return false,
        };

        let hdr = unsafe { &mut *(clb_va as *mut HbaCmdHdr) };
        hdr.flags = fis_cfl() | (if write { 1 << 6 } else { 0 });
        hdr.prdtl = 1;
        hdr.prdbc = (count as u32) * SECTOR_SIZE;
        hdr.ctba = ct.phys();
        hdr.ctbau = 0;
        hdr.rsv3 = [0; 4];

        let tbl = unsafe { &mut *(ct.va() as *mut HbaCmdTbl) };
        tbl.cfis = [0; 64];
        tbl.acmd = [0; 16];
        tbl.rsv = [0; 48];
        write_fis(
            &mut tbl.cfis,
            if write { CMD_WRITE_DMA_EXT } else { CMD_READ_DMA_EXT },
            lba,
            count,
        );
        tbl.prdt[0].dba = data_phys;
        tbl.prdt[0].dbau = 0;
        tbl.prdt[0].rsv = 0;
        tbl.prdt[0].flags = ((count as u32) * SECTOR_SIZE - 1) | (1 << 31);

        port_w(port, REG_CI, 1);
        wait_command(port)
    }
}

/// FIS length field for the 20-byte host-to-device FIS (in dwords).
fn fis_cfl() -> u16 {
    (core::mem::size_of::<[u8; 20]>() / 4) as u16
}

/// Fill a host-to-device FIS at `cfis` (a 64-byte command-table FIS area).
fn write_fis(cfis: &mut [u8; 64], cmd: u8, lba: u64, count: u8) {
    cfis[0] = H2D_FIS_TYPE;
    cfis[1] = 0x80; // C bit
    cfis[2] = cmd;
    cfis[3] = 0; // feature low
    cfis[4] = (lba & 0xff) as u8;
    cfis[5] = ((lba >> 8) & 0xff) as u8;
    cfis[6] = ((lba >> 16) & 0xff) as u8;
    cfis[7] = 0x40; // device
    cfis[8] = ((lba >> 24) & 0xff) as u8;
    cfis[9] = ((lba >> 32) & 0xff) as u8;
    cfis[10] = ((lba >> 40) & 0xff) as u8;
    cfis[11] = 0; // feature high
    cfis[12] = count & 0xff;
    cfis[13] = ((count as u16) >> 8) as u8;
}

/// Extract the sector count from an IDENTIFY DEVICE payload.
fn sectors_from_identify(words: &[u16]) -> u64 {
    if words[83] & (1 << 10) != 0 {
        let lo = words[100] as u64 | ((words[101] as u64) << 16);
        let hi = words[102] as u64 | ((words[103] as u64) << 16);
        lo | (hi << 32)
    } else {
        words[60] as u64 | ((words[61] as u64) << 16)
    }
}

/// Wait until the port's task-file busy/drq bits clear, up to `ms` millis.
fn spin_ready(port: i32, ms: u32) -> bool {
    for _ in 0..(ms * 10_000) {
        let tfd = port_r(port, REG_TFD);
        if tfd & PORT_TFD_BSY == 0 && tfd & PORT_TFD_DRQ == 0 {
            return true;
        }
    }
    false
}

/// Wait for the issued command to complete (the CI bit clears).
fn wait_command(port: i32) -> bool {
    for _ in 0..10_000_000 {
        if port_r(port, REG_CI) & 1 == 0 {
            return true;
        }
    }
    false
}

#[inline]
fn mmio_r32(offset: u32) -> u32 {
    unsafe { core::ptr::read_volatile((G_ABAR_VA + offset as usize) as *const u32) }
}

#[inline]
fn mmio_w32(offset: u32, value: u32) {
    unsafe { core::ptr::write_volatile((G_ABAR_VA + offset as usize) as *mut u32, value) }
}

#[inline]
fn port_r(port: i32, offset: u32) -> u32 {
    mmio_r32(PORT_BASE + (port as u32) * PORT_STRIDE + offset)
}

#[inline]
fn port_w(port: i32, offset: u32, value: u32) {
    mmio_w32(PORT_BASE + (port as u32) * PORT_STRIDE + offset, value);
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
