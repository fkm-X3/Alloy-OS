use alloc::vec::Vec;
use core::fmt;

use crate::apps::window_manager::{WindowId, WindowManager, WindowOptions, WindowState};
use crate::protocol::{ClientId, DisplayRequest, DisplayResponse, PixelFormat, SurfaceId};
use crate::server::{DisplayBackend, DisplayServer, ServerError};

pub const SHELL_CLIENT_ID: ClientId = ClientId::new(4096);
pub const PANEL_HEIGHT: u32 = 30;

const BACKGROUND_Z_ORDER: u32 = 0;
const PANEL_Z_ORDER: u32 = 4096;
const LAUNCHER_Z_ORDER: u32 = 4097;

const DEFAULT_LAUNCHER_WIDTH: u32 = 300;
const DEFAULT_LAUNCHER_HEIGHT: u32 = 170;
const PANEL_PADDING: u32 = 8;
const PANEL_SLOT_GAP: u32 = 8;

const KEY_ESCAPE: u8 = 27;
const KEY_TAB: u8 = b'\t';
const KEY_ENTER: u8 = b'\n';
const KEY_SPACE: u8 = b' ';
const KEY_SPECIAL_UP: u8 = 128;
const KEY_SPECIAL_DOWN: u8 = 129;
const KEY_SPECIAL_LEFT: u8 = 130;
const KEY_SPECIAL_RIGHT: u8 = 131;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellApp {
    Terminal,
    InfoPanel,
}

impl ShellApp {
    pub const ALL: [ShellApp; 2] = [ShellApp::Terminal, ShellApp::InfoPanel];

