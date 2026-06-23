#Requires -Version 7.0

<#
.SYNOPSIS
    Alloy OS Development Environment Setup (PowerShell)
.DESCRIPTION
    Installs the required toolchains for building Alloy OS on Windows:
      - MSYS2 (if missing): provides GCC, nasm, make, QEMU, aarch64 cross-compiler
      - Rust nightly (if missing): builds the Rust kernel components
      - Python 3 (if missing): required for screenshot/smoke-test scripts
.NOTES
    This script will self-elevate to Administrator if not already running as admin.
    After completion, use `make` from a Git Bash / MSYS2 shell:
      make ARCH=i686       # 32-bit x86
      make ARCH=x86_64     # 64-bit x86
      make ARCH=aarch64    # ARM64
#>

$ErrorActionPreference = 'Stop'

# --------------- self-elevation ---------------
$currentPrincipal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "Not running as Administrator. Elevating..." -ForegroundColor Yellow
    Start-Process -FilePath pwsh -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs
    exit
}

# --------------- helpers ---------------
function Test-Command($cmd) {
    try { $null = Get-Command $cmd -ErrorAction Stop; return $true }
    catch { return $false }
}

function Install-Msys2Package($pkg) {
    Write-Host "  Installing $pkg ..." -ForegroundColor Cyan
    & "$msys2Root\usr\bin\bash.exe" -lc "pacman -S --noconfirm --needed $pkg 2>&1"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  WARNING: '$pkg' installation failed (exit $LASTEXITCODE)." -ForegroundColor Yellow
    }
}

# --------------- 1. MSYS2 ---------------
Write-Host "`n=== Checking MSYS2 ===" -ForegroundColor Green

# Common MSYS2 install locations
$msys2Candidates = @(
    "C:\msys64",
    "C:\msys32",
    "$env:ProgramFiles\msys64",
    "${env:ProgramFiles(x86)}\msys2",
    "$env:LOCALAPPDATA\msys64",
    "$env:USERPROFILE\scoop\apps\msys2\current"
)

$msys2Root = $null
foreach ($dir in $msys2Candidates) {
    if (Test-Path "$dir\usr\bin\bash.exe") {
        $msys2Root = $dir
        break
    }
}

