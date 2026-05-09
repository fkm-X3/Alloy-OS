Building musl for i686-alloy (instructions for WSL)

Overview
This document describes steps to cross-build musl libc for the i686-alloy target and create a sysroot for static-linked userland testing.

Prerequisites (in WSL):
- Install build essentials: sudo apt-get install build-essential gcc-multilib g++-multilib make wget
- Install a cross-compiler toolchain for i386 if needed (gcc -m32 can be used for i386 builds)

Steps (recommended):
1. In WSL, create a build directory: mkdir -p ~/musl-build && cd ~/musl-build
2. Download musl: wget https://www.musl-libc.org/releases/musl-1.2.4.tar.gz && tar xzf musl-1.2.4.tar.gz
3. Configure musl for a 32-bit build (static):
   mkdir build && cd build
   ../musl-1.2.4/configure --prefix=/usr/i686-alloy --host=i386-linux-gnu CC="gcc -m32" CFLAGS="-O2 -static"
4. make -j$(nproc) && make install DESTDIR=$PWD/sysroot
5. The sysroot will be under build/sysroot/usr/i686-alloy

Notes
- Using "gcc -m32" is a simple approach to produce 32-bit artifacts; for true cross-compilation a proper i386 cross-compiler is preferred.
- The exact host/triple can be adjusted. For initial tests, static linked binaries built with musl and -m32 are sufficient.
- Copy headers and libraries into repo under libs/musl-sysroot/ for integration with build system.

Caveats
- Building musl on Windows directly is not supported; use WSL as documented in project top-level README.
- This guide produces static libc; dynamic linking (ld-musl) can be added later once ELF PT_DYNAMIC support exists.
