# Alloy OS
An os built in c/c++ and rust

## fucking information
```bash
# use linux because ts don't compile on windows
make iso

# or be lazy after making iso for the first time
make fuck

# debug (headless qemu)
make output
```

## experimental window manager
window-manager scaffolding now exists in the display-server path (`os/display/apps/window_manager.rs`).

- it is keyboard-first (floating windows, focus cycling, move/resize controls)
- it is **not** the default boot path yet
- enable it by setting `ENABLE_OS_DISPLAY_SERVER` to `true` in `kernel/rust/src/lib.rs`
