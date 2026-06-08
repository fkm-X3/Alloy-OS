use alloc::vec::Vec;
use crate::block::BlockDevice;
use crate::block::SECTOR_SIZE;

const FAT32_EOC: u32 = 0x0FFFFFF8;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_LONG_NAME: u8 = 0x0F;
const ATTR_VOLUME_ID: u8 = 0x08;
const MAX_PATH_COMPONENTS: usize = 32;

#[repr(C, packed)]
struct Bpb {
    bs_jmp_boot: [u8; 3],
    bs_oem_name: [u8; 8],
    bpb_byts_per_sec: u16,
    bpb_sec_per_clus: u8,
    bpb_rsvd_sec_cnt: u16,
    bpb_num_fats: u8,
    bpb_root_ent_cnt: u16,
    bpb_tot_sec16: u16,
    bpb_media: u8,
    bpb_fat_sz16: u16,
    bpb_sec_per_trk: u16,
    bpb_num_heads: u16,
    bpb_hidd_sec: u32,
    bpb_tot_sec32: u32,
    bpb_fat_sz32: u32,
    bpb_ext_flags: u16,
    bpb_fs_ver: u16,
    bpb_root_clus: u32,
    bpb_fs_info: u16,
    bpb_bk_boot_sec: u16,
    bpb_reserved: [u8; 12],
    bpb_drv_num: u8,
    bpb_reserved1: u8,
    bpb_boot_sig: u8,
    bpb_vol_id: u32,
    bpb_vol_lab: [u8; 11],
    bpb_fil_sys_type: [u8; 8],
}

#[allow(dead_code)]
pub struct Fat32Fs {
    device: usize,
    bytes_per_sec: u16,
    sec_per_clus: u8,
    rsvd_sec_cnt: u32,
    num_fats: u8,
    fat_sz32: u32,
    root_clus: u32,
    first_data_sector: u32,

    fat_cache: Vec<u8>,
    cached_fat_sector: u32,
    fat_dirty: bool,
}

pub struct Fat32File {
    pub name: [u8; 256],
    pub name_len: usize,
    pub size: u32,
    pub is_dir: bool,
    pub first_cluster: u32,
    pub attributes: u8,
}

impl Fat32Fs {
    pub fn new(device_id: usize, dev: &mut dyn BlockDevice) -> Result<Self, ()> {
        let mut sector0 = [0u8; SECTOR_SIZE];
        dev.read_sectors(0, 1, &mut sector0).map_err(|_| ())?;

        let bpb: &Bpb = unsafe { &*(sector0.as_ptr() as *const Bpb) };

        let byts_per_sec = u16::from_le(bpb.bpb_byts_per_sec);
        let sec_per_clus = bpb.bpb_sec_per_clus;
        let rsvd_sec_cnt = bpb.bpb_rsvd_sec_cnt as u32;
        let num_fats = bpb.bpb_num_fats;
        let fat_sz32 = u32::from_le(bpb.bpb_fat_sz32);
        let root_clus = u32::from_le(bpb.bpb_root_clus);

        let first_data_sector = rsvd_sec_cnt + (num_fats as u32) * fat_sz32;

        if byts_per_sec != 512 || sec_per_clus == 0 || fat_sz32 == 0 {
            return Err(());
        }

        Ok(Fat32Fs {
            device: device_id,
            bytes_per_sec: byts_per_sec,
            sec_per_clus,
            rsvd_sec_cnt,
            num_fats,
            fat_sz32,
            root_clus,
            first_data_sector,
            fat_cache: Vec::new(),
            cached_fat_sector: u32::MAX,
            fat_dirty: false,
        })
    }

    fn read_fat_entry(&mut self, cluster: u32, dev: &mut dyn BlockDevice) -> Result<u32, ()> {
        let fat_sec = self.rsvd_sec_cnt + (cluster * 4) / (self.bytes_per_sec as u32);
        let byte_in_sec = ((cluster * 4) as usize) % (self.bytes_per_sec as usize);

        if fat_sec != self.cached_fat_sector {
            self.fat_cache.resize(self.bytes_per_sec as usize, 0);
            dev.read_sectors(fat_sec as u64, 1, self.fat_cache.as_mut_slice())
                .map_err(|_| ())?;
            self.cached_fat_sector = fat_sec;
        }

        let raw = u32::from_le_bytes([
            self.fat_cache[byte_in_sec],
            self.fat_cache[byte_in_sec + 1],
            self.fat_cache[byte_in_sec + 2],
            self.fat_cache[byte_in_sec + 3],
        ]);

        Ok(raw & 0x0FFFFFFF)
    }

    fn cluster_to_sector(&self, cluster: u32) -> u64 {
        ((cluster - 2) as u64) * (self.sec_per_clus as u64) + self.first_data_sector as u64
    }

    fn read_cluster(&self, cluster: u32, dev: &mut dyn BlockDevice, buf: &mut [u8]) -> Result<(), ()> {
        let sector = self.cluster_to_sector(cluster);
        let bytes_per_cluster = (self.sec_per_clus as usize) * (self.bytes_per_sec as usize);
        let count = core::cmp::min(buf.len(), bytes_per_cluster);
        dev.read_sectors(sector, self.sec_per_clus as u64, &mut buf[..count])
            .map_err(|_| ())
    }

