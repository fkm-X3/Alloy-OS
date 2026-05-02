/// Alloy OS Terminal - Ratatui-based TUI
/// 
/// A feature-rich terminal interface for Alloy OS
/// built on Ratatui framework with tab-based multi-view support.

use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;

mod app;
mod commands;
mod ui;

use app::App;
use ui::{Theme, ViewType, views::View};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    mut terminal: Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let theme = Theme::default();

    loop {
        terminal.draw(|f| draw_ui(f, &app, &theme))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_input(&mut app, key).await {
                    return Ok(());
                }
            }
        }

        // Process any pending updates
        app.update().await;
    }
}

fn draw_ui(f: &mut Frame, app: &App, theme: &Theme) {
    // Draw the tab bar at the top
    draw_tab_bar(f, app, theme);

    // Draw the current view
    match app.view_manager.current_view {
        ViewType::Terminal => {
            app.terminal_view.draw(f, app, theme);
        }
        ViewType::Monitor => {
            app.monitor_view.draw(f, app, theme);
        }
        ViewType::Help => {
            app.help_view.draw(f, app, theme);
        }
        ViewType::Logs => {
            app.logs_view.draw(f, app, theme);
        }
    }
}

fn draw_tab_bar(f: &mut Frame, app: &App, theme: &Theme) {
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    let mut tabs = vec![];
    for view in ViewType::all().iter() {
        let is_active = app.view_manager.current_view == *view;
        let style = if is_active {
            theme.status_active_style()
        } else {
            theme.status_bar_style()
        };

        tabs.push(Span::styled(
            format!(" {} ", view.name()),
            style,
        ));
        tabs.push(Span::raw(" "));
    }

    let tab_line = Line::from(tabs);
    let tab_bar = Paragraph::new(tab_line)
        .style(theme.status_bar_style());

    let tab_area = Rect {
        x: 0,
        y: 0,
        width: f.size().width,
        height: 1,
    };

    f.render_widget(tab_bar, tab_area);
}

async fn handle_input(app: &mut App, key: KeyEvent) -> bool {
    use crossterm::event::KeyModifiers;

    // Global hotkeys
    match key.code {
        KeyCode::Esc => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.modifiers.contains(KeyModifiers::SHIFT) {
                app.view_manager.prev_view();
            } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.view_manager.next_view();
            }
            return false;
        }
        _ => {}
    }

    // Route input to current view
    match app.view_manager.current_view {
        ViewType::Terminal => {
            app.handle_input(key).await;
        }
        ViewType::Monitor => {
            app.monitor_view.handle_input(key);
        }
        ViewType::Help => {
            app.help_view.handle_input(key);
        }
        ViewType::Logs => {
            app.logs_view.handle_input(key);
        }
    }

    false
}
