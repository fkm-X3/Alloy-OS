/// System Stats View for Alloy OS Terminal

use iced::widget::{column, container, row, text};
use iced::{Element, Length};
use crate::app::{App, Message};

pub fn view(_app: &App) -> Element<Message> {
    let stats_content = column![
        text("System Statistics").size(24),
        text("").size(4),
        row![
            column![
                text("CPU Usage").size(16),
                text("75.2%").size(14),
            ],
            column![
                text("Memory Usage").size(16),
                text("2.4 GB / 8.0 GB").size(14),
            ],
            column![
                text("Uptime").size(16),
                text("2d 5h 23m").size(14),
            ],
        ]
        .spacing(20)
        .padding(10),
        text("").size(4),
        text("Running Processes").size(16),
        text("init, shell, terminal, kernel...").size(12),
    ]
    .spacing(10)
    .padding(20);

    container(stats_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
}
