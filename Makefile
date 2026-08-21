# Alloy Kernel Makefile

# Architecture selection (default: x86_64)
# Supported: x86_64, aarch64
ARCH ?= x86_64

# Architecture-specific configuration
ifeq ($(ARCH),x86_64)
    TARGET = x86_64-alloy
    LD = ld
    AS = nasm
    ASFLAGS = -f elf64 -dARCH_X86_64
    LDFLAGS_ARCH = -m elf_x86_64
    QEMU = qemu-system-x86_64
    QEMU_FLAGS = -serial stdio
    RUST_TARGET = x86_64-alloy.json
    RUST_FEATURES = --no-default-features --features x86_64
    LINKER = kernel/linker_x86_64.ld
    BOOT_ASM = $(BOOT_DIR)/multiboot2.asm $(BOOT_DIR)/boot_x86_64.asm
    ARCH_ASM = $(ARCH_ASM_DIR)/gdt_flush.asm $(ARCH_ASM_DIR)/idt_stubs.asm $(ARCH_ASM_DIR)/context_switch.asm $(ARCH_ASM_DIR)/syscall_entry.asm
else ifeq ($(ARCH),aarch64)
    TARGET = aarch64-alloy
    LD = aarch64-linux-gnu-ld
    AS = aarch64-linux-gnu-gcc
    ASFLAGS = -c -march=armv8-a
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
    ARCH_ASM = $(ARCH_ASM_DIR)/context_switch.S $(ARCH_ASM_DIR)/exception_vectors.S
else
    $(error Unsupported architecture: $(ARCH). Use x86_64 or aarch64)
endif
RUSTC = rustc
CARGO = $(HOME)/.cargo/bin/cargo

# Flags
LDFLAGS = $(LDFLAGS_ARCH) -T $(LINKER)

# Directories
BUILD_DIR = build
BOOT_DIR = boot
USERLAND_DIR = os/userland
KERNEL_RUST_DIR = kernel/rust
ARCH_ASM_DIR = kernel/asm/$(ARCH)

# Source files (common)
ASM_SOURCES = $(BOOT_ASM) $(ARCH_ASM)

# Object files
ASM_OBJECTS = $(patsubst %.asm,$(BUILD_DIR)/%.o,$(filter %.asm,$(ASM_SOURCES)))
ASM_OBJECTS += $(patsubst %.S,$(BUILD_DIR)/%.o,$(filter %.S,$(ASM_SOURCES)))
RUST_LIB = $(BUILD_DIR)/kernel/rust/liballoy_kernel_rust.a
OBJECTS = $(ASM_OBJECTS)

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
	cp -f os/userland/build/$(ARCH)/forktest forktest 2>/dev/null || true
ifneq ($(ARCH),aarch64)
	cp -f os/userland/build/$(ARCH)/hello_cpp hello_cpp 2>/dev/null || touch hello_cpp
	cp -f os/userland/build/$(ARCH)/test_qml test_window 2>/dev/null || touch test_window
else
	@touch hello_cpp test_window
endif

# Cross-compile the Qt6/QML DE for Alloy OS (requires Qt6 at /opt/alloy/qt6)
# DE is x86_64-only; skipped on aarch64 until Qt6 cross-compilation is ported.
DE_OUT = de/build/alloy_de_qml
QT_HOST_TOOLS ?=
QT_HOST_PATH ?=
CMAKE_QT_HOST_FLAGS = $(if $(QT_HOST_TOOLS),-DQT_HOST_TOOLS_DIR=$(QT_HOST_TOOLS)) \
                      $(if $(QT_HOST_PATH),-DQT_HOST_PATH=$(QT_HOST_PATH))

de-build: userland
ifeq ($(ARCH),aarch64)
	@echo "Skipping DE build (x86_64-only, ARCH=$(ARCH))"
	@touch alloy_de_qml
else
	@echo "Building DE (cross-compile for Alloy OS x86_64)..."
	@mkdir -p de/build
	cd de && cmake -B build -DCMAKE_BUILD_TYPE=Release $(CMAKE_QT_HOST_FLAGS) || true
	cmake --build de/build --target alloy_de_qml || true
	@cp $(DE_OUT) alloy_de_qml 2>/dev/null || touch alloy_de_qml
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

# Boot via ISO (x86: GRUB on ISO) or bare ELF via -kernel (aarch64: QEMU
# virt has no BIOS/IDE; the kernel is loaded at 0x40080000 by the -kernel path)
ifeq ($(ARCH),aarch64)
run: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -no-reboot -no-shutdown -D qemu.log
else
run: $(KERNEL_ISO) $(DISK_IMG)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
endif

run-ahci: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -machine q35 -drive file=$(DISK_IMG),format=raw,if=none,id=disk -device ahci,id=ahci -device ide-hd,drive=disk,bus=ahci.0

ifeq ($(ARCH),aarch64)
output: $(KERNEL_ELF)
	$(QEMU) -kernel $(KERNEL_ELF) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log
