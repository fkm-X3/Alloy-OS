# Controls

| Key | Action |
|---|---|
| **ESC** | Exit display mode |
| **\`** | Toggle keyboard window-control mode |
| **1 / 2 / 3 / 4** | Switch Iced tabs in normal mode |
| **P** | Toggle palette overlay |
| **T / Space** | Toggle accent brightness |
| **PgUp / PgDn** | Cycle focused window |
| **Arrow keys** | Move focused window (control mode) |
| **+ / -** | Resize focused window (control mode) |
| **M** | Minimize focused window (control mode) |
| **H** | Hide focused window (control mode) |
| **R** | Restore next hidden/minimized window |
| **C / X** | Close focused window (control mode) |
| **Mouse move** | Move on-screen pointer |
| **Left click** | Focus top-most window under pointer |
| **Left-drag** (title bar) | Drag focused window |

Mouse input is relative PS/2 — click inside QEMU to grab pointer. `make output` is headless and cannot capture live host mouse; use `make run` or `make mouse-smoke` for pointer testing.
