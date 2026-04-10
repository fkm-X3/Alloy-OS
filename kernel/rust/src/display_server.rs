use alloy_os_display::protocol::{ClientId, DisplayRequest, DisplayResponse, PixelFormat};
use alloy_os_display::server::DisplayServer;

use crate::ffi;
use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::terminal::TerminalSurface;
use crate::graphics::vesa::VesaDisplay;
use crate::terminal::Terminal;

const TERMINAL_CLIENT_ID: ClientId = ClientId::new(1);
const TERMINAL_WIDTH_CHARS: u32 = 80;
const TERMINAL_HEIGHT_CHARS: u32 = 25;
const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;
const ESC_KEY: u8 = 27;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServerBootError {
    ServerStart,
    TerminalSurfaceInit,
    SurfaceCreate,
    SurfaceConfigure,
    SurfaceUpload,
    FramePresent,
}

fn serial_log(message: &'static [u8]) {
    unsafe {
        ffi::serial_print(message.as_ptr());
    }
}

pub fn run(display: VesaDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Bootstrapping display server runtime\n\0");

    let backend = FusionDisplayBackend::new(display);
    let mut server = DisplayServer::new(backend);
    server
        .start()
        .map_err(|_| DisplayServerBootError::ServerStart)?;

    server
        .handle_request(
            TERMINAL_CLIENT_ID,
            DisplayRequest::SetFrameIntervalMs {
                interval_ms: DEFAULT_FRAME_INTERVAL_MS,
            },
        )
        .map_err(|_| DisplayServerBootError::ServerStart)?;

    let mut terminal = Terminal::new();
    let mut terminal_surface =
        TerminalSurface::new(&mut terminal, TERMINAL_WIDTH_CHARS, TERMINAL_HEIGHT_CHARS)
            .map_err(|_| DisplayServerBootError::TerminalSurfaceInit)?;
    terminal_surface.set_position(16, 16);
    terminal_surface.set_visible(true);
    terminal_surface.set_z_order(1);

    let (surface_width, surface_height) = terminal_surface.get_surface_dimensions();
    let surface_id = match server
        .handle_request(
            TERMINAL_CLIENT_ID,
            DisplayRequest::CreateSurface {
                width: surface_width,
                height: surface_height,
                format: PixelFormat::Argb8888,
            },
        )
        .map_err(|_| DisplayServerBootError::SurfaceCreate)?
    {
        DisplayResponse::SurfaceCreated { surface_id } => surface_id,
        _ => return Err(DisplayServerBootError::SurfaceCreate),
    };

    server
        .handle_request(
            TERMINAL_CLIENT_ID,
            DisplayRequest::SetSurfacePosition {
                surface_id,
                x: 16,
                y: 16,
            },
        )
        .map_err(|_| DisplayServerBootError::SurfaceConfigure)?;
    server
        .handle_request(
            TERMINAL_CLIENT_ID,
            DisplayRequest::SetSurfaceVisibility {
                surface_id,
                visible: true,
            },
        )
        .map_err(|_| DisplayServerBootError::SurfaceConfigure)?;
    server
        .handle_request(
            TERMINAL_CLIENT_ID,
            DisplayRequest::SetSurfaceZOrder {
                surface_id,
                z_order: 1,
            },
        )
        .map_err(|_| DisplayServerBootError::SurfaceConfigure)?;
    server
        .handle_request(
            TERMINAL_CLIENT_ID,
            DisplayRequest::RequestFocus {
                surface_id: Some(surface_id),
            },
        )
        .map_err(|_| DisplayServerBootError::SurfaceConfigure)?;

    terminal_surface.mark_full_dirty();
    terminal_surface
        .render()
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    server
        .upload_surface_pixels(
            TERMINAL_CLIENT_ID,
            surface_id,
            surface_width,
            surface_height,
            terminal_surface.surface().get_buffer(),
            None,
        )
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

    let boot_uptime = unsafe { ffi::timer_get_uptime_ms_ffi() };
    server
        .update_frame(boot_uptime)
        .map_err(|_| DisplayServerBootError::FramePresent)?;
    serial_log(b"[DisplayServer] Ready - press ESC to leave display mode\n\0");

    loop {
        if ffi::keyboard_has_key() {
            let key = ffi::keyboard_read();
            if key == ESC_KEY {
                break;
            }

            server
                .route_key_input(key, true)
                .map_err(|_| DisplayServerBootError::FramePresent)?;

            terminal_surface
                .handle_input(key)
                .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
            terminal_surface
                .render()
                .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

            server
                .upload_surface_pixels(
                    TERMINAL_CLIENT_ID,
                    surface_id,
                    surface_width,
                    surface_height,
                    terminal_surface.surface().get_buffer(),
                    None,
                )
                .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
        }

        let uptime_ms = unsafe { ffi::timer_get_uptime_ms_ffi() };
        server
            .update_frame(uptime_ms)
            .map_err(|_| DisplayServerBootError::FramePresent)?;

        while server.poll_event().is_some() {}

        unsafe {
            core::arch::asm!("hlt");
        }
    }

    let diagnostics = server.diagnostics();
    if diagnostics.dropped_events > 0 {
        serial_log(b"[DisplayServer] Warning: event queue overflow detected\n\0");
    }
    if diagnostics.backend_errors > 0 {
        serial_log(b"[DisplayServer] Warning: backend errors detected during runtime\n\0");
    }

    let _ = server.stop();
    serial_log(b"[DisplayServer] Runtime stopped\n\0");
    Ok(())
}