    fn read_dir_entries(&mut self, cluster: u32, dev: &mut dyn BlockDevice) -> Result<Vec<Fat32File>, ()> {
        let mut entries = Vec::new();
        let clus_bytes = (self.sec_per_clus as usize) * (self.bytes_per_sec as usize);
        let mut buf = alloc::vec![0u8; clus_bytes];

        let mut current = cluster;
        loop {
            self.read_cluster(current, dev, &mut buf)?;

            let mut offset = 0;
            while offset + 32 <= clus_bytes {
                let entry_type = buf[offset + 11];
                if entry_type == ATTR_LONG_NAME {
                    offset += 32;
                    continue;
                }
                let first_byte = buf[offset];
                if first_byte == 0x00 {
                    break;
                }
                if first_byte == 0xE5 {
                    offset += 32;
                    continue;
                }
                if entry_type & ATTR_VOLUME_ID != 0 {
                    offset += 32;
                    continue;
                }

                let mut name_buf = [0u8; 256];
                let mut name_len = 0usize;

                let short_name = &buf[offset..offset + 11];
                for &c in short_name.iter() {
                    if c == b' ' { break; }
                    if name_len < 255 { name_buf[name_len] = c; name_len += 1; }
                }
                if name_len < 255 && short_name[0] != b' ' {
                    name_buf[name_len] = b'.';
                    name_len += 1;
                    let mut ext_start = false;
                    for &c in short_name[8..11].iter() {
                        if c == b' ' { continue; }
                        ext_start = true;
                        if name_len < 255 { name_buf[name_len] = c; name_len += 1; }
                    }
                    if !ext_start && name_len > 0 && name_buf[name_len - 1] == b'.' {
                        name_len -= 1;
                    }
                }

                let cluster_lo = u16::from_le_bytes([buf[offset + 26], buf[offset + 27]]);
                let cluster_hi = u16::from_le_bytes([buf[offset + 20], buf[offset + 21]]);
                let first_cluster = ((cluster_hi as u32) << 16) | (cluster_lo as u32);
                let size = u32::from_le_bytes([
                    buf[offset + 28], buf[offset + 29], buf[offset + 30], buf[offset + 31],
                ]);

                entries.push(Fat32File {
                    name: name_buf,
                    name_len,
                    size,
                    is_dir: (entry_type & ATTR_DIRECTORY) != 0,
                    first_cluster,
                    attributes: entry_type,
                });

                offset += 32;
            }

            let next = self.read_fat_entry(current, dev)?;
            if next >= FAT32_EOC {
                break;
            }
            current = next;
        }

        Ok(entries)
    }

    pub fn root_entries(&mut self, dev: &mut dyn BlockDevice) -> Result<Vec<Fat32File>, ()> {
        self.read_dir_entries(self.root_clus, dev)
    }

    pub fn read_file(&mut self, file: &Fat32File, dev: &mut dyn BlockDevice) -> Result<Vec<u8>, ()> {
        if file.is_dir {
            return Err(());
        }
        let mut data = alloc::vec![0u8; file.size as usize];
        let clus_bytes = (self.sec_per_clus as usize) * (self.bytes_per_sec as usize);
        let mut offset = 0usize;
        let mut cluster = file.first_cluster;
        let mut tmp_buf = alloc::vec![0u8; clus_bytes];

        loop {
            self.read_cluster(cluster, dev, &mut tmp_buf)?;
            let remaining = file.size as usize - offset;
            let copy_size = core::cmp::min(remaining, clus_bytes);
            data[offset..offset + copy_size].copy_from_slice(&tmp_buf[..copy_size]);
            offset += copy_size;

            if offset >= file.size as usize {
                break;
            }

            let next = self.read_fat_entry(cluster, dev)?;
            if next >= FAT32_EOC {
                break;
            }
            cluster = next;
        }

        data.truncate(offset);
        Ok(data)
    }

    pub fn find_path(&mut self, path: &str, dev: &mut dyn BlockDevice) -> Result<Fat32File, ()> {
        let trimmed = path.trim_matches('/');
        if trimmed.is_empty() {
            return Err(());
        }

        let components: Vec<&str> = trimmed.split('/').collect();
        if components.len() > MAX_PATH_COMPONENTS {
            return Err(());
        }

        let mut entries = self.root_entries(dev)?;

        for (depth, &part) in components.iter().enumerate() {
            let is_last = depth == components.len() - 1;
            let mut found = false;

            for entry in entries.iter() {
                let entry_name = core::str::from_utf8(&entry.name[..entry.name_len])
                    .unwrap_or("").to_lowercase();
                let search = part.to_lowercase();

                if entry_name == search {
                    if is_last {
                        return Ok(Fat32File {
                            name: entry.name,
                            name_len: entry.name_len,
                            size: entry.size,
                            is_dir: entry.is_dir,
                            first_cluster: entry.first_cluster,
                            attributes: entry.attributes,
                        });
                    }
                    if entry.is_dir {
                        entries = self.read_dir_entries(entry.first_cluster, dev)?;
                        found = true;
                        break;
                    }
                    return Err(());
                }
            }

            if !found {
                return Err(());
            }
        }

        Err(())
    }
}
