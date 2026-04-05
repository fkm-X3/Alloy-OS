# Alloy Kernel Makefile

# Architecture
ARCH ?= x86_64

# Cross-compiler toolchain
AS = nasm
CC = i686-elf-gcc
CXX = i686-elf-g++
LD = i686-elf-ld

# Flags
ASFLAGS = -f elf32
CFLAGS = -m32 -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ikernel/cpp
CXXFLAGS = $(CFLAGS) -fno-use-cxa-atexit
LDFLAGS = -m elf_i386 -T kernel/linker.ld

# Directories
BUILD_DIR = build
BOOT_DIR = boot
KERNEL_CPP_DIR = kernel/cpp
ARCH_DIR = $(KERNEL_CPP_DIR)/arch/$(ARCH)
DRIVERS_DIR = $(KERNEL_CPP_DIR)/drivers

# Source files
ASM_SOURCES = $(BOOT_DIR)/multiboot2.asm \
              $(BOOT_DIR)/boot.asm \
              $(ARCH_DIR)/gdt_flush.asm \
              $(ARCH_DIR)/idt_stubs.asm

CPP_SOURCES = $(KERNEL_CPP_DIR)/boot/main.cpp \
              $(ARCH_DIR)/gdt.cpp \
              $(ARCH_DIR)/idt.cpp \
              $(DRIVERS_DIR)/serial.cpp \
              $(DRIVERS_DIR)/vga.cpp \
              $(DRIVERS_DIR)/keyboard.cpp

# Object files
ASM_OBJECTS = $(patsubst %.asm,$(BUILD_DIR)/%.o,$(ASM_SOURCES))
CPP_OBJECTS = $(patsubst %.cpp,$(BUILD_DIR)/%.o,$(CPP_SOURCES))
OBJECTS = $(ASM_OBJECTS) $(CPP_OBJECTS)

# Output
KERNEL_ELF = $(BUILD_DIR)/alloy.elf
KERNEL_ISO = $(BUILD_DIR)/alloy.iso

.PHONY: all clean run iso

all: $(KERNEL_ELF)

iso: $(KERNEL_ISO)

# Link kernel
$(KERNEL_ELF): $(OBJECTS)
	@echo "Linking kernel..."
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -o $@ $(OBJECTS)
	@echo "Kernel built successfully: $@"

# Assemble .asm files
$(BUILD_DIR)/%.o: %.asm
	@echo "Assembling $<..."
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) $< -o $@

# Compile .cpp files
$(BUILD_DIR)/%.o: %.cpp
	@echo "Compiling $<..."
	@mkdir -p $(dir $@)
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Create bootable ISO
$(KERNEL_ISO): $(KERNEL_ELF)
	@echo "Creating ISO image..."
	@mkdir -p $(BUILD_DIR)/isodir/boot/grub
	@cp $(KERNEL_ELF) $(BUILD_DIR)/isodir/boot/alloy.elf
	@cp $(BOOT_DIR)/grub.cfg $(BUILD_DIR)/isodir/boot/grub/
	grub-mkrescue -o $@ $(BUILD_DIR)/isodir
	@echo "ISO created: $@"

# Run in QEMU
run: $(KERNEL_ISO)
	qemu-system-i386 -cdrom $(KERNEL_ISO) -serial stdio -d int -no-reboot -no-shutdown

# Run in QEMU with debugging
debug: $(KERNEL_ISO)
	qemu-system-i386 -cdrom $(KERNEL_ISO) -serial stdio -s -S

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)

# Print variables for debugging
print-%:
	@echo $* = $($*)
