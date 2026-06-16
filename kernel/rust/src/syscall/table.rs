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
pub const SOCKET: u32 = 12;
pub const BIND: u32 = 13;
pub const LISTEN: u32 = 14;
pub const ACCEPT: u32 = 15;
pub const CONNECT: u32 = 16;
pub const CLOSE_SOCKET: u32 = 17;
pub const WAITPID: u32 = 22;
pub const CLONE: u32 = 21;
pub const FORK: u32 = 20;
pub const BRK: u32 = 19;
pub const HAS_PENDING_CONNECTIONS: u32 = 18;
pub const SOCKET_READ: u32 = 23;
pub const SOCKET_WRITE: u32 = 24;
pub const ALLOC_SHM: u32 = 25;
pub const SHM_USER_VADDR: u32 = 26;

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
    Socket = SOCKET,
    Bind = BIND,
    Listen = LISTEN,
    Accept = ACCEPT,
    Connect = CONNECT,
    CloseSocket = CLOSE_SOCKET,
    HasPendingConnections = HAS_PENDING_CONNECTIONS,
    Brk = BRK,
    Fork = FORK,
    Clone = CLONE,
    WaitPid = WAITPID,
    SocketRead = SOCKET_READ,
    SocketWrite = SOCKET_WRITE,
    AllocShm = ALLOC_SHM,
    ShmUserVaddr = SHM_USER_VADDR,
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
            SOCKET => Some(SyscallNumber::Socket),
            BIND => Some(SyscallNumber::Bind),
            LISTEN => Some(SyscallNumber::Listen),
            ACCEPT => Some(SyscallNumber::Accept),
            CONNECT => Some(SyscallNumber::Connect),
            CLOSE_SOCKET => Some(SyscallNumber::CloseSocket),
            HAS_PENDING_CONNECTIONS => Some(SyscallNumber::HasPendingConnections),
            BRK => Some(SyscallNumber::Brk),
            FORK => Some(SyscallNumber::Fork),
            CLONE => Some(SyscallNumber::Clone),
            WAITPID => Some(SyscallNumber::WaitPid),
            SOCKET_READ => Some(SyscallNumber::SocketRead),
            SOCKET_WRITE => Some(SyscallNumber::SocketWrite),
            ALLOC_SHM => Some(SyscallNumber::AllocShm),
            SHM_USER_VADDR => Some(SyscallNumber::ShmUserVaddr),
            _ => None,
        }
    }
}