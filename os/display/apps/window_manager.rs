use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt;

use crate::protocol::{ClientId, DisplayRequest, DisplayResponse, PixelFormat, SurfaceId};
use crate::server::{DisplayBackend, DisplayServer, ServerError};

pub type WindowId = u32;

const BORDER_THICKNESS: u32 = 2;
const TITLE_BAR_HEIGHT: u32 = 18;
const MIN_WINDOW_WIDTH: u32 = 120;
const MIN_WINDOW_HEIGHT: u32 = 64;
const MAX_WINDOW_WIDTH: u32 = 1600;
const MAX_WINDOW_HEIGHT: u32 = 1200;
const MOVE_STEP: i32 = 12;
const RESIZE_STEP: i32 = 24;

const KEY_ESCAPE: u8 = 27;
const KEY_TAB: u8 = b'\t';
const KEY_TOGGLE_CONTROL: u8 = b'`';
const KEY_SPECIAL_UP: u8 = 128;
const KEY_SPECIAL_DOWN: u8 = 129;
const KEY_SPECIAL_LEFT: u8 = 130;
const KEY_SPECIAL_RIGHT: u8 = 131;
const KEY_SPECIAL_PGUP: u8 = 135;
const KEY_SPECIAL_PGDN: u8 = 136;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowManagerError {
    Server(ServerError),
    InvalidDimensions,
    WindowNotFound,
    UnexpectedResponse,
    ServerRejected,
    WindowIdExhausted,
}

impl fmt::Display for WindowManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowManagerError::Server(err) => write!(f, "server error: {}", err),
            WindowManagerError::InvalidDimensions => write!(f, "invalid window dimensions"),
            WindowManagerError::WindowNotFound => write!(f, "window not found"),
            WindowManagerError::UnexpectedResponse => write!(f, "unexpected server response"),
            WindowManagerError::ServerRejected => write!(f, "server rejected request"),
            WindowManagerError::WindowIdExhausted => write!(f, "window id space exhausted"),
        }
    }
}

