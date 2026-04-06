# GDB initialization file for Alloy OS kernel debugging
#
# Usage:
#   1. Run 'make debug' in one terminal (starts QEMU waiting for GDB)
#   2. Run 'gdb build/alloy.elf' in another terminal
#   3. GDB will automatically load this file and connect to QEMU

# Connect to QEMU (listening on localhost:1234)
target remote localhost:1234

# Load kernel symbols
symbol-file build/alloy.elf

# Set architecture
set architecture i386

# Enable TUI mode for better visibility
layout src
layout regs

# Useful breakpoints
# Uncomment as needed:
# break kernel_main
# break rust_main
# break panic_handler

# Display helpful information
echo \n
echo ╔═══════════════════════════════════════════════╗\n
echo ║  Alloy OS Kernel Debugger                     ║\n
echo ╚═══════════════════════════════════════════════╝\n
echo \n
echo Connected to QEMU on localhost:1234\n
echo Kernel ELF loaded: build/alloy.elf\n
echo \n
echo Useful commands:\n
echo   continue (c)  - Resume execution\n
echo   step (s)      - Step one instruction\n
echo   next (n)      - Step over function calls\n
echo   break <func>  - Set breakpoint\n
echo   info reg      - Show registers\n
echo   x/10i $eip    - Disassemble at current location\n
echo   bt            - Show backtrace\n
echo \n
echo Kernel symbols loaded. Ready to debug.\n
echo \n

# Set convenient aliases
define hook-stop
    info registers
    x/5i $eip
end
