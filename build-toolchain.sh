#!/bin/bash
set -e

# Development environment setup script
# Installs mainstream GCC packages for all three target architectures
# (x86_64 natively, aarch64 via cross-compiler)

if [ "$(id -u)" -eq 0 ]; then
    SUDO=""
else
    SUDO="sudo"
fi

$SUDO apt-get update
$SUDO apt-get install -y --no-install-recommends \
    build-essential \
    gcc-multilib \
    g++-multilib \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu

echo ""
echo "============================================"
echo "Development toolchain installed!"
echo "============================================"
echo ""
echo "  x86_64:  gcc -m64"
echo "  aarch64: aarch64-linux-gnu-gcc"
echo ""
echo "Now run: make ARCH=x86_64     # 64-bit x86 build"
echo "         make ARCH=aarch64    # ARM64 build"
