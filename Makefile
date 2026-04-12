# Alloy Kernel Makefile

# Architecture
ARCH ?= x86_64
TARGET ?= i686-alloy

# Cross-compiler toolchain
AS = nasm
CC = $(HOME)/.local/i686-elf/bin/i686-elf-gcc
CXX = $(HOME)/.local/i686-elf/bin/i686-elf-g++
LD = $(HOME)/.local/i686-elf/bin/i686-elf-ld
RUSTC = rustc
CARGO = $(HOME)/.cargo/bin/cargo

# Flags
ASFLAGS = -f elf32
CFLAGS = -m32 -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ikernel/cpp
CXXFLAGS = $(CFLAGS) -fno-use-cxa-atexit
LDFLAGS = -m elf_i386 -T kernel/linker.ld

# Directories
BUILD_DIR = build
BOOT_DIR = boot
KERNEL_CPP_DIR = kernel/cpp
KERNEL_RUST_DIR = kernel/rust
ARCH_DIR = $(KERNEL_CPP_DIR)/arch/$(ARCH)
DRIVERS_DIR = $(KERNEL_CPP_DIR)/drivers
MM_DIR = $(KERNEL_CPP_DIR)/mm
RUST_FFI_DIR = $(KERNEL_CPP_DIR)/rust

# Source files
ASM_SOURCES = $(BOOT_DIR)/multiboot2.asm \
              $(BOOT_DIR)/boot.asm \
              $(ARCH_DIR)/gdt_flush.asm \
              $(ARCH_DIR)/idt_stubs.asm \
              $(ARCH_DIR)/context_switch.asm \
              $(ARCH_DIR)/syscall_entry.asm

CPP_SOURCES = $(KERNEL_CPP_DIR)/boot/main.cpp \
              $(KERNEL_CPP_DIR)/arch/cpu.cpp \
              $(KERNEL_CPP_DIR)/arch/syscall.cpp \
              $(ARCH_DIR)/gdt.cpp \
              $(ARCH_DIR)/idt.cpp \
              $(DRIVERS_DIR)/serial.cpp \
              $(DRIVERS_DIR)/vga.cpp \
              $(DRIVERS_DIR)/vesa.cpp \
              $(DRIVERS_DIR)/keyboard.cpp \
              $(DRIVERS_DIR)/timer.cpp \
              $(MM_DIR)/pmm.cpp \
              $(MM_DIR)/paging.cpp \
              $(MM_DIR)/vmm.cpp

# Object files
ASM_OBJECTS = $(patsubst %.asm,$(BUILD_DIR)/%.o,$(ASM_SOURCES))
CPP_OBJECTS = $(patsubst %.cpp,$(BUILD_DIR)/%.o,$(CPP_SOURCES))
RUST_LIB = $(BUILD_DIR)/kernel/rust/liballoy_kernel_rust.a
OBJECTS = $(ASM_OBJECTS) $(CPP_OBJECTS)

# Output
KERNEL_ELF = $(BUILD_DIR)/alloy.elf
KERNEL_ISO = $(BUILD_DIR)/alloy.iso

.PHONY: all clean run iso output screenshot debug

all: $(KERNEL_ELF)

iso: $(KERNEL_ISO)

# Link kernel
$(KERNEL_ELF): $(OBJECTS) $(RUST_LIB)
	@echo "Linking kernel..."
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -o $@ $(OBJECTS) $(RUST_LIB)
	@echo "Kernel built successfully: $@"

# Build Rust library
$(RUST_LIB): $(shell find $(KERNEL_RUST_DIR)/src -name '*.rs')
	@echo "Building Rust kernel library..."
	@mkdir -p $(BUILD_DIR)/kernel/rust
	cd $(KERNEL_RUST_DIR) && $(CARGO) +nightly build --release --target i686-alloy.json -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec
	@cp $(KERNEL_RUST_DIR)/target/i686-alloy/release/liballoy_kernel_rust.a $(RUST_LIB)
	@echo "Rust library built: $(RUST_LIB)"

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
	qemu-system-i386 -cdrom $(KERNEL_ISO) -serial stdio -no-reboot -no-shutdown -D qemu.log

output: $(KERNEL_ISO)
	qemu-system-i386 -cdrom $(KERNEL_ISO) -serial stdio -display none -no-reboot -no-shutdown -D qemu.log

# Boot headless and auto-capture desktop shell screenshot (PNG)
screenshot: $(KERNEL_ISO)
	python3 tools/capture_desktop_screenshot.py --iso $(KERNEL_ISO) --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5

# Run in QEMU with debugging
debug: $(KERNEL_ISO)
	qemu-system-i386 -cdrom $(KERNEL_ISO) -serial stdio -s -S

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	cd $(KERNEL_RUST_DIR) && $(CARGO) clean

# Im lazy
fuck:
	@echo "Doing all dat for you."
	rm -rf $(BUILD_DIR)
	cd $(KERNEL_RUST_DIR) && $(CARGO) clean
	@echo "Cleaned your shitass code, compiling the iso."
	make iso
	@echo "Run 'make run' to test the kernel, bozo really though id do everything for you."

# Print variables for debugging
print-%:
	@echo $* = $($*)
