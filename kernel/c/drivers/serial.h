#ifndef ALLOY_SERIAL_H
#define ALLOY_SERIAL_H

#include "boot/types.h"

#ifdef __cplusplus
extern "C" {
#endif

void init_serial();
void serial_print(const char* str);
void serial_print_hex(uint32_t value);
void serial_print_hex64(uint64_t value);
void serial_print_hex_with_prefix(const char* prefix, uint32_t value);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_SERIAL_H */