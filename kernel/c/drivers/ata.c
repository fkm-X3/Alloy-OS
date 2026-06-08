#include "ata.h"
#include "serial.h"

#define ATA_DATA_PORT(base)         (base)
#define ATA_FEATURES_PORT(base)     (base + 1)
#define ATA_SECTOR_COUNT_PORT(base) (base + 2)
#define ATA_LBA_LO_PORT(base)       (base + 3)
#define ATA_LBA_MID_PORT(base)      (base + 4)
#define ATA_LBA_HI_PORT(base)       (base + 5)
#define ATA_DRIVE_PORT(base)        (base + 6)
#define ATA_COMMAND_PORT(base)      (base + 7)
#define ATA_CONTROL_PORT(base)      (base + 0x206)

#define ATA_PRIMARY_IO   0x1F0
#define ATA_PRIMARY_CTRL 0x3F6
#define ATA_SECONDARY_IO 0x170
#define ATA_SECONDARY_CTRL 0x376

static const uint16_t ata_io_bases[2]    = { ATA_PRIMARY_IO, ATA_SECONDARY_IO };
static const uint16_t ata_ctrl_bases[2]  = { ATA_PRIMARY_CTRL, ATA_SECONDARY_CTRL };

static struct ata_drive_info g_drives[2][2];
static int g_ata_initialized = 0;

static inline void outb(uint16_t port, uint8_t value) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline void outw(uint16_t port, uint16_t value) {
    asm volatile("outw %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint16_t inw(uint16_t port) {
    uint16_t ret;
    asm volatile("inw %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static void ata_wait(uint16_t io_base) {
    for (int i = 0; i < 4; i++) {
        inb(ata_ctrl_bases[io_base == ATA_PRIMARY_IO ? 0 : 1]);
    }
}

static int ata_busy_wait(uint16_t io_base, int timeout_ms) {
    for (int i = 0; i < timeout_ms * 1000; i++) {
        uint8_t status = inb(ATA_COMMAND_PORT(io_base));
        if (!(status & ATA_STATUS_BSY)) {
            return 1;
        }
        ata_wait(io_base);
    }
    return 0;
}

static int ata_drq_wait(uint16_t io_base, int timeout_ms) {
    for (int i = 0; i < timeout_ms * 1000; i++) {
        uint8_t status = inb(ATA_COMMAND_PORT(io_base));
        if (status & ATA_STATUS_ERR) {
            return -1;
        }
        if (status & ATA_STATUS_DRQ) {
            return 1;
        }
        if (!(status & ATA_STATUS_BSY)) {
            return 0;
        }
        ata_wait(io_base);
    }
    return -2;
}

static void ata_soft_reset(uint8_t bus) {
    uint16_t ctrl = ata_ctrl_bases[bus];
    outb(ctrl, 0x04);
    ata_wait(ata_io_bases[bus]);
    outb(ctrl, 0x00);
    ata_wait(ata_io_bases[bus]);
}

static void ata_words_to_sectors(struct ata_drive_info* info, uint16_t* buf) {
    if (info->command_sets & (1 << 26)) {
        uint64_t lo = (uint64_t)(uint32_t)buf[100] | ((uint64_t)(uint32_t)buf[101] << 16);
        uint64_t hi = (uint64_t)(uint32_t)buf[102] | ((uint64_t)(uint32_t)buf[103] << 16);
        info->num_sectors = lo | (hi << 32);
    } else {
        info->num_sectors = (uint32_t)buf[60] | ((uint32_t)buf[61] << 16);
    }
}

static int ata_identify(uint8_t bus, uint8_t drive) {
    uint16_t io = ata_io_bases[bus];

    ata_soft_reset(bus);

    if (!ata_busy_wait(io, 1)) {
        return 0;
    }

    outb(ATA_DRIVE_PORT(io), drive == ATA_DRIVE_MASTER ? 0xA0 : 0xB0);
    ata_wait(io);

    outb(ATA_SECTOR_COUNT_PORT(io), 0);
    outb(ATA_LBA_LO_PORT(io), 0);
    outb(ATA_LBA_MID_PORT(io), 0);
    outb(ATA_LBA_HI_PORT(io), 0);

    outb(ATA_COMMAND_PORT(io), ATA_CMD_IDENTIFY);
    ata_wait(io);

    uint8_t status = inb(ATA_COMMAND_PORT(io));
    if (status == 0) {
        return 0;
    }

    if (!ata_busy_wait(io, 1)) {
        return 0;
    }

    if (inb(ATA_LBA_MID_PORT(io)) != 0 && inb(ATA_LBA_HI_PORT(io)) != 0) {
        return 0;
    }

    if (ata_drq_wait(io, 1) < 0) {
        return 0;
    }

    struct ata_drive_info* info = &g_drives[bus][drive];
    info->present = 1;

    uint16_t buf[256];
    for (int i = 0; i < 256; i++) {
        buf[i] = inw(ATA_DATA_PORT(io));
    }

    info->signature     = buf[0];
    info->capabilities  = buf[49];
    info->command_sets  = ((uint32_t)buf[83] << 16) | buf[82];

    info->is_lba48 = (info->command_sets & (1 << 26)) ? 1 : 0;
    ata_words_to_sectors(info, buf);

    for (int i = 0; i < 20; i++) {
        uint16_t w = buf[27 + i];
        info->model[i * 2]     = (char)(w >> 8);
        info->model[i * 2 + 1] = (char)(w & 0xFF);
    }
    info->model[40] = '\0';
    for (int i = 39; i >= 0 && info->model[i] == ' '; i--) {
        info->model[i] = '\0';
    }

    serial_print("[ATA] Drive ");
    serial_print_hex(bus);
    serial_print(":");
    serial_print_hex(drive);
    serial_print(": ");
    serial_print(info->model);
    serial_print(" (");
    serial_print_hex((uint32_t)(info->num_sectors / 2048));
    serial_print(" MB LBA");
    if (info->is_lba48) serial_print("48");
    else serial_print("28");
    serial_print(")\n");

    return 1;
}

int ata_init(void) {
    if (g_ata_initialized) return 1;
    serial_print("[ATA] Initializing ATA PIO driver...\n");

    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < 2; j++) {
            g_drives[i][j].present = 0;
            g_drives[i][j].is_lba48 = 0;
            g_drives[i][j].num_sectors = 0;
            g_drives[i][j].model[0] = '\0';
        }
    }

    for (int bus = 0; bus < 2; bus++) {
        ata_identify(bus, ATA_DRIVE_MASTER);
        ata_identify(bus, ATA_DRIVE_SLAVE);
    }

    g_ata_initialized = 1;
    serial_print("[ATA] ATA PIO driver initialized\n");
    return 1;
}

