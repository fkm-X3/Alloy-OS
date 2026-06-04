# Build & Run

## Docker (recommended)

```sh
docker compose build
docker compose run --rm alloy make output
```

The repo is bind-mounted into `/workspace`, so host edits are visible immediately.

## Native workflow

```sh
make iso       # build kernel + bootable ISO
make lazy      # clean rebuild + ISO
make run       # QEMU with GUI window
make output    # QEMU headless, serial output only (works in Docker)
make screenshot # headless boot + auto-capture desktop PNG
make mouse-smoke        # scripted mouse interaction test
make mouse-screenshot   # mouse test + screenshot
make debug     # QEMU with GDB stub (-s -S)
make clean     # remove all build outputs
```

`make run` opens a GUI QEMU window and may need display forwarding in Docker. Headless targets work well in containers.

## Validation

```sh
# Display crate tests (host, std-available)
cd Alloy-DE && cargo test -p alloy-display-kernel
cd Alloy-DE && cargo test -p alloy-display-kernel <test_name>

# Kernel Rust (requires nightly + custom target)
cd kernel/rust && cargo +nightly fmt --check
cd kernel/rust && cargo +nightly clippy --target i686-alloy.json -Zbuild-std=core,alloc

# Assembly syntax check
nasm -f elf32 -o /dev/null <file.asm>
```

Kernel Rust is `no_std` with `panic = "abort"`. No `cargo test` runs inside the kernel crate.
