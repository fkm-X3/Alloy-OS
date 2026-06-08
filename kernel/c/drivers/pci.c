#include "pci.h"
#include "serial.h"

static struct pci_device g_devices[MAX_PCI_DEVICES];
static int g_device_count = 0;

static inline void outl(uint16_t port, uint32_t value) {
    asm volatile("outl %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint32_t inl(uint16_t port) {
    uint32_t ret;
    asm volatile("inl %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static uint32_t pci_make_addr(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    return 0x80000000 | (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC);
}

uint32_t pci_config_read_dword(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    uint32_t addr = pci_make_addr(bus, slot, func, offset);
    outl(PCI_CONFIG_ADDRESS, addr);
    return inl(PCI_CONFIG_DATA);
}

void pci_config_write_dword(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset, uint32_t value) {
    uint32_t addr = pci_make_addr(bus, slot, func, offset);
    outl(PCI_CONFIG_ADDRESS, addr);
    outl(PCI_CONFIG_DATA, value);
}

uint16_t pci_config_read_word(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    uint32_t dword = pci_config_read_dword(bus, slot, func, offset);
    if (offset & 2) {
        return (uint16_t)(dword >> 16);
    }
    return (uint16_t)(dword & 0xFFFF);
}

static void pci_read_device(uint8_t bus, uint8_t slot, uint8_t func) {
    uint16_t vendor = pci_config_read_word(bus, slot, func, PCI_VENDOR_ID);
    if (vendor == 0xFFFF) return;

    if (g_device_count >= MAX_PCI_DEVICES) return;

    struct pci_device* dev = &g_devices[g_device_count];
    dev->bus        = bus;
    dev->slot       = slot;
    dev->func       = func;
    dev->vendor_id  = vendor;
    dev->device_id  = pci_config_read_word(bus, slot, func, PCI_DEVICE_ID);

    uint32_t class_reg = pci_config_read_dword(bus, slot, func, 0x08);
    dev->revision_id = (uint8_t)(class_reg & 0xFF);
    dev->prog_if     = (uint8_t)((class_reg >> 8) & 0xFF);
    dev->subclass    = (uint8_t)((class_reg >> 16) & 0xFF);
    dev->class_code  = (uint8_t)(class_reg >> 24);

    uint32_t header = pci_config_read_dword(bus, slot, func, 0x0C);
    dev->header_type = (uint8_t)((header >> 16) & 0xFF);

    for (int i = 0; i < 6; i++) {
        dev->bars[i] = pci_config_read_dword(bus, slot, func, 0x10 + i * 4);
    }

    g_device_count++;
}

static void pci_scan_bus(uint8_t bus) {
    for (int slot = 0; slot < 32; slot++) {
        uint16_t vendor = pci_config_read_word(bus, slot, 0, PCI_VENDOR_ID);
        if (vendor == 0xFFFF) continue;

        pci_read_device(bus, slot, 0);

        uint32_t header = pci_config_read_dword(bus, slot, 0, 0x0C);
        uint8_t header_type = (uint8_t)((header >> 16) & 0xFF);

        if (header_type & 0x80) {
            for (int func = 1; func < 8; func++) {
                vendor = pci_config_read_word(bus, slot, func, PCI_VENDOR_ID);
                if (vendor != 0xFFFF) {
                    pci_read_device(bus, slot, func);
                }
            }
        }

        if ((header_type & 0x7F) == PCI_HEADER_TYPE_BRIDGE) {
            uint32_t bus_reg = pci_config_read_dword(bus, slot, 0, PCI_SECONDARY_BUS);
            uint8_t secondary_bus = (uint8_t)((bus_reg >> 8) & 0xFF);
            if (secondary_bus != bus) {
                pci_scan_bus(secondary_bus);
            }
        }
    }
}

void pci_init(void) {
    serial_print("[PCI] Scanning PCI bus...\n");
    g_device_count = 0;

    uint16_t vendor = pci_config_read_word(0, 0, 0, PCI_VENDOR_ID);
    if (vendor == 0xFFFF) {
        serial_print("[PCI] No PCI host controller found\n");
        return;
    }

    pci_scan_bus(0);

    serial_print("[PCI] Found ");
    serial_print_hex(g_device_count);
    serial_print(" devices\n");

    for (int i = 0; i < g_device_count; i++) {
        struct pci_device* dev = &g_devices[i];
        serial_print("  ");
        serial_print_hex(dev->bus);
        serial_print(":");
        serial_print_hex(dev->slot);
        serial_print(".");
        serial_print_hex(dev->func);
        serial_print(" [");
        serial_print_hex(dev->class_code);
        serial_print(".");
        serial_print_hex(dev->subclass);
        serial_print(".");
        serial_print_hex(dev->prog_if);
        serial_print("] vendor=");
        serial_print_hex(dev->vendor_id);
        serial_print(" device=");
        serial_print_hex(dev->device_id);
        serial_print("\n");
    }
}

int pci_device_count(void) {
    return g_device_count;
}

int pci_get_device(int index, struct pci_device* dev) {
    if (index < 0 || index >= g_device_count || !dev) return 0;
    *dev = g_devices[index];
    return 1;
}

int pci_find_devices(uint8_t class_code, uint8_t subclass, uint8_t prog_if,
                     struct pci_device* out, int max_count) {
    int count = 0;
    for (int i = 0; i < g_device_count && count < max_count; i++) {
        if (g_devices[i].class_code == class_code &&
            g_devices[i].subclass == subclass &&
            (prog_if == 0xFF || g_devices[i].prog_if == prog_if)) {
            out[count++] = g_devices[i];
        }
    }
    return count;
}
