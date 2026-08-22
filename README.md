> [!IMPORTANT]
> Alloy-OS is over. It was a good (short) run but I've started to dread working on Alloy-OS so I'm starting a new project [Ferric-K](https://github.com/fkm-X3/ferric-k), it will be a lot more restrictive then this project but it should create a better result.

<div align="center">
  <h1>Alloy OS</h1>
  <p><strong>An operating system built in Rust and Assembly.</strong></p>
  <img src="assets/alloy-os-light.svg" alt="Alloy OS light logo" width="300" />
  <img src="assets/alloy-os-dark.svg" alt="Alloy OS dark logo" width="300" />
</div>

<p align="center">
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/build-and-test.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/build-and-test.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/quick-check.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/quick-check.yml/badge.svg" alt="Quick Check"></a>
</p>

## Quick start

```sh
docker compose build
docker compose run --rm alloy cargo xtask output
```

See [docs/build.md](docs/build.md) for all xtask commands, native builds, and validation commands.

## Architecture

| Directory | Responsibility |
|---|---|
| `kernel/unsafe-core/` | The only crate with `unsafe`: arch layer, drivers, memory, alloc/sync, interrupts, boot init; safe `api` boundary |
| `kernel/hal/` | Safe API contract: re-exports `unsafe-core::api`, `println!`/`log!` macros |
| `kernel/rust/` | Safe kernel: VFS, display server, Fusion Wayland, net, terminal, syscalls (`#![deny(unsafe_code)]`) |
| `boot/` + `kernel/asm/` | Boot and context-switch assembly (the only non-Rust code) |
| `tools/` | Screenshot/smoke-test helpers |

Boot flow: ASM → Rust init (`kernel_main`) → `rust_main()` → display server.  
See [docs/architecture.md](docs/architecture.md) for boot flow, conventions, CI, and gotchas.

## What you see at boot

The kernel enters Rust, initializes VFS, boots the display server with Fusion compositing with LXQt-compatible shell surfaces. Wayland support lives under `kernel/rust/src/fusion/wayland`.

## Controls

See [docs/controls.md](docs/controls.md) for keyboard shortcuts and mouse usage.

## Tools

see [docs/tools.md](docs/controls.md) for what the tools do and how to use them.
  
