#include "initrd.h"
#include "../boot/multiboot2.h"
#include "serial.h"

static struct initrd_module g_modules[MAX_INITRD_MODULES];
static int g_module_count = 0;

void initrd_init(uint32_t multiboot_addr) {
    serial_print("[INITRD] Scanning multiboot modules...\n");

    g_module_count = 0;

    if (multiboot_addr == 0) {
        serial_print("[INITRD] No multiboot info\n");
        return;
    }

    struct multiboot_tag* tag = (struct multiboot_tag*)(multiboot_addr + 8);

    while (tag->type != MULTIBOOT_TAG_TYPE_END) {
        if (tag->type == MULTIBOOT_TAG_TYPE_MODULE) {
            if (g_module_count >= MAX_INITRD_MODULES) {
                serial_print("[INITRD] Too many modules\n");
                break;
            }

            struct multiboot_tag_module* mod = (struct multiboot_tag_module*)tag;
            struct initrd_module* m = &g_modules[g_module_count];

            m->start = mod->mod_start;
            m->end   = mod->mod_end;
            m->size  = mod->mod_end - mod->mod_start;

            int i = 0;
            while (mod->cmdline[i] && i < 63) {
                m->cmdline[i] = mod->cmdline[i];
                i++;
            }
            m->cmdline[i] = '\0';

            serial_print("[INITRD] Module ");
            serial_print_hex(g_module_count);
            serial_print(": start=0x");
            serial_print_hex(m->start);
            serial_print(" end=0x");
            serial_print_hex(m->end);
            serial_print(" size=");
            serial_print_hex(m->size);
            serial_print(" cmdline=\"");
            serial_print(m->cmdline);
            serial_print("\"\n");

            g_module_count++;
        }

        tag = (struct multiboot_tag*)((uint8_t*)tag + ((tag->size + 7) & ~7));
    }

    serial_print("[INITRD] Found ");
    serial_print_hex(g_module_count);
    serial_print(" module(s)\n");
}

int initrd_module_count(void) {
    return g_module_count;
}

int initrd_get_module(int index, struct initrd_module* mod) {
    if (index < 0 || index >= g_module_count || !mod) return 0;
    *mod = g_modules[index];
    return 1;
}

uint32_t initrd_module_start_ffi(int index) {
    if (index < 0 || index >= g_module_count) return 0;
    return g_modules[index].start;
}

uint32_t initrd_module_end_ffi(int index) {
    if (index < 0 || index >= g_module_count) return 0;
    return g_modules[index].end;
}

uint32_t initrd_module_size_ffi(int index) {
    if (index < 0 || index >= g_module_count) return 0;
    return g_modules[index].size;
}

void initrd_module_cmdline_ffi(int index, char* buf, uint32_t max_len) {
    if (index < 0 || index >= g_module_count || !buf || max_len == 0) {
        if (buf && max_len > 0) buf[0] = '\0';
        return;
    }
    uint32_t i = 0;
    while (g_modules[index].cmdline[i] && i < max_len - 1) {
        buf[i] = g_modules[index].cmdline[i];
        i++;
    }
    buf[i] = '\0';
}

int initrd_has_modules_ffi(void) {
    return g_module_count > 0;
}
