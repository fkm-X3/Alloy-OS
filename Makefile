# Alloy Kernel Makefile

# Architecture selection (default: i686 for testing)
# Supported: i686, x86_64 (placeholder), aarch64 (minimal)
ARCH ?= i686

# Architecture-specific configuration
ifeq ($(ARCH),i686)
    TARGET = i686-alloy
    CROSS_PREFIX = $(HOME)/.local/i686-elf/bin/i686-elf-
    AS = nasm
    ASFLAGS = -f elf32
    CFLAGS_ARCH = -m32 -DARCH_I686
    LDFLAGS_ARCH = -m elf_i386
    QEMU = qemu-system-i386
    QEMU_FLAGS = -serial stdio
    RUST_TARGET = i686-alloy.json
    RUST_FEATURES = --features i686
    LINKER = kernel/linker.ld
    BOOT_ASM = $(BOOT_DIR)/multiboot2.asm $(BOOT_DIR)/boot.asm
    ARCH_ASM = $(ARCH_DIR)/gdt_flush.asm $(ARCH_DIR)/idt_stubs.asm $(ARCH_DIR)/context_switch.asm $(ARCH_DIR)/syscall_entry.asm
else ifeq ($(ARCH),x86_64)
    TARGET = x86_64-alloy
    CROSS_PREFIX = $(HOME)/.local/x86_64-elf/bin/x86_64-elf-
    AS = nasm
    ASFLAGS = -f elf64
    CFLAGS_ARCH = -m64 -DARCH_X86_64
    LDFLAGS_ARCH = -m elf_x86_64
    QEMU = qemu-system-x86_64
    QEMU_FLAGS = -serial stdio
    RUST_TARGET = x86_64-alloy.json
    RUST_FEATURES = --features x86_64
    LINKER = kernel/linker_x86_64.ld
    BOOT_ASM = $(BOOT_DIR)/multiboot2.asm $(BOOT_DIR)/boot_x86_64.asm
    ARCH_ASM = $(ARCH_DIR)/gdt_flush.asm $(ARCH_DIR)/idt_stubs.asm $(ARCH_DIR)/context_switch.asm $(ARCH_DIR)/syscall_entry.asm
else ifeq ($(ARCH),aarch64)
    TARGET = aarch64-alloy
    CROSS_PREFIX = $(HOME)/.local/aarch64-elf/bin/aarch64-elf-
    AS = $(CROSS_PREFIX)gcc
    ASFLAGS = -c -march=armv8-a
    CFLAGS_ARCH = -march=armv8-a -DARCH_AARCH64
    LDFLAGS_ARCH = -m aarch64elf
    QEMU = qemu-system-aarch64
    QEMU_FLAGS = -machine virt -cpu cortex-a53 -serial stdio -kernel
    RUST_TARGET = aarch64-alloy.json
    RUST_FEATURES = --features aarch64
    LINKER = kernel/linker_aarch64.ld
    BOOT_ASM = $(BOOT_DIR)/boot_aarch64.S
    ARCH_ASM = $(ARCH_DIR)/context_switch.S $(ARCH_DIR)/exception_vectors.S
else
    $(error Unsupported architecture: $(ARCH). Use i686, x86_64, or aarch64)
endif

# Cross-compiler toolchain
CC = $(CROSS_PREFIX)gcc
CXX = $(CROSS_PREFIX)g++
LD = $(CROSS_PREFIX)ld
RUSTC = rustc
CARGO = $(HOME)/.cargo/bin/cargo

# Flags
CFLAGS = $(CFLAGS_ARCH) -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ikernel/cpp
CXXFLAGS = $(CFLAGS) -fno-use-cxa-atexit
LDFLAGS = $(LDFLAGS_ARCH) -T $(LINKER)

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
ASM_SOURCES = $(BOOT_ASM) $(ARCH_ASM)

CPP_SOURCES = $(KERNEL_CPP_DIR)/boot/main.cpp \
              $(KERNEL_CPP_DIR)/arch/cpu.cpp \
              $(KERNEL_CPP_DIR)/arch/syscall.cpp \
              $(ARCH_DIR)/gdt.cpp \
              $(ARCH_DIR)/idt.cpp \
              $(DRIVERS_DIR)/serial.cpp \
              $(DRIVERS_DIR)/vga.cpp \
              $(DRIVERS_DIR)/vesa.cpp \
              $(DRIVERS_DIR)/keyboard.cpp \
              $(DRIVERS_DIR)/mouse.cpp \
              $(DRIVERS_DIR)/timer.cpp \
              $(MM_DIR)/pmm.cpp \
              $(MM_DIR)/paging.cpp \
              $(MM_DIR)/vmm.cpp

# Object files
ASM_OBJECTS = $(patsubst %.asm,$(BUILD_DIR)/%.o,$(filter %.asm,$(ASM_SOURCES)))
ASM_OBJECTS += $(patsubst %.S,$(BUILD_DIR)/%.o,$(filter %.S,$(ASM_SOURCES)))
CPP_OBJECTS = $(patsubst %.cpp,$(BUILD_DIR)/%.o,$(CPP_SOURCES))
RUST_LIB = $(BUILD_DIR)/kernel/rust/liballoy_kernel_rust.a
OBJECTS = $(ASM_OBJECTS) $(CPP_OBJECTS)

# Output
KERNEL_ELF = $(BUILD_DIR)/alloy.elf
KERNEL_ISO = $(BUILD_DIR)/alloy.iso

