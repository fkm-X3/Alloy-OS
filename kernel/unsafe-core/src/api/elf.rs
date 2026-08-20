//! Safe ELF header and program-header parsing from a byte slice.
//!
//! All pointer casts and alignment assumptions are encapsulated here so the
//! kernel crate can parse ELF images with `#![forbid(unsafe_code)]`.

/// Parsed ELF32 executable header fields (offsets derived from the ELF spec).
#[derive(Debug, Clone, Copy)]
pub struct Elf32Header {
    pub e_type: u16,
    pub e_machine: u16,
    pub e_entry: u32,
    pub e_phoff: u32,
    pub e_shoff: u32,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// Parsed ELF32 program header.
#[derive(Debug, Clone, Copy)]
pub struct Elf32Phdr {
    pub p_type: u32,
    pub p_offset: u32,
    pub p_vaddr: u32,
    pub p_paddr: u32,
    pub p_filesz: u32,
    pub p_memsz: u32,
    pub p_flags: u32,
    pub p_align: u32,
}

/// Parsed ELF64 executable header fields.
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    pub e_type: u16,
    pub e_machine: u16,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// Parsed ELF64 program header.
#[derive(Debug, Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

// ---------------------------------------------------------------------------
// Safe byte-level readers (no pointer casts).
// ---------------------------------------------------------------------------

fn read_u16_le(data: &[u8], off: usize) -> Option<u16> {
    if off + 2 > data.len() { return None; }
    Some(u16::from_le_bytes([data[off], data[off + 1]]))
}

fn read_u32_le(data: &[u8], off: usize) -> Option<u32> {
    if off + 4 > data.len() { return None; }
    Some(u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]))
}

fn read_u64_le(data: &[u8], off: usize) -> Option<u64> {
    if off + 8 > data.len() { return None; }
    Some(u64::from_le_bytes([
        data[off], data[off + 1], data[off + 2], data[off + 3],
        data[off + 4], data[off + 5], data[off + 6], data[off + 7],
    ]))
}

fn is_elf(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0x7f && data[1] == b'E' && data[2] == b'L' && data[3] == b'F'
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse an ELF32 header from `image`. Returns `None` if the data is too
/// short or the magic/class bytes don't match.
pub fn parse_elf32_header(image: &[u8]) -> Option<Elf32Header> {
    if !is_elf(image) || image.len() < 52 { return None; }
    if image[4] != 1 { return None; } // ELFCLASS32
    Some(Elf32Header {
        e_type:       read_u16_le(image, 16)?,
        e_machine:    read_u16_le(image, 18)?,
        e_entry:      read_u32_le(image, 24)?,
        e_phoff:      read_u32_le(image, 28)?,
        e_shoff:      read_u32_le(image, 32)?,
        e_flags:      read_u32_le(image, 36)?,
        e_ehsize:     read_u16_le(image, 40)?,
        e_phentsize:  read_u16_le(image, 42)?,
        e_phnum:      read_u16_le(image, 44)?,
        e_shentsize:  read_u16_le(image, 46)?,
        e_shnum:      read_u16_le(image, 48)?,
        e_shstrndx:   read_u16_le(image, 50)?,
    })
}

/// Parse an ELF32 program header at byte offset `phoff` within `image`.
pub fn parse_elf32_phdr(image: &[u8], phoff: usize) -> Option<Elf32Phdr> {
    if phoff + 32 > image.len() { return None; }
    Some(Elf32Phdr {
        p_type:   read_u32_le(image, phoff)?,
        p_offset: read_u32_le(image, phoff + 4)?,
        p_vaddr:  read_u32_le(image, phoff + 8)?,
        p_paddr:  read_u32_le(image, phoff + 12)?,
        p_filesz: read_u32_le(image, phoff + 16)?,
        p_memsz:  read_u32_le(image, phoff + 20)?,
        p_flags:  read_u32_le(image, phoff + 24)?,
        p_align:  read_u32_le(image, phoff + 28)?,
    })
}

/// Parse an ELF64 header from `image`.
pub fn parse_elf64_header(image: &[u8]) -> Option<Elf64Header> {
    if !is_elf(image) || image.len() < 64 { return None; }
    if image[4] != 2 { return None; } // ELFCLASS64
    Some(Elf64Header {
        e_type:       read_u16_le(image, 16)?,
        e_machine:    read_u16_le(image, 18)?,
        e_entry:      read_u64_le(image, 24)?,
        e_phoff:      read_u64_le(image, 32)?,
        e_shoff:      read_u64_le(image, 40)?,
        e_flags:      read_u32_le(image, 48)?,
        e_ehsize:     read_u16_le(image, 52)?,
        e_phentsize:  read_u16_le(image, 54)?,
        e_phnum:      read_u16_le(image, 56)?,
        e_shentsize:  read_u16_le(image, 58)?,
        e_shnum:      read_u16_le(image, 60)?,
        e_shstrndx:   read_u16_le(image, 62)?,
    })
}

/// Parse an ELF64 program header at byte offset `phoff` within `image`.
pub fn parse_elf64_phdr(image: &[u8], phoff: usize) -> Option<Elf64Phdr> {
    if phoff + 56 > image.len() { return None; }
    Some(Elf64Phdr {
        p_type:   read_u32_le(image, phoff)?,
        p_flags:  read_u32_le(image, phoff + 4)?,
        p_offset: read_u64_le(image, phoff + 8)?,
        p_vaddr:  read_u64_le(image, phoff + 16)?,
        p_paddr:  read_u64_le(image, phoff + 24)?,
        p_filesz: read_u64_le(image, phoff + 32)?,
        p_memsz:  read_u64_le(image, phoff + 40)?,
        p_align:  read_u64_le(image, phoff + 48)?,
    })
}

/// Detect the ELF class (1 = 32-bit, 2 = 64-bit) or `None` if invalid.
pub fn elf_class(image: &[u8]) -> Option<u8> {
    if !is_elf(image) || image.len() < 5 { return None; }
    let class = image[4];
    if class == 1 || class == 2 { Some(class) } else { None }
}
