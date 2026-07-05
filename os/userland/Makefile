# Alloy OS Userland Build System
ARCH ?= x86_64

ifeq ($(ARCH),x86_64)
    CC = gcc
    CXX = g++
    LD = ld
    CFLAGS = -m64 -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    CXXFLAGS = -m64 -std=c++17 -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ilib
    LDFLAGS = -m elf_x86_64 -T userland.ld -static -nostdlib
else ifeq ($(ARCH),aarch64)
    CC = aarch64-linux-gnu-gcc
    CXX = aarch64-linux-gnu-g++
    LD = aarch64-linux-gnu-ld
    CFLAGS = -march=armv8-a -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    CXXFLAGS = -march=armv8-a -std=c++17 -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ilib
    LDFLAGS = -m aarch64elf -T userland.ld -static -nostdlib
else
    $(error Unsupported architecture: $(ARCH). Use x86_64 or aarch64)
endif

LIB_DIR = lib
TEST_DIR = tests
BUILD_DIR = build

# Library objects (order: crt0, crt_cpp, then the rest)
CRT0_OBJ = $(BUILD_DIR)/crt0.o
CRT_CPP_OBJ = $(BUILD_DIR)/crt_cpp.o
C_LIB_OBJS = $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o
CPP_LIB_OBJS = $(BUILD_DIR)/cxxabi.o
LIB_OBJS = $(CRT0_OBJ) $(CRT_CPP_OBJ) $(C_LIB_OBJS) $(CPP_LIB_OBJS)

# Test binaries
TEST_C_SRCS = $(wildcard $(TEST_DIR)/*.c)
TEST_CPP_SRCS = $(wildcard $(TEST_DIR)/*.cpp)
TEST_C_BINS = $(patsubst $(TEST_DIR)/%.c,$(BUILD_DIR)/%,$(TEST_C_SRCS))
TEST_CPP_BINS = $(patsubst $(TEST_DIR)/%.cpp,$(BUILD_DIR)/%,$(TEST_CPP_SRCS))
TEST_BINS = $(TEST_C_BINS) $(TEST_CPP_BINS)

ALL_OBJS = $(LIB_OBJS) $(patsubst $(BUILD_DIR)/%,$(BUILD_DIR)/%.o,$(TEST_BINS))

.PHONY: all clean

all: $(BUILD_DIR) $(LIB_OBJS) $(TEST_BINS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Assembly entry point
$(CRT0_OBJ): $(LIB_DIR)/crt0_$(ARCH).S | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# CRT C++ (compiled as C)
$(CRT_CPP_OBJ): $(LIB_DIR)/crt_cpp.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# C library objects
$(BUILD_DIR)/%.o: $(LIB_DIR)/%.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# C test binaries
$(BUILD_DIR)/%: $(TEST_DIR)/%.c $(LIB_OBJS)
	$(CC) $(CFLAGS) -c $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) -o $@ $(CRT0_OBJ) $(CRT_CPP_OBJ) $(BUILD_DIR)/$*.o $(C_LIB_OBJS) $(CPP_LIB_OBJS)

# C++ test binaries
$(BUILD_DIR)/%: $(TEST_DIR)/%.cpp $(LIB_OBJS)
	$(CXX) $(CXXFLAGS) -c $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) -o $@ $(CRT0_OBJ) $(CRT_CPP_OBJ) $(BUILD_DIR)/$*.o $(C_LIB_OBJS) $(CPP_LIB_OBJS)

clean:
	rm -rf $(BUILD_DIR)
