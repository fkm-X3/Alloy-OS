ARCH ?= i686

ifeq ($(ARCH),i686)
    CC = i686-elf-gcc
    LD = i686-elf-ld
    CFLAGS = -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Isrc -I../os/userland/lib
    LDFLAGS = -T alloy_de.ld -static -nostdlib
endif

LIB_DIR = ../os/userland/lib
BUILD_DIR = build

LIB_OBJS = $(BUILD_DIR)/crt0.o $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o

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

$(BUILD_DIR)/wayland_client.o: $(LIB_DIR)/wayland_client.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

SRCS = src/main.c src/draw.c src/shm.c
OBJS = $(patsubst src/%.c,$(BUILD_DIR)/%.o,$(SRCS))

$(BUILD_DIR)/%.o: src/%.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/alloy_de: $(OBJS) $(LIB_OBJS)
	$(LD) $(LDFLAGS) -o $@ $(BUILD_DIR)/crt0.o $(OBJS) $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o
	@echo "alloy_de built: $@"

clean:
	rm -rf $(BUILD_DIR)
