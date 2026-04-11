use alloc::vec::Vec;

use alloy_os_display::apps::window_manager::{InputOutcome, WindowId, WindowManager, WindowOptions};
use alloy_os_display::protocol::{ClientId, DisplayEvent, DisplayRequest, SurfaceId};
use alloy_os_display::server::{DisplayBackend, DisplayServer};

use crate::ffi;
use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::terminal::TerminalSurface;
use crate::graphics::Display;
use crate::graphics::vesa::VesaDisplay;
use crate::terminal::Terminal;

const TERMINAL_CLIENT_ID: ClientId = ClientId::new(1);
const INFO_CLIENT_ID: ClientId = ClientId::new(2);
const TERMINAL_WIDTH_CHARS: u32 = 80;
const TERMINAL_HEIGHT_CHARS: u32 = 25;
const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;
const TERMINAL_WINDOW_X: i32 = 24;
const TERMINAL_WINDOW_Y: i32 = 24;
const INFO_WINDOW_X: i32 = 520;
const INFO_WINDOW_Y: i32 = 48;
const INFO_WINDOW_WIDTH: u32 = 320;
const INFO_WINDOW_HEIGHT: u32 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServerBootError {
    ServerStart,
    TerminalSurfaceInit,
    SurfaceCreate,
    SurfaceConfigure,
    SurfaceUpload,
    FramePresent,
    WindowManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ManagedWindowBinding {
    client_id: ClientId,
    window_id: WindowId,
    surface_id: SurfaceId,
}

fn serial_log(message: &'static [u8]) {
    unsafe {
        ffi::serial_print(message.as_ptr());
    }
}

fn create_window_binding<B: DisplayBackend>(
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    options: WindowOptions,
) -> Result<ManagedWindowBinding, DisplayServerBootError> {
    let client_id = options.owner;
    let window_id = wm
        .create_window(server, options)
        .map_err(|_| DisplayServerBootError::WindowManager)?;
    let surface_id = wm
        .content_surface(window_id)
        .ok_or(DisplayServerBootError::WindowManager)?;
    Ok(ManagedWindowBinding {
        client_id,
        window_id,
        surface_id,
    })
}

fn fill_rect(
    pixels: &mut [u32],
    stride: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
) {
    if stride == 0 || width == 0 || height == 0 {
        return;
    }

    let max_height = (pixels.len() as u32) / stride;
    let end_x = x.saturating_add(width).min(stride);
    let end_y = y.saturating_add(height).min(max_height);

    for row in y..end_y {
        let row_offset = (row * stride) as usize;
        for col in x..end_x {
            pixels[row_offset + col as usize] = color;
        }
    }
}

fn build_info_surface_pixels(width: u32, height: u32) -> Result<Vec<u32>, DisplayServerBootError> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or(DisplayServerBootError::SurfaceCreate)? as usize;
    let mut pixels = alloc::vec![0xFF121212; pixel_count];

    for y in 0..height {
        let shade = 0x10 + ((y.saturating_mul(0x20) / height.max(1)) & 0x1F);
        let row_color = 0xFF000000 | (shade << 16) | (shade << 8) | shade;
        for x in 0..width {
            pixels[(y * width + x) as usize] = row_color;
        }
    }

    fill_rect(&mut pixels, width, 0, 0, width, 26, 0xFF1D2C42);
    fill_rect(
        &mut pixels,
        width,
        0,
        26,
        width,
        2,
        0xFF79B7FF,
    );
    fill_rect(&mut pixels, width, 16, 48, width.saturating_sub(32), 16, 0xFF27415F);
    fill_rect(&mut pixels, width, 16, 76, width.saturating_sub(32), 16, 0xFF27415F);
    fill_rect(&mut pixels, width, 16, 104, width.saturating_sub(32), 16, 0xFF27415F);
    fill_rect(&mut pixels, width, 16, 134, width.saturating_sub(32), 2, 0xFF79B7FF);
    fill_rect(&mut pixels, width, 16, 145, 44, 18, 0xFF406A9D);
    fill_rect(&mut pixels, width, 68, 145, 44, 18, 0xFF406A9D);
    fill_rect(&mut pixels, width, 120, 145, 44, 18, 0xFF406A9D);

    Ok(pixels)
}

