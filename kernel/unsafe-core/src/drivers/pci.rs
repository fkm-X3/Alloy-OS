//! Safe PCI bus driver (x86_64).
//!
//! Replaces `ported/x86_64/drivers/pci.rs`. Scans the config space through
//! the standard 0xCF8/0xCFC I/O ports and exposes the discovered devices as
//! plain values; the caller-facing API never exposes raw pointers.
//!
//! No C-ABI entry points need to survive here: once the ported AHCI driver is
//! migrated, nothing else in the tree references the old `pci_*` symbols.

use crate::drivers::serial::Serial;
use crate::raw::asm::x86_64::{inl, outl};

/// Config-space I/O port pair (x86 PCI mechanism #1).
const PCI_CONFIG_ADDRESS: u16 = 0xcf8;
const PCI_CONFIG_DATA: u16 = 0xcfc;

/// Offsets in a type-0 config header.
const PCI_VENDOR_ID: u8 = 0;
const PCI_DEVICE_ID: u8 = 0x02;
const PCI_SECONDARY_BUS: u8 = 0x19;
const PCI_HEADER_TYPE_BRIDGE: u8 = 0x1;

const MAX_PCI_DEVICES: usize = 256;

/// Class code for mass-storage controllers (AHCI lookup).
pub const PCI_CLASS_MASS_STORAGE: u8 = 0x01;
/// Subclass code for SATA controllers.
pub const PCI_SUBCLASS_SATA: u8 = 0x06;

/// A single PCI function discovered during the scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u8,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub header_type: u8,
    pub bars: [u32; 6],
}

impl Default for PciDevice {
    fn default() -> Self {
        PciDevice {
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
        }
    }
}

static mut PCI_DEVICES: [PciDevice; MAX_PCI_DEVICES] = [PciDevice {
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
}; MAX_PCI_DEVICES];
static mut PCI_DEVICE_COUNT: usize = 0;

/// Safe PCI bus facade.
pub struct Pci;

impl Pci {
    /// Build the config-space address for a (bus, slot, func, offset) tuple.
    fn make_addr(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
        0x8000_0000
            | ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((func as u32) << 8)
            | ((offset as u32) & 0xfc)
    }

