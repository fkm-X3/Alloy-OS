//! The safe public boundary of `alloy-kernel-unsafe-core`.
//!
//! This is the only surface the safe kernel and the HAL are allowed to use.
//! Every item here must be callable from safe code with no UB possible; raw
//! pointers never cross this boundary.
//!
//! This module re-exports the moved HAL trait definitions, impls, and data
//! types verbatim so `alloy-kernel-hal` can keep exposing its existing
//! public API (`Arch`, `MemoryManager`, `InterruptController`, `SerialPort`,
//! `Timer`, `IoPort`, `Mmio`, ...) without writing any code that contains
//! `unsafe`. As the driver/arch/mem ports land, the raw impls retreat behind
//! safe wrappers and this module becomes the documented safe API.
//!
//! Submodules: io, mem, interrupt, serial, time, drivers, alloc, sync,
//! arch, callback.

pub use crate::alloc::{self, get_stats, KernelAllocator, Slab};
pub use crate::arch::{self, Arch, CpuContext, CpuInfo, cpu_halt};
#[cfg(feature = "x86_64")]
pub use crate::arch::cpu_sti_halt;
pub use crate::callback::{self, FaultAction, SyscallHandler, SyscallTable};
pub use crate::callback::{set_keyboard_wake_handler, set_mouse_wake_handler};
pub use crate::callback::{set_page_fault_handler, set_timer_tick_handler};
pub use crate::interrupt::{self, InterruptController, InterruptGuard, IrqHandler, IrqLine};
pub use crate::io::{self, DefaultMmio, Mmio, MmioReg};
#[cfg(feature = "x86_64")]
pub use crate::io::X86IoPort;
#[cfg(feature = "x86_64")]
pub use crate::io::IoPort;
pub use crate::mem::{self, AddressSpace, PageFlags, PhysFrame, VmRegion};
pub use crate::mem::{heap_start, heap_size, allocated_pages};
pub use crate::mem::user::{copy_from_user, copy_to_user};
pub use crate::arch::syscall_no;
#[cfg(feature = "x86_64")]
pub use crate::arch::x86_64::{current_user_syscall_frame, UserSyscallFrame};
pub use crate::memory::{self, MemoryManager};
pub use crate::serial::{self, SerialPort};
pub use crate::sync::{
    self, irq_disable, irq_enable, irq_restore, irq_save, SpinLock, SpinLockIrq,
};
pub use crate::time::{self, Timer};

// --- Safe driver facades ---
pub use crate::drivers::serial::Serial;
pub use crate::drivers::timer::SystemTimer;
#[cfg(feature = "x86_64")]
pub use crate::drivers::vga::VgaText;
#[cfg(feature = "x86_64")]
pub use crate::drivers::keyboard::{KeyEvent, Keyboard};
#[cfg(feature = "x86_64")]
pub use crate::drivers::keyboard::{
    SPECIAL_KEY_DELETE, SPECIAL_KEY_DOWN, SPECIAL_KEY_END, SPECIAL_KEY_HOME,
    SPECIAL_KEY_LEFT, SPECIAL_KEY_PGDN, SPECIAL_KEY_PGUP, SPECIAL_KEY_RIGHT, SPECIAL_KEY_UP,
};
#[cfg(feature = "x86_64")]
pub use crate::drivers::mouse::{Mouse, MouseEvent};
#[cfg(feature = "x86_64")]
pub use crate::drivers::mouse::{
    MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT, MOUSE_EVENT_FLAG_X_OVERFLOW,
    MOUSE_EVENT_FLAG_Y_OVERFLOW, MOUSE_INIT_ERR_ENABLE_STREAMING,
    MOUSE_INIT_ERR_ENABLE_STREAMING_ACK, MOUSE_INIT_ERR_INPUT_NOT_READY, MOUSE_INIT_ERR_NONE,
    MOUSE_INIT_ERR_OUTPUT_NOT_READY, MOUSE_INIT_ERR_SET_DEFAULTS, MOUSE_INIT_ERR_SET_DEFAULTS_ACK,
};
#[cfg(feature = "aarch64")]
pub use crate::drivers::pl110::Pl110;
#[cfg(feature = "x86_64")]
pub use crate::drivers::pci::{Pci, PciDevice};
#[cfg(feature = "x86_64")]
pub use crate::drivers::ata::{Ata, AtaDriveInfo};
#[cfg(feature = "x86_64")]
pub use crate::drivers::ahci::{Ahci, AhciDriveInfo};
#[cfg(feature = "x86_64")]
pub use crate::drivers::initrd::{Initrd, InitrdModule};
#[cfg(feature = "x86_64")]
pub use crate::drivers::vesa::{Vesa, VesaError, VesaInfo};
