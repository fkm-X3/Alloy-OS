# Architecture

## Boot flow

1. **ASM** (`boot/boot.asm`, `boot/multiboot2.asm`) — Multiboot2 entry
2. **C** (`kernel/c/boot/main.c`) — initializes serial, VGA, GDT/IDT, keyboard, mouse, timer, paging, VMM, VESA, then calls `rust_main()`
3. **Rust** (`kernel/rust/src/lib.rs::rust_main()`) — initializes VFS, runs display server
4. **Display server** (`kernel/rust/src/display_server.rs`) — wires protocol, Fusion backend, window manager, shell

## Repository layout

| Directory | Responsibility |
|---|---|
| `kernel/c/` | Early boot (C), drivers, paging, VMM, handoff to Rust |
| `kernel/rust/` | Rust kernel entry, allocator, VFS, display server, Fusion Wayland |
| `Alloy-DE/alloy-display-kernel/` | Shared display server library — protocol, server core, window manager, desktop shell |
| `boot/` | Bootloader ASM + GRUB config |
| `tools/` | Screenshot and smoke-test Python helpers |

## Key conventions

- `no_std` + `alloc` throughout kernel Rust. Use existing newtype IDs (`ClientId`, `SurfaceId`, `WindowId`) — never raw integers.
- Server state uses `BTreeMap`/`VecDeque` for deterministic ordering, not hash maps.
- Geometry and buffer sizing use checked/saturating arithmetic.
- Serial logging (`ffi::serial_print`) is the primary debug mechanism for boot/headless paths.
- Validate requests via `protocol::validate_request()` before mutating server state.

## CI

| Workflow | What it does |
|---|---|
| `build-and-test.yml` | Full Docker build + ISO + boot verification via serial log |
| `quick-check.yml` | PR syntax checks (nasm, fmt, clippy) |

All CI runs inside Docker via `.github/actions/docker-ci`.

## Gotchas

- Mouse input is relative PS/2. Click inside QEMU to grab pointer. `make output` is headless and cannot capture live mouse.
- The repo targets `i686-alloy` (32-bit x86), not `x86_64`, despite `ARCH ?= x86_64` in the Makefile.
- Rust nightly is required with `-Zbuild-std=core,alloc` and the custom target spec `i686-alloy.json`.
- Not a Cargo workspace — two crate roots: `kernel/rust`, `Alloy-DE/alloy-display-kernel`.

## Terminal mode

If Alloy falls back to terminal mode, these built-ins are available:

`help [command]`, `clear`, `echo`, `version`, `sysinfo`, `uname`, `free`, `ticks`, `meminfo`, `cpuinfo`, `uptime`
