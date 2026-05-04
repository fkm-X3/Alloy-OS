use alloy_os_display::apps::desktop_shell::{
    DesktopShell, ShellAction, ShellApp, ShellInputOutcome, default_window_options_for_app,
};
use alloy_os_display::apps::toolbox_apps::{ToolboxAppState, render_toolbox_app};
use alloy_os_display::apps::window_manager::{InputOutcome, WindowId, WindowManager, WindowOptions, WindowState};
use alloy_os_display::protocol::{
    ClientId, DisplayEvent, DisplayRequest, DisplayResponse, MouseButton, PixelFormat, SurfaceId,
};
use alloy_os_display::server::{DisplayBackend, DisplayServer};

use crate::ffi;
use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::framebuffer::{Color, FramebufferRenderer};
use crate::fusion::terminal::TerminalSurface;
use crate::graphics::Display;
use crate::graphics::vesa::VesaDisplay;
use crate::terminal::Terminal;
use crate::utils::pointer;

const TERMINAL_CLIENT_ID: ClientId = ClientId::new(1);
const CURSOR_CLIENT_ID: ClientId = ClientId::new(2);
const TERMINAL_WIDTH_CHARS: u32 = 80;
const TERMINAL_HEIGHT_CHARS: u32 = 25;
const DEFAULT_FRAME_INTERVAL_MS: u32 = 16;
const CURSOR_WIDTH: u32 = 12;
const CURSOR_HEIGHT: u32 = 18;
const CURSOR_Z_ORDER: u32 = 65535;
const ICED_CLIENT_ID: ClientId = ClientId::new(32);
const ICED_WINDOW_Z_ORDER: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootUiMode {
    IcedPrimary,
    DesktopShell,
}

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

#[derive(Debug, Clone, Copy)]
struct AppRuntime {
    terminal_binding: Option<ManagedWindowBinding>,
    settings_binding: Option<ManagedWindowBinding>,
    file_explorer_binding: Option<ManagedWindowBinding>,
    text_editor_binding: Option<ManagedWindowBinding>,
    calculator_binding: Option<ManagedWindowBinding>,
    settings_state: ToolboxAppState,
    file_explorer_state: ToolboxAppState,
    text_editor_state: ToolboxAppState,
    calculator_state: ToolboxAppState,
}

#[derive(Debug, Clone, Copy)]
struct PointerState {
    x: i32,
    y: i32,
    buttons: u8,
    dragging_window: Option<WindowId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IcedWindowBinding {
    client_id: ClientId,
    window_id: WindowId,
    surface_id: SurfaceId,
}

#[derive(Debug, Clone, Copy)]
struct IcedUiState {
    active_tab: u8,
    show_palette: bool,
    accent_bright: bool,
}

#[derive(Debug, Clone, Copy)]
struct IcedUiRuntime {
    binding: IcedWindowBinding,
    state: IcedUiState,
    surface_width: u32,
    surface_height: u32,
    dirty: bool,
}

impl AppRuntime {
    fn new() -> Self {
        Self {
            terminal_binding: None,
            settings_binding: None,
            file_explorer_binding: None,
            text_editor_binding: None,
            calculator_binding: None,
            settings_state: ToolboxAppState::new(ShellApp::Settings),
            file_explorer_state: ToolboxAppState::new(ShellApp::FileExplorer),
            text_editor_state: ToolboxAppState::new(ShellApp::TextEditor),
            calculator_state: ToolboxAppState::new(ShellApp::Calculator),
        }
    }

    fn binding(&self, app: ShellApp) -> Option<ManagedWindowBinding> {
        match app {
            ShellApp::Terminal => self.terminal_binding,
            ShellApp::Settings => self.settings_binding,
            ShellApp::FileExplorer => self.file_explorer_binding,
            ShellApp::TextEditor => self.text_editor_binding,
            ShellApp::Calculator => self.calculator_binding,
        }
    }

    fn binding_mut(&mut self, app: ShellApp) -> &mut Option<ManagedWindowBinding> {
        match app {
            ShellApp::Terminal => &mut self.terminal_binding,
            ShellApp::Settings => &mut self.settings_binding,
            ShellApp::FileExplorer => &mut self.file_explorer_binding,
            ShellApp::TextEditor => &mut self.text_editor_binding,
            ShellApp::Calculator => &mut self.calculator_binding,
        }
    }

