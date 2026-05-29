# Architecture Abstraction Layer

Alloy OS supports multiple CPU architectures through a Hardware Abstraction Layer (HAL).

## Supported Architectures

| Architecture | Status | Description |
|---|---|---|
| **i686** | **Main/Default** | 32-bit x86, fully working, used for testing |
| **x86_64** | Placeholder | 64-bit x86, stubs only |
| **aarch64** | Minimal | 64-bit ARM, basic working support |

## Building for Different Architectures

```bash
# Default: i686 (32-bit x86)
make
make run

# x86_64 (placeholder)
make ARCH=x86_64

# ARM64 (minimal)
make ARCH=aarch64
```

## Directory Structure

```
kernel/
  hal/                          # Rust HAL crate
    src/
      arch/
        i686/                   # i686 implementation
        x86_64/                 # x86_64 placeholder
        aarch64/                # ARM64 minimal
      io/                       # I/O abstraction (port I/O vs MMIO)
      interrupt/                # PIC/APIC/GIC
      memory/                   # Paging structures per arch
      serial/                   # UART16550/PL011
      time/                     # PIT/ARM Generic Timer
  cpp/
    arch/
      i686/                     # i686 C++ code (GDT, IDT, context switch)
      x86_64/                   # x86_64 placeholder stubs
      aarch64/                  # ARM64 minimal stubs
    boot/
      main.cpp                  # Architecture-aware boot flow
  linker.ld                     # i686 linker script
  linker_x86_64.ld              # x86_64 linker script
  linker_aarch64.ld             # ARM64 linker script
  rust/
    i686-alloy.json             # i686 Rust target spec
    x86_64-alloy.json           # x86_64 Rust target spec
    aarch64-alloy.json          # ARM64 Rust target spec
```

## Key Abstractions

### I/O Operations

- **x86**: Port I/O (`inb`/`outb` instructions)
- **ARM64**: Memory-mapped I/O (volatile reads/writes)

### Interrupt Controllers

- **i686/x86_64**: PIC 8259 (legacy), APIC (future)
- **aarch64**: GICv2 (Generic Interrupt Controller)

### Timers

- **x86**: PIT (Programmable Interval Timer) at 1193182 Hz
- **ARM64**: ARM Generic Timer (CNTFRQ_EL0, CNTVCT_EL0)

### Serial Ports

- **x86**: UART 16550 via port I/O (COM1 at 0x3F8)
- **ARM64**: PL011 UART via MMIO (QEMU virt at 0x09000000)

### Memory Management

- **i686**: 2-level paging (PD + PT), 4GB address space
- **x86_64**: 4-level paging (PML4 + PDPT + PD + PT), 128TB+ address space
- **aarch64**: 4-level translation tables (L0-L3), 256TB address space

### CPU Context

- **i686**: EAX-EDI, EBP, ESP, EIP, segment registers, EFLAGS, CR3
- **x86_64**: RAX-R15, RBP, RSP, RIP, segment registers, RFLAGS, CR3
- **aarch64**: x19-x30, SP_EL1, ELR_EL1, SPSR_EL1, TTBR0_EL1

## Architecture-Specific Notes

### i686 (Default)

- Booted via GRUB/Multiboot2
- VGA text mode available
- PS/2 keyboard and mouse
- PIT timer
- PIC interrupt controller
- QEMU: `qemu-system-i386`

### x86_64 (Placeholder)

- Requires long mode entry from bootloader
- VGA text mode still available in compatibility
- Same drivers as i686 (PIT, PIC, PS/2)
- QEMU: `qemu-system-x86_64`
- **Not yet functional**

### aarch64 (Minimal)

- Booted via UEFI or direct kernel loading
- No VGA text mode (framebuffer only)
- No PS/2 (HID/USB needed)
- ARM Generic Timer
- GICv2 interrupt controller
- QEMU: `qemu-system-aarch64 -machine virt -cpu cortex-a53`
- **Basic boot only, no GUI**

## Adding a New Architecture

1. Create `kernel/hal/src/arch/<arch>/mod.rs`
2. Create `kernel/c/arch/<arch>/` with C stubs
3. Create `kernel/rust/<arch>-alloy.json` target spec
4. Create `kernel/linker_<arch>.ld` linker script
5. Add architecture to Makefile conditional
6. Add feature flag to `kernel/hal/Cargo.toml`
7. Update `kernel/c/arch/context.h` with register layout
