use crate::fusion::backend::FusionDisplayBackend;
use crate::graphics::Display;

pub mod desktop;
pub mod panel;
pub mod window_manager;

pub use desktop::LxqtDesktop;
pub use panel::LxqtPanel;
pub use window_manager::LxqtWindowManager;

pub struct LxqtShell {
    pub desktop: LxqtDesktop,
    pub panel: LxqtPanel,
    pub window_manager: LxqtWindowManager,
    display_width: u32,
    display_height: u32,
}

impl LxqtShell {
    pub fn new(display: &impl Display) -> Self {
        let (width, height) = display.get_resolution();
        Self {
            desktop: LxqtDesktop::new(width, height),
            panel: LxqtPanel::new(width, 36),
            window_manager: LxqtWindowManager::new(width, height),
            display_width: width,
            display_height: height,
        }
    }

    pub fn init_surfaces(&mut self, backend: &mut FusionDisplayBackend) {
        self.desktop.create_surface(backend);
        self.panel.create_surface(backend);
        self.panel
            .set_position(self.display_width, self.display_height);
    }

    pub fn render(&mut self, backend: &mut FusionDisplayBackend, uptime_ms: u64) {
        self.desktop.render(backend);
        self.panel.render(backend, uptime_ms);
        let _windows = self.window_manager.windows_to_render();
    }
}
