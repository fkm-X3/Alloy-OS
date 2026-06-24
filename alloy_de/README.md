# alloy_de — Desktop Environment

This is the Alloy OS desktop environment (GUI shell), originally built for **i686 (32-bit)**.

It needs porting to **x86_64**. The linker script (`alloy_de.ld`) uses `elf32-i386` output format, and `crt0.S` in `os/userland/lib/` is 32-bit assembly. Porting requires:

1. Rewrite `alloy_de.ld` for `elf64-x86-64` output
2. Port `os/userland/lib/crt0.S` to x86_64
3. Update pointer widths in C source (shm, draw, wl_client)
4. Restore the `#[cfg(feature = "i686")]` → `#[cfg(feature = "x86_64")]` block in `kernel/rust/src/fs/mod.rs` for VFS embedding

Build is skipped automatically on non-i686 arches.