    /// Read a 32-bit dword from a function's config space.
    pub fn config_read_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
        outl(PCI_CONFIG_ADDRESS, Self::make_addr(bus, slot, func, offset));
        inl(PCI_CONFIG_DATA)
    }

    /// Write a 32-bit dword into a function's config space.
    pub fn config_write_dword(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
        outl(PCI_CONFIG_ADDRESS, Self::make_addr(bus, slot, func, offset));
        outl(PCI_CONFIG_DATA, value);
    }

    /// Read a 16-bit word from a function's config space.
    pub fn config_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
        let dword = Self::config_read_dword(bus, slot, func, offset);
        if offset & 2 != 0 {
            (dword >> 16) as u16
        } else {
            dword as u16
        }
    }

    /// Scan the whole bus hierarchy and record every present function.
    pub fn init() {
        Serial::write_str("[PCI] Scanning PCI bus...\n");
        unsafe {
            PCI_DEVICE_COUNT = 0;
        }
        let vendor = Self::config_read_word(0, 0, 0, PCI_VENDOR_ID);
        if vendor == 0xffff {
            Serial::write_str("[PCI] No PCI host controller found\n");
            return;
        }
        Self::scan_bus(0);
        Serial::write_str("[PCI] Found ");
        Serial::write_hex(unsafe { PCI_DEVICE_COUNT } as u32);
        Serial::write_str(" devices\n");
        for i in 0..Self::device_count() {
            let dev = Self::get_device(i).unwrap();
            Serial::write_str("  ");
            Serial::write_hex(dev.bus as u32);
            Serial::write_str(":");
            Serial::write_hex(dev.slot as u32);
            Serial::write_str(".");
            Serial::write_hex(dev.func as u32);
            Serial::write_str(" [");
            Serial::write_hex(dev.class_code as u32);
            Serial::write_str(".");
            Serial::write_hex(dev.subclass as u32);
            Serial::write_str(".");
            Serial::write_hex(dev.prog_if as u32);
            Serial::write_str("] vendor=");
            Serial::write_hex(dev.vendor_id as u32);
            Serial::write_str(" device=");
            Serial::write_hex(dev.device_id as u32);
            Serial::write_str("\n");
        }
    }

    /// Number of devices discovered by [`Pci::init`].
    pub fn device_count() -> usize {
        unsafe { PCI_DEVICE_COUNT }
    }

    /// Copy out the device at `index`, if present.
    pub fn get_device(index: usize) -> Option<PciDevice> {
        if index >= unsafe { PCI_DEVICE_COUNT } {
            return None;
        }
        Some(unsafe { PCI_DEVICES[index] })
    }

    /// Collect devices matching `class`/`subclass` (`prog_if == 0xff` matches
    /// any programming interface) into `out`. Returns the number written.
    pub fn find_devices(
        class_code: u8,
        subclass: u8,
        prog_if: u8,
        out: &mut [PciDevice],
    ) -> usize {
        let mut count = 0usize;
        for i in 0..Self::device_count() {
            if count >= out.len() {
                break;
            }
            let dev = Self::get_device(i).unwrap();
            if dev.class_code == class_code
                && dev.subclass == subclass
                && (prog_if == 0xff || dev.prog_if == prog_if)
            {
                out[count] = dev;
                count += 1;
            }
        }
        count
    }

    fn read_device(bus: u8, slot: u8, func: u8) {
        let vendor = Self::config_read_word(bus, slot, func, PCI_VENDOR_ID);
        if vendor == 0xffff {
            return;
        }
        unsafe {
            if PCI_DEVICE_COUNT >= MAX_PCI_DEVICES {
                return;
            }
            let dev = &mut PCI_DEVICES[PCI_DEVICE_COUNT];
            dev.bus = bus;
            dev.slot = slot;
            dev.func = func;
            dev.vendor_id = vendor;
            dev.device_id = Self::config_read_word(bus, slot, func, PCI_DEVICE_ID);
            let class_reg = Self::config_read_dword(bus, slot, func, 0x08);
            dev.revision_id = (class_reg & 0xff) as u8;
            dev.prog_if = ((class_reg >> 8) & 0xff) as u8;
            dev.subclass = ((class_reg >> 16) & 0xff) as u8;
            dev.class_code = (class_reg >> 24) as u8;
            let header = Self::config_read_dword(bus, slot, func, 0x0c);
            dev.header_type = ((header >> 16) & 0xff) as u8;
            for (i, bar) in dev.bars.iter_mut().enumerate() {
                *bar = Self::config_read_dword(bus, slot, func, (0x10 + i as u8 * 4) as u8);
            }
            PCI_DEVICE_COUNT += 1;
        }
    }

    fn scan_bus(bus: u8) {
        for slot in 0..32u8 {
            let vendor = Self::config_read_word(bus, slot, 0, PCI_VENDOR_ID);
            if vendor == 0xffff {
                continue;
            }
            Self::read_device(bus, slot, 0);
            let header = Self::config_read_dword(bus, slot, 0, 0x0c);
            let header_type = ((header >> 16) & 0xff) as u8;
            if header_type & 0x80 != 0 {
                for func in 1..8u8 {
                    let v = Self::config_read_word(bus, slot, func, PCI_VENDOR_ID);
                    if v != 0xffff {
                        Self::read_device(bus, slot, func);
                    }
                }
            }
            if header_type & 0x7f == PCI_HEADER_TYPE_BRIDGE {
                let bus_reg = Self::config_read_dword(bus, slot, 0, PCI_SECONDARY_BUS);
                let secondary_bus = ((bus_reg >> 8) & 0xff) as u8;
                if secondary_bus != bus {
                    Self::scan_bus(secondary_bus);
                }
            }
        }
    }
}