impl From<ServerError> for WindowManagerError {
    fn from(value: ServerError) -> Self {
        WindowManagerError::Server(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputOutcome {
    ExitDisplay,
    Consumed,
    ForwardToWindow(WindowId),
}

#[derive(Debug, Clone, Copy)]
pub struct WindowOptions {
    pub owner: ClientId,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_order: u32,
    pub visible: bool,
    pub focused: bool,
    pub resizable: bool,
}

impl WindowOptions {
    pub fn new(owner: ClientId, width: u32, height: u32) -> Self {
        Self {
            owner,
            x: 0,
            y: 0,
            width,
            height,
            z_order: 1,
            visible: true,
            focused: false,
            resizable: true,
        }
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_z_order(mut self, z_order: u32) -> Self {
        self.z_order = z_order;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

#[derive(Debug, Clone, Copy)]
struct ManagedWindow {
    id: WindowId,
    owner: ClientId,
    frame_surface: SurfaceId,
    content_surface: SurfaceId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    visible: bool,
    z_order: u32,
    resizable: bool,
}

pub struct WindowManager {
    windows: BTreeMap<WindowId, ManagedWindow>,
    focused_window: Option<WindowId>,
    next_window_id: WindowId,
    next_z_order: u32,
    control_mode: bool,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: BTreeMap::new(),
            focused_window: None,
            next_window_id: 1,
            next_z_order: 1,
            control_mode: false,
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    pub fn is_control_mode(&self) -> bool {
        self.control_mode
    }

    pub fn content_surface(&self, window_id: WindowId) -> Option<SurfaceId> {
        self.windows.get(&window_id).map(|window| window.content_surface)
    }

    pub fn create_window<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
        options: WindowOptions,
    ) -> Result<WindowId, WindowManagerError> {
        if options.width == 0 || options.height == 0 {
            return Err(WindowManagerError::InvalidDimensions);
        }

        let window_id = self.allocate_window_id()?;
        let width = options.width.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
        let height = options.height.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);
        let (frame_width, frame_height) = Self::frame_dimensions(width, height)?;

        let frame_surface = Self::create_surface(server, options.owner, frame_width, frame_height)?;
        let content_surface = match Self::create_surface(server, options.owner, width, height) {
            Ok(surface_id) => surface_id,
            Err(err) => {
                let _ = Self::destroy_surface(server, options.owner, frame_surface);
                return Err(err);
            }
        };

        let z_order = options.z_order.max(1);
        self.next_z_order = self.next_z_order.max(z_order.saturating_add(1));

        let window = ManagedWindow {
            id: window_id,
            owner: options.owner,
            frame_surface,
            content_surface,
            x: options.x,
            y: options.y,
            width,
            height,
            visible: options.visible,
            z_order,
            resizable: options.resizable,
        };

        self.windows.insert(window_id, window);

        if let Err(err) = Self::apply_window_geometry(server, &window) {
            self.windows.remove(&window_id);
            let _ = Self::destroy_surface(server, options.owner, content_surface);
            let _ = Self::destroy_surface(server, options.owner, frame_surface);
            return Err(err);
        }

        self.upload_frame(server, window_id, false)?;

        if options.focused || self.focused_window.is_none() {
            self.focus_window(server, window_id)?;
        }

        Ok(window_id)
    }

    pub fn destroy_window<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
        window_id: WindowId,
    ) -> Result<(), WindowManagerError> {
        let window = self
            .windows
            .remove(&window_id)
            .ok_or(WindowManagerError::WindowNotFound)?;

        let content_destroy = Self::destroy_surface(server, window.owner, window.content_surface);
        let frame_destroy = Self::destroy_surface(server, window.owner, window.frame_surface);

        if content_destroy.is_err() || frame_destroy.is_err() {
            return Err(WindowManagerError::ServerRejected);
        }

        if self.focused_window == Some(window_id) {
            self.focused_window = None;
            if let Some(next_focus) = self.ordered_window_ids().last().copied() {
                self.focus_window(server, next_focus)?;
            }
        } else {
            self.refresh_frames(server)?;
        }

        Ok(())
    }

    pub fn focus_window<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
        window_id: WindowId,
    ) -> Result<(), WindowManagerError> {
        let mut window = self
            .windows
            .get(&window_id)
            .copied()
            .ok_or(WindowManagerError::WindowNotFound)?;

        window.z_order = self.next_z_order.max(1);
        self.next_z_order = self.next_z_order.saturating_add(1).max(window.z_order.saturating_add(1));
        self.windows.insert(window_id, window);

        Self::apply_window_geometry(server, &window)?;
        Self::request_focus(server, window.owner, Some(window.content_surface))?;

        self.focused_window = Some(window_id);
        self.refresh_frames(server)?;
        Ok(())
    }

    pub fn focus_next_window<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
    ) -> Result<(), WindowManagerError> {
        let ordered = self.ordered_window_ids();
        if ordered.is_empty() {
            return Ok(());
        }

        let next_index = match self
            .focused_window
            .and_then(|focused| ordered.iter().position(|id| *id == focused))
        {
            Some(index) => (index + 1) % ordered.len(),
            None => 0,
        };

        self.focus_window(server, ordered[next_index])
    }

    pub fn focus_prev_window<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
    ) -> Result<(), WindowManagerError> {
        let ordered = self.ordered_window_ids();
        if ordered.is_empty() {
            return Ok(());
        }

        let prev_index = match self
            .focused_window
            .and_then(|focused| ordered.iter().position(|id| *id == focused))
        {
            Some(0) => ordered.len() - 1,
            Some(index) => index - 1,
            None => ordered.len() - 1,
        };

        self.focus_window(server, ordered[prev_index])
    }

