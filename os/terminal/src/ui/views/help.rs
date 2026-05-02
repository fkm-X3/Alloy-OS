/// Help view - displays command reference
/// 
/// Shows available commands in a formatted table with descriptions

use crate::app::App;
use crate::ui::theme::Theme;
use crate::ui::views::View;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct HelpView {
    /// Scrollable list of commands
    commands: Vec<(String, String)>,
    /// Currently selected command index
    selected: usize,
}

impl HelpView {
    pub fn new() -> Self {
        let commands = vec![
            ("help [COMMAND]".to_string(), "Show this help message or info about a command".to_string()),
            ("echo [TEXT]".to_string(), "Print text to the output".to_string()),
            ("clear".to_string(), "Clear the screen".to_string()),
            ("version".to_string(), "Show OS version and features".to_string()),
            ("sysinfo".to_string(), "Display system summary (requires FFI)".to_string()),
            ("free".to_string(), "Show memory usage statistics".to_string()),
            ("uptime".to_string(), "Display system uptime".to_string()),
            ("meminfo".to_string(), "Show detailed memory information".to_string()),
            ("cpuinfo".to_string(), "Display CPU information and features".to_string()),
            ("ticks".to_string(), "Show system timer statistics".to_string()),
            ("uname".to_string(), "Print OS name".to_string()),
        ];

        Self {
            commands,
            selected: 0,
        }
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for HelpView {
    fn draw(&self, f: &mut Frame, _app: &App, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Command list
                Constraint::Length(1),  // Status bar
            ])
            .split(f.size());

        draw_header(f, chunks[0], theme);
        draw_command_list(f, chunks[1], self, theme);
        draw_status_bar(f, chunks[2], theme);
    }

    fn handle_input(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected < self.commands.len() - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::End => {
                self.selected = self.commands.len() - 1;
            }
            _ => {}
        }
    }

    fn update(&mut self, _app: &App) {
        // Nothing to update
    }

    fn name(&self) -> &'static str {
        "Help"
    }
}

fn draw_header(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .title(" Command Reference ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let title = Paragraph::new(Line::from(vec![
        Span::styled("Available Commands", theme.title_style()),
    ]))
        .block(block)
        .alignment(Alignment::Center)
        .style(theme.base_style());

    f.render_widget(title, area);
}

fn draw_command_list(f: &mut Frame, area: Rect, view: &HelpView, theme: &Theme) {
    let block = Block::default()
        .title(" Use ↑↓ to navigate ")
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let items: Vec<ListItem> = view.commands
        .iter()
        .enumerate()
        .map(|(idx, (cmd, desc))| {
            let is_selected = idx == view.selected;
            let style = if is_selected {
                theme.selected_style()
            } else {
                theme.base_style()
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:<20}", cmd),
                    theme.active_style(),
                ),
                Span::styled(" - ", theme.muted_style()),
                Span::raw(desc),
            ]);

            if is_selected {
                ListItem::new(content).style(style)
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(theme.base_style());

    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = "Use Up/Down arrows to navigate, or type a command to run it";
    let status_line = Line::from(vec![
        Span::styled(help_text, theme.muted_style()),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(theme.status_bar_style());

    f.render_widget(paragraph, area);
}
