# Alloy Kernel Makefile

# Architecture selection (default: x86_64)
# Supported: x86_64, aarch64
ARCH ?= x86_64

# Architecture-specific configuration
ifeq ($(ARCH),x86_64)
    TARGET = x86_64-alloy
    CC = gcc
    LD = ld
    AS = nasm
    ASFLAGS = -f elf64 -dARCH_X86_64
    CFLAGS_ARCH = -m64 -DARCH_X86_64 -mno-sse -mno-sse2 -mno-mmx -mno-avx -mno-80387 -mno-fp-ret-in-387
    LDFLAGS_ARCH = -m elf_x86_64
    QEMU = qemu-system-x86_64
    QEMU_FLAGS = -serial stdio
    RUST_TARGET = x86_64-alloy.json
    RUST_FEATURES = --no-default-features --features x86_64
    LINKER = kernel/linker_x86_64.ld
    BOOT_ASM = $(BOOT_DIR)/multiboot2.asm $(BOOT_DIR)/boot_x86_64.asm
    ARCH_ASM = $(ARCH_DIR)/gdt_flush.asm $(ARCH_DIR)/idt_stubs.asm $(ARCH_DIR)/context_switch.asm $(ARCH_DIR)/syscall_entry.asm
else ifeq ($(ARCH),aarch64)
    TARGET = aarch64-alloy
    CC = aarch64-linux-gnu-gcc
    LD = aarch64-linux-gnu-ld
    AS = aarch64-linux-gnu-gcc
    ASFLAGS = -c -march=armv8-a
    CFLAGS_ARCH = -march=armv8-a -DARCH_AARCH64
    LDFLAGS_ARCH = -m aarch64elf
    QEMU = qemu-system-aarch64
    QEMU_FLAGS = -machine virt -cpu cortex-a53 -serial stdio
    QEMU_FW = $(shell \
        for f in /usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
                  /usr/share/AAVMF/AAVMF_CODE.fd \
                  /usr/share/edk2/aarch64/QEMU_EFI.fd \
                  /opt/homebrew/share/qemu-efi-aarch64/QEMU_EFI.fd \
                  C:/msys64/usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
                  C:/Program\ */qemu/share/qemu-efi-aarch64/QEMU_EFI.fd; do \
            if [ -f "$$f" ]; then echo "-bios $$f"; break; fi; \
        done)
    RUST_TARGET = aarch64-alloy.json
    RUST_FEATURES = --no-default-features --features aarch64
    LINKER = kernel/linker_aarch64.ld
    BOOT_ASM = $(BOOT_DIR)/boot_aarch64.S
    ARCH_ASM = $(ARCH_DIR)/context_switch.S $(ARCH_DIR)/exception_vectors.S
else
    $(error Unsupported architecture: $(ARCH). Use x86_64 or aarch64)
endif
RUSTC = rustc
CARGO = $(HOME)/.cargo/bin/cargo

# Flags
CFLAGS = $(CFLAGS_ARCH) -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ikernel/c
LDFLAGS = $(LDFLAGS_ARCH) -T $(LINKER)

# Directories
BUILD_DIR = build
BOOT_DIR = boot
KERNEL_C_DIR = kernel/c
USERLAND_DIR = os/userland
KERNEL_RUST_DIR = kernel/rust
ARCH_DIR = $(KERNEL_C_DIR)/arch/$(ARCH)
DRIVERS_DIR = $(KERNEL_C_DIR)/drivers
MM_DIR = $(KERNEL_C_DIR)/mm

# Source files (common)
ASM_SOURCES = $(BOOT_ASM) $(ARCH_ASM)

C_SOURCES = $(KERNEL_C_DIR)/arch/cpu.c \
            $(KERNEL_C_DIR)/arch/syscall.c \
            $(ARCH_DIR)/gdt.c \
            $(ARCH_DIR)/idt.c \
            $(MM_DIR)/pmm.c \
            $(MM_DIR)/vmm.c \
            $(DRIVERS_DIR)/serial.c \
            $(DRIVERS_DIR)/timer.c

