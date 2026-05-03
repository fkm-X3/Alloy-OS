/// Alloy OS Terminal - Iced GUI
/// 
/// A modern GUI interface for Alloy OS built on the Iced framework
/// with System Stats and Terminal Emulator views

use iced::widget::{column, container, row, text, button};
use iced::{Application, Command, Element, Settings, Length, executor, window};

mod app;
mod theme;
mod ui;

use app::{App, Message};

pub fn main() -> iced::Result {
    TerminalApp::run(Settings {
        window: window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

struct TerminalApp {
    app: App,
}

impl Application for TerminalApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                app: App::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Alloy OS Terminal")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.app.update(message);
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Tab bar with view switcher
        let tab_bar = {
            let tabs: Vec<_> = ui::ViewType::all()
                .iter()
                .map(|view| {
                    button(text(view.name()))
                        .on_press(Message::SwitchView(*view))
                        .into()
                })
                .collect();

            row(tabs).spacing(5).padding(10)
        };

        // Main content area
        let content = match self.app.view_manager.current_view() {
            ui::ViewType::Splash => ui::views::splash_view(&self.app),
            ui::ViewType::Terminal => ui::views::terminal_view(&self.app),
        };

        // Combine tab bar and content
        let layout = column![
            tab_bar,
            content,
        ]
        .spacing(0);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }
}