int ata_drive_present(uint8_t bus, uint8_t drive) {
    if (bus > 1 || drive > 1) return 0;
    return g_drives[bus][drive].present;
}

static int ata_pio_read_lba28(uint16_t io, uint8_t drive, uint32_t lba, uint8_t count, uint16_t* buffer) {
    outb(ATA_DRIVE_PORT(io), 0xE0 | (drive << 4) | ((lba >> 24) & 0x0F));
    outb(ATA_SECTOR_COUNT_PORT(io), count);
    outb(ATA_LBA_LO_PORT(io), lba & 0xFF);
    outb(ATA_LBA_MID_PORT(io), (lba >> 8) & 0xFF);
    outb(ATA_LBA_HI_PORT(io), (lba >> 16) & 0xFF);
    outb(ATA_COMMAND_PORT(io), ATA_CMD_READ_PIO);

    for (int s = 0; s < count; s++) {
        if (ata_drq_wait(io, 5) < 0) return 0;
        for (int i = 0; i < 256; i++) {
            buffer[s * 256 + i] = inw(ATA_DATA_PORT(io));
        }
    }

    return 1;
}

static int ata_pio_read_lba48(uint16_t io, uint8_t drive, uint64_t lba, uint16_t count, uint16_t* buffer) {
    outb(ATA_DRIVE_PORT(io), 0x40 | (drive << 4));
    outb(ATA_LBA_HI_PORT(io), (lba >> 24) & 0xFF);
    outb(ATA_LBA_MID_PORT(io), (lba >> 32) & 0xFF);
    outb(ATA_LBA_LO_PORT(io), (lba >> 40) & 0xFF);
    outb(ATA_SECTOR_COUNT_PORT(io), (count >> 8) & 0xFF);
    outb(ATA_LBA_LO_PORT(io), lba & 0xFF);
    outb(ATA_LBA_MID_PORT(io), (lba >> 8) & 0xFF);
    outb(ATA_LBA_HI_PORT(io), (lba >> 16) & 0xFF);
    outb(ATA_SECTOR_COUNT_PORT(io), count & 0xFF);
    outb(ATA_COMMAND_PORT(io), ATA_CMD_READ_PIO_EXT);

    for (int s = 0; s < count; s++) {
        if (ata_drq_wait(io, 5) < 0) return 0;
        for (int i = 0; i < 256; i++) {
            buffer[s * 256 + i] = inw(ATA_DATA_PORT(io));
        }
    }

    return 1;
}