    pub const fn index(self) -> usize {
        match self {
            ShellApp::Terminal => 0,
            ShellApp::InfoPanel => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellWindowStatus {
    Closed,
    Normal,
    Minimized,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellWindowEntry {
    pub app: ShellApp,
    pub window_id: Option<WindowId>,
    pub status: ShellWindowStatus,
    pub focused: bool,
}

impl ShellWindowEntry {
    const fn new(app: ShellApp) -> Self {
        Self {
            app,
            window_id: None,
            status: ShellWindowStatus::Closed,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAction {
    ActivateApp(ShellApp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellInputOutcome {
    Consumed,
    Ignored,
    Action(ShellAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopShellError {
    Server(ServerError),
    UnexpectedResponse,
    InvalidWorkspace,
    InvalidPixelBuffer,
}

impl fmt::Display for DesktopShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesktopShellError::Server(err) => write!(f, "server error: {}", err),
            DesktopShellError::UnexpectedResponse => write!(f, "unexpected display response"),
            DesktopShellError::InvalidWorkspace => write!(f, "invalid workspace dimensions"),
            DesktopShellError::InvalidPixelBuffer => write!(f, "invalid pixel buffer dimensions"),
        }
    }
}

impl From<ServerError> for DesktopShellError {
    fn from(value: ServerError) -> Self {
        DesktopShellError::Server(value)
    }
}

pub struct DesktopShell {
    workspace_width: u32,
    workspace_height: u32,
    background_surface: SurfaceId,
    panel_surface: SurfaceId,
    launcher_surface: SurfaceId,
    launcher_width: u32,
    launcher_height: u32,
    launcher_visible: bool,
    launcher_selection: usize,
    control_mode: bool,
    dirty: bool,
    entries: [ShellWindowEntry; 2],
}

impl DesktopShell {
    pub fn bootstrap<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        workspace_width: u32,
        workspace_height: u32,
    ) -> Result<Self, DesktopShellError> {
        if workspace_width == 0 || workspace_height == 0 {
            return Err(DesktopShellError::InvalidWorkspace);
        }

        let launcher_width = DEFAULT_LAUNCHER_WIDTH.min(workspace_width.max(1));
        let launcher_height = DEFAULT_LAUNCHER_HEIGHT.min(workspace_height.max(1));

        let background_surface = Self::create_surface(server, workspace_width, workspace_height)?;
        let panel_surface = Self::create_surface(server, workspace_width, PANEL_HEIGHT)?;
        let launcher_surface = Self::create_surface(server, launcher_width, launcher_height)?;

        let panel_y = workspace_height.saturating_sub(PANEL_HEIGHT) as i32;
        let (launcher_x, launcher_y) = Self::launcher_position(
            workspace_width,
            workspace_height,
            launcher_width,
            launcher_height,
        );

        Self::configure_surface(
            server,
            background_surface,
            0,
            0,
            BACKGROUND_Z_ORDER,
            true,
        )?;
        Self::configure_surface(
            server,
            panel_surface,
            0,
            panel_y,
            PANEL_Z_ORDER,
            true,
        )?;
        Self::configure_surface(
            server,
            launcher_surface,
            launcher_x,
            launcher_y,
            LAUNCHER_Z_ORDER,
            false,
        )?;

        let mut shell = Self {
            workspace_width,
            workspace_height,
            background_surface,
            panel_surface,
            launcher_surface,
            launcher_width,
            launcher_height,
            launcher_visible: false,
            launcher_selection: 0,
            control_mode: false,
            dirty: true,
            entries: [
                ShellWindowEntry::new(ShellApp::Terminal),
                ShellWindowEntry::new(ShellApp::InfoPanel),
            ],
        };

        shell.upload_background(server)?;
        shell.render(server)?;
        Ok(shell)
    }

    pub fn panel_height(&self) -> u32 {
        PANEL_HEIGHT
    }

    pub fn launcher_visible(&self) -> bool {
        self.launcher_visible
    }

    pub fn set_launcher_visible(&mut self, visible: bool) {
        if self.launcher_visible != visible {
            self.launcher_visible = visible;
            self.dirty = true;
        }
    }

    pub fn set_control_mode(&mut self, active: bool) {
        if self.control_mode != active {
            self.control_mode = active;
            self.dirty = true;
        }
    }

    pub fn bind_window(&mut self, app: ShellApp, window_id: WindowId) {
        let index = app.index();
        let entry = &mut self.entries[index];
        entry.window_id = Some(window_id);
        entry.status = ShellWindowStatus::Normal;
        self.dirty = true;
    }

    pub fn clear_binding(&mut self, app: ShellApp) {
        let index = app.index();
        let entry = &mut self.entries[index];
        entry.window_id = None;
        entry.status = ShellWindowStatus::Closed;
        entry.focused = false;
        self.dirty = true;
    }

    pub fn app_for_window(&self, window_id: WindowId) -> Option<ShellApp> {
        self.entries
            .iter()
            .find(|entry| entry.window_id == Some(window_id))
            .map(|entry| entry.app)
    }

    pub fn window_id_for_app(&self, app: ShellApp) -> Option<WindowId> {
        self.entries[app.index()].window_id
    }

    pub fn entries(&self) -> &[ShellWindowEntry] {
        &self.entries
    }

    pub fn sync_from_window_manager(&mut self, wm: &WindowManager) {
        let mut changed = false;
        let focused_window = wm.focused_window();

        for entry in self.entries.iter_mut() {
            let prev = *entry;
            if let Some(window_id) = entry.window_id {
                match wm.window_state(window_id) {
                    Some(state) => {
                        entry.status = Self::status_from_window_state(state);
                    }
                    None => {
                        entry.window_id = None;
                        entry.status = ShellWindowStatus::Closed;
                    }
                }
            } else {
                entry.status = ShellWindowStatus::Closed;
            }

            entry.focused = entry.window_id == focused_window && entry.status == ShellWindowStatus::Normal;
            if *entry != prev {
                changed = true;
            }
        }

        if changed {
            self.dirty = true;
        }
    }

    pub fn handle_control_key(&mut self, key: u8) -> ShellInputOutcome {
        match key {
            b'l' | b'L' => {
                self.launcher_visible = !self.launcher_visible;
                self.dirty = true;
                return ShellInputOutcome::Consumed;
            }
            b'1' => {
                self.launcher_selection = 0;
                self.launcher_visible = false;
                self.dirty = true;
                return ShellInputOutcome::Action(ShellAction::ActivateApp(ShellApp::Terminal));
            }
            b'2' => {
                self.launcher_selection = 1;
                self.launcher_visible = false;
                self.dirty = true;
                return ShellInputOutcome::Action(ShellAction::ActivateApp(ShellApp::InfoPanel));
            }
            _ => {}
        }

        if !self.launcher_visible {
            return ShellInputOutcome::Ignored;
        }

        match key {
            KEY_SPECIAL_LEFT | KEY_SPECIAL_UP => {
                self.select_prev_launcher_item();
                self.dirty = true;
                ShellInputOutcome::Consumed
            }
            KEY_SPECIAL_RIGHT | KEY_SPECIAL_DOWN | KEY_TAB => {
                self.select_next_launcher_item();
                self.dirty = true;
                ShellInputOutcome::Consumed
            }
            KEY_ENTER | KEY_SPACE => {
                let app = ShellApp::ALL[self.launcher_selection];
                self.launcher_visible = false;
                self.dirty = true;
                ShellInputOutcome::Action(ShellAction::ActivateApp(app))
            }
            KEY_ESCAPE => {
                self.launcher_visible = false;
                self.dirty = true;
                ShellInputOutcome::Consumed
            }
            _ => ShellInputOutcome::Ignored,
        }
    }

    pub fn render<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
    ) -> Result<(), DesktopShellError> {
        if !self.dirty {
            return Ok(());
        }

        let panel_pixels = self.build_panel_pixels()?;
        server
            .upload_surface_pixels(
                SHELL_CLIENT_ID,
                self.panel_surface,
                self.workspace_width,
                PANEL_HEIGHT,
                &panel_pixels,
                None,
            )
            .map_err(DesktopShellError::Server)?;

        let launcher_pixels = self.build_launcher_pixels()?;
        server
            .upload_surface_pixels(
                SHELL_CLIENT_ID,
                self.launcher_surface,
                self.launcher_width,
                self.launcher_height,
                &launcher_pixels,
                None,
            )
            .map_err(DesktopShellError::Server)?;

        let launcher_visibility = server
            .handle_request(
                SHELL_CLIENT_ID,
                DisplayRequest::SetSurfaceVisibility {
                    surface_id: self.launcher_surface,
                    visible: self.launcher_visible,
                },
            )
            .map_err(DesktopShellError::Server)?;
        Self::expect_ack(launcher_visibility)?;

        self.dirty = false;
        Ok(())
    }

    fn upload_background<B: DisplayBackend>(
        &self,
        server: &mut DisplayServer<B>,
    ) -> Result<(), DesktopShellError> {
        let background = Self::build_background_pixels(self.workspace_width, self.workspace_height)?;
        server
            .upload_surface_pixels(
                SHELL_CLIENT_ID,
                self.background_surface,
                self.workspace_width,
                self.workspace_height,
                &background,
                None,
            )
            .map_err(DesktopShellError::Server)
    }

    fn select_next_launcher_item(&mut self) {
        self.launcher_selection = (self.launcher_selection + 1) % ShellApp::ALL.len();
    }

    fn select_prev_launcher_item(&mut self) {
        self.launcher_selection = if self.launcher_selection == 0 {
            ShellApp::ALL.len() - 1
        } else {
            self.launcher_selection - 1
        };
    }

    fn status_from_window_state(state: WindowState) -> ShellWindowStatus {
        match state {
            WindowState::Normal => ShellWindowStatus::Normal,
            WindowState::Minimized => ShellWindowStatus::Minimized,
            WindowState::Hidden => ShellWindowStatus::Hidden,
        }
    }

    fn launcher_position(
        workspace_width: u32,
        workspace_height: u32,
        launcher_width: u32,
        launcher_height: u32,
    ) -> (i32, i32) {
        let usable_height = workspace_height.saturating_sub(PANEL_HEIGHT);
        let x = workspace_width.saturating_sub(launcher_width) / 2;
        let y = usable_height.saturating_sub(launcher_height) / 2;
        (x as i32, y as i32)
    }

    fn create_surface<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        width: u32,
        height: u32,
    ) -> Result<SurfaceId, DesktopShellError> {
        let response = server
            .handle_request(
                SHELL_CLIENT_ID,
                DisplayRequest::CreateSurface {
                    width,
                    height,
                    format: PixelFormat::Argb8888,
                },
            )
            .map_err(DesktopShellError::Server)?;

        match response {
            DisplayResponse::SurfaceCreated { surface_id } => Ok(surface_id),
            DisplayResponse::Error(_) => Err(DesktopShellError::UnexpectedResponse),
            _ => Err(DesktopShellError::UnexpectedResponse),
        }
    }

    fn configure_surface<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        surface_id: SurfaceId,
        x: i32,
        y: i32,
        z_order: u32,
        visible: bool,
    ) -> Result<(), DesktopShellError> {
        let position_response = server
            .handle_request(
                SHELL_CLIENT_ID,
                DisplayRequest::SetSurfacePosition { surface_id, x, y },
            )
            .map_err(DesktopShellError::Server)?;
        Self::expect_ack(position_response)?;

        let z_response = server
            .handle_request(
                SHELL_CLIENT_ID,
                DisplayRequest::SetSurfaceZOrder {
                    surface_id,
                    z_order,
                },
            )
            .map_err(DesktopShellError::Server)?;
        Self::expect_ack(z_response)?;

        let vis_response = server
            .handle_request(
                SHELL_CLIENT_ID,
                DisplayRequest::SetSurfaceVisibility { surface_id, visible },
            )
            .map_err(DesktopShellError::Server)?;
        Self::expect_ack(vis_response)
    }

    fn expect_ack(response: DisplayResponse) -> Result<(), DesktopShellError> {
        match response {
            DisplayResponse::Ack => Ok(()),
            DisplayResponse::Error(_) => Err(DesktopShellError::UnexpectedResponse),
            _ => Err(DesktopShellError::UnexpectedResponse),
        }
    }

    fn build_background_pixels(width: u32, height: u32) -> Result<Vec<u32>, DesktopShellError> {
        let mut pixels = Self::allocate_pixels(width, height, 0xFF0B0D12)?;
        for y in 0..height {
            let shade = 0x12 + ((y.saturating_mul(0x26) / height.max(1)) & 0x3F);
            let row_color = 0xFF000000 | (shade << 16) | (shade << 8) | (shade + 0x10).min(0xFF);
            let row_start = (y * width) as usize;
            let row_end = row_start.saturating_add(width as usize).min(pixels.len());
            for pixel in pixels[row_start..row_end].iter_mut() {
                *pixel = row_color;
            }
        }
        Ok(pixels)
    }

    fn build_panel_pixels(&self) -> Result<Vec<u32>, DesktopShellError> {
        let mut pixels = Self::allocate_pixels(self.workspace_width, PANEL_HEIGHT, 0xFF1B1E25)?;

        let top_line = if self.control_mode {
            0xFF4FA8FF
        } else {
            0xFF2E3644
        };
        Self::fill_rect(&mut pixels, self.workspace_width, 0, 0, self.workspace_width, 1, top_line);

        let slot_count = self.entries.len() as u32;
        let total_gap = PANEL_SLOT_GAP.saturating_mul(slot_count.saturating_sub(1));
        let available_width = self
            .workspace_width
            .saturating_sub(PANEL_PADDING.saturating_mul(2))
            .saturating_sub(total_gap);
        let slot_width = (available_width / slot_count.max(1)).clamp(30, 180);
        let slot_height = PANEL_HEIGHT.saturating_sub(10).max(8);
        let total_width = slot_width
            .saturating_mul(slot_count)
            .saturating_add(total_gap);
        let start_x = self.workspace_width.saturating_sub(total_width) / 2;
        let slot_y = 5;

        for (index, entry) in self.entries.iter().enumerate() {
            let x = start_x + (index as u32).saturating_mul(slot_width + PANEL_SLOT_GAP);
            let base = Self::slot_color(*entry);
            Self::fill_rect(&mut pixels, self.workspace_width, x, slot_y, slot_width, slot_height, base);

            let accent = match entry.app {
                ShellApp::Terminal => 0xFF66D9EF,
                ShellApp::InfoPanel => 0xFFF5B971,
            };
            Self::fill_rect(
                &mut pixels,
                self.workspace_width,
                x.saturating_add(4),
                slot_y.saturating_add(4),
                8,
                slot_height.saturating_sub(8).max(2),
                accent,
            );

            if entry.focused {
                Self::fill_rect(
                    &mut pixels,
                    self.workspace_width,
                    x.saturating_add(2),
                    slot_y.saturating_add(slot_height.saturating_sub(3)),
                    slot_width.saturating_sub(4),
                    2,
                    0xFFD4EAFF,
                );
            }

            if self.launcher_visible && self.launcher_selection == index {
                Self::stroke_rect(
                    &mut pixels,
                    self.workspace_width,
                    x,
                    slot_y,
                    slot_width,
                    slot_height,
                    0xFF8ED1FF,
                );
            }
        }

        let mode_color = if self.control_mode {
            0xFF58D273
        } else {
            0xFF6B6F78
        };
        let mode_size = PANEL_HEIGHT.saturating_sub(12).max(6);
        Self::fill_rect(
            &mut pixels,
            self.workspace_width,
            self.workspace_width.saturating_sub(mode_size + 8),
            6,
            mode_size,
            mode_size,
            mode_color,
        );

        Ok(pixels)
    }

    fn build_launcher_pixels(&self) -> Result<Vec<u32>, DesktopShellError> {
        let mut pixels = Self::allocate_pixels(self.launcher_width, self.launcher_height, 0xFF151922)?;
        Self::fill_rect(
            &mut pixels,
            self.launcher_width,
            0,
            0,
            self.launcher_width,
            22,
            0xFF1F2F46,
        );
        Self::fill_rect(
            &mut pixels,
            self.launcher_width,
            0,
            22,
            self.launcher_width,
            1,
            0xFF84C0FF,
        );

        let item_height = 44;
        let item_gap = 10;
        let start_y = 34;
        for (index, entry) in self.entries.iter().enumerate() {
            let y = start_y + (index as u32).saturating_mul(item_height + item_gap);
            let mut color = match entry.status {
                ShellWindowStatus::Closed => 0xFF342427,
                ShellWindowStatus::Normal => 0xFF2C3B53,
                ShellWindowStatus::Minimized => 0xFF2E3239,
                ShellWindowStatus::Hidden => 0xFF2A2F35,
            };

            if self.launcher_selection == index {
                color = 0xFF3A5B86;
            }
            if entry.focused {
                color = 0xFF486EA3;
            }

            Self::fill_rect(
                &mut pixels,
                self.launcher_width,
                14,
                y,
                self.launcher_width.saturating_sub(28),
                item_height,
                color,
            );

            let accent = match entry.app {
                ShellApp::Terminal => 0xFF66D9EF,
                ShellApp::InfoPanel => 0xFFF5B971,
            };
            Self::fill_rect(&mut pixels, self.launcher_width, 22, y + 9, 18, 26, accent);

            let status_dot = match entry.status {
                ShellWindowStatus::Closed => 0xFFB26A6A,
                ShellWindowStatus::Normal => 0xFF58D273,
                ShellWindowStatus::Minimized => 0xFFE6C15A,
                ShellWindowStatus::Hidden => 0xFF8A8F99,
            };
            Self::fill_rect(
                &mut pixels,
                self.launcher_width,
                self.launcher_width.saturating_sub(30),
                y + 15,
                10,
                10,
                status_dot,
            );
        }

        Ok(pixels)
    }

    fn slot_color(entry: ShellWindowEntry) -> u32 {
        match entry.status {
            ShellWindowStatus::Closed => 0xFF342426,
            ShellWindowStatus::Normal if entry.focused => 0xFF2F7FDB,
            ShellWindowStatus::Normal => 0xFF454D5D,
            ShellWindowStatus::Minimized => 0xFF3B3F46,
            ShellWindowStatus::Hidden => 0xFF333842,
        }
    }

    fn allocate_pixels(width: u32, height: u32, fill: u32) -> Result<Vec<u32>, DesktopShellError> {
        let pixel_count = width
            .checked_mul(height)
            .ok_or(DesktopShellError::InvalidPixelBuffer)? as usize;
        Ok(alloc::vec![fill; pixel_count])
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

    fn stroke_rect(
        pixels: &mut [u32],
        stride: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: u32,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        Self::fill_rect(pixels, stride, x, y, width, 1, color);
        Self::fill_rect(
            pixels,
            stride,
            x,
            y.saturating_add(height.saturating_sub(1)),
            width,
            1,
            color,
        );
        Self::fill_rect(pixels, stride, x, y, 1, height, color);
        Self::fill_rect(
            pixels,
            stride,
            x.saturating_add(width.saturating_sub(1)),
            y,
            1,
            height,
            color,
        );
    }
}

pub fn default_window_options_for_app(
    app: ShellApp,
    workspace_width: u32,
    workspace_height: u32,
    terminal_width: u32,
    terminal_height: u32,
) -> WindowOptions {
    match app {
        ShellApp::Terminal => {
            let x = 24i32;
            let y = 24i32;
            WindowOptions::new(ClientId::new(1), terminal_width, terminal_height)
                .with_position(x, y)
                .with_z_order(2)
                .with_focused(true)
                .with_resizable(false)
        }
        ShellApp::InfoPanel => {
            let width = 320u32.min(workspace_width.saturating_sub(32).max(120));
            let max_height = workspace_height.saturating_sub(PANEL_HEIGHT + 32).max(96);
            let height = 180u32.min(max_height);
            let x = workspace_width.saturating_sub(width + 28) as i32;
            let max_y = workspace_height
                .saturating_sub(PANEL_HEIGHT.saturating_add(height).saturating_add(8))
                as i32;
            let y = 48i32.min(max_y.max(12));
            WindowOptions::new(ClientId::new(2), width, height)
                .with_position(x.max(12), y.max(12))
                .with_z_order(1)
                .with_visibility(true)
                .with_focused(false)
                .with_resizable(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use crate::apps::window_manager::WindowManager;
    use crate::protocol::{Rect, SurfaceId};

    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct MockSurface {
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        visible: bool,
        z_order: u32,
    }

    #[derive(Debug, Default)]
    struct MockBackend {
        surfaces: BTreeMap<SurfaceId, MockSurface>,
    }

    impl DisplayBackend for MockBackend {
        type Error = ();

        fn create_surface(
            &mut self,
            surface_id: SurfaceId,
            width: u32,
            height: u32,
            _format: PixelFormat,
        ) -> Result<(), Self::Error> {
            self.surfaces.insert(
                surface_id,
                MockSurface {
                    width,
                    height,
                    x: 0,
                    y: 0,
                    visible: true,
                    z_order: 0,
                },
            );
            Ok(())
        }

        fn destroy_surface(&mut self, surface_id: SurfaceId) -> Result<(), Self::Error> {
            self.surfaces.remove(&surface_id).map(|_| ()).ok_or(())
        }

        fn set_surface_position(
            &mut self,
            surface_id: SurfaceId,
            x: i32,
            y: i32,
        ) -> Result<(), Self::Error> {
            let surface = self.surfaces.get_mut(&surface_id).ok_or(())?;
            surface.x = x;
            surface.y = y;
            Ok(())
        }

        fn resize_surface(
            &mut self,
            surface_id: SurfaceId,
            width: u32,
            height: u32,
        ) -> Result<(), Self::Error> {
            let surface = self.surfaces.get_mut(&surface_id).ok_or(())?;
            surface.width = width;
            surface.height = height;
            Ok(())
        }

        fn set_surface_visibility(
            &mut self,
            surface_id: SurfaceId,
            visible: bool,
        ) -> Result<(), Self::Error> {
            let surface = self.surfaces.get_mut(&surface_id).ok_or(())?;
            surface.visible = visible;
            Ok(())
        }

        fn set_surface_z_order(
            &mut self,
            surface_id: SurfaceId,
            z_order: u32,
        ) -> Result<(), Self::Error> {
            let surface = self.surfaces.get_mut(&surface_id).ok_or(())?;
            surface.z_order = z_order;
            Ok(())
        }

        fn commit_surface(
            &mut self,
            surface_id: SurfaceId,
            _damage: Option<Rect>,
        ) -> Result<(), Self::Error> {
            self.surfaces.get(&surface_id).map(|_| ()).ok_or(())
        }

        fn upload_surface_pixels(
            &mut self,
            surface_id: SurfaceId,
            width: u32,
            height: u32,
            pixels: &[u32],
            _damage: Option<Rect>,
        ) -> Result<(), Self::Error> {
            let surface = self.surfaces.get(&surface_id).ok_or(())?;
            if surface.width != width || surface.height != height {
                return Err(());
            }

            let required = (width as usize).saturating_mul(height as usize);
            if pixels.len() < required {
                return Err(());
            }
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn launcher_toggle_and_navigation_work_in_control_mode() {
        let mut server = DisplayServer::new(MockBackend::default());
        server.start().expect("server should start");

        let mut shell = DesktopShell::bootstrap(&mut server, 640, 480)
            .expect("shell bootstrap should succeed");
        assert!(!shell.launcher_visible());

        assert_eq!(
            shell.handle_control_key(b'l'),
            ShellInputOutcome::Consumed,
            "toggle should be consumed"
        );
        assert!(shell.launcher_visible(), "launcher should be visible");

        assert_eq!(
            shell.handle_control_key(KEY_SPECIAL_RIGHT),
            ShellInputOutcome::Consumed,
            "navigation should be consumed"
        );
        assert_eq!(
            shell.handle_control_key(KEY_ENTER),
            ShellInputOutcome::Action(ShellAction::ActivateApp(ShellApp::InfoPanel)),
            "selected app should activate"
        );
        assert!(!shell.launcher_visible(), "launcher should close after activation");
    }

    #[test]
    fn sync_tracks_window_state_changes() {
        let mut server = DisplayServer::new(MockBackend::default());
        server.start().expect("server should start");

        let mut shell = DesktopShell::bootstrap(&mut server, 800, 600)
            .expect("shell bootstrap should succeed");

        let mut wm = WindowManager::new();
        wm.set_workspace_bounds(800, 600).expect("workspace should set");

        let terminal_id = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(1), 320, 200)
                    .with_position(16, 16)
                    .with_focused(true),
            )
            .expect("terminal window should create");

        let info_id = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(2), 240, 180)
                    .with_position(90, 64),
            )
            .expect("info window should create");

        shell.bind_window(ShellApp::Terminal, terminal_id);
        shell.bind_window(ShellApp::InfoPanel, info_id);
        shell.sync_from_window_manager(&wm);
        let entries = shell.entries();
        assert_eq!(entries[0].status, ShellWindowStatus::Normal);
        assert!(entries[0].focused, "terminal should start focused");

        wm.focus_window(&mut server, info_id)
            .expect("info window should focus");
        wm.minimize_focused(&mut server)
            .expect("focused window should minimize");
        shell.sync_from_window_manager(&wm);
        let entries = shell.entries();
        assert_eq!(entries[1].status, ShellWindowStatus::Minimized);
        assert!(
            !entries[1].focused,
            "minimized window should not remain focused"
        );
    }
}
