use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::framebuffer::{Color, FramebufferRenderer};

pub struct LxqtPanel {
    pub surface_id: u32,
    pub width: u32,
    pub height: u32,
    pub dirty: bool,
}

impl LxqtPanel {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            surface_id: 0,
            width,
            height,
            dirty: true,
        }
    }

    pub fn create_surface(&mut self, backend: &mut FusionDisplayBackend) {
        if self.surface_id == 0 {
            self.surface_id = backend.create_surface(self.width, self.height).unwrap_or(0);
            if self.surface_id != 0 {
                let _ = backend.set_z_order(self.surface_id, 65534);
            }
        }
    }

    pub fn set_position(&mut self, display_width: u32, display_height: u32) {
        if self.surface_id != 0 {
            let _ = self.width;
            let y = (display_height.saturating_sub(self.height)) as i32;
            let _ = y;
            self.width = display_width;
            self.dirty = true;
        }
    }

    pub fn render(&mut self, backend: &mut FusionDisplayBackend, uptime_ms: u64) {
        if !self.dirty || self.surface_id == 0 {
            return;
        }

        let mut renderer = match FramebufferRenderer::new(self.width, self.height) {
            Ok(r) => r,
            Err(_) => return,
        };

        let bg = Color::from_rgb(18, 22, 32);
        renderer.clear(bg);

        let border = Color::from_rgb(42, 50, 68);
        renderer.h_line(0, self.width - 1, 0, border, 1);

        let btn_color = Color::from_rgb(65, 130, 220);
        renderer.fill_rect(4, 4, 28, self.height - 8, btn_color);
        renderer.fill_rect(8, 10, 20, 2, Color::white());
        renderer.fill_rect(8, 15, 20, 2, Color::white());
        renderer.fill_rect(8, 20, 20, 2, Color::white());

        let task_y = 34u32;
        let _ = task_y;

        let seconds = (uptime_ms / 1000) % 60;
        let minutes = (uptime_ms / 60000) % 60;
        let hours = (uptime_ms / 3600000) % 24;

        let time_str = alloc::format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let time_w = renderer.text_width(&time_str);
        let time_x = self.width.saturating_sub(time_w + 10);

        let clock_color = Color::from_rgb(180, 190, 210);
        renderer.draw_text(time_x, 6, &time_str, clock_color, Some(bg));

        let _ = backend.upload_pixels(self.surface_id, self.width, self.height, renderer.pixels());
        self.dirty = false;
    }
}
