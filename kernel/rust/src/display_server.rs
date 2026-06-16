use crate::ffi;
use crate::graphics::PlatformDisplay;
use crate::fusion::FusionDisplayBackend;
use crate::fusion::WaylandServer;

fn serial_log(message: &'static [u8]) {
    unsafe {
        ffi::serial_print(message.as_ptr());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServerBootError {
    ServerStart,
}

impl DisplayServerBootError {
    pub const fn code(self) -> &'static str {
        match self {
            DisplayServerBootError::ServerStart => "DS-001",
        }
    }

    pub const fn serial_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart =>
                b"[DisplayServer][DS-001] Failed to start display server runtime\n\0",
        }
    }

    pub const fn vga_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart =>
                b"[DisplayServer][DS-001] Failed to start display server\n\0",
        }
    }
}

pub fn run(display: PlatformDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Starting Wayland compositor backend\n\0");

    let mut wayland = WaylandServer::new();
    if wayland.init().is_err() {
        serial_log(b"[DisplayServer] Wayland init failed\n\0");
        return Err(DisplayServerBootError::ServerStart);
    }
    serial_log(b"[DisplayServer] Wayland compositor ready at /tmp/wayland-0\n\0");

    // Move the display into the Wayland server's framebuffer for compositing
    wayland.set_framebuffer(FusionDisplayBackend::new(display));
    wayland.composite_frame();

    let mut frame_counter: u64 = 0;

    loop {
        // 1. Accept new clients
        if wayland.has_pending_connections() {
            if wayland.accept_client().is_ok() {
                serial_log(b"[DisplayServer] Client connected\n\0");
            }
        }

        // 2. Poll all clients for incoming messages and dispatch
        wayland.poll_clients();

        // 3. Process frame callbacks (sync done events)
        wayland.process_frame_callbacks();

        // 4. Composite every 10 iterations (~frame rate throttle)
        frame_counter += 1;
        if frame_counter % 10 == 0 {
            wayland.composite_frame();
        }

        // 5. Check for keyboard exit
        #[cfg(any(feature = "i686", feature = "x86_64"))]
        if ffi::keyboard_has_key() {
            if ffi::keyboard_read() == b'`' {
                serial_log(b"[DisplayServer] Exit\n\0");
                break;
            }
        }

        // 6. Yield CPU
        unsafe {
            core::arch::asm!("hlt");
        }
    }

    serial_log(b"[DisplayServer] Stopped\n\0");
    Ok(())
}
