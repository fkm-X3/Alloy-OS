# Build & Run

All build/run commands go through `xtask` (`cargo xtask <command>`). Select the
architecture with the `ARCH` env var (`x86_64` default, or `aarch64`).

## Docker (recommended)

```sh
docker compose build
docker compose run --rm alloy cargo xtask output
```

The repo is bind-mounted into `/workspace`, so host edits are visible immediately.

## Native workflow

```sh
cargo xtask iso       # build kernel + bootable ISO
cargo xtask lazy      # clean rebuild + ISO
cargo xtask run       # QEMU with GUI window
cargo xtask output    # QEMU headless, serial output only (works in Docker)
cargo xtask screenshot # headless boot + auto-capture desktop PNG
cargo xtask mouse-smoke        # scripted mouse interaction test
cargo xtask mouse-screenshot   # mouse test + screenshot
cargo xtask debug     # QEMU with GDB stub (-s -S)
cargo xtask clean     # remove all build outputs
```

`cargo xtask run` opens a GUI QEMU window and may need display forwarding in Docker. Headless targets work well in containers.

## Validation

```sh
# Kernel Rust (requires nightly + custom target)
cd kernel/rust && cargo +nightly fmt --check
cd kernel/rust && cargo +nightly clippy --target x86_64-alloy.json -Zbuild-std=core,alloc

# Assembly syntax check (x86_64)
nasm -f elf64 -o /dev/null <file.asm>
```

Kernel Rust is `no_std` with `panic = "abort"`. No `cargo test` runs inside the kernel crate. The display crate (`Fusion`) is inlined under `kernel/rust/src/fusion/`.
