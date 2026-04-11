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
- it is the default boot path yet
- dissable it by setting `ENABLE_OS_DISPLAY_SERVER` to `false` in `kernel/rust/src/lib.rs`

### controls
- **ESC**: exit display mode
- **PgUp / PgDn**: cycle focused window
- **`**: toggle keyboard window-control mode
- **Arrow keys** (control mode): move focused window
- **+ / -** (control mode): resize focused window
- **M** (control mode): minimize focused window
- **H** (control mode): hide focused window
- **R** (control mode): restore next hidden/minimized window
- **C / X** (control mode): close focused window

### default-boot promotion gate
the display-server + window-manager path should stay gated until all are true:
- wm unit tests pass in `os/display` (including focus/state/bounds scenarios)
- kernel builds cleanly with the display-server path integrated (`make`)
- headless runtime smoke (`make output`) exercises focus/move/resize/minimize/hide/restore/close without lockups
- terminal fallback remains intact if display-server startup fails
