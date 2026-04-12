use alloy_os_display::apps::desktop_shell::{
    DesktopShell, ShellAction, ShellApp, ShellInputOutcome, default_window_options_for_app,
};
use alloy_os_display::apps::window_manager::{InputOutcome, WindowId, WindowManager, WindowOptions, WindowState};
use alloy_os_display::protocol::{ClientId, DisplayEvent, DisplayRequest, SurfaceId};
use alloy_os_display::server::{DisplayBackend, DisplayServer};

use crate::ffi;
use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::terminal::TerminalSurface;
use crate::graphics::Display;
use crate::graphics::vesa::VesaDisplay;
use crate::terminal::Terminal;

const TERMINAL_CLIENT_ID: ClientId = ClientId::new(1);
const TERMINAL_WIDTH_CHARS: u32 = 80;
const TERMINAL_HEIGHT_CHARS: u32 = 25;
const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServerBootError {
    ServerStart,
    TerminalSurfaceInit,
    SurfaceUpload,
    FramePresent,
    WindowManager,
    Shell,
}

impl DisplayServerBootError {
    pub const fn code(self) -> &'static str {
        match self {
            DisplayServerBootError::ServerStart => "DS-001",
            DisplayServerBootError::TerminalSurfaceInit => "DS-002",
            DisplayServerBootError::SurfaceUpload => "DS-003",
            DisplayServerBootError::FramePresent => "DS-004",
            DisplayServerBootError::WindowManager => "DS-005",
            DisplayServerBootError::Shell => "DS-006",
        }
    }

    pub const fn serial_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart => {
                b"[DisplayServer][DS-001] Failed to start display server runtime\n\0"
            }
            DisplayServerBootError::TerminalSurfaceInit => {
                b"[DisplayServer][DS-002] Failed to initialize terminal surface\n\0"
            }
            DisplayServerBootError::SurfaceUpload => {
                b"[DisplayServer][DS-003] Failed to upload surface pixels\n\0"
            }
            DisplayServerBootError::FramePresent => {
                b"[DisplayServer][DS-004] Failed to present display frame\n\0"
            }
            DisplayServerBootError::WindowManager => {
                b"[DisplayServer][DS-005] Window manager operation failed\n\0"
            }
            DisplayServerBootError::Shell => {
                b"[DisplayServer][DS-006] Desktop shell operation failed\n\0"
            }
        }
    }

    pub const fn vga_message(self) -> &'static [u8] {
        match self {
            DisplayServerBootError::ServerStart => {
                b"[DisplayServer][DS-001] Failed to start display server\n\0"
            }
            DisplayServerBootError::TerminalSurfaceInit => {
                b"[DisplayServer][DS-002] Terminal surface initialization failed\n\0"
            }
            DisplayServerBootError::SurfaceUpload => {
                b"[DisplayServer][DS-003] Display upload failed\n\0"
            }
            DisplayServerBootError::FramePresent => {
                b"[DisplayServer][DS-004] Display frame presentation failed\n\0"
            }
            DisplayServerBootError::WindowManager => {
                b"[DisplayServer][DS-005] Window manager failure\n\0"
            }
            DisplayServerBootError::Shell => {
                b"[DisplayServer][DS-006] Desktop shell failure\n\0"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ManagedWindowBinding {
    app: ShellApp,
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
    app: ShellApp,
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
        app,
        client_id,
        window_id,
        surface_id,
    })
}

fn upload_terminal_surface<B: DisplayBackend>(
    server: &mut DisplayServer<B>,
    terminal_surface: &mut TerminalSurface,
    binding: ManagedWindowBinding,
    width: u32,
    height: u32,
) -> Result<(), DisplayServerBootError> {
    terminal_surface.mark_full_dirty();
    terminal_surface
        .render()
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    server
        .upload_surface_pixels(
            binding.client_id,
            binding.surface_id,
            width,
            height,
            terminal_surface.surface().get_buffer(),
            None,
        )
        .map_err(|_| DisplayServerBootError::SurfaceUpload)
}

fn spawn_terminal_window<B: DisplayBackend>(
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    terminal_surface: &mut TerminalSurface,
    workspace_width: u32,
    workspace_height: u32,
) -> Result<ManagedWindowBinding, DisplayServerBootError> {
    let (surface_width, surface_height) = terminal_surface.get_surface_dimensions();
    let options = default_window_options_for_app(
        ShellApp::Terminal,
        workspace_width,
        workspace_height,
        surface_width,
        surface_height,
    );
    let binding = create_window_binding(wm, server, ShellApp::Terminal, options)?;
    upload_terminal_surface(server, terminal_surface, binding, surface_width, surface_height)?;
    Ok(binding)
}

fn focus_or_restore_binding<B: DisplayBackend>(
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    binding: ManagedWindowBinding,
) -> Result<(), DisplayServerBootError> {
    match wm.window_state(binding.window_id) {
        Some(WindowState::Normal) => wm
            .focus_window(server, binding.window_id)
            .map_err(|_| DisplayServerBootError::WindowManager),
        Some(WindowState::Minimized) | Some(WindowState::Hidden) => wm
            .restore_window(server, binding.window_id)
            .map_err(|_| DisplayServerBootError::WindowManager),
        None => Err(DisplayServerBootError::WindowManager),
    }
}

