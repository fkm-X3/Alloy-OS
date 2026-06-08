#include "ahci.h"
#include "pci.h"
#include "../mm/pmm.h"
#include "../mm/vmm.h"
#include "../mm/paging.h"
#include "serial.h"

#define HBA_PI   0x00C
#define HBA_GHC  0x004
#define HBA_GHC_AE (1U << 31)
#define HBA_CAP  0x000
#define HBA_CAP_NP 0x1F

#define PORT_CMD_ST      (1U << 0)
#define PORT_CMD_FRE     (1U << 4)
#define PORT_CMD_FR      (1U << 14)
#define PORT_CMD_CR      (1U << 15)
#define PORT_CMD_SPIN_UP (1U << 1)
#define PORT_CMD_POWER_ON (1U << 2)

#define PORT_TFD_BSY (1U << 7)
#define PORT_TFD_DRQ (1U << 3)
#define PORT_SSTS_DET 3
#define PORT_SIG_ATA 0x00000101

#define REG_CMD  0x18
#define REG_TFD  0x20
#define REG_SIG  0x24
#define REG_SSTS 0x28
#define REG_SERR 0x30
#define REG_CI   0x38
#define REG_CLB  0x00
#define REG_CLBU 0x04
#define REG_FB   0x08
#define REG_FBU  0x0C
#define REG_IE   0x14
#define REG_SCTL 0x2C

#define CMD_IDENTIFY     0xEC
#define CMD_READ_DMA_EXT 0x25
#define CMD_WRITE_DMA_EXT 0x35

#define H2D_FIS_TYPE 0x27

struct hba_cmd_hdr {
    uint16_t cfl:5;
    uint16_t a:1;
    uint16_t w:1;
    uint16_t p:1;
    uint16_t r:1;
    uint16_t b:1;
    uint16_t c:1;
    uint16_t rsv:1;
    uint16_t rsv2:4;
    uint16_t prdtl;
    uint32_t prdbc;
    uint32_t ctba;
    uint32_t ctbau;
    uint32_t rsv3[4];
} __attribute__((packed));

struct prdt_entry {
    uint32_t dba;
    uint32_t dbau;
    uint32_t rsv;
    uint32_t dbc:22;
    uint32_t rsv2:9;
    uint32_t i:1;
} __attribute__((packed));

struct hba_cmd_tbl {
    uint8_t cfis[64];
    uint8_t acmd[16];
    uint8_t rsv[48];
    struct prdt_entry prdt[1];
} __attribute__((packed));

struct h2d_fis {
    uint8_t type;
    uint8_t pmport:4;
    uint8_t rsv0:3;
    uint8_t c:1;
    uint8_t cmd;
    uint8_t feat_lo;
    uint8_t lba0, lba1, lba2;
    uint8_t dev;
    uint8_t lba3, lba4, lba5;
    uint8_t feat_hi;
    uint8_t rsv1[4];
} __attribute__((packed));

static uint8_t* g_abar = NULL;
static int g_initialized = 0;
static int g_drives = 0;

struct drive {
    int port;
    int present;
    uint64_t sectors;
    char model[41];
} g_devs[32];

static inline uint32_t mmio_r32(uint32_t o) { return *(volatile uint32_t*)(g_abar + o); }
static inline void mmio_w32(uint32_t o, uint32_t v) { *(volatile uint32_t*)(g_abar + o) = v; }
static inline uint32_t port_r(int p, uint32_t o) { return mmio_r32(0x100 + p * 0x80 + o); }
static inline void port_w(int p, uint32_t o, uint32_t v) { mmio_w32(0x100 + p * 0x80 + o, v); }

static int alloc_page(uint32_t* phys, uint8_t** virt) {
    void* p = pmm_alloc_frame();
    if (!p) return 0;
    *phys = (uint32_t)p;

    void* v = vmm_alloc_region(4096, PAGE_PRESENT | PAGE_WRITE);
    if (!v) { pmm_free_frame(p); return 0; }

    uint32_t va = (uint32_t)v;
    vmm_unmap((void*)va);
    if (!vmm_map((void*)va, (void*)*phys, PAGE_PRESENT | PAGE_WRITE)) {
        pmm_free_frame(p);
        return 0;
    }
    *virt = (uint8_t*)va;
    return 1;
}

static int spin_ready(int p, int ms) {
    for (int i = 0; i < ms * 10000; i++) {
        uint32_t tfd = port_r(p, REG_TFD);
        if (!(tfd & PORT_TFD_BSY) && !(tfd & PORT_TFD_DRQ)) return 1;
    }
    return 0;
}

