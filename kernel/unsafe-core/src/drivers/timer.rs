//! Safe system-timer driver: PIT on x86_64, ARM Generic Timer + GICv2 on
//! aarch64.
//!
//! Replaces `ported/x86_64/drivers/timer.rs` and
//! `ported/aarch64/drivers/timer.rs`. The C-ABI entry points
//! (`timer_init_ffi`, `timer_handler`, `timer_get_*_ffi`, `gic_init`) are
//! kept because surviving ported modules (idt.rs on both arches, the aarch64
//! boot main) still call them by symbol. The serial markers printed during
//! init match the C driver exactly (boot gate depends on them).

use crate::drivers::serial::Serial;

#[cfg(feature = "x86_64")]
use crate::raw::asm::x86_64::outb;

#[cfg(feature = "aarch64")]
use crate::io::{DefaultMmio, Mmio};
#[cfg(feature = "aarch64")]
use crate::raw::asm::aarch64::{read_cntfrq_el0, read_cntpct_el0, write_timer_ctl, write_timer_cval};

/// Total PIT ticks since boot (IRQ-incremented). Kept `#[no_mangle]` to
/// preserve the C symbol surface for compat.
#[no_mangle]
pub static mut g_timer_ticks: u64 = 0;

/// Programmed timer frequency (Hz). The aarch64 path also caches the system
/// counter frequency separately (see `g_timer_freq_hz`).
static mut g_timer_frequency: u32 = 0;

#[cfg(feature = "aarch64")]
static mut g_timer_freq_hz: u64 = 0;

#[cfg(feature = "x86_64")]
const PIT_BASE_FREQ: u32 = 1193180;

/// PIT command: channel 0, RW both, mode 3 (square wave), binary = 0x36.
#[cfg(feature = "x86_64")]
const PIT_CMD_INIT: u8 = 0x36;

// --- aarch64 GICv2 (moved verbatim from the ported timer) ---

#[cfg(feature = "aarch64")]
const GICD_BASE: usize = 0x0800_0000;
#[cfg(feature = "aarch64")]
const GICC_BASE: usize = 0x0801_0000;
#[cfg(feature = "aarch64")]
const GICD_CTLR: usize = 0x000;
#[cfg(feature = "aarch64")]
const GICD_ISENABLER: usize = 0x100;
#[cfg(feature = "aarch64")]
const GICD_IPRIORITYR: usize = 0x400;
#[cfg(feature = "aarch64")]
const GICC_CTLR: usize = 0x000;
#[cfg(feature = "aarch64")]
const GICC_PMR: usize = 0x004;
#[cfg(feature = "aarch64")]
const GICC_EOIR: usize = 0x010;
#[cfg(feature = "aarch64")]
const GIC_PPI_PHYS_TIMER: u32 = 30;

#[cfg(feature = "aarch64")]
#[inline]
fn gic_write(base: usize, offset: usize, value: u32) {
    unsafe { <DefaultMmio as Mmio>::write32(base + offset, value) };
}

#[cfg(feature = "aarch64")]
#[inline]
fn gic_read(base: usize, offset: usize) -> u32 {
    unsafe { <DefaultMmio as Mmio>::read32(base + offset) }
}

/// Safe system-timer facade.
///
/// Mirrors the C driver exactly: `init` programs the PIT (x86_64) or the ARM
/// generic timer + GICv2 (aarch64), `ticks`/`uptime_ms`/`frequency` read the
/// current state, and the IRQ handler increments the tick counter.
pub struct SystemTimer;

impl SystemTimer {
    /// Initialize the timer at `frequency` Hz. Idempotent-safe only if called
    /// before interrupts for that timer are enabled; matches the C call
    /// sites (once at boot).
    pub fn init(frequency: u32) {
        #[cfg(feature = "x86_64")]
        {
            Serial::write_str("[Timer] Initializing PIT timer\n");
            unsafe {
                g_timer_frequency = frequency;
            }
            let mut divisor = PIT_BASE_FREQ / frequency;
            if divisor > 65535 {
                divisor = 65535;
            }
            unsafe {
                outb(0x43, PIT_CMD_INIT);
                outb(0x40, (divisor & 0xff) as u8);
                outb(0x40, ((divisor >> 8) & 0xff) as u8);
            }
            Serial::write_str("[Timer] PIT initialized\n");
        }

        #[cfg(feature = "aarch64")]
        {
            Serial::write_str("[Timer] Initializing ARM Generic Timer\n");
            unsafe {
                g_timer_frequency = frequency;
            }
            let mut freq_hz = read_cntfrq_el0();
            if freq_hz == 0 {
                freq_hz = 62_500_000;
            }
            unsafe {
                g_timer_freq_hz = freq_hz;
            }
            Serial::write_str("[Timer] System counter frequency: ");
            Serial::write_hex(freq_hz as u32);
            Serial::write_str("\n");
            gic_init();
            let period = freq_hz / frequency as u64;
            let now = read_cntpct_el0();
            write_timer_cval(now.wrapping_add(period));
            write_timer_ctl(1);
            Serial::write_str("[Timer] ARM Generic Timer initialized\n");
        }
    }

