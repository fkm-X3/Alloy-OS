//! Timer abstraction

#[cfg(any(feature = "i686", feature = "x86_64"))]
use crate::io::{IoPort, X86IoPort};

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
#[cfg(any(feature = "i686", feature = "x86_64"))]
pub struct Pit {
    pub frequency: u32,
    pub ticks: u64,
}

#[cfg(any(feature = "i686", feature = "x86_64"))]
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

#[cfg(any(feature = "i686", feature = "x86_64"))]
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
            #[cfg(any(feature = "i686", feature = "x86_64"))]
            unsafe {
                core::arch::asm!("hlt");
            }
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
        let freq: u64;
        unsafe {
            core::arch::asm!("mrs {}, S3_3_C14_C0_0", out(reg) freq);
        }
        self.frequency = freq as u32;
    }

    fn ticks(&self) -> u64 {
        let count: u64;
        unsafe {
            core::arch::asm!("mrs {}, S3_3_C14_C0_1", out(reg) count);
        }
        count
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

        unsafe {
            // Set compare value and enable timer
            core::arch::asm!("msr S3_3_C14_C2_0, {}", in(reg) target_ticks);
            core::arch::asm!("msr S3_3_C14_C2_1, {}", in(reg) 1);

            // Wait until timer fires
            while self.ticks() < compare {
                core::arch::asm!("wfi");
            }

            core::arch::asm!("msr S3_3_C14_C2_1, {}", in(reg) 0);
        }
    }
}
