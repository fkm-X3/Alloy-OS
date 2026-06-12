# Build & Run

## Prerequisites

- Cross-compiler (`i686-elf-gcc`) — built via `build-toolchain.sh`
- nasm, QEMU, Rust nightly
- On Linux: `build-essential`, `grub-pc-bin`, `xorriso`, `mtools`, `dosfstools`

## Build and run

```sh
make iso       # build kernel + bootable ISO
make lazy      # clean rebuild + ISO
make run       # QEMU with GUI window
make output    # QEMU headless, serial output only
make screenshot # headless boot + auto-capture desktop PNG
make mouse-smoke        # scripted mouse interaction test
make mouse-screenshot   # mouse test + screenshot
make debug     # QEMU with GDB stub (-s -S)
make clean     # remove all build outputs
```

`make run` opens a GUI QEMU window.

## Validation

```sh
# Kernel Rust (requires nightly + custom target)
cd kernel/rust && cargo +nightly fmt --check
cd kernel/rust && cargo +nightly clippy --target i686-alloy.json -Zbuild-std=core,alloc

# Assembly syntax check
nasm -f elf32 -o /dev/null <file.asm>
```

Kernel Rust is `no_std` with `panic = "abort"`. No `cargo test` runs inside the kernel crate. The display crate (`Fusion`) is inlined under `kernel/rust/src/fusion/`.
