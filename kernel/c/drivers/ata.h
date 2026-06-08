#ifndef ALLOY_ATA_H
#define ALLOY_ATA_H

#include "boot/types.h"

#define ATA_BUS_PRIMARY   0
#define ATA_BUS_SECONDARY 1
#define ATA_DRIVE_MASTER  0
#define ATA_DRIVE_SLAVE   1

#define ATA_SECTOR_SIZE 512

#define ATA_CMD_READ_PIO       0x20
#define ATA_CMD_READ_PIO_EXT   0x24
#define ATA_CMD_WRITE_PIO      0x30
#define ATA_CMD_WRITE_PIO_EXT  0x34
#define ATA_CMD_IDENTIFY       0xEC
#define ATA_CMD_FLUSH_CACHE    0xE7
#define ATA_CMD_READ_MULTIPLE  0xC4
#define ATA_CMD_WRITE_MULTIPLE 0xC5
#define ATA_CMD_SET_MULTIPLE   0xC6
#define ATA_CMD_SET_FEATURES   0xEF

#define ATA_STATUS_ERR  0x01
#define ATA_STATUS_DRQ  0x08
#define ATA_STATUS_DF   0x20
#define ATA_STATUS_DRDY 0x40
#define ATA_STATUS_BSY  0x80

struct ata_drive_info {
    uint8_t  present;
    uint8_t  is_lba48;
    uint16_t signature;
    uint16_t capabilities;
    uint32_t command_sets;
    uint64_t num_sectors;
    char     model[41];
};

#ifdef __cplusplus
extern "C" {
#endif

int  ata_init(void);
int  ata_drive_present(uint8_t bus, uint8_t drive);
int  ata_read_sectors(uint8_t bus, uint8_t drive, uint64_t lba, uint8_t count, void* buffer);
int  ata_write_sectors(uint8_t bus, uint8_t drive, uint64_t lba, uint8_t count, const void* buffer);
void ata_get_info(uint8_t bus, uint8_t drive, struct ata_drive_info* info);
void ata_flush_cache(uint8_t bus, uint8_t drive);

#ifdef __cplusplus
}
#endif

#endif
