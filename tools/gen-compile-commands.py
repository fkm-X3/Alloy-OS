#!/usr/bin/env python3
"""Generate clang compile_commands.json for the C2Rust transpile step.

Mirrors the Makefile's C_SOURCES selection and the per-arch CFLAGS exactly.
Run once per architecture:

    python3 tools/gen-compile-commands.py --arch x86_64  -o tools/compile_commands/x86_64/compile_commands.json
    python3 tools/gen-compile-commands.py --arch aarch64 -o tools/compile_commands/aarch64/compile_commands.json

Each entry uses `arguments` so c2rust replays clang with identical flags.
"""

import argparse
import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KERNEL_C = os.path.join(ROOT, "kernel", "c")

# Common sources compiled for every arch (Makefile C_SOURCES first block).
COMMON_SOURCES = [
    "arch/cpu.c",
    "arch/syscall.c",
    "{arch}/gdt.c",
    "{arch}/idt.c",
    "mm/pmm.c",
    "mm/vmm.c",
    "drivers/serial.c",
    "drivers/timer.c",
]

# Arch-specific sources (Makefile C_SOURCES second block).
ARCH_SOURCES = {
    "x86_64": [
        "boot/main.c",
        "mm/paging.c",
        "drivers/vga.c",
        "drivers/vesa.c",
        "drivers/keyboard.c",
        "drivers/mouse.c",
        "drivers/ata.c",
        "drivers/pci.c",
        "drivers/ahci.c",
        "drivers/initrd.c",
    ],
    "aarch64": [
        "drivers/pl110.c",
        "mm/paging_aarch64.c",
        "boot/main_aarch64.c",
    ],
}

# clang flags per the plan (matching Makefile CFLAGS, clang-syntax).
ARCH_FLAGS = {
    "x86_64": [
        "--target=x86_64-unknown-none",
        "-std=gnu11",
        "-ffreestanding",
        "-nostdlib",
        "-fno-builtin",
        "-m64",
        "-mno-sse",
        "-mno-sse2",
        "-mno-mmx",
        "-mno-avx",
        "-mno-80387",
        "-DARCH_X86_64",
        "-Ikernel/c",
        "-O2",
    ],
    "aarch64": [
        "--target=aarch64-unknown-none",
        "-std=gnu11",
        "-ffreestanding",
        "-nostdlib",
        "-fno-builtin",
        "-march=armv8-a",
        "-DARCH_AARCH64",
        "-Ikernel/c",
        "-O2",
    ],
}


def sources_for(arch):
    sources = []
    for rel in COMMON_SOURCES:
        sources.append(os.path.join(KERNEL_C, rel.format(arch=os.path.join("arch", arch))))
    for rel in ARCH_SOURCES[arch]:
        sources.append(os.path.join(KERNEL_C, rel))
    return sources


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--arch", required=True, choices=["x86_64", "aarch64"])
    parser.add_argument("-o", "--output", required=True,
                        help="output compile_commands.json path")
    args = parser.parse_args()

    flags = ARCH_FLAGS[args.arch]
    entries = []
    for src in sources_for(args.arch):
        if not os.path.isfile(src):
            raise SystemExit(f"missing source: {src}")
        entries.append({
            "directory": ROOT,
            "arguments": ["clang"] + flags + ["-fsyntax-only", src],
            "file": src,
        })

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, "w") as fh:
        json.dump(entries, fh, indent=2)
        fh.write("\n")

    print(f"wrote {len(entries)} entries for {args.arch} -> {args.output}")


if __name__ == "__main__":
    main()