# Architecture-specific C sources
ifeq ($(ARCH),aarch64)
C_SOURCES += $(DRIVERS_DIR)/pl110.c \
             $(MM_DIR)/paging_aarch64.c \
             $(KERNEL_C_DIR)/boot/main_aarch64.c
else
C_SOURCES += $(KERNEL_C_DIR)/boot/main.c \
             $(MM_DIR)/paging.c \
             $(DRIVERS_DIR)/vga.c \
             $(DRIVERS_DIR)/vesa.c \
             $(DRIVERS_DIR)/keyboard.c \
             $(DRIVERS_DIR)/mouse.c \
             $(DRIVERS_DIR)/ata.c \
             $(DRIVERS_DIR)/pci.c \
             $(DRIVERS_DIR)/ahci.c \
             $(DRIVERS_DIR)/initrd.c
endif

# Object files
ASM_OBJECTS = $(patsubst %.asm,$(BUILD_DIR)/%.o,$(filter %.asm,$(ASM_SOURCES)))
ASM_OBJECTS += $(patsubst %.S,$(BUILD_DIR)/%.o,$(filter %.S,$(ASM_SOURCES)))
C_OBJECTS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(C_SOURCES))
RUST_LIB = $(BUILD_DIR)/kernel/rust/liballoy_kernel_rust.a
OBJECTS = $(ASM_OBJECTS) $(C_OBJECTS)

# Output
KERNEL_ELF = $(BUILD_DIR)/alloy.elf
KERNEL_ISO = $(BUILD_DIR)/alloy.iso

.PHONY: all clean run iso output screenshot screenshot-elf mouse-smoke mouse-smoke-elf mouse-screenshot debug debug-elf review-install review docker-build docker-run print-arch userland de-build run-de run-elf output-elf

all: userland de-build $(KERNEL_ELF)

iso: $(KERNEL_ISO)

# Build userland binaries (embedded into kernel VFS via include_bytes!)
userland:
	$(MAKE) -C os/userland ARCH=$(ARCH)
	cp -f os/userland/build/$(ARCH)/hello hello 2>/dev/null || true
	cp -f os/userland/build/$(ARCH)/compositor compositor 2>/dev/null || true
	cp -f os/userland/build/$(ARCH)/test_wl_client test_wl_client 2>/dev/null || true
ifneq ($(ARCH),aarch64)
	cp -f os/userland/build/$(ARCH)/hello_cpp hello_cpp 2>/dev/null || true
	cp -f os/userland/build/$(ARCH)/test_qml test_window 2>/dev/null || true
endif

# Cross-compile the Qt6/QML DE for Alloy OS (requires Qt6 at /opt/alloy/qt6)
# DE is x86_64-only; skipped on aarch64 until Qt6 cross-compilation is ported.
DE_OUT = de/build/alloy_de_qml

de-build: userland
ifeq ($(ARCH),aarch64)
	@echo "Skipping DE build (x86_64-only, ARCH=$(ARCH))"
else
	@echo "Building DE (cross-compile for Alloy OS x86_64)..."
	@mkdir -p de/build
	cd de && cmake -B build -DCMAKE_BUILD_TYPE=Release
	cmake --build de/build --target alloy_de_qml
	@cp $(DE_OUT) alloy_de_qml
	@echo "DE binary at: $(DE_OUT) -> alloy_de_qml"
endif

# Run the DE on the host for development/testing
run-de:
	@echo "Running DE (host)..."
	cd de && cmake -B build-host -DCMAKE_BUILD_TYPE=Release && cmake --build build-host
	cd de/build-host && ./AlloyDE

# Link kernel
$(KERNEL_ELF): userland $(OBJECTS) $(RUST_LIB)
	@echo "Linking kernel ($(ARCH))..."
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -o $@ $(OBJECTS) $(RUST_LIB)
	@echo "Kernel built successfully: $@"