static int send_cmd(struct drive* d, int write, uint64_t lba, uint8_t count,
                    uint32_t data_phys) {
    if (!spin_ready(d->port, 10)) return 0;

    uint8_t* clb_v = NULL;
    uint32_t clb_p = 0;
    uint8_t* ct_v = NULL;
    uint32_t ct_p = 0;
    if (!alloc_page(&clb_p, &clb_v)) return 0;
    if (!alloc_page(&ct_p, &ct_v)) return 0;

    struct hba_cmd_hdr* hdr = (struct hba_cmd_hdr*)clb_v;
    struct hba_cmd_tbl* tbl = (struct hba_cmd_tbl*)ct_v;

    struct h2d_fis* fis = (struct h2d_fis*)tbl->cfis;
    fis->type    = H2D_FIS_TYPE;
    fis->c       = 1;
    fis->cmd     = write ? CMD_WRITE_DMA_EXT : CMD_READ_DMA_EXT;
    fis->lba0    = (uint8_t)(lba);
    fis->lba1    = (uint8_t)(lba >> 8);
    fis->lba2    = (uint8_t)(lba >> 16);
    fis->dev     = 0x40;
    fis->lba3    = (uint8_t)(lba >> 24);
    fis->lba4    = (uint8_t)(lba >> 32);
    fis->lba5    = (uint8_t)(lba >> 40);
    fis->feat_lo = 0;
    fis->feat_hi = 0;
    tbl->cfis[12] = count;

    hdr->cfl   = sizeof(struct h2d_fis) / 4;
    hdr->w     = write;
    hdr->prdtl = 1;
    hdr->prdbc = count * 512;
    hdr->ctba  = ct_p;

    tbl->prdt[0].dba = data_phys;
    tbl->prdt[0].dbc = count * 512 - 1;
    tbl->prdt[0].i   = 1;

    port_w(d->port, REG_CLB,  clb_p);
    port_w(d->port, REG_CLBU, 0);
    port_w(d->port, REG_FB,   0);
    port_w(d->port, REG_FBU,  0);

    port_w(d->port, REG_CI, 1);

    int ok = 0;
    for (int i = 0; i < 10000000; i++) {
        if (!(port_r(d->port, REG_CI) & 1)) { ok = 1; break; }
    }

    pmm_free_frame((void*)clb_p);
    pmm_free_frame((void*)ct_p);
    return ok;
}

