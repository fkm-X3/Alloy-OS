from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

from capture_desktop_screenshot import (
    connect_qmp,
    convert_ppm_to_png,
    qmp_execute,
    read_qmp_message,
    wait_for_serial_marker,
)


def send_relative_motion(stream, dx: int, dy: int) -> None:
    qmp_execute(
        stream,
        {
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "rel", "data": {"axis": "x", "value": dx}},
                    {"type": "rel", "data": {"axis": "y", "value": dy}},
                ]
            },
        },
    )


def send_button(stream, button: str, down: bool) -> None:
    qmp_execute(
        stream,
        {
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "btn", "data": {"button": button, "down": down}},
                ]
            },
        },
    )


def click_left(stream, delay_seconds: float) -> None:
    send_button(stream, "left", True)
    time.sleep(delay_seconds)
    send_button(stream, "left", False)


def perform_mouse_smoke(stream, step_delay_seconds: float) -> None:
    # Pointer starts at display center in runtime. First click activates launcher tile.
    click_left(stream, step_delay_seconds)
    time.sleep(0.35)

    # Move to a predictable title-bar area for terminal/default app windows.
    send_relative_motion(stream, -220, -190)
    time.sleep(step_delay_seconds)

    # Drag window diagonally.
    send_button(stream, "left", True)
    time.sleep(step_delay_seconds)
    for _ in range(6):
        send_relative_motion(stream, 12, 8)
        time.sleep(step_delay_seconds)
    send_button(stream, "left", False)
    time.sleep(step_delay_seconds)


def _detect_firmware() -> str | None:
    """Probe well-known aarch64 UEFI firmware paths."""
    candidates = [
        "/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
        "/usr/share/AAVMF/AAVMF_CODE.fd",
        "/usr/share/edk2/aarch64/QEMU_EFI.fd",
        "/opt/homebrew/share/qemu-efi-aarch64/QEMU_EFI.fd",
        "C:/msys64/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
        "C:/Program Files/qemu/share/qemu-efi-aarch64/QEMU_EFI.fd",
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return None


def run_smoke(args: argparse.Namespace) -> None:
    iso_path = Path(args.iso).resolve() if args.iso else None
    kernel_path = Path(args.kernel).resolve() if args.kernel else None
    serial_log = Path(args.serial_log).resolve()
    qmp_socket = Path(args.qmp_socket).resolve()
    qemu_log = Path(args.qemu_log).resolve()
    screenshot_png = Path(args.screenshot).resolve() if args.screenshot else None
    screenshot_ppm = screenshot_png.with_suffix(".ppm") if screenshot_png else None

    if not iso_path and not kernel_path:
        raise ValueError("Must provide either --iso or --kernel")
    if iso_path and not iso_path.exists():
        raise FileNotFoundError(f"ISO not found: {iso_path}")
    if kernel_path and not kernel_path.exists():
        raise FileNotFoundError(f"Kernel not found: {kernel_path}")

    serial_log.parent.mkdir(parents=True, exist_ok=True)
    qemu_log.parent.mkdir(parents=True, exist_ok=True)
    if screenshot_png is not None:
        screenshot_png.parent.mkdir(parents=True, exist_ok=True)

    for path in (serial_log, qmp_socket, screenshot_png, screenshot_ppm):
        if path is not None and path.exists():
            path.unlink()

    qemu_cmd = [
        args.qemu_binary,
        "-serial",
        f"file:{serial_log}",
        "-display",
        "none",
        "-qmp",
        f"unix:{qmp_socket},server=on,wait=off",
        "-no-reboot",
        "-no-shutdown",
        "-D",
        str(qemu_log),
    ]

    if kernel_path is not None:
        qemu_cmd.append("-kernel")
        qemu_cmd.append(str(kernel_path))
    else:
        qemu_cmd.append("-cdrom")
        qemu_cmd.append(str(iso_path))

    # Add UEFI firmware for aarch64 if user didn't explicitly set --bios ""
    bios = args.bios
    if bios is None:
        detected = _detect_firmware()
        if detected:
            bios = detected
    if bios:
        qemu_cmd.append("-bios")
        qemu_cmd.append(bios)

    print(f"[mouse-smoke] Starting QEMU: {' '.join(qemu_cmd)}")
    proc = subprocess.Popen(
        qemu_cmd,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
    )

    try:
        wait_for_serial_marker(serial_log, args.marker, args.timeout_seconds, process=proc)
        print(
            f"[mouse-smoke] Marker detected; waiting {args.settle_seconds:.1f}s before input..."
        )
        time.sleep(args.settle_seconds)

        qmp_sock = connect_qmp(qmp_socket, timeout_seconds=5.0)
        with qmp_sock:
            stream = qmp_sock.makefile("rwb", buffering=0)
            _ = read_qmp_message(stream)  # Greeting
            qmp_execute(stream, {"execute": "qmp_capabilities"})
            perform_mouse_smoke(stream, args.step_delay_seconds)

            if screenshot_ppm is not None:
                qmp_execute(
                    stream,
                    {
                        "execute": "screendump",
                        "arguments": {"filename": str(screenshot_ppm)},
                    },
                )

            qmp_execute(stream, {"execute": "quit"})

        try:
            proc.wait(timeout=5.0)
        except subprocess.TimeoutExpired:
            proc.terminate()
            proc.wait(timeout=5.0)

        if screenshot_ppm is not None and screenshot_png is not None:
            convert_ppm_to_png(screenshot_ppm, screenshot_png)
            if not args.keep_ppm and screenshot_ppm.exists():
                screenshot_ppm.unlink()
            print(f"[mouse-smoke] Saved screenshot: {screenshot_png}")

        print(f"[mouse-smoke] Serial log: {serial_log}")
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5.0)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5.0)
        if qmp_socket.exists():
            qmp_socket.unlink()
        if screenshot_ppm is not None and screenshot_ppm.exists() and not args.keep_ppm:
            screenshot_ppm.unlink()