int ata_read_sectors(uint8_t bus, uint8_t drive, uint64_t lba, uint8_t count, void* buffer) {
    if (bus > 1 || drive > 1 || !g_drives[bus][drive].present || count == 0) {
        return 0;
    }

    uint16_t io = ata_io_bases[bus];
    if (!ata_busy_wait(io, 5)) return 0;

    int result;
    if (g_drives[bus][drive].is_lba48) {
        result = ata_pio_read_lba48(io, drive, lba, count, (uint16_t*)buffer);
    } else {
        result = ata_pio_read_lba28(io, drive, (uint32_t)lba, count, (uint16_t*)buffer);
    }

    return result;
}

static int ata_pio_write_lba28(uint16_t io, uint8_t drive, uint32_t lba, uint8_t count, const uint16_t* buffer) {
    outb(ATA_DRIVE_PORT(io), 0xE0 | (drive << 4) | ((lba >> 24) & 0x0F));
    outb(ATA_SECTOR_COUNT_PORT(io), count);
    outb(ATA_LBA_LO_PORT(io), lba & 0xFF);
    outb(ATA_LBA_MID_PORT(io), (lba >> 8) & 0xFF);
    outb(ATA_LBA_HI_PORT(io), (lba >> 16) & 0xFF);
    outb(ATA_COMMAND_PORT(io), ATA_CMD_WRITE_PIO);

    for (int s = 0; s < count; s++) {
        if (ata_drq_wait(io, 5) < 0) return 0;
        for (int i = 0; i < 256; i++) {
            outw(ATA_DATA_PORT(io), buffer[s * 256 + i]);
        }
    }

    return 1;
}

static int ata_pio_write_lba48(uint16_t io, uint8_t drive, uint64_t lba, uint16_t count, const uint16_t* buffer) {
    outb(ATA_DRIVE_PORT(io), 0x40 | (drive << 4));
    outb(ATA_LBA_HI_PORT(io), (lba >> 40) & 0xFF);
    outb(ATA_LBA_MID_PORT(io), (lba >> 32) & 0xFF);
    outb(ATA_LBA_LO_PORT(io), (lba >> 24) & 0xFF);
    outb(ATA_SECTOR_COUNT_PORT(io), (count >> 8) & 0xFF);
    outb(ATA_LBA_LO_PORT(io), lba & 0xFF);
    outb(ATA_LBA_MID_PORT(io), (lba >> 8) & 0xFF);
    outb(ATA_LBA_HI_PORT(io), (lba >> 16) & 0xFF);
    outb(ATA_SECTOR_COUNT_PORT(io), count & 0xFF);
    outb(ATA_COMMAND_PORT(io), ATA_CMD_WRITE_PIO_EXT);

    for (int s = 0; s < count; s++) {
        if (ata_drq_wait(io, 5) < 0) return 0;
        for (int i = 0; i < 256; i++) {
            outw(ATA_DATA_PORT(io), buffer[s * 256 + i]);
        }
    }

    return 1;
}

int ata_write_sectors(uint8_t bus, uint8_t drive, uint64_t lba, uint8_t count, const void* buffer) {
    if (bus > 1 || drive > 1 || !g_drives[bus][drive].present || count == 0) {
        return 0;
    }

    uint16_t io = ata_io_bases[bus];
    if (!ata_busy_wait(io, 5)) return 0;

    int result;
    if (g_drives[bus][drive].is_lba48) {
        result = ata_pio_write_lba48(io, drive, lba, count, (const uint16_t*)buffer);
    } else {
        result = ata_pio_write_lba28(io, drive, (uint32_t)lba, count, (const uint16_t*)buffer);
    }

    if (result) {
        outb(ATA_COMMAND_PORT(io), ATA_CMD_FLUSH_CACHE);
        ata_busy_wait(io, 5);
    }

    return result;
}

void ata_get_info(uint8_t bus, uint8_t drive, struct ata_drive_info* info) {
    if (bus > 1 || drive > 1 || !info) return;
    *info = g_drives[bus][drive];
}

void ata_flush_cache(uint8_t bus, uint8_t drive) {
    if (bus > 1 || drive > 1) return;
    uint16_t io = ata_io_bases[bus];
    outb(ATA_COMMAND_PORT(io), ATA_CMD_FLUSH_CACHE);
    ata_busy_wait(io, 5);
}
