#![allow(dead_code)]

/// Logs view - displays system events and logs
/// 
/// Shows timestamped log entries with color-coded severity levels

use crate::app::App;
use crate::ui::theme::Theme;
use crate::ui::views::View;
use crossterm::event::KeyEvent;
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

pub struct LogsView {
    /// Log entries (most recent at the end)
    logs: Vec<LogEntry>,
    /// Scroll offset
    scroll: usize,
}

impl LogsView {
    pub fn new() -> Self {
        let logs = vec![
            LogEntry {
                timestamp: Local::now(),
                level: LogLevel::Info,
                message: "Terminal initialized".to_string(),
            },
            LogEntry {
                timestamp: Local::now(),
                level: LogLevel::Info,
                message: "System metrics collection started".to_string(),
            },
            LogEntry {
                timestamp: Local::now(),
                level: LogLevel::Info,
                message: "Display server ready".to_string(),
            },
            LogEntry {
                timestamp: Local::now(),
                level: LogLevel::Debug,
                message: "Input handler attached".to_string(),
            },
        ];

        Self {
            logs,
            scroll: 0,
        }
    }

    /// Add a new log entry
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push(LogEntry {
            timestamp: Local::now(),
            level,
            message,
        });

        // Keep last 1000 logs
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }
}

impl Default for LogsView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for LogsView {
    fn draw(&self, f: &mut Frame, _app: &App, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Log list
                Constraint::Length(1),  // Status bar
            ])
            .split(f.size());

        draw_header(f, chunks[0], theme);
        draw_log_list(f, chunks[1], self, theme);
        draw_status_bar(f, chunks[2], theme);
    }

    fn handle_input(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;

        let page_size = 10;

        match key.code {
            KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll < self.logs.len().saturating_sub(1) {
                    self.scroll += 1;
                }
            }
            KeyCode::Home => {
                self.scroll = 0;
            }
            KeyCode::End => {
                self.scroll = self.logs.len().saturating_sub(1);
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(page_size);
            }
            KeyCode::PageDown => {
                self.scroll = (self.scroll + page_size).min(self.logs.len().saturating_sub(1));
            }
            _ => {}
        }
    }

    fn update(&mut self, _app: &App) {
        // Could periodically fetch new logs
    }

    fn name(&self) -> &'static str {
        "Logs"
    }
}

fn draw_header(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .title(" System Logs ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let title = Paragraph::new(Line::from(vec![
        Span::styled("Event Log", theme.title_style()),
    ]))
        .block(block)
        .alignment(Alignment::Center)
        .style(theme.base_style());

    f.render_widget(title, area);
}

fn draw_log_list(f: &mut Frame, area: Rect, view: &LogsView, theme: &Theme) {
    let block = Block::default()
        .title(format!(" {} entries ", view.logs.len()))
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style());

    let items: Vec<ListItem> = view.logs
        .iter()
        .map(|entry| {
            let timestamp = entry.timestamp.format("%H:%M:%S").to_string();
            let level_style = match entry.level {
                LogLevel::Error => theme.error_style(),
                LogLevel::Warning => theme.warning_style(),
                LogLevel::Info => theme.success_style(),
                LogLevel::Debug => theme.muted_style(),
            };

            let level_text = match entry.level {
                LogLevel::Error => "ERROR",
                LogLevel::Warning => "WARN",
                LogLevel::Info => "INFO",
                LogLevel::Debug => "DEBUG",
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:6}", level_text),
                    level_style,
                ),
                Span::styled(
                    format!(" {} ", timestamp),
                    theme.muted_style(),
                ),
                Span::raw(&entry.message),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(theme.base_style());

    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = "↑↓/Page Up Down: Scroll | Home/End: Jump";
    let status_line = Line::from(vec![
        Span::styled(help_text, theme.muted_style()),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(theme.status_bar_style());

    f.render_widget(paragraph, area);
}