# Build Rust library
$(RUST_LIB): userland $(shell find $(KERNEL_RUST_DIR)/src -name '*.rs')
	@echo "Building Rust kernel library ($(ARCH))..."
	@mkdir -p $(BUILD_DIR)/kernel/rust
	cd $(KERNEL_RUST_DIR) && $(CARGO) +nightly build --release --target $(RUST_TARGET) $(RUST_FEATURES) -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec
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

# Compile .c files
$(BUILD_DIR)/%.o: %.c
	@echo "Compiling $<..."
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

# Create bootable ISO (x86 only)
$(KERNEL_ISO): $(KERNEL_ELF)
	@echo "Creating ISO image..."
	@mkdir -p $(BUILD_DIR)/isodir/boot/grub
	@cp $(KERNEL_ELF) $(BUILD_DIR)/isodir/boot/alloy.elf
	@cp $(BOOT_DIR)/grub.cfg $(BUILD_DIR)/isodir/boot/grub/
	grub-mkrescue -o $@ $(BUILD_DIR)/isodir
	@echo "ISO created: $@"

# Run in QEMU
# Disk image for storage testing
DISK_IMG = $(BUILD_DIR)/disk.img
DISK_SIZE_MB = 64

$(DISK_IMG):
	@echo "Creating $(DISK_SIZE_MB)MB disk image..."
	@mkdir -p $(BUILD_DIR)
	qemu-img create -f raw $@ $(DISK_SIZE_MB)M 2>/dev/null || dd if=/dev/zero of=$@ bs=1M count=$(DISK_SIZE_MB) 2>/dev/null

# Boot via ISO (x86: GRUB on ISO; aarch64: ISO + UEFI firmware)
run: $(KERNEL_ISO) $(DISK_IMG)
ifeq ($(ARCH),aarch64)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) $(QEMU_FW) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
else
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
endif

run-ahci: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -machine q35 -drive file=$(DISK_IMG),format=raw,if=none,id=disk -device ahci,id=ahci -device ide-hd,drive=disk,bus=ahci.0

output: $(KERNEL_ISO) $(DISK_IMG)
ifeq ($(ARCH),aarch64)
	$(QEMU) -cdrom $(KERNEL_ISO) -display none $(QEMU_FLAGS) $(QEMU_FW) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
else
	$(QEMU) -cdrom $(KERNEL_ISO) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
endif

output-ahci: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -machine q35 -drive file=$(DISK_IMG),format=raw,if=none,id=disk -device ahci,id=ahci -device ide-hd,drive=disk,bus=ahci.0

# Boot kernel ELF directly (no ISO/GRUB) – works for all arches
run-elf: $(KERNEL_ELF) $(DISK_IMG)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) $(QEMU_FW) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide

output-elf: $(KERNEL_ELF) $(DISK_IMG)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) $(QEMU_FW) -display none -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide

# Boot headless and auto-capture desktop shell screenshot (PNG)
screenshot: $(KERNEL_ISO)
ifeq ($(ARCH),aarch64)
	python3 tools/capture_desktop_screenshot.py --iso $(KERNEL_ISO) --bios '$(QEMU_FW)' --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5
else
	python3 tools/capture_desktop_screenshot.py --iso $(KERNEL_ISO) --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5
endif

# Boot headless with kernel ELF and capture screenshot (works for all arches)
screenshot-elf: $(KERNEL_ELF)
	python3 tools/capture_desktop_screenshot.py --kernel $(KERNEL_ELF) --bios '$(QEMU_FW)' --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5

# Boot headless and run scripted mouse interactions (no screenshot)
mouse-smoke: $(KERNEL_ISO)
ifeq ($(ARCH),aarch64)
	python3 tools/mouse_smoke.py --iso $(KERNEL_ISO) --bios '$(QEMU_FW)' --serial-log $(BUILD_DIR)/mouse-smoke-boot.log --qemu-log $(BUILD_DIR)/qemu-mouse-smoke.log