    fn toolbox_state_mut(&mut self, app: ShellApp) -> Option<&mut ToolboxAppState> {
        match app {
            ShellApp::Settings => Some(&mut self.settings_state),
            ShellApp::FileExplorer => Some(&mut self.file_explorer_state),
            ShellApp::TextEditor => Some(&mut self.text_editor_state),
            ShellApp::Calculator => Some(&mut self.calculator_state),
            ShellApp::Terminal => None,
        }
    }

    fn app_for_window(&self, window_id: WindowId) -> Option<ShellApp> {
        for app in ShellApp::ALL {
            if self.binding(app).map(|binding| binding.window_id) == Some(window_id) {
                return Some(app);
            }
        }
        None
    }

    fn app_for_surface(&self, surface_id: SurfaceId) -> Option<ShellApp> {
        for app in ShellApp::ALL {
            if self.binding(app).map(|binding| binding.surface_id) == Some(surface_id) {
                return Some(app);
            }
        }
        None
    }
}

impl IcedUiState {
    fn new() -> Self {
        Self {
            active_tab: 0,
            show_palette: false,
            accent_bright: true,
        }
    }

    fn handle_key(&mut self, key: u8) -> bool {
        match key {
            b'1' => {
                self.active_tab = 0;
                true
            }
            b'2' => {
                self.active_tab = 1;
                true
            }
            b'3' => {
                self.active_tab = 2;
                true
            }
            b'4' => {
                self.active_tab = 3;
                true
            }
            b'p' | b'P' => {
                self.show_palette = !self.show_palette;
                true
            }
            b't' | b'T' | b' ' => {
                self.accent_bright = !self.accent_bright;
                true
            }
            _ => false,
        }
    }
}

impl IcedUiRuntime {
    fn new(binding: IcedWindowBinding, surface_width: u32, surface_height: u32) -> Self {
        Self {
            binding,
            state: IcedUiState::new(),
            surface_width,
            surface_height,
            dirty: true,
        }
    }

    fn handle_key(&mut self, key: u8) -> bool {
        if self.state.handle_key(key) {
            self.dirty = true;
            return true;
        }
        false
    }

    fn sync_surface_dimensions<B: DisplayBackend>(
        &mut self,
        server: &DisplayServer<B>,
    ) -> Result<(), DisplayServerBootError> {
        let surface = server
            .surface(self.binding.surface_id)
            .ok_or(DisplayServerBootError::SurfaceUpload)?;
        if surface.width != self.surface_width || surface.height != self.surface_height {
            self.surface_width = surface.width;
            self.surface_height = surface.height;
            self.dirty = true;
        }
        Ok(())
    }

    fn render_if_dirty<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
    ) -> Result<(), DisplayServerBootError> {
        if !self.dirty {
            return Ok(());
        }

        let pixels =
            build_iced_primary_pixels(self.surface_width, self.surface_height, self.state)?;
        server
            .upload_surface_pixels(
                self.binding.client_id,
                self.binding.surface_id,
                self.surface_width,
                self.surface_height,
                &pixels,
                None,
            )
            .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
        self.dirty = false;
        Ok(())
    }
}

fn serial_log(message: &'static [u8]) {
    unsafe {
        ffi::serial_print(message.as_ptr());
    }
}

