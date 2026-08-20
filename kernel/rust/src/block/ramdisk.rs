use super::BlockDevice;
use super::SECTOR_SIZE;

pub struct Ramdisk {
    phys_start: usize,
    num_sectors: u64,
}

impl Ramdisk {
    pub fn new(phys_start: usize, size_bytes: usize) -> Self {
        Ramdisk {
            phys_start,
            num_sectors: size_bytes as u64 / SECTOR_SIZE as u64,
        }
    }
}

impl BlockDevice for Ramdisk {
    fn num_sectors(&self) -> u64 {
        self.num_sectors
    }

    fn read_sectors(&mut self, lba: u64, count: u64, buf: &mut [u8]) -> Result<(), ()> {
        if lba
            .checked_add(count)
            .map_or(true, |end| end > self.num_sectors)
        {
            return Err(());
        }
        let byte_off = (lba as usize) * SECTOR_SIZE;
        let total = (count as usize) * SECTOR_SIZE;
        if buf.len() < total {
            return Err(());
        }
        let src_phys = self.phys_start as usize + byte_off;

        alloy_kernel_hal::mem::read_phys_bytes(src_phys, &mut buf[..total]);
        Ok(())
    }

    fn write_sectors(&mut self, lba: u64, count: u64, buf: &[u8]) -> Result<(), ()> {
        if lba
            .checked_add(count)
            .map_or(true, |end| end > self.num_sectors)
        {
            return Err(());
        }
        let byte_off = (lba as usize) * SECTOR_SIZE;
        let total = (count as usize) * SECTOR_SIZE;
        if buf.len() < total {
            return Err(());
        }
        let dst_phys = self.phys_start as usize + byte_off;

        alloy_kernel_hal::mem::write_phys_bytes(dst_phys, &buf[..total]);
        Ok(())
    }
}
