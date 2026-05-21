use crate::app::Message;
use crate::styles::{AccentButton, BorderButton, PanelBg};
use iced::widget::{button, column, container, row, text, vertical_space};
use iced::{Alignment, Element, Length};

pub fn view<'a>() -> Element<'a, Message> {
    let title = text("Slay the Spire Mod Loader")
        .size(28)
        .style(iced::Color::from_rgb(0.9, 0.9, 0.95));

    let subtitle = text("To begin, we need to locate your Slay the Spire installation directory.")
        .size(16)
        .style(iced::Color::from_rgb(0.6, 0.65, 0.7));

    let auto_detect_btn = button(
        text("Auto-Detect Path")
            .size(16)
            .horizontal_alignment(iced::alignment::Horizontal::Center),
    )
    .padding(12)
    .width(200)
    .on_press(Message::AutoDetectPath)
    .style(iced::theme::Button::Custom(Box::new(AccentButton)));

    let browse_btn = button(
        text("Browse Manually...")
            .size(16)
            .horizontal_alignment(iced::alignment::Horizontal::Center),
    )
    .padding(12)
    .width(200)
    .on_press(Message::BrowsePath)
    .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let buttons = row![auto_detect_btn, browse_btn]
        .spacing(20)
        .align_items(Alignment::Center);

    let content = column![
        title,
        vertical_space().height(10),
        subtitle,
        vertical_space().height(30),
        buttons
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(PanelBg)))
        .into()
}
