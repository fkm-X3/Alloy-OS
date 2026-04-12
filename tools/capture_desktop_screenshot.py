from __future__ import annotations

import argparse
import json
import os
import socket
import struct
import subprocess
import sys
import time
import zlib
from pathlib import Path


def wait_for_serial_marker(
    log_path: Path,
    marker: str,
    timeout_seconds: float,
    process: subprocess.Popen | None = None,
) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        if process is not None and process.poll() is not None:
            code = process.returncode
            raise RuntimeError(
                f"QEMU exited early with code {code} while waiting for marker {marker!r}"
            )
        if log_path.exists():
            text = log_path.read_text(encoding="utf-8", errors="ignore")
            if marker in text:
                return
        time.sleep(0.25)
    raise TimeoutError(
        f"Timed out after {timeout_seconds:.0f}s waiting for serial marker: {marker!r}"
    )


def connect_qmp(socket_path: Path, timeout_seconds: float) -> socket.socket:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.connect(str(socket_path))
            return sock
        except OSError as exc:
            last_error = exc
            time.sleep(0.1)
    raise TimeoutError(f"Unable to connect QMP socket {socket_path}: {last_error}")


def read_qmp_message(stream) -> dict:
    while True:
        line = stream.readline()
        if not line:
            raise RuntimeError("QMP connection closed unexpectedly")
        message = json.loads(line.decode("utf-8", errors="replace").strip())
        if message:
            return message


def qmp_execute(stream, payload: dict) -> dict:
    stream.write((json.dumps(payload) + "\r\n").encode("utf-8"))
    stream.flush()

    while True:
        message = read_qmp_message(stream)
        if "return" in message:
            return message["return"]
        if "error" in message:
            raise RuntimeError(f"QMP command failed: {message['error']}")
        # Ignore async events and keep waiting for return/error.


def read_ppm_token(file_obj) -> bytes:
    token = bytearray()

    while True:
        ch = file_obj.read(1)
        if not ch:
            return b""
        if ch == b"#":
            file_obj.readline()
            continue
        if ch in b" \t\r\n":
            continue
        token.extend(ch)
        break

    while True:
        ch = file_obj.read(1)
        if not ch or ch in b" \t\r\n":
            break
        token.extend(ch)

    return bytes(token)


def convert_ppm_to_png(ppm_path: Path, png_path: Path) -> None:
    with ppm_path.open("rb") as f:
        magic = read_ppm_token(f)
        if magic != b"P6":
            raise RuntimeError(f"Unsupported PPM format: {magic!r}")

        width = int(read_ppm_token(f))
        height = int(read_ppm_token(f))
        max_value = int(read_ppm_token(f))
        if max_value != 255:
            raise RuntimeError(f"Unsupported PPM max value: {max_value}")

        expected_size = width * height * 3
        rgb = f.read(expected_size)
        if len(rgb) != expected_size:
            raise RuntimeError(
                f"Incomplete PPM pixel data: expected {expected_size}, got {len(rgb)}"
            )

    stride = width * 3
    scanlines = bytearray()
    for row in range(height):
        start = row * stride
        end = start + stride
        scanlines.append(0)  # PNG filter type 0 (None)
        scanlines.extend(rgb[start:end])

    compressed = zlib.compress(bytes(scanlines), level=9)

    def png_chunk(tag: bytes, data: bytes) -> bytes:
        body = tag + data
        return (
            struct.pack(">I", len(data))
            + body
            + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", compressed)
        + png_chunk(b"IEND", b"")
    )
    png_path.write_bytes(png)


def run_capture(args: argparse.Namespace) -> None:
    iso_path = Path(args.iso).resolve()
    output_png = Path(args.output).resolve()
    serial_log = Path(args.serial_log).resolve()
    qmp_socket = Path(args.qmp_socket).resolve()
    qemu_log = Path(args.qemu_log).resolve()
    output_ppm = output_png.with_suffix(".ppm")

    if not iso_path.exists():
        raise FileNotFoundError(f"ISO not found: {iso_path}")

    output_png.parent.mkdir(parents=True, exist_ok=True)
    serial_log.parent.mkdir(parents=True, exist_ok=True)
    qemu_log.parent.mkdir(parents=True, exist_ok=True)

    for path in (output_png, output_ppm, serial_log, qmp_socket):
        if path.exists():
            path.unlink()

    qemu_cmd = [
        args.qemu_binary,
        "-cdrom",
        str(iso_path),
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

    print(f"[screenshot] Starting QEMU: {' '.join(qemu_cmd)}")
    proc = subprocess.Popen(
        qemu_cmd,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
    )

    try:
        wait_for_serial_marker(serial_log, args.marker, args.timeout_seconds, process=proc)
        print(
            f"[screenshot] Marker detected; waiting {args.settle_seconds:.1f}s before capture..."
        )
        time.sleep(args.settle_seconds)

        qmp_sock = connect_qmp(qmp_socket, timeout_seconds=5.0)
        with qmp_sock:
            stream = qmp_sock.makefile("rwb", buffering=0)
            _ = read_qmp_message(stream)  # Greeting
            qmp_execute(stream, {"execute": "qmp_capabilities"})
            qmp_execute(
                stream,
                {"execute": "screendump", "arguments": {"filename": str(output_ppm)}},
            )
            qmp_execute(stream, {"execute": "quit"})

        try:
            proc.wait(timeout=5.0)
        except subprocess.TimeoutExpired:
            proc.terminate()
            proc.wait(timeout=5.0)

        convert_ppm_to_png(output_ppm, output_png)
        if not args.keep_ppm and output_ppm.exists():
            output_ppm.unlink()

        print(f"[screenshot] Saved screenshot: {output_png}")
        print(f"[screenshot] Serial log: {serial_log}")
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


def build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Boot QEMU, wait for first displayed desktop frame marker, "
            "then capture screenshot to PNG."
        )
    )
    parser.add_argument("--iso", required=True, help="Path to bootable ISO")
    parser.add_argument(
        "--output",
        default="build/desktop-shell-grid.png",
        help="Output PNG path",
    )
    parser.add_argument(
        "--serial-log",
        default="build/desktop-shell-boot.log",
        help="Serial log file path",
    )
    parser.add_argument(
        "--qmp-socket",
        default="/tmp/alloy-qmp-screenshot.sock",
        help="QMP unix socket path",
    )
    parser.add_argument(
        "--qemu-log",
        default="build/qemu-screenshot.log",
        help="QEMU debug log path",
    )
    parser.add_argument(
        "--marker",
        default="[DisplayServer] First frame presented",
        help="Serial log marker to wait for before capture",
    )
    parser.add_argument(
        "--settle-seconds",
        type=float,
        default=5.0,
        help="Seconds to wait after marker before taking screenshot",
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
        help="Keep intermediate PPM dump next to output PNG",
    )
    return parser


def main() -> int:
    parser = build_arg_parser()
    args = parser.parse_args()

    try:
        run_capture(args)
        return 0
    except Exception as exc:
        print(f"[screenshot] ERROR: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

