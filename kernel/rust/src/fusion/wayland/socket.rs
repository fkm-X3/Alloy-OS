//! Unix domain socket management for Wayland connections
//!
//! Provides socket creation, binding, listening, and accepting connections
//! at the standard Wayland socket path.

use crate::ffi;

use super::WaylandError;

/// Constants for Unix domain sockets
const AF_UNIX: u32 = 1;
const SOCK_STREAM: u32 = 1;
const PF_UNIX: u32 = 1;

// Max backlog for listen queue
const LISTEN_BACKLOG: u32 = 5;

/// Unix domain socket for Wayland connections
pub struct UnixSocket {
    /// Socket file descriptor
    fd: Option<u32>,
    /// Whether socket is bound
    bound: bool,
    /// Whether socket is listening
    listening: bool,
}

impl UnixSocket {
    /// Create a new Unix domain socket
    pub fn new() -> Result<Self, WaylandError> {
        // NOTE: socket() syscall would be called here in a real implementation
        // For now, we create a placeholder that the kernel integration will handle
        
        let socket = Self {
            fd: Some(0), // Placeholder FD; will be replaced by actual syscall
            bound: false,
            listening: false,
        };

        unsafe {
            ffi::serial_print(b"[Wayland Socket] Created Unix domain socket\n\0".as_ptr());
        }

        Ok(socket)
    }

    /// Bind socket to the standard Wayland path
    pub fn bind(&mut self, path: &str) -> Result<(), WaylandError> {
        if self.fd.is_none() {
            return Err(WaylandError::InvalidFd);
        }

        // Ensure path is within reasonable length for Unix socket (typically 108 bytes)
        if path.len() > 107 {
            return Err(WaylandError::ProtocolViolation);
        }

        // In a real implementation, would call bind() syscall here:
        // bind(fd, sockaddr_un { sun_family: AF_UNIX, sun_path: path }, ...)
        
        self.bound = true;

        unsafe {
            ffi::serial_print(b"[Wayland Socket] Bound to \0".as_ptr());
            // Print the path in groups of chars (can't easily print a string slice)
            for byte in path.as_bytes().iter() {
                ffi::vga_putchar(*byte);
            }
            ffi::serial_print(b"\n\0".as_ptr());
        }

        Ok(())
    }

    /// Start listening for incoming connections
    pub fn listen(&mut self) -> Result<(), WaylandError> {
        if !self.bound {
            return Err(WaylandError::SocketBindFailed);
        }

        if self.fd.is_none() {
            return Err(WaylandError::InvalidFd);
        }

        // In a real implementation, would call listen() syscall
        // listen(fd, LISTEN_BACKLOG)
        
        self.listening = true;

        unsafe {
            ffi::serial_print(b"[Wayland Socket] Listening for connections\n\0".as_ptr());
        }

        Ok(())
    }

    /// Accept an incoming client connection
    pub fn accept(&self) -> Result<u32, WaylandError> {
        if !self.listening {
            return Err(WaylandError::SocketListenFailed);
        }

        if self.fd.is_none() {
            return Err(WaylandError::InvalidFd);
        }

        // In a real implementation, would call accept() syscall
        // accept(fd, NULL, NULL) which returns new client FD
        // For now, return a placeholder FD
        
        let client_fd = 4; // Placeholder client FD

        unsafe {
            ffi::serial_print(b"[Wayland Socket] Accepted connection on fd \0".as_ptr());
        }

        Ok(client_fd)
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
        if let Some(_fd) = self.fd {
            // In a real implementation, would call close() syscall
            // close(fd)
            
            unsafe {
                ffi::serial_print(b"[Wayland Socket] Closed socket\n\0".as_ptr());
            }
        }
    }
}
