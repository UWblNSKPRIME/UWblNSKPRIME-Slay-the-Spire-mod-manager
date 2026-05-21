use iced::widget::{button, checkbox, row, text, pick_list};
use iced::{Element, Length, Alignment};
use crate::app::{AppState, Message};
use crate::styles::{PlayButton, StopButton};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let theme = state.active_theme();
    let muted_color = crate::styles::get_muted_text_color(&theme);

    let debug_checkbox = checkbox("Enable Debug Console Mode", state.config.debug_mode)
        .on_toggle(Message::ToggleDebug)
        .text_size(14);

    let play_button = if state.is_game_running {
        button(
            text("GAME RUNNING...")
                .size(16)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .padding(12)
        .width(200)
        .style(iced::theme::Button::Custom(Box::new(StopButton)))
    } else {
        button(
            text("PLAY GAME")
                .size(16)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .padding(12)
        .width(200)
        .on_press(Message::LaunchGame)
        .style(iced::theme::Button::Custom(Box::new(PlayButton)))
    };

    let themes = vec!["Dark".to_string(), "Light".to_string()];
    let current_theme = Some(if state.config.theme.as_deref() == Some("Light") {
        "Light".to_string()
    } else {
        "Dark".to_string()
    });

    let theme_picker = pick_list(
        themes,
        current_theme,
        Message::SelectTheme
    )
    .width(100);

    let theme_row = row![
        text("Theme:").size(14).style(muted_color),
        theme_picker
    ]
    .spacing(8)
    .align_items(Alignment::Center);

    row![
        debug_checkbox,
        iced::widget::horizontal_space().width(20),
        theme_row,
        iced::widget::horizontal_space().width(Length::Fill),
        play_button
    ]
    .spacing(20)
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .into()
}
