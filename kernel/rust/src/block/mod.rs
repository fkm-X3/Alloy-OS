pub mod ramdisk;

use alloc::vec::Vec;
use alloc::boxed::Box;

pub const SECTOR_SIZE: usize = 512;

pub trait BlockDevice: Send {
    fn num_sectors(&self) -> u64;
    fn read_sectors(&mut self, lba: u64, count: u64, buf: &mut [u8]) -> Result<(), ()>;
    fn write_sectors(&mut self, lba: u64, count: u64, buf: &[u8]) -> Result<(), ()>;
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub struct AtaDevice {
    bus: u8,
    drive: u8,
    num_sectors: u64,
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
impl AtaDevice {
    pub fn new(bus: u8, drive: u8) -> Option<Self> {
        if !crate::ffi::ata_drive_exists(bus, drive) {
            return None;
        }
        Some(AtaDevice { bus, drive, num_sectors: 0 })
    }

    pub fn probe(&mut self) {
        self.num_sectors = 0;
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
impl BlockDevice for AtaDevice {
    fn num_sectors(&self) -> u64 {
        self.num_sectors
    }

    fn read_sectors(&mut self, lba: u64, count: u64, buf: &mut [u8]) -> Result<(), ()> {
        let total = (count as usize) * SECTOR_SIZE;
        if buf.len() < total {
            return Err(());
        }
        let mut offset = 0usize;
        let mut current_lba = lba;
        let mut done = 0u64;

        while done < count {
            let batch = core::cmp::min(count - done, 256) as u8;
            let chunk_len = (batch as usize) * SECTOR_SIZE;
            if !crate::ffi::ata_read(self.bus, self.drive, current_lba, batch,
                                      &mut buf[offset..offset + chunk_len]) {
                return Err(());
            }
            current_lba += batch as u64;
            offset += chunk_len;
            done += batch as u64;
        }
        Ok(())
    }

    fn write_sectors(&mut self, lba: u64, count: u64, buf: &[u8]) -> Result<(), ()> {
        let total = (count as usize) * SECTOR_SIZE;
        if buf.len() < total {
            return Err(());
        }
        let mut offset = 0usize;
        let mut current_lba = lba;
        let mut done = 0u64;

        while done < count {
            let batch = core::cmp::min(count - done, 256) as u8;
            let chunk_len = (batch as usize) * SECTOR_SIZE;
            if !crate::ffi::ata_write(self.bus, self.drive, current_lba, batch,
                                       &buf[offset..offset + chunk_len]) {
                return Err(());
            }
            current_lba += batch as u64;
            offset += chunk_len;
            done += batch as u64;
        }
        Ok(())
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
pub struct AhciDevice {
    index: i32,
    num_sectors: u64,
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
impl AhciDevice {
    pub fn new(index: i32) -> Option<Self> {
        Some(AhciDevice { index, num_sectors: 0 })
    }

    pub fn probe(&mut self) {
        self.num_sectors = 0;
    }
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
impl BlockDevice for AhciDevice {
    fn num_sectors(&self) -> u64 {
        self.num_sectors
    }

    fn read_sectors(&mut self, lba: u64, count: u64, buf: &mut [u8]) -> Result<(), ()> {
        let total = (count as usize) * SECTOR_SIZE;
        if buf.len() < total {
            return Err(());
        }
        let mut offset = 0usize;
        let mut current_lba = lba;
        let mut done = 0u64;

        while done < count {
            let batch = core::cmp::min(count - done, 255) as u8;
            let chunk = (batch as usize) * SECTOR_SIZE;
            if !crate::ffi::ahci_read(self.index, current_lba, batch,
                                       &mut buf[offset..offset + chunk]) {
                return Err(());
            }
            current_lba += batch as u64;
            offset += chunk;
            done += batch as u64;
        }
        Ok(())
    }

    fn write_sectors(&mut self, lba: u64, count: u64, buf: &[u8]) -> Result<(), ()> {
        let total = (count as usize) * SECTOR_SIZE;
        if buf.len() < total {
            return Err(());
        }
        let mut offset = 0usize;
        let mut current_lba = lba;
        let mut done = 0u64;

        while done < count {
            let batch = core::cmp::min(count - done, 255) as u8;
            let chunk = (batch as usize) * SECTOR_SIZE;
            if !crate::ffi::ahci_write(self.index, current_lba, batch,
                                        &buf[offset..offset + chunk]) {
                return Err(());
            }
            current_lba += batch as u64;
            offset += chunk;
            done += batch as u64;
        }
        Ok(())
    }
}

pub fn init_block_devices() -> Vec<Box<dyn BlockDevice>> {
    #[cfg(any(feature = "i686", feature = "x86_64"))]
    {
        let mut devices: Vec<Box<dyn BlockDevice>> = Vec::new();

        for bus in 0..=1u8 {
            for drive in 0..=1u8 {
                if crate::ffi::ata_drive_exists(bus, drive) {
                    crate::ffi::print_str(&alloc::format!("[block] ATA {bus}:{drive} detected\n"));
                    let mut dev = AtaDevice::new(bus, drive).unwrap();
                    dev.probe();
                    devices.push(Box::new(dev));
                }
            }
        }

        let ahci_count = crate::ffi::ahci_drive_count_ffi();
        for i in 0..ahci_count {
            let mut dev = AhciDevice::new(i).unwrap();
            dev.probe();
            devices.push(Box::new(dev));
        }

        if crate::ffi::initrd_has_modules() {
            let count = crate::ffi::initrd_module_count_ffi();
            for i in 0..count {
                let start = crate::ffi::initrd_module_start(i);
                let size = crate::ffi::initrd_module_size(i);
                let cmdline = crate::ffi::initrd_module_cmdline(i);
                if size > 0 && (size % SECTOR_SIZE) == 0 {
                    let name = core::str::from_utf8(&cmdline).unwrap_or("initrd");
                    crate::ffi::print_str(&alloc::format!("[block] ramdisk #{i}: {name} start=0x{start:x} size={size}\n"));
                    devices.push(Box::new(ramdisk::Ramdisk::new(start, size)));
                }
            }
        }

        devices
    }
    #[cfg(feature = "aarch64")]
    {
        Vec::new()
    }
}