else
	python3 tools/mouse_smoke.py --iso $(KERNEL_ISO) --serial-log $(BUILD_DIR)/mouse-smoke-boot.log --qemu-log $(BUILD_DIR)/qemu-mouse-smoke.log
endif

# Boot headless with kernel ELF and run scripted mouse interactions
mouse-smoke-elf: $(KERNEL_ELF)
	python3 tools/mouse_smoke.py --kernel $(KERNEL_ELF) --bios '$(QEMU_FW)' --serial-log $(BUILD_DIR)/mouse-smoke-boot.log --qemu-log $(BUILD_DIR)/qemu-mouse-smoke.log

# Run scripted mouse interactions and capture a screenshot artifact
mouse-screenshot: $(KERNEL_ISO)
ifeq ($(ARCH),aarch64)
	python3 tools/mouse_smoke.py --iso $(KERNEL_ISO) --bios '$(QEMU_FW)' --serial-log $(BUILD_DIR)/mouse-screenshot-boot.log --qemu-log $(BUILD_DIR)/mouse-screenshot.log --screenshot $(BUILD_DIR)/mouse-smoke.png .
else
	python3 tools/mouse_smoke.py --iso $(KERNEL_ISO) --serial-log $(BUILD_DIR)/mouse-screenshot-boot.log --qemu-log $(BUILD_DIR)/mouse-screenshot.log --screenshot $(BUILD_DIR)/mouse-smoke.png .
endif

# Run in QEMU with debugging
debug: $(KERNEL_ISO)
ifeq ($(ARCH),aarch64)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) $(QEMU_FW) -s -S
else
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -s -S
endif

debug-elf: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) $(QEMU_FW) -s -S

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	cd $(KERNEL_RUST_DIR) && $(CARGO) clean
	rm -rf de/build de/build-host
	rm -f hello compositor test_window alloy_de_qml

# Create a FAT32 disk image (requires mkfs.fat in PATH)
fat32-img: $(BUILD_DIR)
	@echo "Creating FAT32 disk image..."
	qemu-img create -f raw $(BUILD_DIR)/fat32.img 64M 2>/dev/null || \
	  dd if=/dev/zero of=$(BUILD_DIR)/fat32.img bs=1M count=64 2>/dev/null
	mkfs.fat -F 32 $(BUILD_DIR)/fat32.img 2>/dev/null || \
	  echo "WARNING: mkfs.fat not found. Install dosfstools. Created empty image."
	@echo "FAT32 image: $(BUILD_DIR)/fat32.img"

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

# Docker build: Create image for the selected ARCH
docker-build:
	@echo "Building Alloy OS Docker image for $(ARCH)..."
	docker build -t alloy-os-dev-$(ARCH):latest -f Dockerfile.$(ARCH) .
	@echo "Docker image built: alloy-os-dev-$(ARCH):latest"

docker-build-prod:
	@echo "Building Alloy OS Docker image for $(ARCH)..."
	docker build -t alloy-os-$(ARCH):latest -f Dockerfile.$(ARCH) .
	@echo "Docker image built: alloy-os-$(ARCH):latest"

# Docker run: Start container
docker-run: docker-build
	@echo "Starting Alloy OS container for $(ARCH)..."
	ALLOY_ARCH=$(ARCH) docker compose run --rm -it alloy

docker-run-prod: docker-build
	@echo "Starting Alloy OS container for $(ARCH)..."
	docker run --rm -it \
		-p 22:22 \
		-v "$(CURDIR):/workspace" \
		-v "$(HOME)/.local/x86_64-elf:/root/.local/x86_64-elf" \
		-v "$(HOME)/.cargo/registry:/root/.cargo/registry" \
		-v "$(HOME)/.cargo/git:/root/.cargo/git" \
		-w /workspace \
		--name alloy-os-$(ARCH) \
		alloy-os-$(ARCH):latest \
		bash
