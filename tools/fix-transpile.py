#!/usr/bin/env python3
"""Apply the mechanical fixes C2Rust output needs before it compiles.

The C2Rust translation of `kernel/c/` does not compile as-is inside
`alloy-kernel-unsafe-core`. This script applies the exact set of fixes from
the bulk-port, so a re-transpile can be re-fixed deterministically:

1. `::libc::` -> `crate::raw::string::` and drop generated `use ::libc;`
   lines. Only `memcpy`, `memset`, and `size_t` are used; `raw::string`
   provides the freestanding shims (resolved via compiler-builtins-mem on
   the alloy targets).

2. `use ::c2rust_bitfields;` -> `use ::c2rust_bitfields::BitfieldStruct;`
   A bare crate import does not bring the `BitfieldStruct` derive into
   scope, so the generated `#[BitfieldStruct]` attrs fail to resolve.

3. x86_64 io inline-asm. C2Rust emits `inlateout("N\"dx")` (an invalid `N`
   immediate constraint on an in/out operand), templates that reference
   operands by index (`{0}`/`{1}`), and register class `"ax"` regardless of
   operand width. LLVM rejects the `N"dx` constraint and a `u8` cannot use
   the `ax` class. Rewrite to literal AT&T registers with width-correct
   classes:
     u8  -> al   ("outb %al, %dx" / "inb %dx, %al")
     u16 -> ax   ("outw %ax, %dx" / "inw %dx, %ax")
     u32 -> eax  ("outl %eax, %dx" / "inl %dx, %eax")

Usage:

    # freshly transpiled tree (before copying into ported/), e.g.
    python3 tools/fix-transpile.py /tmp/port_x86/src
    python3 tools/fix-transpile.py /tmp/port_a64/src

    # or the already-merged tree (idempotent no-op when clean)
    python3 tools/fix-transpile.py kernel/unsafe-core/src/ported

Pass --check to fail if any file still needs fixes (useful as a guard).
"""

import argparse
import pathlib
import re
import sys

VALREG = {"b": "al", "w": "ax", "l": "eax"}

# Matches an io asm template + its operand list, e.g.
#   "outb {0}, {1}\n", inlateout("ax") value => _, inlateout("dx") port => _,
# or the multi-line vesa variant where the port operand continues on the
# next line. The operand list is captured lazily (DOTALL) up to the first
# `options(` of the block's own asm!(...), not consumed (lookahead), so the
# operand quotes don't truncate the capture. Blocks like `"invlpg ({0})\n"`
# or `"mov %cr3, {0}\n"` are left alone (not in/out instructions).
IO_ASM_RE = re.compile(
    r'"(?P<ins>out[bwl]|in[bwl]) \{(?P<a>[01])\}, \{(?P<b>[01])\}\\n",'
    r'(?P<ops>.*?)(?=\s*options\()',
    re.DOTALL,
)

LIBUSE_RE = re.compile(r"^use ::libc;\n", re.MULTILINE)


def fix_io_asm_line(match: "re.Match[str]") -> str:
    ins = match.group("ins")
    width = ins[-1]
    valreg = VALREG[width]
    ops = match.group("ops")
    # Fix the value/ret operand's register class; the port operand is
    # already "dx". u16 keeps "ax" so this is an identity there.
    ops = ops.replace('("ax")', '("{valreg}")'.format(valreg=valreg))
    if ins.startswith("out"):
        template = '"out{op} %{reg}, %dx\\n"'.format(op=width, reg=valreg)
    else:
        template = '"in{op} %dx, %{reg}\\n"'.format(op=width, reg=valreg)
    return template + "," + ops


def fix_text(text: str) -> str:
    # 1. libc -> crate::raw::string
    text = LIBUSE_RE.sub("", text)
    text = text.replace("::libc::", "crate::raw::string::")
    # 2. bitfields derive import
    text = text.replace(
        "use ::c2rust_bitfields;", "use ::c2rust_bitfields::BitfieldStruct;"
    )
    # 3. io inline-asm: invalid N"dx constraint, then template/register
    #    class rewrite.
    text = text.replace('inlateout("N\\"dx")', 'inlateout("dx")')
    text = IO_ASM_RE.sub(fix_io_asm_line, text)
    return text


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Apply mechanical fixes to C2Rust output (see docstring)."
    )
    parser.add_argument("paths", nargs="+", type=pathlib.Path,
                        help="Directories (or files) of translated .rs files.")
    parser.add_argument("--check", action="store_true",
                        help="Exit non-zero if any file still needs fixes.")
    args = parser.parse_args()

    files = []
    for p in args.paths:
        if p.is_file():
            files.append(p)
        else:
            files.extend(sorted(p.rglob("*.rs")))

    if not files:
        print("fix-transpile: no .rs files found", file=sys.stderr)
        return 2

    changed = []
    dirty = []
    for f in files:
        orig = f.read_text()
        fixed = fix_text(orig)
        if fixed != orig:
            changed.append(f)
        if re.search(r'use ::libc;|::libc::|inlateout\("N\\\\"dx"\)|use ::c2rust_bitfields;|"out[bwl] \{[01]\}|"in[bwl] \{[01]\}',
                     fixed):
            dirty.append(f)
        f.write_text(fixed)

    if changed:
        print("fix-transpile: rewrote:")
        for f in changed:
            print("  -", f)
    else:
        print("fix-transpile: no changes (already clean)")

    if dirty:
        print("fix-transpile: still-dirty files:", file=sys.stderr)
        for f in dirty:
            print("  -", f, file=sys.stderr)
        if args.check:
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