.PHONY: all clean run iso output screenshot mouse-smoke mouse-screenshot debug review-install review docker-build docker-run print-arch

all: $(KERNEL_ELF)

iso: $(KERNEL_ISO)

# Link kernel
$(KERNEL_ELF): $(OBJECTS) $(RUST_LIB)
	@echo "Linking kernel ($(ARCH))..."
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -o $@ $(OBJECTS) $(RUST_LIB)
	@echo "Kernel built successfully: $@"

# Build Rust library
$(RUST_LIB): $(shell find $(KERNEL_RUST_DIR)/src -name '*.rs')
	@echo "Building Rust kernel library ($(ARCH))..."
	@mkdir -p $(BUILD_DIR)/kernel/rust
	cd $(KERNEL_RUST_DIR) && $(CARGO) +nightly build --release --target $(RUST_TARGET) -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec
	@cp $(KERNEL_RUST_DIR)/target/$(TARGET)/release/liballoy_kernel_rust.a $(RUST_LIB)
	@echo "Rust library built: $(RUST_LIB)"

# Assemble .asm files (NASM)
$(BUILD_DIR)/%.o: %.asm
	@echo "Assembling $<..."
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) $< -o $@

# Assemble .S files (GAS)
$(BUILD_DIR)/%.o: %.S
	@echo "Assembling $<..."
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) $< -o $@

# Compile .cpp files
$(BUILD_DIR)/%.o: %.cpp
	@echo "Compiling $<..."
	@mkdir -p $(dir $@)
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Create bootable ISO (x86 only)
$(KERNEL_ISO): $(KERNEL_ELF)
	@echo "Creating ISO image..."
	@mkdir -p $(BUILD_DIR)/isodir/boot/grub
	@cp $(KERNEL_ELF) $(BUILD_DIR)/isodir/boot/alloy.elf
	@cp $(BOOT_DIR)/grub.cfg $(BUILD_DIR)/isodir/boot/grub/
	grub-mkrescue -o $@ $(BUILD_DIR)/isodir
	@echo "ISO created: $@"

# Run in QEMU
run: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log

output: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log

# Boot headless and auto-capture desktop shell screenshot (PNG)
screenshot: $(KERNEL_ISO)
	python3 tools/capture_desktop_screenshot.py --iso $(KERNEL_ISO) --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5

# Boot headless and run scripted mouse interactions (no screenshot)
mouse-smoke: $(KERNEL_ISO)
	python3 tools/mouse_smoke.py --iso $(KERNEL_ISO) --serial-log $(BUILD_DIR)/mouse-smoke-boot.log --qemu-log $(BUILD_DIR)/qemu-mouse-smoke.log

# Run scripted mouse interactions and capture a screenshot artifact
mouse-screenshot: $(KERNEL_ISO)
	python3 tools/mouse_smoke.py --iso $(KERNEL_ISO) --serial-log $(BUILD_DIR)/mouse-screenshot-boot.log --qemu-log $(BUILD_DIR)/mouse-screenshot.log --screenshot $(BUILD_DIR)/mouse-smoke.png .

# Run in QEMU with debugging
debug: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -s -S

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	cd $(KERNEL_RUST_DIR) && $(CARGO) clean

# Im lazy
lazy:
	@echo "Doing all dat for you."
	rm -rf $(BUILD_DIR)
	cd $(KERNEL_RUST_DIR) && $(CARGO) clean
	@echo "Cleaned your shitass code, compiling the iso."
	make iso
	@echo "Run 'make run' to test."

# Print architecture configuration
print-arch:
	@echo "ARCH = $(ARCH)"
	@echo "TARGET = $(TARGET)"
	@echo "CC = $(CC)"
	@echo "CXX = $(CXX)"
	@echo "LD = $(LD)"
	@echo "AS = $(AS)"
	@echo "ASFLAGS = $(ASFLAGS)"
	@echo "CFLAGS_ARCH = $(CFLAGS_ARCH)"
	@echo "LDFLAGS_ARCH = $(LDFLAGS_ARCH)"
	@echo "QEMU = $(QEMU)"
	@echo "RUST_TARGET = $(RUST_TARGET)"
	@echo "RUST_FEATURES = $(RUST_FEATURES)"
	@echo "ARCH_DIR = $(ARCH_DIR)"

# Print variables for debugging
print-%:
	@echo $* = $($*)

# Docker build: Create image
docker-build:
	@echo "Building Alloy OS Docker image..."
	docker build -t alloy-os-dev:latest .
	@echo "Docker image built: alloy-os-dev:latest"

docker-build-prod:
		@echo "Building Alloy OS Docker image..."
		docker build -t alloy-os:latest .
		@echo "Docker image built: alloy-os:latest"

# Docker run: Start container
docker-run: docker-build
	@echo "Starting Alloy OS container..."
	docker compose run --rm -it alloy

docker-run-prod: docker-build
	@echo "Starting Alloy OS container..."
	docker run --rm -it \
		-p 22:22 \
		-v "$(CURDIR):/workspace" \
		-v "$(HOME)/.local/i686-elf:/root/.local/i686-elf" \
		-v "$(HOME)/.cargo/registry:/root/.cargo/registry" \
		-v "$(HOME)/.cargo/git:/root/.cargo/git" \
		-w /workspace \
		--name alloy-os \
		alloy-os:latest \
		bash