pub fn run(display: VesaDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Bootstrapping display server runtime\n\0");
    let (display_width, display_height) = display.get_resolution();

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

    let mut wm = WindowManager::new();
    wm.set_workspace_bounds(display_width, display_height)
        .map_err(|_| DisplayServerBootError::WindowManager)?;
    let (surface_width, surface_height) = terminal_surface.get_surface_dimensions();
    let mut terminal_binding = Some(create_window_binding(
        &mut wm,
        &mut server,
        WindowOptions::new(TERMINAL_CLIENT_ID, surface_width, surface_height)
            .with_position(TERMINAL_WINDOW_X, TERMINAL_WINDOW_Y)
            .with_z_order(2)
            .with_focused(true)
            .with_resizable(false),
    )?);
    let mut info_binding = Some(create_window_binding(
        &mut wm,
        &mut server,
        WindowOptions::new(INFO_CLIENT_ID, INFO_WINDOW_WIDTH, INFO_WINDOW_HEIGHT)
            .with_position(INFO_WINDOW_X, INFO_WINDOW_Y)
            .with_z_order(1)
            .with_visibility(true)
            .with_focused(false)
            .with_resizable(true),
    )?);
    let info_pixels = build_info_surface_pixels(INFO_WINDOW_WIDTH, INFO_WINDOW_HEIGHT)?;
    let info_surface_id = info_binding
        .as_ref()
        .map(|binding| binding.surface_id)
        .ok_or(DisplayServerBootError::WindowManager)?;
    server
        .upload_surface_pixels(
            INFO_CLIENT_ID,
            info_surface_id,
            INFO_WINDOW_WIDTH,
            INFO_WINDOW_HEIGHT,
            &info_pixels,
            None,
        )
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

    terminal_surface.mark_full_dirty();
    terminal_surface
        .render()
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    let terminal_surface_id = terminal_binding
        .as_ref()
        .map(|binding| binding.surface_id)
        .ok_or(DisplayServerBootError::WindowManager)?;
    server
        .upload_surface_pixels(
            TERMINAL_CLIENT_ID,
            terminal_surface_id,
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
    serial_log(
        b"[DisplayServer] WM ready - ESC exits, ` toggles control mode, PgUp/PgDn cycles focus, M/H/C/R manage windows in control mode\n\0",
    );

    loop {
        if ffi::keyboard_has_key() {
            let key = ffi::keyboard_read();
            if key != 0 {
                match wm
                    .handle_key(&mut server, key)
                    .map_err(|_| DisplayServerBootError::WindowManager)?
                {
                    InputOutcome::ExitDisplay => break,
                    InputOutcome::Consumed => {}
                    InputOutcome::ForwardToWindow(window_id) => {
                        server
                            .route_key_input(key, true)
                            .map_err(|_| DisplayServerBootError::FramePresent)?;

                        if terminal_binding
                            .as_ref()
                            .map(|binding| binding.window_id)
                            == Some(window_id)
                        {
                            terminal_surface
                                .handle_input(key)
                                .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
                            terminal_surface
                                .render()
                                .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

                            if let Some(binding) = terminal_binding.as_ref().copied() {
                                server
                                    .upload_surface_pixels(
                                        binding.client_id,
                                        binding.surface_id,
                                        surface_width,
                                        surface_height,
                                        terminal_surface.surface().get_buffer(),
                                        None,
                                    )
                                    .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
                            }
                        }
                    }
                }
            }
        }

        let uptime_ms = unsafe { ffi::timer_get_uptime_ms_ffi() };
        server
            .update_frame(uptime_ms)
            .map_err(|_| DisplayServerBootError::FramePresent)?;

        while let Some(event) = server.poll_event() {
            match event {
                DisplayEvent::FocusChanged {
                    surface_id: Some(surface_id),
                } => {
                    if terminal_binding
                        .as_ref()
                        .map(|binding| binding.surface_id)
                        == Some(surface_id)
                    {
                        serial_log(b"[DisplayServer] Focus changed -> terminal\n\0");
                    } else if info_binding
                        .as_ref()
                        .map(|binding| binding.surface_id)
                        == Some(surface_id)
                    {
                        serial_log(b"[DisplayServer] Focus changed -> info\n\0");
                    } else {
                        serial_log(b"[DisplayServer] Focus changed -> unmanaged surface\n\0");
                    }
                }
                DisplayEvent::FocusChanged { surface_id: None } => {
                    serial_log(b"[DisplayServer] Focus cleared\n\0");
                }
                DisplayEvent::SurfaceDestroyed { surface_id } => {
                    if terminal_binding
                        .as_ref()
                        .map(|binding| binding.surface_id)
                        == Some(surface_id)
                    {
                        terminal_binding = None;
                        serial_log(b"[DisplayServer] Terminal surface destroyed\n\0");
                    } else if info_binding
                        .as_ref()
                        .map(|binding| binding.surface_id)
                        == Some(surface_id)
                    {
                        info_binding = None;
                        serial_log(b"[DisplayServer] Info surface destroyed\n\0");
                    }
                }
                _ => {}
            }
        }

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
