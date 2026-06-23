<div align="center">
  <h1>Alloy OS</h1>
  <p><strong>An operating system built in Rust, C/C++, and Assembly.</strong></p>
  <img src="assets/alloy-os-light.svg" alt="Alloy OS light logo" width="300" />
  <img src="assets/alloy-os-dark.svg" alt="Alloy OS dark logo" width="300" />
</div>

<p align="center">
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/build-and-test.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/build-and-test.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/quick-check.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/quick-check.yml/badge.svg" alt="Quick Check"></a>
</p>

Kernel boot, Fusion display, Wayland support, and desktop runtime in one repo. Boots the Rust display server in Iced-primary software-rendered mode with Fusion as the compositor/backend layer.

## Quick start

```sh
docker compose build
docker compose run --rm alloy make output
```

See [docs/build.md](docs/build.md) for all make targets, native builds, and validation commands.

## Architecture

| Directory | Responsibility |
|---|---|
| `kernel/c/` | Early boot, drivers, paging, VMM |
| `kernel/rust/` | Rust kernel entry, VFS, display server, Fusion Wayland |
| `boot/` | Bootloader ASM + GRUB config |
| `tools/` | Screenshot/smoke-test helpers |

Boot flow: ASM → C init → `rust_main()` → display server.  
See [docs/architecture.md](docs/architecture.md) for boot flow, conventions, CI, and gotchas.

## What you see at boot

The kernel enters Rust, initializes VFS, boots the display server with Fusion compositing with LXQt-compatible shell surfaces. Wayland support lives under `kernel/rust/src/fusion/wayland`.

## Controls

See [docs/controls.md](docs/controls.md) for keyboard shortcuts and mouse usage.

## Tools

see [docs/tools.md](docs/controls.md) for what the tools do and how to use them.