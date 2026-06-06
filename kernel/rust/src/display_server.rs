use crate::ffi;
use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::shell::LxqtShell;
use crate::graphics::vesa::VesaDisplay;
use crate::fusion::WaylandServer;
use crate::utils::pointer;

const CURSOR_WIDTH: u32 = 12;
const CURSOR_HEIGHT: u32 = 18;
const CURSOR_Z_ORDER: u32 = 65535;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServerBootError {
    ServerStart,
    SurfaceUpload,
    FramePresent,
    LxqtShell,
}

impl DisplayServerBootError {
    pub const fn code(self) -> &'static str {
        match self {
            DisplayServerBootError::ServerStart => "DS-001",
            DisplayServerBootError::SurfaceUpload => "DS-003",
            DisplayServerBootError::FramePresent => "DS-004",
            DisplayServerBootError::LxqtShell => "DS-002",
        }
    }

    pub const fn serial_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart =>
                b"[DisplayServer][DS-001] Failed to start display server runtime\n\0",
            DisplayServerBootError::SurfaceUpload =>
                b"[DisplayServer][DS-003] Failed to upload surface pixels\n\0",
            DisplayServerBootError::FramePresent =>
                b"[DisplayServer][DS-004] Failed to present display frame\n\0",
            DisplayServerBootError::LxqtShell =>
                b"[DisplayServer][DS-002] LXQt shell initialization failed\n\0",
        }
    }

    pub const fn vga_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart =>
                b"[DisplayServer][DS-001] Failed to start display server\n\0",
            DisplayServerBootError::SurfaceUpload =>
                b"[DisplayServer][DS-003] Display upload failed\n\0",
            DisplayServerBootError::FramePresent =>
                b"[DisplayServer][DS-004] Display frame presentation failed\n\0",
            DisplayServerBootError::LxqtShell =>
                b"[DisplayServer][DS-002] LXQt shell failure\n\0",
        }
    }
}

fn serial_log(message: &'static [u8]) {
    unsafe {
        ffi::serial_print(message.as_ptr());
    }
}

struct PointerState {
    x: i32,
    y: i32,
    buttons: u8,
    dragging_window: Option<crate::fusion::shell::window_manager::WindowId>,
}

fn build_cursor_pixels() -> alloc::vec::Vec<u32> {
    let mut pixels = alloc::vec![0u32; (CURSOR_WIDTH * CURSOR_HEIGHT) as usize];
    for y in 0..CURSOR_HEIGHT {
        let fill_width = (y / 2 + 1).min(CURSOR_WIDTH.saturating_sub(1));
        for x in 0..=fill_width {
            let idx = (y * CURSOR_WIDTH + x) as usize;
            let border = x == 0 || y == 0 || x == fill_width || y == CURSOR_HEIGHT - 1;
            pixels[idx] = if border { 0xFF000000 } else { 0xFFF2F2F2 };
        }
    }
    pixels
}

fn create_cursor_surface(
    backend: &mut FusionDisplayBackend,
) -> Result<u32, DisplayServerBootError> {
    let surface_id = backend
        .create_surface(CURSOR_WIDTH, CURSOR_HEIGHT)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    backend
        .set_z_order(surface_id, CURSOR_Z_ORDER)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    backend
        .set_visibility(surface_id, true)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

    let cursor_pixels = build_cursor_pixels();
    backend
        .upload_pixels(surface_id, CURSOR_WIDTH, CURSOR_HEIGHT, &cursor_pixels)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

    Ok(surface_id)
}

fn set_cursor_position(
    backend: &mut FusionDisplayBackend,
    cursor_surface: u32,
    x: i32,
    y: i32,
) -> Result<(), DisplayServerBootError> {
    backend
        .set_position(cursor_surface, x, y)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)
}

fn set_cursor_visibility(
    backend: &mut FusionDisplayBackend,
    cursor_surface: u32,
    visible: bool,
) -> Result<(), DisplayServerBootError> {
    backend
        .set_visibility(cursor_surface, visible)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)
}

