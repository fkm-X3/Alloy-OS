//! Interrupt controller abstraction
//!
//! Moved from `hal/src/interrupt/mod.rs`; the HAL re-exports this
//! module.

#[cfg(feature = "x86_64")]
use crate::io::{IoPort, X86IoPort};

#[cfg(feature = "aarch64")]
use crate::io::{Mmio, DefaultMmio};

/// IRQ handler function type
pub type IrqHandler = fn(irq: u32);

/// Interrupt controller trait
pub trait InterruptController {
    /// Initialize the interrupt controller
    fn init(&mut self);

    /// Register an IRQ handler
    fn register_handler(&mut self, irq: u32, handler: IrqHandler);

    /// Enable a specific IRQ line
    fn enable_irq(&mut self, irq: u32);

    /// Disable a specific IRQ line
    fn disable_irq(&mut self, irq: u32);

    /// Send end-of-interrupt signal
    fn send_eoi(&mut self, irq: u32);

    /// Remap IRQs to the given base vector
    fn remap(&mut self, base: u32);
}

/// PIC (Programmable Interrupt Controller) for x86
#[cfg(feature = "x86_64")]
pub struct Pic8259 {
    pub master_base: u8,
    pub slave_base: u8,
}

#[cfg(feature = "x86_64")]
impl Pic8259 {
    pub const fn new(master_base: u8, slave_base: u8) -> Self {
        Self {
            master_base,
            slave_base,
        }
    }
}

#[cfg(feature = "x86_64")]
impl InterruptController for Pic8259 {
    fn init(&mut self) {
        // PIC initialization (ICW1-ICW4)
        unsafe {
            // ICW1: Start initialization
            <X86IoPort as IoPort>::outb(0x20, 0x11);
            <X86IoPort as IoPort>::outb(0xA0, 0x11);

            // ICW2: Set base vectors
            <X86IoPort as IoPort>::outb(0x21, self.master_base);
            <X86IoPort as IoPort>::outb(0xA1, self.slave_base);

            // ICW3: Tell master about slave at IRQ2
            <X86IoPort as IoPort>::outb(0x21, 0x04);
            // Tell slave its cascade identity
            <X86IoPort as IoPort>::outb(0xA1, 0x02);

            // ICW4: 8086 mode
            <X86IoPort as IoPort>::outb(0x21, 0x01);
            <X86IoPort as IoPort>::outb(0xA1, 0x01);

            // Mask all interrupts initially
            <X86IoPort as IoPort>::outb(0x21, 0xFF);
            <X86IoPort as IoPort>::outb(0xA1, 0xFF);
        }
    }

    fn register_handler(&mut self, _irq: u32, _handler: IrqHandler) {
        // Handlers are registered in the IDT
    }

    fn enable_irq(&mut self, irq: u32) {
        let port: u16 = if irq < 8 { 0x21 } else { 0xA1 };
        let mask: u8 = if irq < 8 {
            !(1 << irq)
        } else {
            !(1 << (irq - 8))
        };
        unsafe {
            let current = <X86IoPort as IoPort>::inb(port);
            <X86IoPort as IoPort>::outb(port, current & mask);
        }
    }

    fn disable_irq(&mut self, irq: u32) {
        let port: u16 = if irq < 8 { 0x21 } else { 0xA1 };
        let mask: u8 = if irq < 8 {
            1 << irq
        } else {
            1 << (irq - 8)
        };
        unsafe {
            let current = <X86IoPort as IoPort>::inb(port);
            <X86IoPort as IoPort>::outb(port, current | mask);
        }
    }

    fn send_eoi(&mut self, irq: u32) {
        if irq >= 8 {
            unsafe {
                <X86IoPort as IoPort>::outb(0xA0, 0x20);
            }
        }
        unsafe {
            <X86IoPort as IoPort>::outb(0x20, 0x20);
        }
    }

    fn remap(&mut self, base: u32) {
        self.master_base = base as u8;
        self.slave_base = base as u8 + 8;
    }
}

/// GIC (Generic Interrupt Controller) for ARM
#[cfg(feature = "aarch64")]
pub struct Gic {
    pub dist_base: u64,
    pub cpu_base: u64,
}

#[cfg(feature = "aarch64")]
impl Gic {
    pub const fn new(dist_base: u64, cpu_base: u64) -> Self {
        Self {
            dist_base,
            cpu_base,
        }
    }

    /// Default configuration for QEMU virt machine
    pub const fn qemu_virt() -> Self {
        Self {
            dist_base: 0x0800_0000,
            cpu_base: 0x0801_0000,
        }
    }
}

#[cfg(feature = "aarch64")]
impl InterruptController for Gic {
    fn init(&mut self) {
        // GICv2 initialization
        unsafe {
            // Enable GIC distributor
            DefaultMmio::write32((self.dist_base + 0x000) as usize, 0x01); // GICD_CTLR

            // Set priority for all SPIs (interrupts 32-1019)
            for i in (32..1020).step_by(4) {
                DefaultMmio::write32((self.dist_base + 0x400 + (i as u64)) as usize, 0xA0A0A0A0);
            }

            // Set all SPIs to group 0
            for i in (32..1020).step_by(32) {
                DefaultMmio::write32((self.dist_base + 0x080 + (i as u64)) as usize, 0x00000000);
            }

            // Enable all SPIs
            for i in (32..1020).step_by(32) {
                DefaultMmio::write32((self.dist_base + 0x100 + (i as u64)) as usize, 0xFFFFFFFF);
            }

            // Enable CPU interface
            DefaultMmio::write32((self.cpu_base + 0x000) as usize, 0x01); // GICC_CTLR
            // Set priority mask to allow all interrupts
            DefaultMmio::write32((self.cpu_base + 0x004) as usize, 0xFF); // GICC_PMR
        }
    }

    fn register_handler(&mut self, _irq: u32, _handler: IrqHandler) {
        // Handlers are registered in the exception vector table
    }

    fn enable_irq(&mut self, irq: u32) {
        let reg = (irq / 32) * 4;
        let bit = 1 << (irq % 32);
        unsafe {
            DefaultMmio::write32((self.dist_base + 0x100 + reg as u64) as usize, bit);
        }
    }

    fn disable_irq(&mut self, irq: u32) {
        let reg = (irq / 32) * 4;
        let bit = 1 << (irq % 32);
        unsafe {
            DefaultMmio::write32((self.dist_base + 0x180 + reg as u64) as usize, bit);
        }
    }

    fn send_eoi(&mut self, _irq: u32) {
        // Write to GICC_EOIR
        unsafe {
            DefaultMmio::write32((self.cpu_base + 0x010) as usize, _irq);
        }
    }

    fn remap(&mut self, _base: u32) {
        // GIC doesn't need remapping like PIC
    }
}
