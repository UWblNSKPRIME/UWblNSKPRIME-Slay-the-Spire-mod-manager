use iced::widget::{button, row, text, pick_list, text_input, column};
use iced::{Element, Length, Alignment};
use crate::app::{AppState, Message};
use crate::styles::{AccentButton, BorderButton, TextInputStyle, StopButton};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let theme = state.active_theme();
    let text_color = crate::styles::get_text_color(&theme);
    let muted_color = crate::styles::get_muted_text_color(&theme);

    let profile_names: Vec<String> = state
        .config
        .profiles
        .iter()
        .map(|p| p.name.clone())
        .collect();
    let active_profile = state.config.active_profile.clone();

    let pick_list_widget = pick_list(
        profile_names,
        active_profile,
        Message::SelectProfile
    )
    .placeholder("Select Profile")
    .width(200);

    let add_profile_btn = button(text("+").size(18))
        .padding(6)
        .on_press(Message::OpenNewProfileDialog)
        .style(iced::theme::Button::Custom(Box::new(AccentButton)));

    let is_default_active = state.config.active_profile.as_deref() == Some("Default");
    let delete_profile_btn = if !is_default_active && state.config.active_profile.is_some() {
        button(text("-").size(18))
            .padding(6)
            .on_press(Message::DeleteActiveProfile)
            .style(iced::theme::Button::Custom(Box::new(StopButton)))
    } else {
        button(text("-").size(18))
            .padding(6)
            .style(iced::theme::Button::Custom(Box::new(BorderButton)))
    };

    let refresh_btn = button(text("Refresh List"))
        .padding(8)
        .on_press(Message::ScanMods)
        .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let export_btn = button(text("Export"))
        .padding(8)
        .on_press(Message::ExportMods)
        .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let import_btn = button(text("Import"))
        .padding(8)
        .on_press(Message::OpenImportModal)
        .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let status_text = text(if state.is_game_running { "Status: Running" } else { "Status: OK" })
        .style(muted_color)
        .size(14);

    let mut main_row = row![
        text("Profile:").size(14).style(muted_color),
        pick_list_widget,
        add_profile_btn,
        delete_profile_btn,
        refresh_btn,
        export_btn,
        import_btn
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    main_row = main_row.push(iced::widget::horizontal_space().width(Length::Fill));
    main_row = main_row.push(status_text);

    if state.show_profile_input {
        let name_input = text_input("New profile name...", &state.new_profile_name)
        .on_input(Message::ProfileNameInput)
        .on_submit(Message::CreateNewProfile)
        .padding(6)
        .width(180)
        .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle)));

        let save_profile_btn = button(text("Save"))
            .padding(6)
            .on_press(Message::CreateNewProfile)
            .style(iced::theme::Button::Custom(Box::new(AccentButton)));

        let cancel_profile_btn = button(text("Cancel"))
            .padding(6)
            .on_press(Message::OpenNewProfileDialog)
            .style(iced::theme::Button::Custom(Box::new(BorderButton)));

        let input_row = row![
            text("New Profile Name:").size(14).style(text_color),
            name_input,
            save_profile_btn,
            cancel_profile_btn
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        column![main_row, input_row]
            .spacing(10)
            .into()
    } else {
        main_row.into()
    }
}
