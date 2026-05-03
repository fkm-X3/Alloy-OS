/// Alloy OS Terminal - Iced GUI
/// 
/// A modern GUI interface for Alloy OS built on the Iced framework
/// with System Stats and Terminal Emulator views

#[cfg(feature = "desktop")]
use iced::widget::{column, container, row, text, button};
#[cfg(feature = "desktop")]
use iced::{Application, Command, Element, Settings, Length, executor, window};

mod framebuffer_renderer;
mod splash_renderer;

#[cfg(feature = "desktop")]
mod app;
#[cfg(feature = "desktop")]
mod theme;
#[cfg(feature = "desktop")]
mod ui;

#[cfg(feature = "desktop")]
use app::{App, Message};

/// Desktop build: run Iced application
#[cfg(feature = "desktop")]
pub fn main() -> iced::Result {
    TerminalApp::run(Settings {
        window: window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

/// Kernel headless build: render splash to framebuffer
#[cfg(feature = "kernel-headless")]
pub fn main() {
    // Render splash screen to framebuffer for kernel display server
    let mut fb = framebuffer_renderer::Framebuffer::new(1024, 768);
    splash_renderer::render_splash_to_framebuffer(&mut fb, 0.0);
    // Pixels are available via fb.pixels() for kernel to upload to display server
}

#[cfg(feature = "desktop")]
struct TerminalApp {
    app: App,
}

#[cfg(feature = "desktop")]
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