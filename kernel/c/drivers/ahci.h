#ifndef ALLOY_AHCI_H
#define ALLOY_AHCI_H

#include "boot/types.h"

#define AHCI_MAX_PORTS 32
#define AHCI_SECTOR_SIZE 512
#define AHCI_COMMAND_SLOTS 32

struct ahci_drive_info {
    uint8_t  present;
    uint8_t  port_num;
    uint64_t num_sectors;
    char     serial[21];
    char     firmware[9];
    char     model[41];
};

#ifdef __cplusplus
extern "C" {
#endif

int  ahci_init(void);
int  ahci_drive_count(void);
int  ahci_get_drive(int index, struct ahci_drive_info* info);
int  ahci_read_sectors(int drive_index, uint64_t lba, uint8_t count, void* buffer);
int  ahci_write_sectors(int drive_index, uint64_t lba, uint8_t count, const void* buffer);

#ifdef __cplusplus
}
#endif

#endif
