use crate::fusion::FusionDisplayBackend;
use crate::fusion::WaylandServer;
use crate::graphics::{Display, PlatformDisplay};

fn serial_log(message: &'static [u8]) {
    let len = message
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(message.len());
    crate::Serial::write_bytes(&message[..len]);
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
            DisplayServerBootError::ServerStart => {
                b"[DisplayServer][DS-001] Failed to start display server runtime\n\0"
            }
        }
    }

    pub const fn vga_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart => {
                b"[DisplayServer][DS-001] Failed to start display server\n\0"
            }
        }
    }
}

pub fn run(display: PlatformDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Starting Wayland compositor backend\n\0");

    #[cfg(feature = "x86_64")]
    let (screen_w, screen_h) = display.get_resolution();

    let mut wayland = WaylandServer::new();
    if wayland.init().is_err() {
        serial_log(b"[DisplayServer] Wayland init failed\n\0");
        return Err(DisplayServerBootError::ServerStart);
    }
    serial_log(b"[DisplayServer] Wayland compositor ready at /tmp/wayland-0\n\0");

    // Move the display into the Wayland server's framebuffer for compositing
    wayland.set_framebuffer(FusionDisplayBackend::new(display));
    wayland.composite_frame();
    crate::render_trace!(
        "[T7] First frame presented (uptime {}ms)",
        crate::SystemTimer::uptime_ms()
    );
    serial_log(b"[DisplayServer] First frame presented\n\0");

    let mut frame_counter: u64 = 0;
    static LOOP_TICKS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

    // Cursor position starts at screen center; mouse deltas accumulate here.
    #[cfg(feature = "x86_64")]
    let mut pointer_x: i32 = (screen_w / 2) as i32;
    #[cfg(feature = "x86_64")]
    let mut pointer_y: i32 = (screen_h / 2) as i32;
    #[cfg(feature = "x86_64")]
    let mut prev_buttons: u8 = 0;

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

        // 4. Route hardware input through the Wayland input router (x86_64)
        #[cfg(feature = "x86_64")]
        {
            use crate::ffi::{MOUSE_EVENT_FLAG_X_OVERFLOW, MOUSE_EVENT_FLAG_Y_OVERFLOW};
            use crate::fusion::wayland::seat::button_codes;
            use alloy_kernel_hal::{Keyboard, Mouse};

            let surfaces = wayland.surface_geometries();

            while let Some(event) = Mouse::read() {
                if event.flags & (MOUSE_EVENT_FLAG_X_OVERFLOW | MOUSE_EVENT_FLAG_Y_OVERFLOW) == 0 {
                    pointer_x = pointer_x.saturating_add(event.dx as i32);
                    pointer_y = pointer_y.saturating_add(event.dy as i32);
                }
                pointer_x = pointer_x.clamp(0, screen_w as i32 - 1);
                pointer_y = pointer_y.clamp(0, screen_h as i32 - 1);

                let _ = wayland
                    .input_router_mut()
                    .handle_pointer_motion(&surfaces, pointer_x, pointer_y);

                let changed = prev_buttons ^ event.buttons;
                for (bit, code) in [
                    (0x01, button_codes::LEFT),
                    (0x02, button_codes::RIGHT),
                    (0x04, button_codes::MIDDLE),
                ] {
                    if changed & bit != 0 {
                        let pressed = event.buttons & bit != 0;
                        let _ = wayland
                            .input_router_mut()
                            .handle_pointer_button(&surfaces, code, pressed, pointer_x, pointer_y);
                        if pressed {
                            let focus = wayland.input_router().pointer_focus();
                            wayland.input_router_mut().set_keyboard_focus(focus);
                        }
                    }
                }
                prev_buttons = event.buttons;

                if event.wheel != 0 {
                    let _ = wayland.input_router_mut().handle_pointer_axis(
                        &surfaces,
                        true,
                        event.wheel as i32,
                    );
                }
            }

            while let Some(key_event) = Keyboard::read_scancode() {
                let _ = wayland
                    .input_router_mut()
                    .handle_key(key_event.code as u32, key_event.pressed);
            }
        }

        // 5. Flush pending input events to clients
        wayland.flush_input_events();

        // 6. Composite every 10 iterations (~frame rate throttle)
        frame_counter += 1;
        if frame_counter % 10 == 0 {
            wayland.composite_frame();
        }

        // 7. Check for keyboard exit
        #[cfg(feature = "x86_64")]
        if crate::ffi::keyboard_has_key() {
            if crate::ffi::keyboard_read() == b'`' {
                serial_log(b"[DisplayServer] Exit\n\0");
                break;
            }
        }

        // 8. Liveness heartbeat — long uptime gaps between consecutive ticks
        //    are compositor-starvation evidence (Session 0.3 fix target).
        let ticks = LOOP_TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if ticks % 2000 == 0 {
            crate::render_trace!(
                "[T8] loop tick {} (uptime {}ms, frame_counter={})",
                ticks,
                crate::SystemTimer::uptime_ms(),
                frame_counter
            );
        }

        // 9. Yield CPU
        #[cfg(feature = "x86_64")]
        alloy_kernel_hal::cpu_halt();
        #[cfg(feature = "aarch64")]
        alloy_kernel_hal::cpu_halt();
    }

    serial_log(b"[DisplayServer] Stopped\n\0");
    Ok(())
}
