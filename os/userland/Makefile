# Alloy OS Userland Build System
ARCH ?= x86_64

ifeq ($(ARCH),x86_64)
    CC = gcc
    LD = ld
    CFLAGS = -m64 -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    LDFLAGS = -m elf_x86_64 -T userland.ld -static -nostdlib
else ifeq ($(ARCH),aarch64)
    CC = aarch64-linux-gnu-gcc
    LD = aarch64-linux-gnu-ld
    CFLAGS = -march=armv8-a -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    LDFLAGS = -m aarch64elf -T userland.ld -static -nostdlib
else
    $(error Unsupported architecture: $(ARCH). Use x86_64 or aarch64)
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

.PHONY: all clean

all: $(LIB_OBJS) $(TEST_BINS)

$(BUILD_DIR)/%.o: $(LIB_DIR)/%.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%: $(TEST_DIR)/%.c $(LIB_OBJS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) -o $@ $(BUILD_DIR)/crt0.o $(BUILD_DIR)/$*.o $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o

clean:
	rm -rf $(BUILD_DIR)