int ahci_init(void) {
    if (g_initialized) return 1;
    serial_print("[AHCI] Scanning SATA...\n");

    struct pci_device devs[8];
    int n = pci_find_devices(PCI_CLASS_MASS_STORAGE, PCI_SUBCLASS_SATA, 0xFF, devs, 8);
    if (!n) {
        serial_print("[AHCI] No SATA host\n");
        g_initialized = 1;
        return 0;
    }

    struct pci_device* c = &devs[0];
    uint16_t cmd = pci_config_read_word(c->bus, c->slot, c->func, PCI_COMMAND);
    cmd |= 7;
    pci_config_write_dword(c->bus, c->slot, c->func, PCI_COMMAND, cmd);

    uint32_t abar = c->bars[5] & ~1;
    if (!abar) {
        serial_print("[AHCI] No ABAR\n");
        g_initialized = 1;
        return 0;
    }

    serial_print("[AHCI] ABAR=0x");
    serial_print_hex(abar);
    serial_print("\n");

    void* v = vmm_alloc_region(8192, PAGE_PRESENT | PAGE_WRITE);
    if (!v) { g_initialized = 1; return 0; }

    g_abar = (uint8_t*)v;
    uint32_t va = (uint32_t)g_abar;
    for (int off = 0; off < 8192; off += 4096) {
        vmm_unmap((void*)(va + off));
        vmm_map((void*)(va + off), (void*)(abar + off), PAGE_PRESENT | PAGE_WRITE);
    }

    mmio_w32(HBA_GHC, HBA_GHC_AE);

    uint32_t pi = mmio_r32(HBA_PI);
    int ports = mmio_r32(HBA_CAP) & HBA_CAP_NP;

    g_drives = 0;
    for (int p = 0; p < ports && p < 32; p++) {
        if (!(pi & (1 << p))) continue;

        port_w(p, REG_CMD, port_r(p, REG_CMD) | PORT_CMD_SPIN_UP | PORT_CMD_POWER_ON);
        uint32_t ssts = port_r(p, REG_SSTS);
        if ((ssts & 0x0F) != PORT_SSTS_DET) continue;
        if (port_r(p, REG_SIG) != PORT_SIG_ATA) continue;

        struct drive* d = &g_devs[g_drives];
        d->port = p;

        uint8_t *clb_v, *ct_v, *id_v;
        uint32_t clb_p, ct_p, id_p;
        if (!alloc_page(&clb_p, &clb_v)) continue;
        if (!alloc_page(&ct_p, &ct_v)) { pmm_free_frame((void*)clb_p); continue; }
        if (!alloc_page(&id_p, &id_v)) { pmm_free_frame((void*)clb_p); pmm_free_frame((void*)ct_p); continue; }

        struct hba_cmd_hdr* hdr = (struct hba_cmd_hdr*)clb_v;
        struct hba_cmd_tbl* tbl = (struct hba_cmd_tbl*)ct_v;

        struct h2d_fis* fis = (struct h2d_fis*)tbl->cfis;
        fis->type = H2D_FIS_TYPE;
        fis->c    = 1;
        fis->cmd  = CMD_IDENTIFY;

        hdr->cfl   = sizeof(struct h2d_fis) / 4;
        hdr->w     = 0;
        hdr->prdtl = 1;
        hdr->ctba  = ct_p;
        tbl->prdt[0].dba = id_p;
        tbl->prdt[0].dbc = 512 - 1;
        tbl->prdt[0].i   = 1;

        port_w(p, REG_CLB,  clb_p);
        port_w(p, REG_CLBU, 0);
        port_w(p, REG_FB,   0);
        port_w(p, REG_FBU,  0);
        port_w(p, REG_SERR, ~0U);

        port_w(p, REG_CMD, port_r(p, REG_CMD) | PORT_CMD_FRE | PORT_CMD_ST);

        if (spin_ready(p, 5)) {
            port_w(p, REG_CI, 1);
            int ok = 0;
            for (int i = 0; i < 5000000; i++) {
                if (!(port_r(p, REG_CI) & 1)) { ok = 1; break; }
            }
            if (ok) {
                uint16_t* id = (uint16_t*)id_v;
                d->present = 1;
                if (id[83] & (1 << 10)) {
                    uint64_t lo = (uint64_t)(uint32_t)id[100] | ((uint64_t)(uint32_t)id[101] << 16);
                    uint64_t hi = (uint64_t)(uint32_t)id[102] | ((uint64_t)(uint32_t)id[103] << 16);
                    d->sectors = lo | (hi << 32);
                } else {
                    d->sectors = (uint32_t)id[60] | ((uint32_t)id[61] << 16);
                }

                for (int i = 0; i < 20; i++) {
                    uint16_t w = id[27 + i];
                    d->model[i*2]   = (char)(w >> 8);
                    d->model[i*2+1] = (char)(w & 0xFF);
                }
                d->model[40] = 0;
                for (int i = 39; i >= 0 && d->model[i] == ' '; i--) d->model[i] = 0;

                serial_print("[AHCI] Port ");
                serial_print_hex(p);
                serial_print(": ");
                serial_print(d->model);
                serial_print(" (");
                serial_print_hex((uint32_t)(d->sectors / 2048));
                serial_print(" MB)\n");
                g_drives++;
            }
        }

        pmm_free_frame((void*)clb_p);
        pmm_free_frame((void*)ct_p);
        pmm_free_frame((void*)id_p);
    }

    g_initialized = 1;
    serial_print("[AHCI] ");
    serial_print_hex(g_drives);
    serial_print(" SATA drive(s)\n");
    return g_drives > 0;
}

int ahci_drive_count(void) { return g_drives; }

int ahci_get_drive(int idx, struct ahci_drive_info* info) {
    if (idx < 0 || idx >= g_drives || !info) return 0;
    info->present = g_devs[idx].present;
    info->port_num = g_devs[idx].port;
    info->num_sectors = g_devs[idx].sectors;
    for (int i = 0; i < 41; i++) info->model[i] = g_devs[idx].model[i];
    return 1;
}

int ahci_read_sectors(int idx, uint64_t lba, uint8_t count, void* buf) {
    if (idx < 0 || idx >= g_drives) return 0;

    uint32_t dma_p;
    uint8_t* dma_v;
    if (!alloc_page(&dma_p, &dma_v)) return 0;

    uint32_t total = count * 512;
    if (!send_cmd(&g_devs[idx], 0, lba, count, dma_p)) {
        pmm_free_frame((void*)dma_p);
        return 0;
    }

    for (uint32_t i = 0; i < total; i++) ((uint8_t*)buf)[i] = dma_v[i];

    pmm_free_frame((void*)dma_p);
    return 1;
}

int ahci_write_sectors(int idx, uint64_t lba, uint8_t count, const void* buf) {
    if (idx < 0 || idx >= g_drives) return 0;

    uint32_t dma_p;
    uint8_t* dma_v;
    if (!alloc_page(&dma_p, &dma_v)) return 0;

    uint32_t total = count * 512;
    for (uint32_t i = 0; i < total; i++) dma_v[i] = ((const uint8_t*)buf)[i];

    int r = send_cmd(&g_devs[idx], 1, lba, count, dma_p);

    pmm_free_frame((void*)dma_p);
    return r;
}
