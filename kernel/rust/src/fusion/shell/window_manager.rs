use crate::fusion::backend::{FusionDisplayBackend, FusionError};
use crate::fusion::framebuffer::{Color, FramebufferRenderer};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub content_surface_id: u32,
    pub frame_surface_id: u32,
    pub title: alloc::string::String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_order: u32,
    pub state: WindowState,
    pub focused: bool,
}

pub struct LxqtWindowManager {
    windows: BTreeMap<WindowId, ManagedWindow>,
    next_window_id: u32,
    focus_order: Vec<WindowId>,
    display_width: u32,
    display_height: u32,
    focused_window: Option<WindowId>,
}

impl LxqtWindowManager {
    pub fn new(display_width: u32, display_height: u32) -> Self {
        Self {
            windows: BTreeMap::new(),
            next_window_id: 1,
            focus_order: Vec::new(),
            display_width,
            display_height,
            focused_window: None,
        }
    }

    pub fn create_window(
        &mut self,
        backend: &mut FusionDisplayBackend,
        title: &str,
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<WindowId, FusionError> {
        let id = WindowId(self.next_window_id);
        self.next_window_id = self.next_window_id.saturating_add(1);

        let frame_w = width.saturating_add(8);
        let frame_h = height.saturating_add(28);
        let frame_id = backend.create_surface(frame_w, frame_h)?;
        let content_id = backend.create_surface(width, height)?;

        let win = ManagedWindow {
            content_surface_id: content_id,
            frame_surface_id: frame_id,
            title: alloc::string::String::from(title),
            x,
            y,
            width,
            height,
            z_order: 256,
            state: WindowState::Normal,
            focused: false,
        };
        self.windows.insert(id, win);
        self.focus_order.push(id);

        backend.set_position(frame_id, x, y)?;
        backend.set_position(content_id, x + 4, y + 24)?;
        backend.set_z_order(frame_id, 256)?;
        backend.set_z_order(content_id, 257)?;

        self.render_window_frame(id, backend);
        self.focus_window(id, backend);

        Ok(id)
    }

    pub fn destroy_window(
        &mut self,
        id: WindowId,
        backend: &mut FusionDisplayBackend,
    ) -> Result<(), FusionError> {
        if let Some(win) = self.windows.remove(&id) {
            let _ = backend.destroy_surface(win.content_surface_id);
            let _ = backend.destroy_surface(win.frame_surface_id);
        }
        self.focus_order.retain(|&fid| fid != id);
        if self.focused_window == Some(id) {
            self.focused_window = self.focus_order.last().copied();
        }
        Ok(())
    }

    pub fn focus_window(&mut self, id: WindowId, backend: &mut FusionDisplayBackend) {
        if !self.windows.contains_key(&id) {
            return;
        }
        if let Some(old) = self.focused_window {
            if let Some(old_win) = self.windows.get_mut(&old) {
                old_win.focused = false;
                self.render_window_frame(old, backend);
            }
        }
        if let Some(win) = self.windows.get_mut(&id) {
            win.focused = true;
            self.render_window_frame(id, backend);
        }
        self.focused_window = Some(id);
        self.focus_order.retain(|&fid| fid != id);
        self.focus_order.push(id);
    }

    pub fn move_window(
        &mut self,
        id: WindowId,
        dx: i32,
        dy: i32,
        backend: &mut FusionDisplayBackend,
    ) -> Result<(), FusionError> {
        if let Some(win) = self.windows.get_mut(&id) {
            win.x = win.x.saturating_add(dx);
            win.y = win.y.saturating_add(dy);
            backend.set_position(win.frame_surface_id, win.x, win.y)?;
            backend.set_position(win.content_surface_id, win.x + 4, win.y + 24)?;
        }
        Ok(())
    }

    pub fn window_at_point(&self, x: i32, y: i32) -> Option<WindowId> {
        for id in self.focus_order.iter().rev() {
            if let Some(win) = self.windows.get(id) {
                if win.state == WindowState::Minimized || win.state == WindowState::Hidden {
                    continue;
                }
                let frame_w = win.width.saturating_add(8) as i32;
                let frame_h = win.height.saturating_add(28) as i32;
                if x >= win.x && x < win.x + frame_w && y >= win.y && y < win.y + frame_h {
                    return Some(*id);
                }
            }
        }
        None
    }

    pub fn title_bar_at_point(&self, x: i32, y: i32) -> Option<WindowId> {
        for id in self.focus_order.iter().rev() {
            if let Some(win) = self.windows.get(id) {
                if y >= win.y && y < win.y + 24 && x >= win.x && x < win.x + win.width as i32 + 8 {
                    return Some(*id);
                }
            }
        }
        None
    }

    pub fn render_window_frame(&self, id: WindowId, backend: &mut FusionDisplayBackend) {
        let Some(win) = self.windows.get(&id) else {
            return;
        };
        let frame_w = win.width.saturating_add(8);
        let frame_h = win.height.saturating_add(28);
        let mut r = match FramebufferRenderer::new(frame_w, frame_h) {
            Ok(r) => r,
            Err(_) => return,
        };

        let title_bar_color = if win.focused {
            Color::from_rgb(48, 90, 150)
        } else {
            Color::from_rgb(40, 44, 56)
        };
        let border_color = Color::from_rgb(35, 40, 52);

        r.clear(Color::from_rgb(25, 30, 42));
        r.fill_rect(0, 0, frame_w, 24, title_bar_color);
        r.stroke_rect(0, 0, frame_w, frame_h, border_color, 1);

        let close_x = frame_w.saturating_sub(22);
        r.fill_rect(close_x, 6, 16, 12, Color::from_rgb(200, 60, 60));

        let title_color = if win.focused {
            Color::white()
        } else {
            Color::from_rgb(180, 190, 210)
        };
        let max_title_w = close_x.saturating_sub(10);
        let cw = r.char_width();
        let display_title = if win.title.len().saturating_mul(cw as usize) > max_title_w as usize {
            let max_chars = (max_title_w / cw).saturating_sub(1) as usize;
            let mut t = alloc::string::String::with_capacity(max_chars + 1);
            for (i, ch) in win.title.chars().enumerate() {
                if i >= max_chars {
                    t.push('…');
                    break;
                }
                t.push(ch);
            }
            t
        } else {
            win.title.clone()
        };
        r.draw_text(6, 4, &display_title, title_color, Some(title_bar_color));

        let _ = backend.upload_pixels(win.frame_surface_id, frame_w, frame_h, r.pixels());
    }

    pub fn set_workspace_bounds(&mut self, width: u32, height: u32) {
        self.display_width = width;
        self.display_height = height;
    }

    pub fn windows_to_render(&self) -> Vec<(WindowId, i32, i32, u32, u32, u32, &[u32])> {
        let result = Vec::new();
        for (_id, win) in &self.windows {
            if win.state == WindowState::Minimized || win.state == WindowState::Hidden {
                continue;
            }
        }
        result
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focused_window
    }
}
