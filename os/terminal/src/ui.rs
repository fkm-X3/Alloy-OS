/// UI rendering for Alloy OS Terminal
/// 
/// Handles layout and drawing of the Ratatui interface

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Header
                Constraint::Min(5),     // Output
                Constraint::Length(3),  // Input
                Constraint::Length(1),  // Status bar
            ]
            .as_ref(),
        )
        .split(f.size());

    // Draw header
    draw_header(f, chunks[0]);

    // Draw output area
    draw_output(f, chunks[1], app);

    // Draw input area
    draw_input(f, chunks[2], app);

    // Draw status bar
    draw_status_bar(f, chunks[3], app);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let title = "Alloy OS Terminal - Ratatui Edition";
    let subtitle = "v0.1.0 | Type 'help' for commands";

    let block = Block::default()
        .title(" Alloy OS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::raw(subtitle)]),
    ];

    let paragraph = Paragraph::new(header_lines)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_output(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Output ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

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
        .map(|line| Line::from(vec![Span::raw(line.clone())]))
        .collect();

    let paragraph = Paragraph::new(output_lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_input(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Input ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let prompt = "> ";

    let input_lines = vec![Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(app.input.clone()),
    ])];

    let paragraph = Paragraph::new(input_lines)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);

    // Draw cursor
    let cursor_x = area.x + (prompt.len() as u16) + (app.cursor_pos as u16);
    let cursor_y = area.y + 1; // +1 for border

    if cursor_x < area.right() && cursor_y < area.bottom() {
        f.set_cursor(cursor_x, cursor_y);
    }
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status = match app.input_mode {
        crate::app::InputMode::Normal => "NORMAL",
        crate::app::InputMode::Insert => "INSERT",
    };

    let help_text = "Esc/Ctrl+C: Quit | Up/Down: History | Ctrl+U: Clear Line";
    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", status),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(help_text),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}
