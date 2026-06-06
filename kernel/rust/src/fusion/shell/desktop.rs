use crate::fusion::backend::FusionDisplayBackend;
use crate::fusion::framebuffer::{Color, FramebufferRenderer};

pub struct LxqtDesktop {
    pub surface_id: u32,
    pub width: u32,
    pub height: u32,
    pub dirty: bool,
}

impl LxqtDesktop {
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
                let _ = backend.set_z_order(self.surface_id, 0);
                let _ = backend.set_position(self.surface_id, 0, 0);
            }
        }
    }

    pub fn render(&mut self, backend: &mut FusionDisplayBackend) {
        if !self.dirty || self.surface_id == 0 {
            return;
        }

        let mut renderer = FramebufferRenderer::new(self.width, self.height).ok();
        let renderer = match renderer.as_mut() {
            Some(r) => r,
            None => return,
        };

        renderer.clear(Color::from_rgb(30, 34, 46));

        let accent = Color::from_rgb(55, 68, 92);
        let stripe_h = (self.height / 16).max(8);
        for i in 0..16 {
            if i % 2 == 0 {
                renderer.fill_rect(0, i * stripe_h, self.width, stripe_h, accent);
            }
        }

        let _ = backend.upload_pixels(self.surface_id, self.width, self.height, renderer.pixels());
        self.dirty = false;
    }
}
