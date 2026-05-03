/// Terminal Emulator View for Alloy OS Terminal

use iced::widget::{column, container, row, text, text_input, scrollable};
use iced::{Element, Length};
use crate::app::{App, Message};

pub fn view(app: &App) -> Element<Message> {
    // Create output display area
    let output_area = {
        let mut output_col = column![];
        
        for line in app.terminal_output.iter() {
            output_col = output_col.push(text(line).size(12));
        }
        
        scrollable(output_col.spacing(2).padding(10))
            .width(Length::Fill)
            .height(Length::Fill)
    };
    
    // Create input bar
    let input_bar = row![
        text("> ").size(14),
        text_input("Enter command...", &app.terminal_input)
            .on_input(Message::TerminalInput)
            .on_submit(Message::TerminalSubmit)
            .padding(5)
            .width(Length::Fill),
    ]
    .spacing(5)
    .padding(10);
    
    // Combine output and input
    let content = column![
        output_area,
        input_bar,
    ]
    .spacing(10)
    .padding(10);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
