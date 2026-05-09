#!/usr/bin/env bash
# Build static hello test using musl (run inside WSL)
set -euo pipefail

SRC="./os/userland/tests/hello.c"
OUT=hello

# If musl-gcc is available, prefer it
if command -v musl-gcc >/dev/null 2>&1; then
  musl-gcc -static -o "$OUT" "$SRC"
else
  # Try gcc -m32 with static
  gcc -m32 -static -o "$OUT" "$SRC" -O2
fi

echo "Built $OUT. Copy to QEMU disk or run in emulator."