fn build_iced_primary_pixels(
    width: u32,
    height: u32,
    state: IcedUiState,
) -> Result<alloc::vec::Vec<u32>, DisplayServerBootError> {
    let mut renderer =
        FramebufferRenderer::new(width, height).map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    renderer.clear(Color::from_rgb(17, 20, 28));

    let header_h = (height / 10).clamp(34, 56);
    renderer.fill_rect(0, 0, width, header_h, Color::from_rgb(27, 39, 56));
    renderer.stroke_rect(0, 0, width, header_h, Color::from_rgb(99, 161, 255), 1);

    let tabs = 4u32;
    let tab_gap = 10u32;
    let available_tabs_width = width.saturating_sub(20).saturating_sub(tab_gap * (tabs - 1));
    let tab_width = (available_tabs_width / tabs.max(1)).max(32);
    for tab in 0..tabs {
        let tab_x = 10 + tab * (tab_width + tab_gap);
        let active = tab as u8 == state.active_tab;
        let tab_color = if active {
            if state.accent_bright {
                Color::from_rgb(79, 168, 255)
            } else {
                Color::from_rgb(62, 130, 201)
            }
        } else {
            Color::from_rgb(52, 60, 74)
        };
        renderer.fill_rect(tab_x, 8, tab_width, header_h.saturating_sub(16), tab_color);
        renderer.stroke_rect(
            tab_x,
            8,
            tab_width,
            header_h.saturating_sub(16),
            Color::from_rgb(170, 210, 255),
            1,
        );
    }

    let content_y = header_h.saturating_add(10);
    let content_h = height.saturating_sub(content_y.saturating_add(10));
    renderer.fill_rect(
        10,
        content_y,
        width.saturating_sub(20),
        content_h,
        Color::from_rgb(22, 26, 35),
    );
    renderer.stroke_rect(
        10,
        content_y,
        width.saturating_sub(20),
        content_h,
        Color::from_rgb(74, 88, 110),
        1,
    );

    let card_gap = 12u32;
    let card_w = ((width.saturating_sub(20)).saturating_sub(card_gap * 3) / 2).max(48);
    let card_h = (content_h.saturating_sub(card_gap * 3) / 2).max(48);
    for row in 0..2u32 {
        for col in 0..2u32 {
            let card_x = 10 + card_gap + col * (card_w + card_gap);
            let card_y = content_y + card_gap + row * (card_h + card_gap);
            let accent = if state.active_tab as u32 == row * 2 + col {
                if state.accent_bright {
                    Color::from_rgb(94, 184, 255)
                } else {
                    Color::from_rgb(67, 132, 191)
                }
            } else {
                Color::from_rgb(50, 60, 79)
            };
            renderer.fill_rect(card_x, card_y, card_w, card_h, Color::from_rgb(29, 35, 48));
            renderer.stroke_rect(card_x, card_y, card_w, card_h, accent, 2);
            renderer.fill_rect(card_x + 10, card_y + 10, 22, 22, accent);
            renderer.h_line(
                card_x + 40,
                card_x + card_w.saturating_sub(12),
                card_y + 16,
                Color::light_gray(),
                2,
            );
            renderer.h_line(
                card_x + 40,
                card_x + card_w.saturating_sub(24),
                card_y + 26,
                Color::dark_gray(),
                2,
            );
        }
    }

    if state.show_palette {
        let overlay_w = width.saturating_sub(120).max(80);
        let overlay_h = height.saturating_sub(140).max(80);
        let overlay_x = (width.saturating_sub(overlay_w)) / 2;
        let overlay_y = (height.saturating_sub(overlay_h)) / 2;
        renderer.fill_rect(
            overlay_x,
            overlay_y,
            overlay_w,
            overlay_h,
            Color::from_argb(220, 10, 14, 22),
        );
        renderer.stroke_rect(
            overlay_x,
            overlay_y,
            overlay_w,
            overlay_h,
            Color::from_rgb(119, 176, 255),
            2,
        );
        let swatch_gap = 12u32;
        let swatch_w = ((overlay_w.saturating_sub(30)).saturating_sub(swatch_gap * 2) / 3).max(14);
        let swatch_h = overlay_h.saturating_sub(44);
        let swatches = [
            Color::from_rgb(74, 138, 224),
            Color::from_rgb(84, 196, 128),
            Color::from_rgb(224, 164, 89),
        ];
        for (index, swatch) in swatches.iter().enumerate() {
            let x = overlay_x + 15 + (index as u32) * (swatch_w + swatch_gap);
            let y = overlay_y + 20;
            renderer.fill_rect(x, y, swatch_w, swatch_h, *swatch);
        }
    }

    Ok(renderer.pixels().to_vec())
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
    runtime: &mut AppRuntime,
    workspace_width: u32,
    workspace_height: u32,
) -> Result<ManagedWindowBinding, DisplayServerBootError> {
    if let Some(binding) = runtime.terminal_binding {
        if wm.window_state(binding.window_id).is_some() {
            focus_or_restore_binding(wm, server, binding)?;
            shell.bind_window(ShellApp::Terminal, binding.window_id);
            return Ok(binding);
        }
    }

    runtime.terminal_binding = None;
    shell.clear_binding(ShellApp::Terminal);

    let created = spawn_terminal_window(wm, server, terminal_surface, workspace_width, workspace_height)?;
    shell.bind_window(ShellApp::Terminal, created.window_id);
    runtime.terminal_binding = Some(created);
    Ok(created)
}