    pub fn move_focused_by<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
        delta_x: i32,
        delta_y: i32,
    ) -> Result<(), WindowManagerError> {
        let window_id = self.focused_window.ok_or(WindowManagerError::WindowNotFound)?;
        let mut window = self
            .windows
            .get(&window_id)
            .copied()
            .ok_or(WindowManagerError::WindowNotFound)?;

        window.x = window.x.saturating_add(delta_x);
        window.y = window.y.saturating_add(delta_y);
        self.windows.insert(window_id, window);

        Self::apply_window_geometry(server, &window)
    }

    pub fn resize_focused_by<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
        delta_width: i32,
        delta_height: i32,
    ) -> Result<(), WindowManagerError> {
        let window_id = self.focused_window.ok_or(WindowManagerError::WindowNotFound)?;
        let mut window = self
            .windows
            .get(&window_id)
            .copied()
            .ok_or(WindowManagerError::WindowNotFound)?;

        if !window.resizable {
            return Ok(());
        }

        window.width = Self::clamp_dimension(window.width, delta_width, MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
        window.height =
            Self::clamp_dimension(window.height, delta_height, MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);

        Self::resize_window_surfaces(server, &window)?;
        Self::apply_window_geometry(server, &window)?;

        self.windows.insert(window_id, window);
        self.upload_frame(server, window_id, self.focused_window == Some(window_id))
    }

    pub fn handle_key<B: DisplayBackend>(
        &mut self,
        server: &mut DisplayServer<B>,
        key: u8,
    ) -> Result<InputOutcome, WindowManagerError> {
        if self.control_mode {
            match key {
                KEY_ESCAPE | KEY_TOGGLE_CONTROL => {
                    self.control_mode = false;
                    serial_log(b"[WM] Keyboard control mode disabled\n\0");
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_UP => {
                    self.move_focused_by(server, 0, -MOVE_STEP)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_DOWN => {
                    self.move_focused_by(server, 0, MOVE_STEP)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_LEFT => {
                    self.move_focused_by(server, -MOVE_STEP, 0)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_RIGHT => {
                    self.move_focused_by(server, MOVE_STEP, 0)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_PGUP => {
                    self.focus_prev_window(server)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_PGDN | KEY_TAB => {
                    self.focus_next_window(server)?;
                    Ok(InputOutcome::Consumed)
                }
                b'+' | b'=' => {
                    self.resize_focused_by(server, RESIZE_STEP, RESIZE_STEP)?;
                    Ok(InputOutcome::Consumed)
                }
                b'-' | b'_' => {
                    self.resize_focused_by(server, -RESIZE_STEP, -RESIZE_STEP)?;
                    Ok(InputOutcome::Consumed)
                }
                b'q' | b'Q' => Ok(InputOutcome::ExitDisplay),
                _ => Ok(InputOutcome::Consumed),
            }
        } else {
            match key {
                KEY_TOGGLE_CONTROL => {
                    self.control_mode = true;
                    serial_log(b"[WM] Keyboard control mode enabled\n\0");
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_PGUP => {
                    self.focus_prev_window(server)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_SPECIAL_PGDN => {
                    self.focus_next_window(server)?;
                    Ok(InputOutcome::Consumed)
                }
                KEY_ESCAPE => Ok(InputOutcome::ExitDisplay),
                _ => Ok(self
                    .focused_window
                    .map(InputOutcome::ForwardToWindow)
                    .unwrap_or(InputOutcome::Consumed)),
            }
        }
    }

    fn refresh_frames<B: DisplayBackend>(
        &self,
        server: &mut DisplayServer<B>,
    ) -> Result<(), WindowManagerError> {
        let ordered = self.ordered_window_ids();
        for window_id in ordered {
            let active = self.focused_window == Some(window_id);
            self.upload_frame(server, window_id, active)?;
        }
        Ok(())
    }

    fn upload_frame<B: DisplayBackend>(
        &self,
        server: &mut DisplayServer<B>,
        window_id: WindowId,
        active: bool,
    ) -> Result<(), WindowManagerError> {
        let window = self
            .windows
            .get(&window_id)
            .copied()
            .ok_or(WindowManagerError::WindowNotFound)?;

        let (frame_width, frame_height) = Self::frame_dimensions(window.width, window.height)?;
        let frame_pixels = Self::render_frame_pixels(window.width, window.height, active)?;
        Self::upload_surface(
            server,
            window.owner,
            window.frame_surface,
            frame_width,
            frame_height,
            &frame_pixels,
        )
    }

    fn ordered_window_ids(&self) -> Vec<WindowId> {
        let mut ordered: Vec<(u32, WindowId)> = self
            .windows
            .values()
            .map(|window| (window.z_order, window.id))
            .collect();
        ordered.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        ordered.into_iter().map(|(_, id)| id).collect()
    }

    fn allocate_window_id(&mut self) -> Result<WindowId, WindowManagerError> {
        let start = self.next_window_id.max(1);
        let mut candidate = start;
        loop {
            if candidate == 0 {
                candidate = 1;
            }

            if !self.windows.contains_key(&candidate) {
                self.next_window_id = candidate.wrapping_add(1).max(1);
                return Ok(candidate);
            }

            candidate = candidate.wrapping_add(1);
            if candidate == start {
                return Err(WindowManagerError::WindowIdExhausted);
            }
        }
    }

    fn content_offset_y() -> u32 {
        BORDER_THICKNESS + TITLE_BAR_HEIGHT
    }

    fn frame_dimensions(content_width: u32, content_height: u32) -> Result<(u32, u32), WindowManagerError> {
        let frame_width = content_width
            .checked_add(BORDER_THICKNESS.saturating_mul(2))
            .ok_or(WindowManagerError::InvalidDimensions)?;
        let frame_height = content_height
            .checked_add(Self::content_offset_y())
            .and_then(|value| value.checked_add(BORDER_THICKNESS))
            .ok_or(WindowManagerError::InvalidDimensions)?;

        Ok((frame_width, frame_height))
    }

    fn render_frame_pixels(
        content_width: u32,
        content_height: u32,
        active: bool,
    ) -> Result<Vec<u32>, WindowManagerError> {
        let (frame_width, frame_height) = Self::frame_dimensions(content_width, content_height)?;
        let pixel_count = frame_width
            .checked_mul(frame_height)
            .ok_or(WindowManagerError::InvalidDimensions)? as usize;

        let border_color = if active { 0xFF4A9EFF } else { 0xFF4F4F4F };
        let title_color = if active { 0xFF1B2A40 } else { 0xFF272727 };
        let body_color = if active { 0xFF10151C } else { 0xFF141414 };
        let accent_color = if active { 0xFF88C4FF } else { 0xFF707070 };

        let mut pixels = alloc::vec![border_color; pixel_count];

        let inner_width = frame_width.saturating_sub(BORDER_THICKNESS.saturating_mul(2));
        Self::paint_rect(
            &mut pixels,
            frame_width,
            BORDER_THICKNESS,
            BORDER_THICKNESS,
            inner_width,
            TITLE_BAR_HEIGHT,
            title_color,
        );

        let content_y = Self::content_offset_y();
        let content_height_px = frame_height.saturating_sub(content_y.saturating_add(BORDER_THICKNESS));
        Self::paint_rect(
            &mut pixels,
            frame_width,
            BORDER_THICKNESS,
            content_y,
            inner_width,
            content_height_px,
            body_color,
        );

        let accent_y = BORDER_THICKNESS + TITLE_BAR_HEIGHT.saturating_sub(2);
        Self::paint_rect(
            &mut pixels,
            frame_width,
            BORDER_THICKNESS,
            accent_y,
            inner_width,
            2,
            accent_color,
        );

        let button_size = 6;
        let button_gap = 4;
        let total_button_width = button_size * 3 + button_gap * 2;
        if inner_width > total_button_width + 8 {
            let button_base_x =
                frame_width.saturating_sub(BORDER_THICKNESS + total_button_width + 6);
            let button_y = BORDER_THICKNESS + 5;
            let button_colors = [0xFFD16B6B, 0xFFD1AE6B, 0xFF79BD79];
            for (index, color) in button_colors.iter().enumerate() {
                let x = button_base_x + (index as u32) * (button_size + button_gap);
                Self::paint_rect(
                    &mut pixels,
                    frame_width,
                    x,
                    button_y,
                    button_size,
                    button_size,
                    *color,
                );
            }
        }

        Ok(pixels)
    }

    fn paint_rect(
        pixels: &mut [u32],
        stride: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: u32,
    ) {
        if width == 0 || height == 0 || stride == 0 {
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

    fn clamp_dimension(current: u32, delta: i32, min: u32, max: u32) -> u32 {
        let resized = if delta >= 0 {
            current.saturating_add(delta as u32)
        } else {
            current.saturating_sub((-delta) as u32)
        };
        resized.clamp(min, max)
    }

    fn create_surface<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        owner: ClientId,
        width: u32,
        height: u32,
    ) -> Result<SurfaceId, WindowManagerError> {
        let response = server
            .handle_request(
                owner,
                DisplayRequest::CreateSurface {
                    width,
                    height,
                    format: PixelFormat::Argb8888,
                },
            )
            .map_err(WindowManagerError::Server)?;

        match response {
            DisplayResponse::SurfaceCreated { surface_id } => Ok(surface_id),
            DisplayResponse::Error(_) => Err(WindowManagerError::ServerRejected),
            _ => Err(WindowManagerError::UnexpectedResponse),
        }
    }

    fn destroy_surface<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        owner: ClientId,
        surface_id: SurfaceId,
    ) -> Result<(), WindowManagerError> {
        let response = server
            .handle_request(owner, DisplayRequest::DestroySurface { surface_id })
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(response)
    }

    fn resize_window_surfaces<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        window: &ManagedWindow,
    ) -> Result<(), WindowManagerError> {
        let (frame_width, frame_height) = Self::frame_dimensions(window.width, window.height)?;

        let frame_response = server
            .handle_request(
                window.owner,
                DisplayRequest::ResizeSurface {
                    surface_id: window.frame_surface,
                    width: frame_width,
                    height: frame_height,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(frame_response)?;

        let content_response = server
            .handle_request(
                window.owner,
                DisplayRequest::ResizeSurface {
                    surface_id: window.content_surface,
                    width: window.width,
                    height: window.height,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(content_response)
    }

    fn apply_window_geometry<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        window: &ManagedWindow,
    ) -> Result<(), WindowManagerError> {
        let content_x = window.x.saturating_add(BORDER_THICKNESS as i32);
        let content_y = window.y.saturating_add(Self::content_offset_y() as i32);
        let frame_z = window.z_order.saturating_mul(2);
        let content_z = frame_z.saturating_add(1);

        let frame_pos = server
            .handle_request(
                window.owner,
                DisplayRequest::SetSurfacePosition {
                    surface_id: window.frame_surface,
                    x: window.x,
                    y: window.y,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(frame_pos)?;

        let content_pos = server
            .handle_request(
                window.owner,
                DisplayRequest::SetSurfacePosition {
                    surface_id: window.content_surface,
                    x: content_x,
                    y: content_y,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(content_pos)?;

        let frame_z_response = server
            .handle_request(
                window.owner,
                DisplayRequest::SetSurfaceZOrder {
                    surface_id: window.frame_surface,
                    z_order: frame_z,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(frame_z_response)?;

        let content_z_response = server
            .handle_request(
                window.owner,
                DisplayRequest::SetSurfaceZOrder {
                    surface_id: window.content_surface,
                    z_order: content_z,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(content_z_response)?;

        let frame_visibility = server
            .handle_request(
                window.owner,
                DisplayRequest::SetSurfaceVisibility {
                    surface_id: window.frame_surface,
                    visible: window.visible,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(frame_visibility)?;

        let content_visibility = server
            .handle_request(
                window.owner,
                DisplayRequest::SetSurfaceVisibility {
                    surface_id: window.content_surface,
                    visible: window.visible,
                },
            )
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(content_visibility)
    }

    fn request_focus<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        owner: ClientId,
        surface_id: Option<SurfaceId>,
    ) -> Result<(), WindowManagerError> {
        let response = server
            .handle_request(owner, DisplayRequest::RequestFocus { surface_id })
            .map_err(WindowManagerError::Server)?;
        Self::expect_ack(response)
    }

    fn upload_surface<B: DisplayBackend>(
        server: &mut DisplayServer<B>,
        owner: ClientId,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        pixels: &[u32],
    ) -> Result<(), WindowManagerError> {
        server
            .upload_surface_pixels(owner, surface_id, width, height, pixels, None)
            .map_err(WindowManagerError::Server)
    }

    fn expect_ack(response: DisplayResponse) -> Result<(), WindowManagerError> {
        match response {
            DisplayResponse::Ack => Ok(()),
            DisplayResponse::Error(_) => Err(WindowManagerError::ServerRejected),
            _ => Err(WindowManagerError::UnexpectedResponse),
        }
    }
}

#[inline]
fn serial_log(_message: &'static [u8]) {}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

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
        flush_count: u32,
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
            self.flush_count = self.flush_count.saturating_add(1);
            Ok(())
        }
    }

    #[test]
    fn create_windows_and_cycle_focus() {
        let mut server = DisplayServer::new(MockBackend::default());
        server.start().expect("server should start");

        let mut wm = WindowManager::new();
        let first = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(1), 320, 200)
                    .with_position(10, 12)
                    .with_focused(true),
            )
            .expect("first window should be created");
        let second = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(1), 280, 180).with_position(64, 40),
            )
            .expect("second window should be created");

        assert_eq!(wm.window_count(), 2);
        assert_eq!(wm.focused_window(), Some(first));

        wm.focus_next_window(&mut server)
            .expect("focus should switch to next");
        assert_eq!(wm.focused_window(), Some(second));

        wm.focus_prev_window(&mut server)
            .expect("focus should switch to previous");
        assert_eq!(wm.focused_window(), Some(first));
    }

    #[test]
    fn control_mode_moves_focused_window() {
        let mut server = DisplayServer::new(MockBackend::default());
        server.start().expect("server should start");

        let mut wm = WindowManager::new();
        let window_id = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(1), 320, 200)
                    .with_position(20, 30)
                    .with_focused(true),
            )
            .expect("window should be created");
        let content_surface = wm.content_surface(window_id).expect("content surface should exist");

        let before = server
            .surface(content_surface)
            .copied()
            .expect("surface metadata should exist");

        wm.handle_key(&mut server, KEY_TOGGLE_CONTROL)
            .expect("should enter control mode");
        assert!(wm.is_control_mode());

        wm.handle_key(&mut server, KEY_SPECIAL_RIGHT)
            .expect("window should move right");
        wm.handle_key(&mut server, KEY_SPECIAL_DOWN)
            .expect("window should move down");

        let after = server
            .surface(content_surface)
            .copied()
            .expect("surface metadata should exist");

        assert_eq!(after.x, before.x + MOVE_STEP);
        assert_eq!(after.y, before.y + MOVE_STEP);

        wm.handle_key(&mut server, KEY_ESCAPE)
            .expect("should leave control mode");
        assert!(!wm.is_control_mode());
    }

    #[test]
    fn resize_respects_window_policy() {
        let mut server = DisplayServer::new(MockBackend::default());
        server.start().expect("server should start");

        let mut wm = WindowManager::new();
        let fixed = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(1), 320, 200)
                    .with_focused(true)
                    .with_resizable(false),
            )
            .expect("fixed window should be created");
        let flexible = wm
            .create_window(
                &mut server,
                WindowOptions::new(ClientId::new(1), 240, 160).with_resizable(true),
            )
            .expect("resizable window should be created");

        let fixed_surface = wm.content_surface(fixed).expect("fixed surface should exist");
        let fixed_before = server
            .surface(fixed_surface)
            .copied()
            .expect("surface metadata should exist");

        wm.handle_key(&mut server, KEY_TOGGLE_CONTROL)
            .expect("should enter control mode");
        wm.handle_key(&mut server, b'+')
            .expect("resize request should be accepted");
        wm.handle_key(&mut server, KEY_ESCAPE)
            .expect("should leave control mode");

        let fixed_after = server
            .surface(fixed_surface)
            .copied()
            .expect("surface metadata should exist");
        assert_eq!(fixed_before.width, fixed_after.width);
        assert_eq!(fixed_before.height, fixed_after.height);

        wm.focus_window(&mut server, flexible)
            .expect("should focus resizable window");
        let flexible_surface = wm.content_surface(flexible).expect("surface should exist");
        let flexible_before = server
            .surface(flexible_surface)
            .copied()
            .expect("surface metadata should exist");

        wm.handle_key(&mut server, KEY_TOGGLE_CONTROL)
            .expect("should enter control mode");
        wm.handle_key(&mut server, b'+')
            .expect("resize should succeed");
        wm.handle_key(&mut server, KEY_ESCAPE)
            .expect("should leave control mode");

        let flexible_after = server
            .surface(flexible_surface)
            .copied()
            .expect("surface metadata should exist");
        assert!(flexible_after.width > flexible_before.width);
        assert!(flexible_after.height > flexible_before.height);
    }
}
