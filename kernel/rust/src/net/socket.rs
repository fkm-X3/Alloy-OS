use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use crate::sync::SpinLock;
use crate::process::WaitQueue;
use crate::process::Scheduler;

pub const AF_UNIX: i32 = 1;
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const LISTEN_BACKLOG: usize = 8;
pub const SOCKET_BUFFER_SIZE: usize = 4096;

const MAX_SOCKETS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SocketState {
    #[allow(dead_code)]
    Free,
    Created,
    Bound,
    Listening,
    Connected,
}

#[derive(Debug)]
struct SocketInner {
    state: SocketState,
    #[allow(dead_code)]
    domain: i32,
    #[allow(dead_code)]
    socket_type: i32,
    bound_path: Option<String>,
    pending_connections: VecDeque<u32>,
    read_buffer: VecDeque<u8>,
    write_buffer: VecDeque<u8>,
    peer_fd: Option<u32>,
}

impl SocketInner {
    fn new(domain: i32, socket_type: i32) -> Self {
        Self {
            state: SocketState::Created,
            domain,
            socket_type,
            bound_path: None,
            pending_connections: VecDeque::new(),
            read_buffer: VecDeque::new(),
            write_buffer: VecDeque::new(),
            peer_fd: None,
        }
    }
}

struct SocketTable {
    sockets: [Option<SocketInner>; MAX_SOCKETS],
    next_fd: u32,
}

impl SocketTable {
    fn new() -> Self {
        let mut sockets = alloc::vec::Vec::with_capacity(MAX_SOCKETS);
        for _ in 0..MAX_SOCKETS {
            sockets.push(None);
        }
        Self {
            sockets: sockets.try_into().unwrap(),
            next_fd: 3,
        }
    }

    fn alloc_fd(&mut self) -> Option<u32> {
        for i in 0..MAX_SOCKETS {
            if self.sockets[i].is_none() {
                let fd = self.next_fd;
                self.next_fd = self.next_fd.wrapping_add(1);
                if self.next_fd >= MAX_SOCKETS as u32 {
                    self.next_fd = 3;
                }
                return Some(fd);
            }
        }
        None
    }

    fn insert(&mut self, fd: u32, socket: SocketInner) {
        let idx = fd as usize;
        if idx < MAX_SOCKETS {
            self.sockets[idx] = Some(socket);
        }
    }

    fn get(&self, fd: u32) -> Option<&SocketInner> {
        let idx = fd as usize;
        if idx < MAX_SOCKETS {
            self.sockets[idx].as_ref()
        } else {
            None
        }
    }

    fn get_mut(&mut self, fd: u32) -> Option<&mut SocketInner> {
        let idx = fd as usize;
        if idx < MAX_SOCKETS {
            self.sockets[idx].as_mut()
        } else {
            None
        }
    }

    fn remove(&mut self, fd: u32) -> Option<SocketInner> {
        let idx = fd as usize;
        if idx < MAX_SOCKETS {
            self.sockets[idx].take()
        } else {
            None
        }
    }

    fn find_by_path(&self, path: &str) -> Option<u32> {
        for (i, sock) in self.sockets.iter().enumerate() {
            if let Some(s) = sock {
                if let Some(ref p) = s.bound_path {
                    if p == path {
                        return Some(i as u32);
                    }
                }
            }
        }
        None
    }

    fn has_pending_connections(&self, fd: u32) -> bool {
        match self.get(fd) {
            Some(s) => !s.pending_connections.is_empty(),
            None => false,
        }
    }
}

static SOCKET_TABLE: SpinLock<Option<SocketTable>> = SpinLock::new(None);

/// Wait queue for socket reads — tasks block here when no data is available.
static SOCKET_READ_WAIT: WaitQueue = WaitQueue::new();

fn ensure_table() {
    let mut guard = SOCKET_TABLE.lock();
    if guard.is_none() {
        *guard = Some(SocketTable::new());
    }
}

pub fn socket_create(domain: i32, socket_type: i32, _protocol: i32) -> i32 {
    if domain != AF_UNIX {
        return -1;
    }
    if socket_type != SOCK_STREAM && socket_type != SOCK_DGRAM {
        return -1;
    }

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();
    match table.alloc_fd() {
        Some(fd) => {
            let socket = SocketInner::new(domain, socket_type);
            table.insert(fd, socket);
            fd as i32
        }
        None => -1,
    }
}

pub fn socket_bind(fd: i32, path: &str) -> i32 {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();

    let socket_state = match table.get(ufd) {
        Some(s) => s.state,
        None => return -1,
    };

    if socket_state != SocketState::Created {
        return -1;
    }

    if table.find_by_path(path).is_some() {
        return -1;
    }

    if let Some(socket) = table.get_mut(ufd) {
        socket.bound_path = Some(path.to_string());
        socket.state = SocketState::Bound;
    }
    0
}