fn upload_toolbox_surface<B: DisplayBackend>(
    server: &mut DisplayServer<B>,
    binding: ManagedWindowBinding,
    state: &ToolboxAppState,
) -> Result<(), DisplayServerBootError> {
    let (surface_width, surface_height) = {
        let surface = server
            .surface(binding.surface_id)
            .ok_or(DisplayServerBootError::SurfaceUpload)?;
        (surface.width, surface.height)
    };
    let pixels = render_toolbox_app(binding.app, state, surface_width, surface_height)
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
    server
        .upload_surface_pixels(
            binding.client_id,
            binding.surface_id,
            surface_width,
            surface_height,
            &pixels,
            None,
        )
        .map_err(|_| DisplayServerBootError::SurfaceUpload)
}

fn spawn_toolbox_window<B: DisplayBackend>(
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    app: ShellApp,
    state: &ToolboxAppState,
    workspace_width: u32,
    workspace_height: u32,
    terminal_width: u32,
    terminal_height: u32,
) -> Result<ManagedWindowBinding, DisplayServerBootError> {
    let options = default_window_options_for_app(
        app,
        workspace_width,
        workspace_height,
        terminal_width,
        terminal_height,
    );
    let binding = create_window_binding(wm, server, app, options)?;
    upload_toolbox_surface(server, binding, state)?;
    Ok(binding)
}

fn activate_shell_app<B: DisplayBackend>(
    app: ShellApp,
    shell: &mut DesktopShell,
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    terminal_surface: &mut TerminalSurface,
    runtime: &mut AppRuntime,
    workspace_width: u32,
    workspace_height: u32,
    terminal_width: u32,
    terminal_height: u32,
) -> Result<(), DisplayServerBootError> {
    if let Some(binding) = runtime.binding(app) {
        if wm.window_state(binding.window_id).is_some() {
            focus_or_restore_binding(wm, server, binding)?;
            shell.bind_window(app, binding.window_id);
            return Ok(());
        }
    }

    *runtime.binding_mut(app) = None;
    shell.clear_binding(app);

    if app == ShellApp::Terminal {
        let _ = ensure_terminal_binding(
            shell,
            wm,
            server,
            terminal_surface,
            runtime,
            workspace_width,
            workspace_height,
        )?;
        return Ok(());
    }

    let state = *runtime
        .toolbox_state_mut(app)
        .ok_or(DisplayServerBootError::SurfaceUpload)?;
    let created = spawn_toolbox_window(
        wm,
        server,
        app,
        &state,
        workspace_width,
        workspace_height,
        terminal_width,
        terminal_height,
    )?;
    shell.bind_window(app, created.window_id);
    *runtime.binding_mut(app) = Some(created);
    Ok(())
}

fn clear_dead_bindings(
    wm: &WindowManager,
    shell: &mut DesktopShell,
    runtime: &mut AppRuntime,
) {
    for app in ShellApp::ALL {
        let binding = runtime.binding(app);
        if let Some(current) = binding {
            if wm.window_state(current.window_id).is_none() {
                shell.clear_binding(current.app);
                *runtime.binding_mut(app) = None;
            }
        }
    }
}

