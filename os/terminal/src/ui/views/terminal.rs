/// Terminal view - main command input/output area
/// 
/// Refactored version of the original ui.rs with modern styling and enhanced layout

use crate::app::App;
use crate::ui::theme::Theme;
use crate::ui::views::View;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct TerminalView;

impl TerminalView {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TerminalView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TerminalView {
    fn draw(&self, f: &mut Frame, app: &App, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(5),     // Output
                Constraint::Length(3),  // Input
                Constraint::Length(1),  // Status bar
            ])
            .split(f.size());

        draw_header(f, chunks[0], theme);
        draw_output(f, chunks[1], app, theme);
        draw_input(f, chunks[2], app, theme);
        draw_status_bar(f, chunks[3], app, theme);
    }

    fn handle_input(&mut self, _key: KeyEvent) {
        // Handled by main app
    }

    fn update(&mut self, _app: &App) {
        // Nothing to update for terminal view
    }

    fn name(&self) -> &'static str {
        "Terminal"
    }
}

fn draw_header(f: &mut Frame, area: Rect, theme: &Theme) {
    let title = "Alloy OS Terminal";
    let subtitle = "v0.1.0 | Type 'help' for commands";

    let block = Block::default()
        .title(" Alloy OS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let header_lines = vec![
        Line::from(vec![Span::styled(title, theme.title_style())]),
        Line::from(vec![Span::styled(subtitle, theme.muted_style())]),
    ];

    let paragraph = Paragraph::new(header_lines)
        .block(block)
        .alignment(Alignment::Center)
        .style(theme.base_style());

    f.render_widget(paragraph, area);
}

fn draw_output(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(" Output ")
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    // Convert output deque to lines, taking only the last N lines that fit
    let max_lines = area.height as usize;
    let start = if app.output.len() > max_lines {
        app.output.len() - max_lines
    } else {
        0
    };

    let output_lines: Vec<Line> = app.output
        .iter()
        .skip(start)
        .map(|line| {
            // Color-code different types of output
            if line.starts_with('>') {
                // Command prompt
                Line::from(vec![Span::styled(line.clone(), theme.secondary_style())])
            } else if line.contains("Error") || line.contains("error") {
                // Error messages
                Line::from(vec![Span::styled(line.clone(), theme.error_style())])
            } else if line.contains("Warning") || line.contains("warning") {
                // Warning messages
                Line::from(vec![Span::styled(line.clone(), theme.warning_style())])
            } else {
                // Normal output
                Line::from(vec![Span::raw(line.clone())])
            }
        })
        .collect();

    let paragraph = Paragraph::new(output_lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(theme.base_style());

    f.render_widget(paragraph, area);
}

fn draw_input(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(" Input ")
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let prompt = "❯ ";

    let input_lines = vec![Line::from(vec![
        Span::styled(prompt, theme.active_style().add_modifier(Modifier::BOLD)),
        Span::raw(app.input.clone()),
    ])];

    let paragraph = Paragraph::new(input_lines)
        .block(block)
        .style(theme.base_style());

    f.render_widget(paragraph, area);

    // Draw cursor
    let cursor_x = area.x + (prompt.len() as u16) + (app.cursor_pos as u16);
    let cursor_y = area.y + 1; // +1 for border

    if cursor_x < area.right() && cursor_y < area.bottom() {
        f.set_cursor(cursor_x, cursor_y);
    }
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let status = match app.input_mode {
        crate::app::InputMode::Normal => "NORMAL",
        crate::app::InputMode::Insert => "INSERT",
    };

    let help_text = "^Tab: Next | ^Shift+Tab: Prev | ^C/Esc: Quit";
    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", status),
            theme.status_active_style(),
        ),
        Span::raw(" "),
        Span::styled(help_text, theme.muted_style()),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(theme.status_bar_style());

    f.render_widget(paragraph, area);
}
