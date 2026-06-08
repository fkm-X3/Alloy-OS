#ifndef ALLOY_PCI_H
#define ALLOY_PCI_H

#include "boot/types.h"

#define PCI_CONFIG_ADDRESS  0xCF8
#define PCI_CONFIG_DATA     0xCFC

#define PCI_VENDOR_ID           0x00
#define PCI_DEVICE_ID           0x02
#define PCI_COMMAND             0x04
#define PCI_STATUS              0x06
#define PCI_REVISION_ID         0x08
#define PCI_PROG_IF             0x09
#define PCI_SUBCLASS            0x0A
#define PCI_CLASS_CODE          0x0B
#define PCI_HEADER_TYPE         0x0E
#define PCI_BAR0                0x10
#define PCI_BAR1                0x14
#define PCI_BAR2                0x18
#define PCI_BAR3                0x1C
#define PCI_BAR4                0x20
#define PCI_BAR5                0x24
#define PCI_SECONDARY_BUS       0x19

#define PCI_CLASS_MASS_STORAGE  0x01
#define PCI_SUBCLASS_IDE        0x01
#define PCI_SUBCLASS_SATA       0x06
#define PCI_SUBCLASS_NVMe       0x08
#define PCI_PROG_IF_AHCI        0x01

#define PCI_HEADER_TYPE_BRIDGE  0x01

#define MAX_PCI_DEVICES 256

struct pci_device {
    uint8_t  bus;
    uint8_t  slot;
    uint8_t  func;
    uint16_t vendor_id;
    uint16_t device_id;
    uint8_t  revision_id;
    uint8_t  class_code;
    uint8_t  subclass;
    uint8_t  prog_if;
    uint8_t  header_type;
    uint32_t bars[6];
};

#ifdef __cplusplus
extern "C" {
#endif

void pci_init(void);
int  pci_device_count(void);
int  pci_get_device(int index, struct pci_device* dev);
int  pci_find_devices(uint8_t class_code, uint8_t subclass, uint8_t prog_if,
                      struct pci_device* out, int max_count);

uint16_t pci_config_read_word(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset);
uint32_t pci_config_read_dword(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset);
void     pci_config_write_dword(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset, uint32_t value);

#ifdef __cplusplus
}
#endif

#endif