pub fn run(display: VesaDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Booting LXQt-compatible Fusion runtime\n\0");
    let display_width = display.framebuffer().width();
    let display_height = display.framebuffer().height();

    let mut backend = FusionDisplayBackend::new(display);

    let mut lxqt_shell = LxqtShell::new(&backend.display_mut());
    lxqt_shell.init_surfaces(&mut backend);

    let mut wayland = WaylandServer::new();
    let _ = wayland.init();

    let cursor_surface = create_cursor_surface(&mut backend)?;
    let mut pointer = PointerState {
        x: (display_width / 2) as i32,
        y: (display_height / 2) as i32,
        buttons: 0,
        dragging_window: None,
    };
    let mouse_ready = ffi::mouse_ready();
    if mouse_ready {
        set_cursor_position(&mut backend, cursor_surface, pointer.x, pointer.y)?;
    } else {
        set_cursor_visibility(&mut backend, cursor_surface, false)?;
        serial_log(b"[DisplayServer] Mouse unavailable\n\0");
    }
    let max_pointer_x = display_width.saturating_sub(1) as i32;
    let max_pointer_y = display_height.saturating_sub(1) as i32;

    backend.flush().map_err(|_| DisplayServerBootError::FramePresent)?;
    serial_log(b"[DisplayServer] First frame presented\n\0");

    loop {
        if ffi::keyboard_has_key() {
            let key = ffi::keyboard_read();
            if key == b'`' {
                serial_log(b"[DisplayServer] Exit key pressed\n\0");
                break;
            }
            if key != 0 {
                if let Some(focused) = lxqt_shell.window_manager.focused() {
                    let _ = lxqt_shell.window_manager.destroy_window(focused, &mut backend);
                }
            }
        }

        while ffi::mouse_has_event() {
            let Some(mouse_event) = ffi::mouse_read() else {
                break;
            };

            let mut delta_x = mouse_event.dx as i32;
            let mut delta_y = -(mouse_event.dy as i32);
            if (mouse_event.flags & ffi::MOUSE_EVENT_FLAG_X_OVERFLOW) != 0 {
                delta_x = 0;
            }
            if (mouse_event.flags & ffi::MOUSE_EVENT_FLAG_Y_OVERFLOW) != 0 {
                delta_y = 0;
            }

            if delta_x != 0 || delta_y != 0 {
                let movement = pointer::apply_relative_motion(
                    pointer.x,
                    pointer.y,
                    delta_x,
                    delta_y,
                    max_pointer_x,
                    max_pointer_y,
                );

                if movement.actual_dx != 0 || movement.actual_dy != 0 {
                    pointer.x = movement.next_x;
                    pointer.y = movement.next_y;
                    set_cursor_position(&mut backend, cursor_surface, pointer.x, pointer.y)?;

                    if let Some(drag_win) = pointer.dragging_window {
                        let _ = lxqt_shell.window_manager.move_window(
                            drag_win,
                            movement.actual_dx,
                            movement.actual_dy,
                            &mut backend,
                        );
                    }
                }
            }

            let left_pressed = (mouse_event.buttons & ffi::MOUSE_BUTTON_LEFT) != 0;
            let left_was = (pointer.buttons & ffi::MOUSE_BUTTON_LEFT) != 0;

            if left_pressed && !left_was {
                if let Some(window_id) = lxqt_shell.window_manager.window_at_point(pointer.x, pointer.y)
                {
                    lxqt_shell.window_manager.focus_window(window_id, &mut backend);
                    if lxqt_shell.window_manager.title_bar_at_point(pointer.x, pointer.y) == Some(window_id) {
                        pointer.dragging_window = Some(window_id);
                    }
                }
            } else if !left_pressed && left_was {
                pointer.dragging_window = None;
            }

            pointer.buttons = mouse_event.buttons;
        }

        let uptime_ms = unsafe { ffi::timer_get_uptime_ms_ffi() };

        lxqt_shell.desktop.dirty = true;
        lxqt_shell.panel.dirty = true;
        lxqt_shell.render(&mut backend, uptime_ms);

        backend.flush().map_err(|_| DisplayServerBootError::FramePresent)?;

        unsafe {
            core::arch::asm!("hlt");
        }
    }

    serial_log(b"[DisplayServer] Runtime stopped\n\0");
    Ok(())
}
