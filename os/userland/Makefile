# Alloy OS Userland Build System
ARCH ?= i686

ifeq ($(ARCH),i686)
    CC = i686-elf-gcc
    LD = i686-elf-ld
    CFLAGS = -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    LDFLAGS = -T userland.ld -static -nostdlib
endif

LIB_DIR = lib
TEST_DIR = tests
BUILD_DIR = build

LIB_OBJS = $(BUILD_DIR)/crt0.o $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o

$(BUILD_DIR)/crt0.o: $(LIB_DIR)/crt0.S
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

TEST_SRCS = $(wildcard $(TEST_DIR)/*.c)
TEST_BINS = $(patsubst $(TEST_DIR)/%.c,$(BUILD_DIR)/%,$(TEST_SRCS))

# The alloy_de binary is the primary DE — always copy to project root
DE_BIN = $(BUILD_DIR)/alloy_de

.PHONY: all clean

all: $(LIB_OBJS) $(TEST_BINS) $(DE_BIN)

$(BUILD_DIR)/%.o: $(LIB_DIR)/%.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%: $(TEST_DIR)/%.c $(LIB_OBJS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) -o $@ $(BUILD_DIR)/crt0.o $(BUILD_DIR)/$*.o $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o

$(BUILD_DIR)/alloy_de: $(TEST_DIR)/alloy_de.c $(LIB_OBJS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $(TEST_DIR)/alloy_de.c -o $(BUILD_DIR)/alloy_de.o
	$(LD) $(LDFLAGS) -o $@ $(BUILD_DIR)/crt0.o $(BUILD_DIR)/alloy_de.o $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o
	@cp $@ $(BUILD_DIR)/../../
	@echo "alloy_de built and copied to project root"

clean:
	rm -rf $(BUILD_DIR)
