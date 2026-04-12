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

# boot headless, wait for first rendered desktop frame, then save screenshot png
make screenshot
```

## desktop shell (keyboard-first)
the display-server path now boots into a keyboard-first desktop shell (`os/display/apps/desktop_shell.rs`) layered on top of the floating window manager (`os/display/apps/window_manager.rs`).

- desktop background + panel/taskbar + launcher + window switcher
- default boot path while `ENABLE_OS_DISPLAY_SERVER` is `true`
- disable it by setting `ENABLE_OS_DISPLAY_SERVER` to `false` in `kernel/rust/src/lib.rs`

### controls
- **ESC**: exit display mode
- **`**: toggle keyboard window-control mode
- **L** (control mode): toggle launcher
- **1 / 2** (control mode): quick-switch terminal/info app
- **Arrow keys / Tab** (launcher open): move launcher selection
- **Enter / Space** (launcher open): activate selected launcher app
- **PgUp / PgDn**: cycle focused window
- **Arrow keys** (control mode, launcher closed): move focused window
- **+ / -** (control mode): resize focused window
- **M** (control mode): minimize focused window
- **H** (control mode): hide focused window
- **R** (control mode): restore next hidden/minimized window
- **C / X** (control mode): close focused window

### default-boot promotion gate
the display-server + window-manager path should stay gated until all are true:
- wm + shell unit tests pass in `os/display` (focus/state/bounds + shell behavior)
- kernel builds cleanly with the display-server path integrated (`make`)
- headless runtime smoke (`make output`) exercises launcher/quick-switch + focus/move/resize/minimize/hide/restore/close without lockups
- terminal fallback remains intact if display-server startup fails