pub fn socket_listen(fd: i32, _backlog: i32) -> i32 {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();
    let socket = match table.get_mut(ufd) {
        Some(s) => s,
        None => return -1,
    };

    if socket.state != SocketState::Bound {
        return -1;
    }

    socket.state = SocketState::Listening;
    0
}

pub fn socket_accept(fd: i32) -> i32 {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();
    let socket = match table.get_mut(ufd) {
        Some(s) => s,
        None => return -1,
    };

    if socket.state != SocketState::Listening {
        return -1;
    }

    match socket.pending_connections.pop_front() {
        Some(client_fd) => client_fd as i32,
        None => -1,
    }
}

pub fn socket_connect(fd: i32, path: &str) -> i32 {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();

    let client_state = match table.get(ufd) {
        Some(s) => s.state,
        None => return -1,
    };

    if client_state != SocketState::Created && client_state != SocketState::Bound {
        return -1;
    }

    let server_fd = match table.find_by_path(path) {
        Some(sfd) => sfd,
        None => return -1,
    };

    let server_listening = match table.get(server_fd) {
        Some(s) => s.state == SocketState::Listening,
        None => return -1,
    };

    if !server_listening {
        return -1;
    }

    let client_fd = match table.alloc_fd() {
        Some(cfd) => cfd,
        None => return -1,
    };

    let server_peer = SocketInner::new(AF_UNIX, SOCK_STREAM);
    table.insert(client_fd, server_peer);

    if let Some(cs) = table.get_mut(client_fd) {
        cs.state = SocketState::Connected;
        cs.peer_fd = Some(ufd);
    }

    if let Some(client_sock) = table.get_mut(ufd) {
        client_sock.state = SocketState::Connected;
        client_sock.peer_fd = Some(client_fd);
    }

    if let Some(ss) = table.get_mut(server_fd) {
        ss.pending_connections.push_back(client_fd);
    }

    0
}

pub fn socket_read(fd: i32, buf: &mut [u8]) -> isize {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    loop {
        ensure_table();
        let mut guard = SOCKET_TABLE.lock();
        let table = guard.as_mut().unwrap();
        let socket = match table.get_mut(ufd) {
            Some(s) => s,
            None => return -1,
        };

        if socket.state != SocketState::Connected {
            return -1;
        }

        let mut bytes_read = 0;
        for b in buf.iter_mut() {
            match socket.read_buffer.pop_front() {
                Some(byte) => {
                    *b = byte;
                    bytes_read += 1;
                }
                None => break,
            }
        }

        if bytes_read > 0 {
            return bytes_read as isize;
        }

        // No data available — block until something is written
        drop(guard);
        Scheduler::block_current_on(&SOCKET_READ_WAIT);
    }
}

pub fn socket_write(fd: i32, buf: &[u8]) -> isize {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();
    let socket = match table.get_mut(ufd) {
        Some(s) => s,
        None => return -1,
    };

    if socket.state != SocketState::Connected {
        return -1;
    }

    let peer_fd = match socket.peer_fd {
        Some(pfd) => pfd,
        None => return -1,
    };

    let bytes_to_write = core::cmp::min(buf.len(), SOCKET_BUFFER_SIZE - socket.write_buffer.len());
    if bytes_to_write == 0 {
        return -1;
    }

    for &b in buf.iter().take(bytes_to_write) {
        socket.write_buffer.push_back(b);
    }

    let had_data = if let Some(peer) = table.get_mut(peer_fd) {
        let was_empty = peer.read_buffer.is_empty();
        for &b in buf.iter().take(bytes_to_write) {
            if peer.read_buffer.len() < SOCKET_BUFFER_SIZE {
                peer.read_buffer.push_back(b);
            }
        }
        was_empty && bytes_to_write > 0
    } else {
        false
    };

    drop(guard);

    // Wake any readers blocking on this socket
    if had_data {
        Scheduler::wake_waiters(&SOCKET_READ_WAIT, 1);
    }

    bytes_to_write as isize
}

pub fn socket_close(fd: i32) -> i32 {
    if fd < 0 {
        return -1;
    }
    let ufd = fd as u32;

    ensure_table();
    let mut guard = SOCKET_TABLE.lock();
    let table = guard.as_mut().unwrap();
    match table.remove(ufd) {
        Some(socket) => {
            if let Some(peer_fd) = socket.peer_fd {
                if let Some(peer) = table.get_mut(peer_fd) {
                    peer.peer_fd = None;
                    peer.state = SocketState::Created;
                }
            }
            0
        }
        None => -1,
    }
}

pub fn socket_has_pending_connections(fd: i32) -> i32 {
    if fd < 0 {
        return 0;
    }
    let ufd = fd as u32;

    ensure_table();
    let guard = SOCKET_TABLE.lock();
    let table = guard.as_ref().unwrap();
    if table.has_pending_connections(ufd) {
        1
    } else {
        0
    }
}
