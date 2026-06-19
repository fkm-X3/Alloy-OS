ARCH ?= i686

# Only i686 is supported (linker script and crt0 are i386-specific)
ifeq ($(ARCH),i686)
    CC = gcc
    LD = ld
    CFLAGS = -m32 -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Isrc -I../os/userland/lib
    LDFLAGS = -m elf_i386 -T alloy_de.ld -static -nostdlib

LIB_DIR = ../os/userland/lib
BUILD_DIR = build

LIB_OBJS = $(BUILD_DIR)/crt0.o $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o

.PHONY: all clean

all: $(BUILD_DIR)/alloy_de

$(BUILD_DIR)/crt0.o: $(LIB_DIR)/crt0.S
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/stdlib.o: $(LIB_DIR)/stdlib.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/stdio.o: $(LIB_DIR)/stdio.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

SRCS = src/main.c src/draw.c src/shm.c src/wl_client.c
OBJS = $(patsubst src/%.c,$(BUILD_DIR)/%.o,$(SRCS))

$(BUILD_DIR)/%.o: src/%.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/alloy_de: $(OBJS) $(LIB_OBJS)
	$(LD) $(LDFLAGS) -o $@ $(BUILD_DIR)/crt0.o $(OBJS) $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o
	@echo "alloy_de built: $@"

clean:
	rm -rf $(BUILD_DIR)

else
# No-op for unsupported architectures (avoids CI failure for x86_64/aarch64)
.PHONY: all clean

all:
	@echo "alloy_de: skipping build for $(ARCH) (i686 only)"

clean:
	@echo "alloy_de: skipping clean for $(ARCH)"
endif
