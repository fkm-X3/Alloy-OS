# Architecture Abstraction Layer

Alloy OS supports multiple CPU architectures through a Hardware Abstraction Layer (HAL).

The kernel is **Rust + assembly only** — there is no C in the tree or the build.

## Supported Architectures

| Architecture | Status | Description |
|---|---|---|
| **x86_64** | **Main/Default** | 64-bit x86, fully working (desktop environment, userland, COW fork, storage) |
| **aarch64** | Working (QEMU virt) | Boots, svc syscalls + userland, PL110 framebuffer; no desktop environment yet |

## Building for Different Architectures

```bash
# Default: x86_64
cargo xtask
cargo xtask run

# ARM64 (QEMU virt, direct -kernel boot)
ARCH=aarch64 cargo xtask output
```

The build is assembly (`nasm` / GAS) plus Rust nightly with a custom target spec — no C toolchain.

## Directory Structure

```
kernel/
  unsafe-core/                  # The ONLY crate allowed to contain `unsafe`
    src/
      api/                      # THE SAFE BOUNDARY — the crate's only public surface
      arch/                     # Context switch, GDT/IDT/vectors, syscall entry, CPU info
        boot/x86_64/            #   x86_64 kernel_main (boot init sequence)
        boot/aarch64/           #   aarch64 kernel_main (boot init sequence)
      drivers/                  # Serial/VGA/VESA/PL110, keyboard/mouse, PCI, ATA/AHCI, initrd
      mem/                      # PhysFrame/VmRegion/AddressSpace, validated user copies
      memory/                   # MemoryManager facade
      interrupt/                # PIC 8259 / GICv2 behind IrqLine + InterruptGuard
      io/                       # IoPort (x86) / Mmio (aarch64)
      serial/ time/             # UART16550 + PL011; PIT + ARM Generic Timer
      alloc/ sync/              # KernelAllocator + Slab; SpinLock/SpinLockIrq
      raw/                      # extern "C" decls, inline-asm shims, linker-visible symbols
  hal/                          # SAFE API CONTRACT crate: re-exports unsafe-core::api,
                                #   println!/print!/log! macros, platform init.
                                #   No implementation code of its own.
  rust/                         # The safe kernel (#![deny(unsafe_code)]): VFS, display
                                #   server, Fusion Wayland, net, terminal, syscall policy
    x86_64-alloy.json           # x86_64 Rust target spec
    aarch64-alloy.json          # ARM64 Rust target spec
  asm/
    x86_64/                     # gdt_flush, idt_stubs, context_switch, syscall_entry (nasm)
    aarch64/                    # context_switch, exception_vectors (GAS)
  linker_x86_64.ld              # x86_64 linker script
  linker_aarch64.ld             # ARM64 linker script
boot/                           # Boot ASM: Multiboot2 + long-mode entry (x86_64),
                                #   EL2→EL1 drop + UART bring-up (aarch64), grub.cfg
```

**Dependency edges (single direction, no cycles):**
`boot asm` → `unsafe-core` → `hal` → `kernel/rust`. The kernel crate depends on exactly one crate (`alloy-kernel-hal`); `hal` re-exports the safe boundary of `unsafe-core`; `unsafe-core` is where every `unsafe` block, inline-asm shim, and linker-visible symbol lives. Not a Cargo workspace — path dependencies between sibling crates.

## Key Abstractions

### I/O Operations

- **x86**: Port I/O (`inb`/`outb` instructions) via `unsafe_core::io::IoPort`
- **ARM64**: Memory-mapped I/O (volatile reads/writes) via `unsafe_core::io::Mmio`

### Interrupt Controllers

- **x86_64**: PIC 8259 (legacy), APIC (future)
- **aarch64**: GICv2 (Generic Interrupt Controller)

Both are wrapped behind `InterruptController` + safe `IrqLine`s and an RAII `InterruptGuard`.

### Timers

- **x86**: PIT (Programmable Interval Timer) at 1193182 Hz
- **ARM64**: ARM Generic Timer (CNTFRQ_EL0, CNTVCT_EL0)

### Serial Ports

- **x86**: UART 16550 via port I/O (COM1 at 0x3F8)
- **ARM64**: PL011 UART via MMIO (QEMU virt at 0x09000000)

Console output goes through the `println!`/`log!` macros from `alloy-kernel-hal`.

### Memory Management

- **x86_64**: 4-level paging (PML4 + PDPT + PD + PT), 128TB+ address space
- **aarch64**: TTBR0 translation tables (L0–L3); userland currently runs identity-mapped with the MMU disabled

Safe surface: `PhysFrame` (RAII), `VmRegion`, `AddressSpace`, validated `copy_from_user`/`copy_to_user`.

### CPU Context

- **x86_64**: RAX-R15, RBP, RSP, RIP, segment registers, RFLAGS, CR3
- **aarch64**: full GPR save area incl. user SP (`sp_el0` restore), ELR_EL1, SPSR_EL1, TTBR0_EL1

## Architecture-Specific Notes

### x86_64 (Default)

- Booted via GRUB/Multiboot2 ISO, or directly from the ELF (`run-elf`/`output-elf`)
- VGA text mode available; VESA linear framebuffer for graphics
- PS/2 keyboard and mouse
- PIT timer
- PIC interrupt controller
- QEMU: `qemu-system-x86_64`

### aarch64

- Direct `-kernel` ELF loading on QEMU `virt`; the boot asm drops EL2→EL1 itself, no firmware needed
- PL110 LCD controller, framebuffer at physical 0x47D00000 (top of the 128 MiB low RAM; PMM reserves it)
- Userland is loaded at a fixed physical base and entered over `svc` syscalls (identity-mapped; per-process TTBR0 pending)
- ARM Generic Timer + GICv2 interrupt controller
- QEMU: `qemu-system-aarch64 -machine virt -cpu cortex-a53`

## Adding a New Architecture

1. Add an `<arch>` feature to all three crates (`kernel/unsafe-core`, `kernel/hal`, `kernel/rust`)
2. Create `kernel/unsafe-core/src/arch/<arch>/` (context/CPU/interrupt/syscall impls) plus a `boot/<arch>/` `kernel_main` boot-init sequence
3. Create `kernel/asm/<arch>/` context-switch and exception-vector assembly
4. Create `kernel/rust/<arch>-alloy.json` target spec
5. Create `kernel/linker_<arch>.ld` linker script
6. Add the architecture variant to `Config::new` in `xtask/src/main.rs` (asm list, cargo target/features, QEMU flags)
