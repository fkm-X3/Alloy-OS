#![allow(dead_code)]

/// View trait and manager for tab-based navigation
/// 
/// Allows switching between different UI views (Terminal, Monitor, Help, Logs)

use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::Frame;
use crossterm::event::KeyEvent;

/// Trait for renderable views in the TUI
pub trait View {
    /// Render this view to the frame
    fn draw(&self, f: &mut Frame, app: &App, theme: &Theme);

    /// Handle keyboard input
    fn handle_input(&mut self, key: KeyEvent);

    /// Update view state
    fn update(&mut self, app: &App);

    /// Get view name for tab display
    fn name(&self) -> &'static str;
}

/// Available views in the application
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewType {
    Terminal,
    Monitor,
    Help,
    Logs,
}

impl ViewType {
    /// Get all available views
    pub fn all() -> [ViewType; 4] {
        [
            ViewType::Terminal,
            ViewType::Monitor,
            ViewType::Help,
            ViewType::Logs,
        ]
    }

    /// Get next view in cycle
    pub fn next(self) -> ViewType {
        match self {
            ViewType::Terminal => ViewType::Monitor,
            ViewType::Monitor => ViewType::Help,
            ViewType::Help => ViewType::Logs,
            ViewType::Logs => ViewType::Terminal,
        }
    }

    /// Get previous view in cycle
    pub fn prev(self) -> ViewType {
        match self {
            ViewType::Terminal => ViewType::Logs,
            ViewType::Monitor => ViewType::Terminal,
            ViewType::Help => ViewType::Monitor,
            ViewType::Logs => ViewType::Help,
        }
    }

    /// Get display name for tab
    pub fn name(self) -> &'static str {
        match self {
            ViewType::Terminal => "Terminal",
            ViewType::Monitor => "Monitor",
            ViewType::Help => "Help",
            ViewType::Logs => "Logs",
        }
    }
}

/// Manages switching between views
pub struct ViewManager {
    pub current_view: ViewType,
}

impl ViewManager {
    pub fn new() -> Self {
        Self {
            current_view: ViewType::Terminal,
        }
    }

    /// Switch to next view
    pub fn next_view(&mut self) {
        self.current_view = self.current_view.next();
    }

    /// Switch to previous view
    pub fn prev_view(&mut self) {
        self.current_view = self.current_view.prev();
    }

    /// Switch to specific view
    pub fn set_view(&mut self, view: ViewType) {
        self.current_view = view;
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}

pub mod terminal;
pub mod monitor;
pub mod help;
pub mod logs;

pub use terminal::TerminalView;
pub use monitor::MonitorView;
pub use help::HelpView;
pub use logs::LogsView;
