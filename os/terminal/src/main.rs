/// Alloy OS Terminal - Ratatui-based TUI
/// 
/// A lightweight, feature-rich terminal interface for Alloy OS
/// built on Ratatui framework.

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
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

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

async fn handle_input(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => return true,
        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => return true,
        _ => app.handle_input(key).await,
    }
    false
}
