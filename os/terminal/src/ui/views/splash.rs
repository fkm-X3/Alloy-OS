/// Splash screen view with rotating visual elements

use iced::widget::{column, container, row, text};
use iced::{Element, Length, Color};
use crate::app::{App, Message};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn view(_app: &App) -> Element<Message> {
    // Calculate rotation based on system time for continuous smooth animation
    let rotation_angle = if let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let millis = elapsed.as_millis() as f32;
        // Rotate 180 degrees per second (360 degrees in 2 seconds)
        (millis / 5.56) % 360.0
    } else {
        0.0
    };
    
    let splash_content = column![
        text("Alloy OS").size(32),
        row![
            container(
                column![
                    text("Light Logo").size(14),
                    text(format!("⟳ {:.0}°", rotation_angle)).size(32),
                ]
                .spacing(10)
            )
            .padding(20)
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0))
            .center_x()
            .center_y(),
            container(
                column![
                    text("Dark Logo").size(14),
                    text(format!("⟳ {:.0}°", rotation_angle)).size(32),
                ]
                .spacing(10)
            )
            .padding(20)
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0))
            .center_x()
            .center_y(),
        ]
        .spacing(20)
        .padding(20),
        text("Spinning logos loaded successfully").size(14),
    ]
    .spacing(20)
    .padding(40)
    .width(Length::Fill)
    .height(Length::Fill);

    container(splash_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}



