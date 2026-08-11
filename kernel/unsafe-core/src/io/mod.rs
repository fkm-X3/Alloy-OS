//! I/O abstraction layer
//!
//! Provides unified interface for port I/O (x86) and MMIO (ARM).
//! Moved from `hal/src/io/mod.rs`; the HAL re-exports this module.

#[cfg(feature = "x86_64")]
use crate::raw::asm;

/// Port I/O trait for architectures that support it (x86)
#[cfg(feature = "x86_64")]
pub trait IoPort {
    unsafe fn outb(port: u16, value: u8);
    unsafe fn outw(port: u16, value: u16);
    unsafe fn outl(port: u16, value: u32);
    unsafe fn inb(port: u16) -> u8;
    unsafe fn inw(port: u16) -> u16;
    unsafe fn inl(port: u16) -> u32;
}

/// x86 port I/O implementation
#[cfg(feature = "x86_64")]
pub struct X86IoPort;

#[cfg(feature = "x86_64")]
impl IoPort for X86IoPort {
    #[inline]
    unsafe fn outb(port: u16, value: u8) {
        asm::x86_64::outb(port, value);
    }

    #[inline]
    unsafe fn outw(port: u16, value: u16) {
        asm::x86_64::outw(port, value);
    }

    #[inline]
    unsafe fn outl(port: u16, value: u32) {
        asm::x86_64::outl(port, value);
    }

    #[inline]
    unsafe fn inb(port: u16) -> u8 {
        asm::x86_64::inb(port)
    }

    #[inline]
    unsafe fn inw(port: u16) -> u16 {
        asm::x86_64::inw(port)
    }

    #[inline]
    unsafe fn inl(port: u16) -> u32 {
        asm::x86_64::inl(port)
    }
}

/// Memory-mapped I/O trait (all architectures)
pub trait Mmio {
    unsafe fn read8(addr: usize) -> u8;
    unsafe fn read16(addr: usize) -> u16;
    unsafe fn read32(addr: usize) -> u32;
    unsafe fn read64(addr: usize) -> u64;
    unsafe fn write8(addr: usize, value: u8);
    unsafe fn write16(addr: usize, value: u16);
    unsafe fn write32(addr: usize, value: u32);
    unsafe fn write64(addr: usize, value: u64);
}

/// Default MMIO implementation using volatile pointers
pub struct DefaultMmio;

impl Mmio for DefaultMmio {
    #[inline]
    unsafe fn read8(addr: usize) -> u8 {
        core::ptr::read_volatile(addr as *const u8)
    }

    #[inline]
    unsafe fn read16(addr: usize) -> u16 {
        core::ptr::read_volatile(addr as *const u16)
    }

    #[inline]
    unsafe fn read32(addr: usize) -> u32 {
        core::ptr::read_volatile(addr as *const u32)
    }

    #[inline]
    unsafe fn read64(addr: usize) -> u64 {
        core::ptr::read_volatile(addr as *const u64)
    }

    #[inline]
    unsafe fn write8(addr: usize, value: u8) {
        core::ptr::write_volatile(addr as *mut u8, value)
    }

    #[inline]
    unsafe fn write16(addr: usize, value: u16) {
        core::ptr::write_volatile(addr as *mut u16, value)
    }

    #[inline]
    unsafe fn write32(addr: usize, value: u32) {
        core::ptr::write_volatile(addr as *mut u32, value)
    }

    #[inline]
    unsafe fn write64(addr: usize, value: u64) {
        core::ptr::write_volatile(addr as *mut u64, value)
    }
}

/// MMIO register helper for structured access
pub struct MmioReg<const ADDR: usize>;

impl<const ADDR: usize> MmioReg<ADDR> {
    #[inline]
    pub unsafe fn read32() -> u32 {
        DefaultMmio::read32(ADDR)
    }

    #[inline]
    pub unsafe fn write32(value: u32) {
        DefaultMmio::write32(ADDR, value)
    }

    #[inline]
    pub unsafe fn read64() -> u64 {
        DefaultMmio::read64(ADDR)
    }

    #[inline]
    pub unsafe fn write64(value: u64) {
        DefaultMmio::write64(ADDR, value)
    }

    #[inline]
    pub unsafe fn set_bits(mask: u32) {
        let val = Self::read32();
        Self::write32(val | mask);
    }

    #[inline]
    pub unsafe fn clear_bits(mask: u32) {
        let val = Self::read32();
        Self::write32(val & !mask);
    }

    #[inline]
    pub unsafe fn write_masked(value: u32, mask: u32) {
        let old = Self::read32();
        Self::write32((old & !mask) | (value & mask));
    }
}
