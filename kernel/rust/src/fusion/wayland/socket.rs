//! Unix domain socket management for Wayland connections
//!
//! Provides socket creation, binding, listening, and accepting connections
//! at the standard Wayland socket path. Supports both direct kernel syscall
//! and fallback modes for environments where syscalls aren't yet available.

use super::WaylandError;

/// Constants for Unix domain sockets
const AF_UNIX: u32 = 1;
const SOCK_STREAM: u32 = 1;
#[allow(dead_code)]
const PF_UNIX: u32 = 1;

// Max backlog for listen queue
const LISTEN_BACKLOG: u32 = 5;

/// Unix domain socket for Wayland connections (primary - uses syscalls)
pub struct UnixSocket {
    /// Socket file descriptor
    fd: Option<u32>,
    /// Whether socket is bound
    bound: bool,
    /// Whether socket is listening
    listening: bool,
    /// Socket path for bind
    path: Option<[u8; 108]>,
}

impl UnixSocket {
    /// Create a new Unix domain socket
    pub fn new() -> Result<Self, WaylandError> {
        // Attempt to use syscall first
        let fd = {
            let result = crate::syscall::rust_sys_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0);
            if result >= 0 {
                Some(result as u32)
            } else {
                None
            }
        };

        match fd {
            Some(f) => {
                crate::println!("[Wayland Socket] Created Unix domain socket via syscall");
                Ok(Self {
                    fd: Some(f),
                    bound: false,
                    listening: false,
                    path: None,
                })
            }
            None => {
                crate::println!("[Wayland Socket] Falling back to placeholder socket");
                // Fallback for environments without full syscall support
                Ok(Self {
                    fd: Some(0), // Placeholder FD
                    bound: false,
                    listening: false,
                    path: None,
                })
            }
        }
    }

    /// Bind socket to the standard Wayland path
    pub fn bind(&mut self, path: &str) -> Result<(), WaylandError> {
        let fd = self.fd.ok_or(WaylandError::InvalidFd)?;

        // Ensure path is within reasonable length for Unix socket (typically 108 bytes)
        if path.len() > 107 {
            return Err(WaylandError::ProtocolViolation);
        }

        // Build sockaddr_un structure: [family(2)][path(108)]
        let mut sockaddr: [u8; 110] = [0; 110];
        sockaddr[0] = AF_UNIX as u8;
        sockaddr[1] = (AF_UNIX >> 8) as u8;
        sockaddr[2..2 + path.len()].copy_from_slice(path.as_bytes());

        let result = crate::syscall::sys_bind_inner(fd as i32, &sockaddr, (path.len() + 2) as u32);

        if result < 0 {
            crate::println!("[Wayland Socket] bind() syscall failed, using fallback");
            // Mark as bound anyway for fallback mode
        }

        self.bound = true;

        // Store path for later reference
        let mut stored_path = [0u8; 108];
        stored_path[..path.len()].copy_from_slice(path.as_bytes());
        self.path = Some(stored_path);

        crate::println!("[Wayland Socket] Bound to {path}");

        Ok(())
    }

    /// Start listening for incoming connections
    pub fn listen(&mut self) -> Result<(), WaylandError> {
        crate::render_trace!("[T9] UnixSocket::listen() entered, fd={:?}", self.fd);
        if !self.bound {
            return Err(WaylandError::SocketBindFailed);
        }

        let fd = self.fd.ok_or(WaylandError::InvalidFd)?;

        let result = crate::syscall::rust_sys_listen(fd as i32, LISTEN_BACKLOG as i32);

        if result < 0 {
            crate::println!("[Wayland Socket] listen() syscall failed, using fallback");
        }

        self.listening = true;

        crate::println!("[Wayland Socket] Listening for connections");

        Ok(())
    }

    /// Accept an incoming client connection
    pub fn accept(&self) -> Result<u32, WaylandError> {
        if !self.listening {
            return Err(WaylandError::SocketListenFailed);
        }

        let fd = self.fd.ok_or(WaylandError::InvalidFd)?;

        let client_fd = crate::syscall::rust_sys_accept(fd as i32);

        if client_fd < 0 {
            return Err(WaylandError::AcceptFailed);
        }

        crate::print!("[Wayland Socket] Accepted connection on fd ");

        Ok(client_fd as u32)
    }

    /// Check if the socket has pending connections waiting to be accepted
    pub fn has_pending_connections(&self) -> bool {
        let fd = match self.fd {
            Some(f) => f,
            None => return false,
        };

        let result = crate::syscall::rust_sys_has_pending_connections(fd as i32);

        result == 1
    }

    /// Connect to an existing socket (for client-side usage)
    pub fn connect(&mut self, path: &str) -> Result<(), WaylandError> {
        let fd = self.fd.ok_or(WaylandError::InvalidFd)?;

        let mut sockaddr: [u8; 110] = [0; 110];
        sockaddr[0] = AF_UNIX as u8;
        sockaddr[1] = (AF_UNIX >> 8) as u8;
        sockaddr[2..2 + path.len()].copy_from_slice(path.as_bytes());

        let result = crate::syscall::sys_connect_inner(fd as i32, &sockaddr, (path.len() + 2) as u32);

        if result < 0 {
            return Err(WaylandError::SocketCreationFailed);
        }

        self.bound = true;
        Ok(())
    }

    /// Get the socket file descriptor
    pub fn fd(&self) -> Option<u32> {
        self.fd
    }

    /// Check if socket is bound
    pub fn is_bound(&self) -> bool {
        self.bound
    }

    /// Check if socket is listening
    pub fn is_listening(&self) -> bool {
        self.listening
    }
}

impl Drop for UnixSocket {
    /// Close the socket when dropped
    fn drop(&mut self) {
        if let Some(fd) = self.fd {
            crate::syscall::rust_sys_close_socket(fd as i32);
            crate::println!("[Wayland Socket] Closed socket");
        }
    }
}