def build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Boot QEMU headless, wait for the first display frame marker, run scripted mouse interactions through QMP, "
            "and optionally capture a screenshot."
        )
    )
    parser.add_argument("--iso", help="Path to bootable ISO (mutually exclusive with --kernel)")
    parser.add_argument("--kernel", help="Path to kernel ELF (mutually exclusive with --iso)")
    parser.add_argument(
        "--serial-log",
        default="build/mouse-smoke-boot.log",
        help="Serial log file path",
    )
    parser.add_argument(
        "--qmp-socket",
        default="/tmp/alloy-qmp-mouse.sock",
        help="QMP unix socket path",
    )
    parser.add_argument(
        "--qemu-log",
        default="build/qemu-mouse-smoke.log",
        help="QEMU debug log path",
    )
    parser.add_argument(
        "--bios",
        default=None,
        help="Path to UEFI firmware file. Auto-detected for aarch64 if omitted. "
             "Set to empty string to disable.",
    )
    parser.add_argument(
        "--screenshot",
        help="Optional screenshot PNG output path",
    )
    parser.add_argument(
        "--marker",
        default="[DisplayServer] First frame presented",
        help="Serial marker to wait for before running mouse scenario (default is DisplayServer first-frame)",
    )
    parser.add_argument(
        "--settle-seconds",
        type=float,
        default=1.5,
        help="Seconds to wait after marker before sending mouse input",
    )
    parser.add_argument(
        "--step-delay-seconds",
        type=float,
        default=0.08,
        help="Delay between scripted mouse steps",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=120.0,
        help="Max seconds to wait for serial marker",
    )
    parser.add_argument(
        "--qemu-binary",
        default="qemu-system-i386",
        help="QEMU executable name/path",
    )
    parser.add_argument(
        "--keep-ppm",
        action="store_true",
        help="Keep intermediate PPM when --screenshot is used",
    )
    return parser


def main() -> int:
    parser = build_arg_parser()
    args = parser.parse_args()

    try:
        run_smoke(args)
        return 0
    except Exception as exc:
        print(f"[mouse-smoke] ERROR: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
