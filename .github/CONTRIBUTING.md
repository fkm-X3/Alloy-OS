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
# Build for x86_64 (64-bit x86, default)

make ARCH=x86_64

...

make ARCH=x86_64 run

...

make ARCH=x86_64 run-elf

...

make ARCH=x86_64 output

...

make ARCH=x86_64 screenshot

...

  Replace `<arch>` with `x86_64` or `aarch64`.

...

make ARCH=x86_64 all

```

- Target architecture (x86_64 or aarch64)
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