if (-not $msys2Root) {
    Write-Host "MSYS2 not found. Installing via winget..." -ForegroundColor Yellow
    if (Test-Command winget) {
        winget install --id MSYS2.MSYS2 --silent --accept-source-agreements
        # winget installs to C:\msys64 by default
        $msys2Root = "C:\msys64"
        if (-not (Test-Path "$msys2Root\usr\bin\bash.exe")) {
            # Retry default location after install
            $msys2Root = "${env:ProgramFiles(x86)}\MSYS2"
        }
        if (-not (Test-Path "$msys2Root\usr\bin\bash.exe")) {
            Write-Host "MSYS2 installation path unknown. Please install manually from https://www.msys2.org/" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "winget not available. Install MSYS2 manually from https://www.msys2.org/" -ForegroundColor Red
        Write-Host "Then re-run this script." -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "Found MSYS2 at: $msys2Root" -ForegroundColor Green
}

# Update package databases (safe to run even if already up-to-date)
Write-Host "Updating MSYS2 package databases..." -ForegroundColor Cyan
& "$msys2Root\usr\bin\bash.exe" -lc "pacman -Sy --noconfirm 2>&1"

# --------------- 2. MSYS2 packages ---------------
Write-Host "`n=== Installing MSYS2 packages ===" -ForegroundColor Green

# Base devel
Install-Msys2Package "make"
Install-Msys2Package "diffutils"
Install-Msys2Package "patch"

# NASM (assembler)
Install-Msys2Package "mingw-w64-ucrt-x86_64-nasm"

# GCC cross-compilers
Install-Msys2Package "mingw-w64-ucrt-x86_64-gcc"       # x86_64 native
Install-Msys2Package "mingw-w64-i686-gcc"               # i686 cross
Install-Msys2Package "mingw-w64-ucrt-x86_64-aarch64-linux-gnu-gcc"  # aarch64 cross

# QEMU
Install-Msys2Package "mingw-w64-ucrt-x86_64-qemu"

# Python (for screenshot/smoke-test scripts)
Install-Msys2Package "mingw-w64-ucrt-x86_64-python"

# --------------- 3. Rust nightly ---------------
Write-Host "`n=== Checking Rust nightly ===" -ForegroundColor Green

$rustup = if (Test-Command rustup) { "rustup" }
          elseif (Test-Command "$env:USERPROFILE\.cargo\bin\rustup.exe") { "$env:USERPROFILE\.cargo\bin\rustup.exe" }
          else { $null }

if (-not $rustup) {
    Write-Host "rustup not found. Installing..." -ForegroundColor Yellow
    & powershell -NoProfile -Command "iex (iwr 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -UseBasicParsing).Content" /VERYSILENT /NORESTART
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
    $rustup = "$env:USERPROFILE\.cargo\bin\rustup.exe"
}

if ($rustup) {
    Write-Host "Installing Rust nightly toolchain..." -ForegroundColor Cyan
    & $rustup toolchain install nightly --allow-downgrade
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Rust nightly installed." -ForegroundColor Green
    }
} else {
    Write-Host "WARNING: Could not install Rust. Do so manually: https://rustup.rs" -ForegroundColor Yellow
}

# --------------- 4. Verify ---------------
Write-Host "`n=== Verification ===" -ForegroundColor Green

$msysBash = "$msys2Root\usr\bin\bash.exe"

function Check-Tool($name, $cmd) {
    $result = & $msysBash -lc "command -v $cmd 2>&1"
    if ($LASTEXITCODE -eq 0 -and $result) {
        Write-Host "  $name : $result" -ForegroundColor Green
    } else {
        Write-Host "  $name : NOT FOUND" -ForegroundColor Red
    }
}

# Run checks inside MSYS2 so PATH is correct
Check-Tool "make (MSYS2)" "make"
Check-Tool "nasm" "nasm"
Check-Tool "gcc (x86_64)" "x86_64-w64-mingw32-gcc"
Check-Tool "gcc (i686)" "i686-w64-mingw32-gcc"
Check-Tool "gcc (aarch64)" "aarch64-linux-gnu-gcc"
Check-Tool "qemu-system-i386" "qemu-system-i386"
Check-Tool "qemu-system-x86_64" "qemu-system-x86_64"
Check-Tool "qemu-system-aarch64" "qemu-system-aarch64"
Check-Tool "python3" "python3"

if (Test-Command rustc) {
    $rv = rustc --version
    Write-Host "  rustc : $rv" -ForegroundColor Green
} else {
    Write-Host "  rustc : NOT FOUND (install from https://rustup.rs)" -ForegroundColor Red
}

if (Test-Command cargo) {
    $cv = cargo --version
    Write-Host "  cargo : $cv" -ForegroundColor Green
} else {
    Write-Host "  cargo : NOT FOUND" -ForegroundColor Red
}

# --------------- summary ---------------
Write-Host @'

============================================
Development toolchain installed!
============================================

MSYS2 root: $msys2Root

To build, open a Git Bash or MSYS2 shell and run:

  make ARCH=i686       # 32-bit x86 build
  make ARCH=x86_64     # 64-bit x86 build
  make ARCH=aarch64    # ARM64 build

NOTE: Use 'make run-elf' / 'make output-elf' instead of 'make run' / 'make output'
      (grub-mkrescue is not available on Windows).

NOTE: Add to your PowerShell profile to use MSYS2 make/gcc from any terminal:
  `$env:CHERE_INVOKING = 1`
  `$env:MSYSTEM = 'UCRT64'`
  `Set-Alias msys2 '$msys2Root\usr\bin\bash.exe'`
'@
