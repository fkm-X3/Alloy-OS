import struct
import subprocess
import sys
import tempfile
import os
import shutil


DEFAULT_SIZE_MB = 64
DEFAULT_OUTPUT = "build/fat32_test.img"


def create_fat32_image(output_path: str, size_mb: int) -> None:
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)

    # Create empty disk image
    with open(output_path, "wb") as f:
        f.truncate(size_mb * 1024 * 1024)

    # Format as FAT32
    try:
        subprocess.run(
            ["mkfs.fat", "-F", "32", output_path],
            check=True, capture_output=True,
        )
        print(f"Formatted {output_path} as FAT32 ({size_mb}MiB)")
    except FileNotFoundError:
        print("WARNING: mkfs.fat not found. Install dosfstools.")
        print(f"Created empty image at {output_path}")
        return

    # Mount and populate with sample files using mdir/mcopy from mtools
    # (mtools uses MTOOLS_SKIP_CHECK=1 to skip fsck)
    env = {**os.environ, "MTOOLS_SKIP_CHECK": "1"}

    try:
        # Create root-level files
        subprocess.run(
            ["mcopy", "-i", output_path, "-", "::README.TXT"],
            input=b"Welcome to Alloy OS!\nThis is the FAT32 test volume.\n",
            check=True, capture_output=True, env=env,
        )
        subprocess.run(
            ["mcopy", "-i", output_path, "-", "::HELLO.TXT"],
            input=b"Hello from Alloy OS kernel!\nFAT32 read test successful.\n",
            check=True, capture_output=True, env=env,
        )

        # Create a subdirectory with files
        subprocess.run(
            ["mmd", "-i", output_path, "::TESTDIR"],
            check=True, capture_output=True, env=env,
        )
        subprocess.run(
            ["mcopy", "-i", output_path, "-", "::TESTDIR\\DATA.BIN"],
            input=bytes(range(256)),
            check=True, capture_output=True, env=env,
        )
        subprocess.run(
            ["mcopy", "-i", output_path, "-", "::TESTDIR\\NOTE.TXT"],
            input=b"Test directory content.\nLine 2.\nLine 3.\n",
            check=True, capture_output=True, env=env,
        )

        print("Populated with sample files:")
        subprocess.run(
            ["mdir", "-i", output_path, "-/", "::"],
            check=True, env=env,
        )

    except FileNotFoundError:
        print("WARNING: mtools not found. Image created but not populated.")
    except subprocess.CalledProcessError as e:
        print(f"WARNING: mtools error: {e.stderr.decode()}")


if __name__ == "__main__":
    output = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_OUTPUT
    size = int(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_SIZE_MB
    create_fat32_image(output, size)
