#include "timer.h"
#include "serial.h"

volatile uint64_t g_timer_ticks = 0;
static uint32_t g_timer_frequency = 0;

#if defined(ARCH_I686) || defined(ARCH_X86_64)
static inline void outb(uint16_t port, uint8_t value) {
    asm volatile ("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t value;
    asm volatile ("inb %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

void timer_init_ffi(uint32_t frequency) {
    serial_print("[Timer] Initializing PIT timer\n");

    g_timer_frequency = frequency;

    uint32_t divisor = PIT_BASE_FREQ / frequency;

    if (divisor > 65535) {
        divisor = 65535;
    }

    outb(PIT_COMMAND, PIT_CMD_INIT);

    outb(PIT_CHANNEL_0, (uint8_t)(divisor & 0xFF));
    outb(PIT_CHANNEL_0, (uint8_t)((divisor >> 8) & 0xFF));

    serial_print("[Timer] PIT initialized\n");
}

void timer_handler() {
    g_timer_ticks++;
    rust_timer_tick();
}
#elif defined(ARCH_AARCH64)
static uint64_t g_timer_freq_hz = 0;

static inline uint64_t read_cntfrq_el0() {
    uint64_t val;
    asm volatile("mrs %0, S3_3_C14_C0_0" : "=r"(val));
    return val;
}

static inline uint64_t read_cntpct_el0() {
    uint64_t val;
    asm volatile("mrs %0, S3_3_C14_C0_1" : "=r"(val));
    return val;
}

static inline void write_cntp_cval_el1(uint64_t val) {
    asm volatile("msr S3_3_C14_C2_2, %0" : : "r"(val));
}

static inline void write_cntp_ctl_el1(uint64_t val) {
    asm volatile("msr S3_3_C14_C2_1, %0" : : "r"(val));
}

// GICv2 MMIO base for QEMU virt
#define GICD_BASE  ((volatile uint32_t*)0x08000000)
#define GICC_BASE  ((volatile uint32_t*)0x08010000)

#define GICD_CTLR       0x000
#define GICD_ISENABLER  0x100
#define GICD_IPRIORITYR 0x400
#define GICC_CTLR       0x0000
#define GICC_PMR        0x0004
#define GICC_EOIR       0x0010

#define GIC_PPI_PHYS_TIMER 30  // Physical timer PPI #14 → GIC ID 30

void gic_init() {
    // Disable distributor during config
    GICD_BASE[GICD_CTLR / 4] = 0;

    // Set priority for timer PPI (non-secure group 1, priority 0x80)
    GICD_BASE[(GICD_IPRIORITYR + (GIC_PPI_PHYS_TIMER / 4) * 4) / 4] = 0x80808080;

    // Enable timer PPI
    GICD_BASE[(GICD_ISENABLER + (GIC_PPI_PHYS_TIMER / 32) * 4) / 4] = 1 << (GIC_PPI_PHYS_TIMER % 32);

    // Enable distributor
    GICD_BASE[GICD_CTLR / 4] = 1;

    // CPU interface: enable, set priority mask to allow all
    GICC_BASE[GICC_CTLR / 4] = 1;
    GICC_BASE[GICC_PMR / 4] = 0xFF;

    serial_print("[Timer] GICv2 initialized\n");
}

void timer_init_ffi(uint32_t frequency) {
    serial_print("[Timer] Initializing ARM Generic Timer\n");

    g_timer_frequency = frequency;
    g_timer_freq_hz = read_cntfrq_el0();

    if (g_timer_freq_hz == 0) {
        g_timer_freq_hz = 62500000;  // QEMU default
    }

    serial_print("[Timer] System counter frequency: ");
    serial_print_hex((uint32_t)g_timer_freq_hz);
    serial_print("\n");

    // Initialize GIC
    gic_init();

    // Set timer to fire every (freq_hz / frequency) ticks
    uint64_t period = g_timer_freq_hz / frequency;
    uint64_t now = read_cntpct_el0();
    write_cntp_cval_el1(now + period);

    // Enable timer (bit 0), mask interrupt (bit 1) - unmask for IRQ generation
    write_cntp_ctl_el1(1);

    serial_print("[Timer] ARM Generic Timer initialized\n");
}

void timer_handler() {
    // Acknowledge interrupt FIRST (write GICC_EOIR with interrupt ID).
    // Doing this before any work prevents the IRQ line staying asserted and
    // re-entering the handler while we run (e.g. while a lock is held).
    GICC_BASE[GICC_EOIR / 4] = GIC_PPI_PHYS_TIMER;

    // Reload timer for next period
    uint64_t period = g_timer_freq_hz / g_timer_frequency;
    uint64_t now = read_cntpct_el0();
    write_cntp_cval_el1(now + period);

    g_timer_ticks++;
    rust_timer_tick();
}
#else
#error "Unsupported architecture for timer"
#endif

uint64_t timer_get_ticks_ffi() {
    return g_timer_ticks;
}

uint64_t timer_get_uptime_ms_ffi() {
#if defined(ARCH_I686) || defined(ARCH_X86_64)
    if (g_timer_frequency == 0) {
        return 0;
    }
    return (g_timer_ticks * 1000) / g_timer_frequency;
#elif defined(ARCH_AARCH64)
    if (g_timer_freq_hz == 0) {
        return 0;
    }
    return (read_cntpct_el0() * 1000) / g_timer_freq_hz;
#endif
}

uint32_t timer_get_frequency_ffi() {
    return g_timer_frequency;
}