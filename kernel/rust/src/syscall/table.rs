// Syscall number table and helpers

/// Syscall number constants (should match syscall.h)
pub const EXIT: u32 = 0;
pub const YIELD: u32 = 1;
pub const GETPID: u32 = 2;
pub const SLEEP: u32 = 3;
pub const OPEN: u32 = 4;
pub const READ: u32 = 5;
pub const WRITE: u32 = 6;
pub const CLOSE: u32 = 7;
pub const DUP: u32 = 8;
pub const LSEEK: u32 = 9;
pub const PIPE: u32 = 10;
pub const EXECVE: u32 = 11;

/// Enum representation of syscall numbers
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SyscallNumber {
    Exit = EXIT,
    Yield = YIELD,
    GetPid = GETPID,
    Sleep = SLEEP,
    Open = OPEN,
    Read = READ,
    Write = WRITE,
    Close = CLOSE,
    Dup = DUP,
    Lseek = LSEEK,
    Pipe = PIPE,
    Execve = EXECVE,
}

impl SyscallNumber {
    pub fn from_u32(n: u32) -> Option<SyscallNumber> {
        match n {
            EXIT => Some(SyscallNumber::Exit),
            YIELD => Some(SyscallNumber::Yield),
            GETPID => Some(SyscallNumber::GetPid),
            SLEEP => Some(SyscallNumber::Sleep),
            OPEN => Some(SyscallNumber::Open),
            READ => Some(SyscallNumber::Read),
            WRITE => Some(SyscallNumber::Write),
            CLOSE => Some(SyscallNumber::Close),
            DUP => Some(SyscallNumber::Dup),
            LSEEK => Some(SyscallNumber::Lseek),
            PIPE => Some(SyscallNumber::Pipe),
            EXECVE => Some(SyscallNumber::Execve),
            _ => None,
        }
    }
}