fn expect_ack(response: DisplayResponse) -> Result<(), DisplayServerBootError> {
    match response {
        DisplayResponse::Ack => Ok(()),
        _ => Err(DisplayServerBootError::SurfaceUpload),
    }
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

fn create_cursor_surface<B: DisplayBackend>(
    server: &mut DisplayServer<B>,
) -> Result<SurfaceId, DisplayServerBootError> {
    let response = server
        .handle_request(
            CURSOR_CLIENT_ID,
            DisplayRequest::CreateSurface {
                width: CURSOR_WIDTH,
                height: CURSOR_HEIGHT,
                format: PixelFormat::Argb8888,
            },
        )
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

    let surface_id = match response {
        DisplayResponse::SurfaceCreated { surface_id } => surface_id,
        _ => return Err(DisplayServerBootError::SurfaceUpload),
    };

    expect_ack(
        server
            .handle_request(
                CURSOR_CLIENT_ID,
                DisplayRequest::SetSurfaceZOrder {
                    surface_id,
                    z_order: CURSOR_Z_ORDER,
                },
            )
            .map_err(|_| DisplayServerBootError::SurfaceUpload)?,
    )?;

    expect_ack(
        server
            .handle_request(
                CURSOR_CLIENT_ID,
                DisplayRequest::SetSurfaceVisibility {
                    surface_id,
                    visible: true,
                },
            )
            .map_err(|_| DisplayServerBootError::SurfaceUpload)?,
    )?;

    let cursor_pixels = build_cursor_pixels();
    server
        .upload_surface_pixels(
            CURSOR_CLIENT_ID,
            surface_id,
            CURSOR_WIDTH,
            CURSOR_HEIGHT,
            &cursor_pixels,
            None,
        )
        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

    Ok(surface_id)
}

fn set_cursor_position<B: DisplayBackend>(
    server: &mut DisplayServer<B>,
    cursor_surface: SurfaceId,
    x: i32,
    y: i32,
) -> Result<(), DisplayServerBootError> {
    expect_ack(
        server
            .handle_request(
                CURSOR_CLIENT_ID,
                DisplayRequest::SetSurfacePosition {
                    surface_id: cursor_surface,
                    x,
                    y,
                },
            )
            .map_err(|_| DisplayServerBootError::SurfaceUpload)?,
    )
}

fn set_cursor_visibility<B: DisplayBackend>(
    server: &mut DisplayServer<B>,
    cursor_surface: SurfaceId,
    visible: bool,
) -> Result<(), DisplayServerBootError> {
    expect_ack(
        server
            .handle_request(
                CURSOR_CLIENT_ID,
                DisplayRequest::SetSurfaceVisibility {
                    surface_id: cursor_surface,
                    visible,
                },
            )
            .map_err(|_| DisplayServerBootError::SurfaceUpload)?,
    )
}

fn spawn_iced_runtime<B: DisplayBackend>(
    wm: &mut WindowManager,
    server: &mut DisplayServer<B>,
    display_width: u32,
    display_height: u32,
) -> Result<IcedUiRuntime, DisplayServerBootError> {
    let content_width = display_width.saturating_sub(200).max(260);
    let content_height = display_height.saturating_sub(180).max(180);
    let frame_width = content_width.saturating_add(4);
    let frame_height = content_height.saturating_add(22);
    let pos_x = (display_width.saturating_sub(frame_width) / 2) as i32;
    let pos_y = (display_height.saturating_sub(frame_height) / 2) as i32;

    let options = WindowOptions::new(ICED_CLIENT_ID, content_width, content_height)
        .with_position(pos_x, pos_y)
        .with_z_order(ICED_WINDOW_Z_ORDER)
        .with_focused(true);
    let window_id = wm
        .create_window(server, options)
        .map_err(|_| DisplayServerBootError::WindowManager)?;
    let surface_id = wm
        .content_surface(window_id)
        .ok_or(DisplayServerBootError::WindowManager)?;
    let (surface_width, surface_height) = {
        let surface = server
            .surface(surface_id)
            .ok_or(DisplayServerBootError::SurfaceUpload)?;
        (surface.width, surface.height)
    };
    let binding = IcedWindowBinding {
        client_id: ICED_CLIENT_ID,
        window_id,
        surface_id,
    };
    let mut runtime = IcedUiRuntime::new(binding, surface_width, surface_height);
    runtime.render_if_dirty(server)?;
    Ok(runtime)
}

pub fn run(display: VesaDisplay, mode: BootUiMode) -> Result<(), DisplayServerBootError> {
    match mode {
        BootUiMode::IcedPrimary => run_iced_primary(display),
        BootUiMode::DesktopShell => run_desktop_shell(display),
    }
}

fn run_iced_primary(display: VesaDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Bootstrapping Iced-primary runtime\n\0");
    let (display_width, display_height) = display.get_resolution();

    let backend = FusionDisplayBackend::new(display);
    let mut server = DisplayServer::new(backend);
    server
        .start()
        .map_err(|_| DisplayServerBootError::ServerStart)?;
    server
        .handle_request(
            ICED_CLIENT_ID,
            DisplayRequest::SetFrameIntervalMs {
                interval_ms: DEFAULT_FRAME_INTERVAL_MS,
            },
        )
        .map_err(|_| DisplayServerBootError::ServerStart)?;

    let mut wm = WindowManager::new();
    wm.set_workspace_bounds(display_width, display_height)
        .map_err(|_| DisplayServerBootError::WindowManager)?;
    let mut runtime = spawn_iced_runtime(&mut wm, &mut server, display_width, display_height)?;

    let cursor_surface = create_cursor_surface(&mut server)?;
    let mut pointer = PointerState {
        x: (display_width / 2) as i32,
        y: (display_height / 2) as i32,
        buttons: 0,
        dragging_window: None,
    };
    let mouse_ready = ffi::mouse_ready();
    if mouse_ready {
        set_cursor_position(&mut server, cursor_surface, pointer.x, pointer.y)?;
    } else {
        set_cursor_visibility(&mut server, cursor_surface, false)?;
        serial_log(b"[DisplayServer] Mouse unavailable; cursor hidden until mouse input is ready\n\0");
    }
    let max_pointer_x = display_width.saturating_sub(1) as i32;
    let max_pointer_y = display_height.saturating_sub(1) as i32;

    let boot_uptime = unsafe { ffi::timer_get_uptime_ms_ffi() };
    let first_present_time = boot_uptime.saturating_add(server.frame_interval_ms() as u64);
    let first_presented = server
        .update_frame(first_present_time)
        .map_err(|_| DisplayServerBootError::FramePresent)?;
    if !first_presented {
        return Err(DisplayServerBootError::FramePresent);
    }

    serial_log(b"[DisplayServer] First frame presented\n\0");
    serial_log(
        b"[DisplayServer] Iced-primary window ready - use ` for control mode, PgUp/PgDn for focus cycle, ESC exits\n\0",
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
                        if window_id == runtime.binding.window_id {
                            runtime.handle_key(key);
                        }
                    }
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
                    set_cursor_position(&mut server, cursor_surface, pointer.x, pointer.y)?;
                    server
                        .route_pointer_motion(pointer.x, pointer.y, movement.actual_dx, movement.actual_dy)
                        .map_err(|_| DisplayServerBootError::FramePresent)?;

                    if pointer.dragging_window.is_some()
                        && (mouse_event.buttons & ffi::MOUSE_BUTTON_LEFT) != 0
                    {
                        wm.move_focused_by(&mut server, movement.actual_dx, movement.actual_dy)
                            .map_err(|_| DisplayServerBootError::WindowManager)?;
                    }
                }
            }

            for (mask, button) in [
                (ffi::MOUSE_BUTTON_LEFT, MouseButton::Left),
                (ffi::MOUSE_BUTTON_RIGHT, MouseButton::Right),
                (ffi::MOUSE_BUTTON_MIDDLE, MouseButton::Middle),
            ] {
                let Some(is_pressed) =
                    pointer::button_state_changed(pointer.buttons, mouse_event.buttons, mask)
                else {
                    continue;
                };

                server
                    .route_mouse_button(button, is_pressed, pointer.x, pointer.y)
                    .map_err(|_| DisplayServerBootError::FramePresent)?;

                if mask != ffi::MOUSE_BUTTON_LEFT {
                    continue;
                }

                if is_pressed {
                    if let Some(window_id) = wm.window_at_point(pointer.x, pointer.y) {
                        wm.focus_window(&mut server, window_id)
                            .map_err(|_| DisplayServerBootError::WindowManager)?;
                        if wm.title_bar_window_at_point(pointer.x, pointer.y) == Some(window_id) {
                            pointer.dragging_window = Some(window_id);
                        } else {
                            pointer.dragging_window = None;
                        }
                    } else {
                        pointer.dragging_window = None;
                    }
                } else {
                    pointer.dragging_window = None;
                }
            }

            if mouse_event.wheel != 0 {
                server
                    .route_mouse_wheel(mouse_event.wheel as i32, pointer.x, pointer.y)
                    .map_err(|_| DisplayServerBootError::FramePresent)?;
            }

            pointer.buttons = mouse_event.buttons;
        }

        if wm.window_state(runtime.binding.window_id).is_none() {
            runtime = spawn_iced_runtime(&mut wm, &mut server, display_width, display_height)?;
        }
        if let Some(window_id) = pointer.dragging_window {
            if wm.window_state(window_id).is_none() {
                pointer.dragging_window = None;
            }
        }

        runtime.sync_surface_dimensions(&server)?;
        runtime.render_if_dirty(&mut server)?;

        let uptime_ms = unsafe { ffi::timer_get_uptime_ms_ffi() };
        server
            .update_frame(uptime_ms)
            .map_err(|_| DisplayServerBootError::FramePresent)?;

        while let Some(event) = server.poll_event() {
            match event {
                DisplayEvent::FocusChanged {
                    surface_id: Some(surface_id),
                } => {
                    if surface_id == runtime.binding.surface_id {
                        serial_log(b"[DisplayServer] Focus changed -> iced primary window\n\0");
                    } else {
                        serial_log(b"[DisplayServer] Focus changed -> unmanaged surface\n\0");
                    }
                }
                DisplayEvent::FocusChanged { surface_id: None } => {
                    serial_log(b"[DisplayServer] Focus cleared\n\0");
                }
                DisplayEvent::SurfaceDestroyed { surface_id } => {
                    if surface_id == runtime.binding.surface_id {
                        serial_log(b"[DisplayServer] Iced primary surface destroyed\n\0");
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

fn run_desktop_shell(display: VesaDisplay) -> Result<(), DisplayServerBootError> {
    serial_log(b"[DisplayServer] Bootstrapping desktop shell fallback runtime\n\0");
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

    let (terminal_surface_width, terminal_surface_height) = terminal_surface.get_surface_dimensions();
    let mut runtime = AppRuntime::new();
    shell.sync_from_window_manager(&wm);
    shell.render(&mut server).map_err(|_| DisplayServerBootError::Shell)?;

    let cursor_surface = create_cursor_surface(&mut server)?;
    let mut pointer = PointerState {
        x: (display_width / 2) as i32,
        y: (workspace_height / 2) as i32,
        buttons: 0,
        dragging_window: None,
    };
    let mouse_ready = ffi::mouse_ready();
    if mouse_ready {
        set_cursor_position(&mut server, cursor_surface, pointer.x, pointer.y)?;
    } else {
        set_cursor_visibility(&mut server, cursor_surface, false)?;
        serial_log(b"[DisplayServer] Mouse unavailable; cursor hidden until mouse input is ready\n\0");
    }
    let max_pointer_x = display_width.saturating_sub(1) as i32;
    let max_pointer_y = display_height.saturating_sub(1) as i32;

    let boot_uptime = unsafe { ffi::timer_get_uptime_ms_ffi() };
    let first_present_time = boot_uptime.saturating_add(server.frame_interval_ms() as u64);
    let first_presented = server
        .update_frame(first_present_time)
        .map_err(|_| DisplayServerBootError::FramePresent)?;
    if !first_presented {
        return Err(DisplayServerBootError::FramePresent);
    }
    serial_log(b"[DisplayServer] First frame presented\n\0");
    serial_log(
        b"[DisplayServer] Desktop shell ready - launcher starts open, Arrow/Tab selects tile, Enter/Space activates tile, ` toggles control mode, 1-5 quick-launch toolbox apps\n\0",
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
                                        &mut runtime,
                                        display_width,
                                        display_height,
                                        terminal_surface_width,
                                        terminal_surface_height,
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

                            if runtime
                                .terminal_binding
                                .map(|binding| binding.window_id)
                                == Some(window_id)
                            {
                                terminal_surface
                                    .handle_input(key)
                                    .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
                                terminal_surface
                                    .render()
                                    .map_err(|_| DisplayServerBootError::SurfaceUpload)?;

                                if let Some(binding) = runtime.terminal_binding {
                                    server
                                        .upload_surface_pixels(
                                            binding.client_id,
                                            binding.surface_id,
                                            terminal_surface_width,
                                            terminal_surface_height,
                                            terminal_surface.surface().get_buffer(),
                                            None,
                                        )
                                        .map_err(|_| DisplayServerBootError::SurfaceUpload)?;
                                }
                            } else if let Some(app) = runtime.app_for_window(window_id) {
                                if let Some(state) = runtime.toolbox_state_mut(app) {
                                    let _ = state.handle_input(key);
                                }
                                if let Some(binding) = runtime.binding(app) {
                                    if let Some(state) = runtime.toolbox_state_mut(app) {
                                        upload_toolbox_surface(&mut server, binding, state)?;
                                    }
                                }
                            }
                        }
                    }
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
                    set_cursor_position(&mut server, cursor_surface, pointer.x, pointer.y)?;
                    server
                        .route_pointer_motion(pointer.x, pointer.y, movement.actual_dx, movement.actual_dy)
                        .map_err(|_| DisplayServerBootError::FramePresent)?;

                    if pointer.dragging_window.is_some()
                        && (mouse_event.buttons & ffi::MOUSE_BUTTON_LEFT) != 0
                    {
                        wm.move_focused_by(&mut server, movement.actual_dx, movement.actual_dy)
                            .map_err(|_| DisplayServerBootError::WindowManager)?;
                    }
                }
            }

            for (mask, button) in [
                (ffi::MOUSE_BUTTON_LEFT, MouseButton::Left),
                (ffi::MOUSE_BUTTON_RIGHT, MouseButton::Right),
                (ffi::MOUSE_BUTTON_MIDDLE, MouseButton::Middle),
            ] {
                let Some(is_pressed) = pointer::button_state_changed(pointer.buttons, mouse_event.buttons, mask) else {
                    continue;
                };

                server
                    .route_mouse_button(button, is_pressed, pointer.x, pointer.y)
                    .map_err(|_| DisplayServerBootError::FramePresent)?;

                if mask != ffi::MOUSE_BUTTON_LEFT {
                    continue;
                }

                if is_pressed {
                    if let Some(app) = shell.launcher_app_at_point(pointer.x, pointer.y) {
                        activate_shell_app(
                            app,
                            &mut shell,
                            &mut wm,
                            &mut server,
                            &mut terminal_surface,
                            &mut runtime,
                            display_width,
                            display_height,
                            terminal_surface_width,
                            terminal_surface_height,
                        )?;
                        pointer.dragging_window = None;
                    } else if let Some(window_id) = wm.window_at_point(pointer.x, pointer.y) {
                        wm.focus_window(&mut server, window_id)
                            .map_err(|_| DisplayServerBootError::WindowManager)?;
                        if wm.title_bar_window_at_point(pointer.x, pointer.y) == Some(window_id) {
                            pointer.dragging_window = Some(window_id);
                        } else {
                            pointer.dragging_window = None;
                        }
                    } else {
                        pointer.dragging_window = None;
                    }
                } else {
                    pointer.dragging_window = None;
                }
            }

            if mouse_event.wheel != 0 {
                server
                    .route_mouse_wheel(mouse_event.wheel as i32, pointer.x, pointer.y)
                    .map_err(|_| DisplayServerBootError::FramePresent)?;
            }

            pointer.buttons = mouse_event.buttons;
        }

        clear_dead_bindings(&wm, &mut shell, &mut runtime);
        if let Some(window_id) = pointer.dragging_window {
            if wm.window_state(window_id).is_none() {
                pointer.dragging_window = None;
            }
        }
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
                    match runtime.app_for_surface(surface_id) {
                        Some(ShellApp::Terminal) => serial_log(b"[DisplayServer] Focus changed -> terminal\n\0"),
                        Some(ShellApp::Settings) => serial_log(b"[DisplayServer] Focus changed -> settings\n\0"),
                        Some(ShellApp::FileExplorer) => serial_log(b"[DisplayServer] Focus changed -> file explorer\n\0"),
                        Some(ShellApp::TextEditor) => serial_log(b"[DisplayServer] Focus changed -> text editor\n\0"),
                        Some(ShellApp::Calculator) => serial_log(b"[DisplayServer] Focus changed -> calculator\n\0"),
                        None => serial_log(b"[DisplayServer] Focus changed -> unmanaged surface\n\0"),
                    }
                }
                DisplayEvent::FocusChanged { surface_id: None } => {
                    serial_log(b"[DisplayServer] Focus cleared\n\0");
                }
                DisplayEvent::SurfaceDestroyed { surface_id } => {
                    if let Some(app) = runtime.app_for_surface(surface_id) {
                        *runtime.binding_mut(app) = None;
                        shell.clear_binding(app);
                        match app {
                            ShellApp::Terminal => serial_log(b"[DisplayServer] Terminal surface destroyed\n\0"),
                            ShellApp::Settings => serial_log(b"[DisplayServer] Settings surface destroyed\n\0"),
                            ShellApp::FileExplorer => {
                                serial_log(b"[DisplayServer] File explorer surface destroyed\n\0")
                            }
                            ShellApp::TextEditor => serial_log(b"[DisplayServer] Text editor surface destroyed\n\0"),
                            ShellApp::Calculator => serial_log(b"[DisplayServer] Calculator surface destroyed\n\0"),
                        }
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