fn ensure_terminal_binding<B: DisplayBackend>(
    shell: &mut DesktopShell,
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    terminal_surface: &mut TerminalSurface,
    terminal_binding: &mut Option<ManagedWindowBinding>,
    workspace_width: u32,
    workspace_height: u32,
) -> Result<ManagedWindowBinding, DisplayServerBootError> {
    if let Some(binding) = terminal_binding.as_ref().copied() {
        if wm.window_state(binding.window_id).is_some() {
            focus_or_restore_binding(wm, server, binding)?;
            shell.bind_window(ShellApp::Terminal, binding.window_id);
            return Ok(binding);
        }
    }

    *terminal_binding = None;
    shell.clear_binding(ShellApp::Terminal);

    let created = spawn_terminal_window(wm, server, terminal_surface, workspace_width, workspace_height)?;
    shell.bind_window(ShellApp::Terminal, created.window_id);
    *terminal_binding = Some(created);
    Ok(created)
}

fn activate_shell_app<B: DisplayBackend>(
    app: ShellApp,
    shell: &mut DesktopShell,
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    terminal_surface: &mut TerminalSurface,
    terminal_binding: &mut Option<ManagedWindowBinding>,
    workspace_width: u32,
    workspace_height: u32,
) -> Result<(), DisplayServerBootError> {
    match app {
        ShellApp::Terminal => {
            let _ = ensure_terminal_binding(
                shell,
                wm,
                server,
                terminal_surface,
                terminal_binding,
                workspace_width,
                workspace_height,
            )?;
            Ok(())
        }
        ShellApp::ComingSoon => {
            let binding = ensure_terminal_binding(
                shell,
                wm,
                server,
                terminal_surface,
                terminal_binding,
                workspace_width,
                workspace_height,
            )?;
            terminal_surface.append_system_message("coming soon");
            let (surface_width, surface_height) = terminal_surface.get_surface_dimensions();
            upload_terminal_surface(server, terminal_surface, binding, surface_width, surface_height)?;
            Ok(())
        }
    }
}

fn clear_dead_binding(
    wm: &WindowManager,
    shell: &mut DesktopShell,
    binding: &mut Option<ManagedWindowBinding>,
) {
    if let Some(current) = binding.as_ref().copied() {
        if wm.window_state(current.window_id).is_none() {
            shell.clear_binding(current.app);
            *binding = None;
        }
    }
}

pub fn run(display: VesaDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Bootstrapping desktop shell runtime\n\0");
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

    let mut shell =
        DesktopShell::bootstrap(&mut server, display_width, display_height).map_err(|_| DisplayServerBootError::Shell)?;

    let mut wm = WindowManager::new();
    let workspace_height = display_height.saturating_sub(shell.panel_height()).max(1);
    wm.set_workspace_bounds(display_width, workspace_height)
        .map_err(|_| DisplayServerBootError::WindowManager)?;

    let (surface_width, surface_height) = terminal_surface.get_surface_dimensions();
    let mut terminal_binding: Option<ManagedWindowBinding> = None;
    shell.sync_from_window_manager(&wm);
    shell.render(&mut server).map_err(|_| DisplayServerBootError::Shell)?;

    let boot_uptime = unsafe { ffi::timer_get_uptime_ms_ffi() };
    server
        .update_frame(boot_uptime)
        .map_err(|_| DisplayServerBootError::FramePresent)?;
    serial_log(
        b"[DisplayServer] Desktop shell ready - launcher starts open, Arrow/Tab selects tile, Enter/Space activates tile, ` toggles control mode, 1 focuses terminal, 2 prints coming soon\n\0",
    );

    loop {
        if ffi::keyboard_has_key() {
            let key = ffi::keyboard_read();
            if key != 0 {
                let mut consumed_by_shell = false;
                if wm.is_control_mode() || shell.launcher_visible() {
                    match shell.handle_control_key(key) {
                        ShellInputOutcome::Consumed => {
                            consumed_by_shell = true;
                        }
                        ShellInputOutcome::Action(action) => {
                            match action {
                                ShellAction::ActivateApp(app) => {
                                    activate_shell_app(
                                        app,
                                        &mut shell,
                                        &mut wm,
                                        &mut server,
                                        &mut terminal_surface,
                                        &mut terminal_binding,
                                        display_width,
                                        display_height,
                                    )?;
                                }
                            }
                            consumed_by_shell = true;
                        }
                        ShellInputOutcome::Ignored => {}
                    }
                }

                if !consumed_by_shell {
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
        }

        clear_dead_binding(&wm, &mut shell, &mut terminal_binding);
        shell.set_control_mode(wm.is_control_mode());
        shell.sync_from_window_manager(&wm);
        shell.render(&mut server).map_err(|_| DisplayServerBootError::Shell)?;

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
                        shell.clear_binding(ShellApp::Terminal);
                        serial_log(b"[DisplayServer] Terminal surface destroyed\n\0");
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
