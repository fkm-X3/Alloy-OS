#!/bin/bash
set -e

# i686-elf Cross-Compiler Build Script
# This builds GCC and binutils for i686-elf target

export PREFIX="$HOME/.local/i686-elf"
export TARGET=i686-elf
export PATH="$PREFIX/bin:$PATH"

BINUTILS_VERSION=2.40
GCC_VERSION=13.2.0

echo "Installing dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential bison flex libgmp3-dev libmpc-dev libmpfr-dev texinfo wget

echo "Creating build directory..."
mkdir -p ~/toolchain-build
cd ~/toolchain-build

# Build Binutils
if [ ! -f "$PREFIX/bin/$TARGET-ld" ]; then
    echo "Building binutils..."
    wget -nc https://ftp.gnu.org/gnu/binutils/binutils-$BINUTILS_VERSION.tar.gz
    tar -xf binutils-$BINUTILS_VERSION.tar.gz
    mkdir -p build-binutils
    cd build-binutils
    ../binutils-$BINUTILS_VERSION/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
    make -j$(nproc)
    make install
    cd ..
else
    echo "Binutils already installed, skipping..."
fi

# Build GCC
if [ ! -f "$PREFIX/bin/$TARGET-gcc" ]; then
    echo "Building GCC..."
    wget -nc https://ftp.gnu.org/gnu/gcc/gcc-$GCC_VERSION/gcc-$GCC_VERSION.tar.gz
    tar -xf gcc-$GCC_VERSION.tar.gz
    mkdir -p build-gcc
    cd build-gcc
    ../gcc-$GCC_VERSION/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers
    make -j$(nproc) all-gcc
    make -j$(nproc) all-target-libgcc
    make install-gcc
    make install-target-libgcc
    cd ..
else
    echo "GCC already installed, skipping..."
fi

echo ""
echo "Cross-compiler toolchain installed successfully!"
echo "Add this to your ~/.bashrc or ~/.profile:"
echo "export PATH=\"$PREFIX/bin:\$PATH\""
echo ""
echo "Then run: source ~/.bashrc"
