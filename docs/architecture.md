# Architecture

## Boot flow

1. **ASM** (`boot/boot_x86_64.asm`, `boot/multiboot2.asm`) — Multiboot2 entry
2. **Rust** (`kernel/unsafe-core/src/arch/boot/x86_64/main.rs::kernel_main()`) — initializes serial, console, GDT/IDT, keyboard, mouse, timer, paging, VMM, VESA, then calls `rust_main()` (aarch64 equivalent: `arch/boot/aarch64/main_aarch64.rs`)
3. **Rust kernel** (`kernel/rust/src/lib.rs::rust_main()`) — registers syscall/timer/page-fault callbacks, initializes VFS, runs display server
4. **Display server** (`kernel/rust/src/display_server.rs`) — wires protocol, Fusion backend, window manager, shell

## Repository layout

| Directory | Responsibility |
|---|---|
| `boot/` | Boot ASM (Multiboot2 entry, long-mode setup, ARM64 EL2→EL1 drop) + GRUB config |
| `kernel/asm/` | Arch context-switch / exception / syscall-entry asm (wrapped by unsafe-core symbols) |
| `kernel/unsafe-core/` | The only crate with `unsafe`: arch layer, drivers, memory, alloc/sync, interrupts, boot init; safe `api` boundary |
| `kernel/hal/` | Safe API contract: re-exports `unsafe-core::api`, `println!`/`log!` macros |
| `kernel/rust/` | Safe kernel (`#![deny(unsafe_code)]`): allocator declaration, VFS, display server, Fusion Wayland, net, terminal, syscall policy |
| `tools/` | Screenshot and smoke-test Python helpers |

Dependency edges are single-direction, no cycles: boot asm → `unsafe-core` → `hal` → `kernel/rust`.

## Key conventions

- `no_std` + `alloc` throughout kernel Rust. Use existing newtype IDs (`ClientId`, `SurfaceId`, `WindowId`) — never raw integers.
- Server state uses `BTreeMap`/`VecDeque` for deterministic ordering, not hash maps.
- Geometry and buffer sizing use checked/saturating arithmetic.
- Serial logging (`println!`/`log!` macros from `alloy-kernel-hal`) is the primary debug mechanism for boot/headless paths.
- Validate requests via `protocol::validate_request()` before mutating server state.

## CI

| Workflow | What it does |
|---|---|
| `build-and-test.yml` | Full Docker build + ISO + boot verification via serial log |
| `quick-check.yml` | PR syntax checks (nasm, fmt, clippy) |

All CI runs inside Docker via `.github/actions/docker-ci`.

## Gotchas

- Mouse input is relative PS/2. Click inside QEMU to grab pointer. `make output` is headless and cannot capture live mouse.
- Rust nightly is required with `-Zbuild-std=core,alloc` and the custom target spec.
- Not a Cargo workspace — three sibling crates with path deps (`kernel/unsafe-core` → `kernel/hal` → `kernel/rust`).

## Terminal mode

If Alloy falls back to terminal mode, these built-ins are available:

`help [command]`, `clear`, `echo`, `version`, `sysinfo`, `uname`, `free`, `ticks`, `meminfo`, `cpuinfo`, `uptime`
