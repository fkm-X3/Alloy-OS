#!/usr/bin/env bash
# Build musl for i686-alloy (run inside WSL)
set -euo pipefail

MUSL_VERSION=1.2.4
BUILD_DIR="$HOME/musl-build"
PREFIX="/usr/i686-alloy"
SYSROOT_DIR="$BUILD_DIR/sysroot"

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [ ! -f "musl-$MUSL_VERSION.tar.gz" ]; then
  wget "https://www.musl-libc.org/releases/musl-$MUSL_VERSION.tar.gz"
fi

if [ ! -d "musl-$MUSL_VERSION" ]; then
  tar xzf "musl-$MUSL_VERSION.tar.gz"
fi

mkdir -p build && cd build

# Configure for 32-bit static build using gcc -m32
# Provide fallback native binutils (ar/ranlib/nm) if cross-prefixed tools are not installed
export AR=${AR:-ar}
export RANLIB=${RANLIB:-ranlib}
export NM=${NM:-nm}

../musl-$MUSL_VERSION/configure --prefix="$PREFIX" --host=i386-linux-gnu CC="gcc -m32" CFLAGS="-O2 -static"
# Pass AR/RANLIB/NM to make to avoid requiring i386-linux-gnu-ar in the host toolchain
make -j$(nproc) AR="$AR" RANLIB="$RANLIB" NM="$NM"
make install DESTDIR="$SYSROOT_DIR"

echo "Musl built and installed to $SYSROOT_DIR. Copy the sysroot contents into repo under libs/musl-sysroot/ if desired."
