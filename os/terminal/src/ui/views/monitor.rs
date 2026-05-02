/// System Monitor view - displays real-time metrics
/// 
/// Shows gauges for memory, CPU, and other system metrics with visual indicators

use crate::app::App;
use crate::ui::theme::Theme;
use crate::ui::views::View;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub struct MonitorView {
    /// Sample memory usage (placeholder)
    memory_percent: u16,
    /// Sample CPU usage (placeholder)
    cpu_percent: u16,
    /// Sample uptime in seconds
    uptime_secs: u64,
}

impl MonitorView {
    pub fn new() -> Self {
        Self {
            memory_percent: 42,
            cpu_percent: 28,
            uptime_secs: 3661, // 1 hour 1 minute
        }
    }
}

impl Default for MonitorView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for MonitorView {
    fn draw(&self, f: &mut Frame, _app: &App, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Min(10),      // Metrics
                Constraint::Length(1),   // Status bar
            ])
            .split(f.size());

        draw_header(f, chunks[0], theme);
        draw_metrics(f, chunks[1], self, theme);
        draw_status_bar(f, chunks[2], theme);
    }

    fn handle_input(&mut self, _key: KeyEvent) {
        // Handle input if needed
    }

    fn update(&mut self, _app: &App) {
        // Update metrics periodically
        // In a real implementation, this would fetch from kernel/system
    }

    fn name(&self) -> &'static str {
        "Monitor"
    }
}

fn draw_header(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .title(" System Monitor ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let title = Paragraph::new(Line::from(vec![
        Span::styled("Real-time System Metrics", theme.title_style()),
    ]))
        .block(block)
        .alignment(Alignment::Center)
        .style(theme.base_style());

    f.render_widget(title, area);
}

fn draw_metrics(f: &mut Frame, area: Rect, view: &MonitorView, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Memory gauge
            Constraint::Length(3),  // CPU gauge
            Constraint::Length(4),  // Uptime and stats
            Constraint::Min(0),     // Filler
        ])
        .split(area);

    // Memory gauge
    let memory_gauge = Gauge::default()
        .block(Block::default()
            .title(" Memory Usage ")
            .borders(Borders::ALL)
            .border_style(theme.border_style()))
        .gauge_style(theme.active_style())
        .percent(view.memory_percent as u16)
        .label(format!("{}%", view.memory_percent));

    f.render_widget(memory_gauge, chunks[0]);

    // CPU gauge
    let cpu_gauge = Gauge::default()
        .block(Block::default()
            .title(" CPU Usage ")
            .borders(Borders::ALL)
            .border_style(theme.border_style()))
        .gauge_style(theme.secondary_style())
        .percent(view.cpu_percent as u16)
        .label(format!("{}%", view.cpu_percent));

    f.render_widget(cpu_gauge, chunks[1]);

    // Uptime and info
    let uptime = format_uptime(view.uptime_secs);
    let stats_lines = vec![
        Line::from(vec![
            Span::styled("Uptime: ", theme.title_style()),
            Span::raw(uptime),
        ]),
        Line::from(vec![
            Span::styled("Load Average: ", theme.muted_style()),
            Span::raw("(placeholder)"),
        ]),
    ];

    let stats_block = Block::default()
        .title(" System Info ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let stats = Paragraph::new(stats_lines)
        .block(stats_block)
        .style(theme.base_style());

    f.render_widget(stats, chunks[2]);
}

fn draw_status_bar(f: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = "Note: Metrics are placeholders (requires kernel FFI integration)";
    let status_line = Line::from(vec![
        Span::styled(help_text, theme.muted_style()),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(theme.status_bar_style());

    f.render_widget(paragraph, area);
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
