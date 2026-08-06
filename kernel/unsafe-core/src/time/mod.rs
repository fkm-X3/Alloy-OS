//! Timer abstraction
//!
//! Moved from `hal/src/time/mod.rs` in Phase 1; the HAL re-exports this
//! module.

#[cfg(feature = "x86_64")]
use crate::io::{IoPort, X86IoPort};

#[cfg(feature = "x86_64")]
use crate::raw::asm::x86_64::hlt;

#[cfg(feature = "aarch64")]
use crate::raw::asm::aarch64::{wfi, write_timer_ctl, write_timer_cval};

/// Timer trait
pub trait Timer {
    /// Initialize the timer with a frequency
    fn init(&mut self, frequency: u32);

    /// Get the number of ticks since boot
    fn ticks(&self) -> u64;

    /// Get system uptime in milliseconds
    fn uptime_ms(&self) -> u64;

    /// Get the timer frequency
    fn frequency(&self) -> u32;

    /// Sleep for the specified number of milliseconds
    fn sleep_ms(&mut self, ms: u64);
}

/// PIT (Programmable Interval Timer) for x86
#[cfg(feature = "x86_64")]
pub struct Pit {
    pub frequency: u32,
    pub ticks: u64,
}

#[cfg(feature = "x86_64")]
impl Pit {
    pub const fn new() -> Self {
        Self {
            frequency: 0,
            ticks: 0,
        }
    }

    pub fn increment_tick(&mut self) {
        self.ticks += 1;
    }
}

#[cfg(feature = "x86_64")]
impl Timer for Pit {
    fn init(&mut self, frequency: u32) {
        self.frequency = frequency;
        self.ticks = 0;

        // Calculate divisor
        let pit_base_freq = 1193182;
        let divisor = (pit_base_freq / frequency) as u16;

        // Send command byte
        unsafe {
            <X86IoPort as IoPort>::outb(0x43, 0x36);

            // Send divisor
            let low = (divisor & 0xFF) as u8;
            let high = ((divisor >> 8) & 0xFF) as u8;
            <X86IoPort as IoPort>::outb(0x40, low);
            <X86IoPort as IoPort>::outb(0x40, high);
        }
    }

    fn ticks(&self) -> u64 {
        self.ticks
    }

    fn uptime_ms(&self) -> u64 {
        (self.ticks * 1000) / self.frequency as u64
    }

    fn frequency(&self) -> u32 {
        self.frequency
    }

    fn sleep_ms(&mut self, ms: u64) {
        let target_ticks = (ms * self.frequency as u64) / 1000;
        let start_ticks = self.ticks;
        while self.ticks - start_ticks < target_ticks {
            hlt();
        }
    }
}

/// ARM Generic Timer (architected timer)
#[cfg(feature = "aarch64")]
pub struct ArmGenericTimer {
    pub frequency: u32,
}

#[cfg(feature = "aarch64")]
impl ArmGenericTimer {
    pub const fn new() -> Self {
        Self { frequency: 0 }
    }
}

#[cfg(feature = "aarch64")]
impl Timer for ArmGenericTimer {
    fn init(&mut self, _frequency: u32) {
        // Read counter frequency from CNTFRQ_EL0
        self.frequency = crate::raw::asm::aarch64::read_cntfrq_el0() as u32;
    }

    fn ticks(&self) -> u64 {
        crate::raw::asm::aarch64::read_cntpct_el0()
    }

    fn uptime_ms(&self) -> u64 {
        (self.ticks() * 1000) / self.frequency as u64
    }

    fn frequency(&self) -> u32 {
        self.frequency
    }

    fn sleep_ms(&mut self, ms: u64) {
        let target_ticks = (ms * self.frequency as u64) / 1000;
        let start_ticks = self.ticks();
        let compare = start_ticks + target_ticks;

        // Set compare value and enable timer
        write_timer_cval(target_ticks);
        write_timer_ctl(1);

        // Wait until timer fires
        while self.ticks() < compare {
            wfi();
        }

        write_timer_ctl(0);
    }
}