else
output: $(KERNEL_ISO) $(DISK_IMG)
	$(QEMU) -cdrom $(KERNEL_ISO) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
endif

output-ahci: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log -machine q35 -drive file=$(DISK_IMG),format=raw,if=none,id=disk -device ahci,id=ahci -device ide-hd,drive=disk,bus=ahci.0

# Boot kernel ELF directly (no ISO/GRUB) – works for all arches.
# aarch64: no UEFI firmware (aarch64 boot asm drops EL2->EL1 itself) and no
# IDE drive (QEMU virt has no IDE controller).
ifeq ($(ARCH),aarch64)
run-elf: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -no-reboot -no-shutdown -D qemu.log
else
run-elf: $(KERNEL_ELF) $(DISK_IMG)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
endif

ifeq ($(ARCH),aarch64)
output-elf: $(KERNEL_ELF)
	$(QEMU) -kernel $(KERNEL_ELF) -display none $(QEMU_FLAGS) -no-reboot -no-shutdown -D qemu.log
else
output-elf: $(KERNEL_ELF) $(DISK_IMG)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -display none -no-reboot -no-shutdown -D qemu.log -drive file=$(DISK_IMG),format=raw,if=ide
endif

# Boot headless and auto-capture desktop shell screenshot (PNG).
# NOTE: --bios '$(QEMU_FW)' expands to '' on x86_64 (undefined var), which the
# harness treats as "no firmware" and skips aarch64 auto-detection.
screenshot: $(KERNEL_ISO)
	python3 os/userland/tools/capture_desktop_screenshot.py --iso $(KERNEL_ISO) --bios '$(QEMU_FW)' --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5

# Boot headless with kernel ELF and capture screenshot (works for all arches)
# aarch64: boot_aarch64.S drops EL2->EL1 itself, so no UEFI firmware is needed
# (--bios '').  The script needs the aarch64 QEMU binary plus machine flags.
ifeq ($(ARCH),aarch64)
screenshot-elf: $(KERNEL_ELF)
	python3 os/userland/tools/capture_desktop_screenshot.py --kernel $(KERNEL_ELF) --bios '' --qemu-binary qemu-system-aarch64 --qemu-extra '-machine virt -cpu cortex-a53 -m 128M' --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5
else
screenshot-elf: $(KERNEL_ELF)
	python3 os/userland/tools/capture_desktop_screenshot.py --kernel $(KERNEL_ELF) --bios '$(QEMU_FW)' --output $(BUILD_DIR)/desktop-shell-grid.png --serial-log $(BUILD_DIR)/desktop-shell-boot.log --qemu-log $(BUILD_DIR)/qemu-screenshot.log --settle-seconds 5
endif

# Boot headless and run scripted mouse interactions (no screenshot)
mouse-smoke: $(KERNEL_ISO)
	python3 os/userland/tools/mouse_smoke.py --iso $(KERNEL_ISO) --bios '$(QEMU_FW)' --serial-log $(BUILD_DIR)/mouse-smoke-boot.log --qemu-log $(BUILD_DIR)/qemu-mouse-smoke.log

# Boot headless with kernel ELF and run scripted mouse interactions
mouse-smoke-elf: $(KERNEL_ELF)
	python3 os/userland/tools/mouse_smoke.py --kernel $(KERNEL_ELF) --bios '$(QEMU_FW)' --serial-log $(BUILD_DIR)/mouse-smoke-boot.log --qemu-log $(BUILD_DIR)/qemu-mouse-smoke.log

# Run scripted mouse interactions and capture a screenshot artifact
mouse-screenshot: $(KERNEL_ISO)
	python3 os/userland/tools/mouse_smoke.py --iso $(KERNEL_ISO) --bios '$(QEMU_FW)' --serial-log $(BUILD_DIR)/mouse-screenshot-boot.log --qemu-log $(BUILD_DIR)/mouse-screenshot.log --screenshot $(BUILD_DIR)/mouse-smoke.png

# Run in QEMU with debugging
ifeq ($(ARCH),aarch64)
debug: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -s -S
else
debug: $(KERNEL_ISO)
	$(QEMU) -cdrom $(KERNEL_ISO) $(QEMU_FLAGS) -s -S
endif

ifeq ($(ARCH),aarch64)
debug-elf: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -s -S
else
debug-elf: $(KERNEL_ELF)
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) $(QEMU_FW) -s -S
endif

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
	@echo "LD = $(LD)"
	@echo "AS = $(AS)"
	@echo "ASFLAGS = $(ASFLAGS)"
	@echo "LDFLAGS_ARCH = $(LDFLAGS_ARCH)"
	@echo "QEMU = $(QEMU)"
	@echo "RUST_TARGET = $(RUST_TARGET)"
	@echo "RUST_FEATURES = $(RUST_FEATURES)"
	@echo "ARCH_ASM_DIR = $(ARCH_ASM_DIR)"

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