    /// Number of timer IRQs since boot.
    pub fn ticks() -> u64 {
        unsafe { g_timer_ticks }
    }

    /// System uptime in milliseconds.
    pub fn uptime_ms() -> u64 {
        #[cfg(feature = "x86_64")]
        {
            let freq = unsafe { g_timer_frequency };
            if freq == 0 {
                return 0;
            }
            unsafe { g_timer_ticks }.wrapping_mul(1000).wrapping_div(freq as u64)
        }
        #[cfg(feature = "aarch64")]
        {
            let freq_hz = unsafe { g_timer_freq_hz };
            if freq_hz == 0 {
                return 0;
            }
            read_cntpct_el0()
                .wrapping_mul(1000)
                .wrapping_div(freq_hz)
        }
    }

    /// Current system time in milliseconds (same as [`uptime_ms`](Self::uptime_ms)).
    pub fn now_ms() -> u64 {
        Self::uptime_ms()
    }

    /// The programmed timer frequency in Hz.
    pub fn frequency() -> u32 {
        unsafe { g_timer_frequency }
    }
}

// ============================================================================
// C-ABI entry points kept for surviving ported callers.
// ============================================================================

#[cfg(feature = "aarch64")]
/// `gic_init()`: enable GICv2 distributor + CPU interface and unmask the
/// physical timer PPI (30).
#[no_mangle]
pub extern "C" fn gic_init() {
    gic_write(GICD_BASE, GICD_CTLR, 0);
    gic_write(
        GICD_BASE,
        GICD_IPRIORITYR + (GIC_PPI_PHYS_TIMER as usize / 4) * 4,
        0x8080_8080,
    );
    gic_write(
        GICD_BASE,
        GICD_ISENABLER + (GIC_PPI_PHYS_TIMER as usize / 32) * 4,
        1 << (GIC_PPI_PHYS_TIMER % 32),
    );
    gic_write(GICD_BASE, GICD_CTLR, 1);
    gic_write(GICC_BASE, GICC_CTLR, 1);
    gic_write(GICC_BASE, GICC_PMR, 0xff);
    Serial::write_str("[Timer] GICv2 initialized\n");
}

/// `timer_init_ffi(frequency)`: program the timer at `frequency` Hz.
#[no_mangle]
pub extern "C" fn timer_init_ffi(frequency: u32) {
    SystemTimer::init(frequency);
}

/// `timer_handler()`: timer IRQ handler — ack, re-arm, count, notify the
/// kernel scheduler.
#[no_mangle]
pub extern "C" fn timer_handler() {
    #[cfg(feature = "aarch64")]
    {
        // EOI the physical timer PPI.
        gic_write(GICC_BASE, GICC_EOIR, GIC_PPI_PHYS_TIMER);
        let freq_hz = unsafe { g_timer_freq_hz };
        let freq = unsafe { g_timer_frequency };
        let period = freq_hz.wrapping_div(freq as u64);
        let now = read_cntpct_el0();
        write_timer_cval(now.wrapping_add(period));
    }
    unsafe {
        g_timer_ticks = g_timer_ticks.wrapping_add(1);
    }
    // [T9] Session 0.1 diagnostic: ground-truth proof that PIT IRQs are
    // (still) arriving. Rate-limited to every 256th tick.
    #[cfg(feature = "x86_64")]
    unsafe {
        if g_timer_ticks % 50 == 0 {
            Serial::write_str("[T9-TICK ");
            Serial::write_hex64(g_timer_ticks);
            Serial::write_str("]\n");
        }
    }
    // The kernel registered its timer-tick handler via
    // `api::callback::set_timer_tick_handler` at boot; unsafe-core never
    // calls `rust_timer_tick` by symbol.
    crate::api::callback::invoke_timer_tick();
}

/// `timer_get_ticks_ffi()`: total timer IRQs since boot.
#[no_mangle]
pub extern "C" fn timer_get_ticks_ffi() -> u64 {
    SystemTimer::ticks()
}

/// `timer_get_uptime_ms_ffi()`: system uptime in milliseconds.
#[no_mangle]
pub extern "C" fn timer_get_uptime_ms_ffi() -> u64 {
    SystemTimer::uptime_ms()
}

/// `timer_get_frequency_ffi()`: programmed timer frequency in Hz.
#[no_mangle]
pub extern "C" fn timer_get_frequency_ffi() -> u32 {
    SystemTimer::frequency()
}
