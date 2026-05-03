/// View manager for tab-based navigation

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewType {
    SystemStats,
    Terminal,
}

impl ViewType {
    pub fn all() -> [ViewType; 2] {
        [ViewType::SystemStats, ViewType::Terminal]
    }

    pub fn next(self) -> ViewType {
        match self {
            ViewType::SystemStats => ViewType::Terminal,
            ViewType::Terminal => ViewType::SystemStats,
        }
    }

    pub fn prev(self) -> ViewType {
        match self {
            ViewType::SystemStats => ViewType::Terminal,
            ViewType::Terminal => ViewType::SystemStats,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ViewType::SystemStats => "System Stats",
            ViewType::Terminal => "Terminal",
        }
    }
}

pub struct ViewManager {
    current: ViewType,
}

impl ViewManager {
    pub fn new() -> Self {
        Self {
            current: ViewType::SystemStats,
        }
    }

    pub fn current_view(&self) -> ViewType {
        self.current
    }

    pub fn set_view(&mut self, view: ViewType) {
        self.current = view;
    }

    pub fn next_view(&mut self) {
        self.current = self.current.next();
    }

    pub fn prev_view(&mut self) {
        self.current = self.current.prev();
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}
