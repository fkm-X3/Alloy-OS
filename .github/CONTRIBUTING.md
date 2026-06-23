# Contributing to Alloy-OS

Thanks for your interest in contributing to Alloy-OS!  
This project is an operating system written primarily in **Rust**, **C/C++**, and **Assembly**, with build tooling and utilities.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Development Setup](#development-setup)
- [Build & Run](#build--run)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Commit & Pull Request Guidelines](#commit--pull-request-guidelines)
- [Issue Guidelines](#issue-guidelines)
- [Security](#security)

## Code of Conduct

By participating in this project, you agree to be respectful and constructive in discussions, issues, and pull requests.

## Ways to Contribute

You can help by:

- Reporting bugs
- Proposing features
- Improving documentation
- Writing tests
- Fixing issues labeled `good first issue` or `help wanted`
- Improving performance, stability, and hardware support

## Development Setup

### Prerequisites

You’ll typically need:

- **Rust toolchain** (`rustup`, `cargo`; nightly required for `-Zbuild-std`)
- **C/C++ toolchain** (`gcc`/`g++`/`clang`/`clang++`, `make`)
- **Assembler tools** (`nasm` for x86, `aarch64-linux-gnu-gcc` for aarch64)
- **QEMU** (or your chosen VM/emulator) for OS testing
- **Python 3** (for scripts/utilities)
- **Docker** (optional, for containerized build environment with all cross-toolchains)

> If you have Docker, prefer the containerized build (`docker compose run --rm alloy make ...`) — no manual toolchain setup needed.

### Clone the repository

```bash
git clone https://github.com/fkm-X3/Alloy-OS.git
cd Alloy-OS
```

## Build & Run

Select the target architecture with `ARCH=`, then build:

```bash
# Build for i686 (32-bit x86, default)
make ARCH=i686

# Build for x86_64
make ARCH=x86_64

# Build for aarch64
make ARCH=aarch64

# Run in QEMU (with GRUB ISO)
make ARCH=i686 run

# Or boot the ELF directly (works for all arches)
make ARCH=i686 run-elf

# Headless boot (serial output only)
make ARCH=i686 output

# Headless boot + auto-screenshot
make ARCH=i686 screenshot
```

If you add or change build commands, please also update the README.

## Project Structure

| Directory | Responsibility |
|---|---|
| `kernel/c/` | Early boot (C), drivers, paging, VMM |
| `kernel/rust/` | Rust kernel entry, allocator, VFS, display server, Fusion Wayland |
| `kernel/hal/` | Hardware abstraction layer |
| `boot/` | Bootloader ASM (multiboot2) + architecture-specific startup |
| `alloy_de/` | Built-in desktop environment (C, Wayland client) |
| `de/` | Host-side Qt6/QML desktop environment for development |
| `os/userland/` | Userland programs |
| `tools/` | Screenshot and smoke-test Python helpers |
| `docs/` | Design notes and documentation |

Use existing module boundaries and naming patterns when adding new code.

## Coding Standards

### General

- Keep changes focused and minimal.
- Prefer small, reviewable PRs.
- Document non-obvious behavior with comments.
- Avoid unrelated refactors in bug-fix PRs.

### Rust

- Run formatting and linting before pushing (inside `kernel/rust/`):
  ```bash
  cd kernel/rust
  cargo +nightly fmt --check
  cargo +nightly clippy --target <arch>-alloy.json -Zbuild-std=core,alloc -D warnings
  ```
  Replace `<arch>` with `i686`, `x86_64`, or `aarch64`.
- Avoid `unwrap()`/`expect()` in kernel-critical paths unless justified.
- Prefer explicit error handling and clear panic boundaries.
- Keep `unsafe` blocks as small as possible and explain safety assumptions.

### C

- Follow the existing style (brace placement, naming, header order).
- Compile with warnings enabled and fix new warnings.
- Prefer fixed-width integer types where relevant (`uint32_t`, etc.).
- Be explicit about ownership, lifetimes, and buffer sizes.

### Assembly

- Keep assembly routines minimal and documented.
- Clearly state calling conventions, register usage, and clobbers.
- Add comments for control register/mode transitions.

## Testing

Before opening a PR, run:

```bash
# Rust formatting/linting (inside kernel/rust/)
cd kernel/rust
cargo +nightly fmt --check
cargo +nightly clippy --target <arch>-alloy.json -Zbuild-std=core,alloc -D warnings

# Build (all arches)
make clean
make ARCH=i686 all
make ARCH=x86_64 all
make ARCH=aarch64 all
```

> There is no `cargo test` inside the kernel crate (`no_std`, `panic = "abort"`). Use QEMU boot tests instead.

Also test boot/runtime behavior in QEMU (or the project’s standard emulator flow).

When changing low-level code (memory, interrupts, scheduler, boot path), include a short test plan in your PR description.

## Commit & Pull Request Guidelines

### Commits

- Use clear, imperative commit messages:
  - `kernel: fix page table flag masking`
  - `x86_64: initialize IDT before enabling interrupts`
  - `docs: add boot flow diagram`
- Keep commits logically grouped.

### Pull Requests

Each PR should include:

1. **What** changed
2. **Why** it changed
3. **How** it was tested
4. Any known limitations or follow-up work

Checklist:

- [ ] Code builds successfully
- [ ] Formatting/linting passes
- [ ] Tests added/updated where appropriate
- [ ] Documentation updated if behavior changed

PRs that are easier to review get merged faster.

## Issue Guidelines

When filing a bug, please include:

- Host OS and toolchain versions
- Target architecture (i686, x86_64, or aarch64)
- Steps to reproduce
- Expected vs actual behavior
- Logs/screenshot/serial output (if available)

For feature requests, describe the use case and potential implementation direction.

## Security

Please do **not** disclose security vulnerabilities in public issues.

Instead, contact the maintainers privately.

You can contact the owner of this repo (fkm-X3) via discord DM, at i_love_eating_uranium

---

Thanks again for helping improve Alloy-OS