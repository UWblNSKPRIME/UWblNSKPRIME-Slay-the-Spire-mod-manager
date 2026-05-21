use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Alignment};
use crate::app::{AppState, Message};
use crate::styles::{TextInputStyle, BorderButton, PanelBg};

// Custom button stylesheet for mod list items
struct ModListItemButton {
    is_selected: bool,
}

impl button::StyleSheet for ModListItemButton {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        let border_color = if is_light { crate::styles::COLOR_ACCENT_LIGHT } else { crate::styles::COLOR_ACCENT_DARK };
        let text_color = if is_light { crate::styles::COLOR_TEXT_FG_LIGHT } else { crate::styles::COLOR_TEXT_FG_DARK };
        button::Appearance {
            background: if self.is_selected {
                if is_light {
                    Some(iced::Background::Color(iced::Color::from_rgb(0.90, 0.93, 0.98)))
                } else {
                    Some(iced::Background::Color(iced::Color::from_rgb(0.20, 0.20, 0.28)))
                }
            } else {
                None
            },
            border: iced::Border {
                color: border_color,
                width: if self.is_selected { 1.0 } else { 0.0 },
                radius: 4.0.into(),
            },
            text_color,
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);
        let is_light = matches!(style, iced::Theme::Light);
        if !self.is_selected {
            active.background = Some(iced::Background::Color(if is_light {
                iced::Color::from_rgb(0.95, 0.96, 0.98)
            } else {
                iced::Color::from_rgb(0.16, 0.16, 0.22)
            }));
        }
        active
    }
}

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let theme = state.active_theme();
    let text_color = crate::styles::get_text_color(&theme);
    let muted_color = crate::styles::get_muted_text_color(&theme);

    let query = state.search_query.to_lowercase();
    let filtered_mods: Vec<&sts_modloader_core::ModInfo> = state
        .mods
        .iter()
        .filter(|m| {
            m.name.to_lowercase().contains(&query)
                || m.id.to_lowercase().contains(&query)
        })
        .collect();

    let search_box = text_input("Search mods...", &state.search_query)
        .on_input(Message::SearchQueryChanged)
        .padding(8)
        .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle)));

    // Toggle all filtered buttons
    let enable_all_btn = button(text("Enable All").size(12))
        .padding(6)
        .on_press(Message::ToggleAllMods(true))
        .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let disable_all_btn = button(text("Disable All").size(12))
        .padding(6)
        .on_press(Message::ToggleAllMods(false))
        .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let toggle_row = row![
        text(format!("Mods ({})", filtered_mods.len())).size(14).style(muted_color),
        iced::widget::horizontal_space().width(Length::Fill),
        enable_all_btn,
        disable_all_btn
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    // Order modification buttons (Move Up, Move Down, Auto Sort)
    let is_search_empty = state.search_query.trim().is_empty();
    
    let move_up_btn = if let Some(ref selected_id) = state.selected_mod_id {
        if is_search_empty {
            button(text("Move Up ▲").size(12))
                .padding(6)
                .on_press(Message::MoveModUp(selected_id.clone()))
                .style(iced::theme::Button::Custom(Box::new(BorderButton)))
        } else {
            button(text("Move Up ▲").size(12))
                .padding(6)
                .style(iced::theme::Button::Custom(Box::new(BorderButton)))
        }
    } else {
        button(text("Move Up ▲").size(12))
            .padding(6)
            .style(iced::theme::Button::Custom(Box::new(BorderButton)))
    };

    let move_down_btn = if let Some(ref selected_id) = state.selected_mod_id {
        if is_search_empty {
            button(text("Move Down ▼").size(12))
                .padding(6)
                .on_press(Message::MoveModDown(selected_id.clone()))
                .style(iced::theme::Button::Custom(Box::new(BorderButton)))
        } else {
            button(text("Move Down ▼").size(12))
                .padding(6)
                .style(iced::theme::Button::Custom(Box::new(BorderButton)))
        }
    } else {
        button(text("Move Down ▼").size(12))
            .padding(6)
            .style(iced::theme::Button::Custom(Box::new(BorderButton)))
    };

    let auto_sort_btn = button(text("⚡ Auto Sort").size(12))
        .padding(6)
        .on_press(Message::AutoSortMods)
        .style(iced::theme::Button::Custom(Box::new(BorderButton)));

    let ordering_row = row![
        text("Order:").size(12).style(muted_color),
        iced::widget::horizontal_space().width(Length::Fill),
        move_up_btn,
        move_down_btn,
        auto_sort_btn
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    let mut list_col = column![].spacing(4);
    for m in &filtered_mods {
        let is_selected = state.selected_mod_id.as_deref() == Some(&m.id);
        
        let cb = checkbox("", m.enabled).on_toggle({
            let id = m.id.clone();
            move |_| Message::ToggleMod(id.clone())
        });

        let item_btn = button(
            row![
                text(&m.name).size(14).style(text_color),
                iced::widget::horizontal_space().width(Length::Fill),
                text(format!("v{}", m.version))
                    .size(12)
                    .style(muted_color),
                iced::widget::horizontal_space().width(12) // indent version from the scrollbar
            ]
            .align_items(Alignment::Center),
        )
        .padding(8)
        .width(Length::Fill)
        .on_press(Message::SelectMod(m.id.clone()))
        .style(iced::theme::Button::Custom(Box::new(ModListItemButton {
            is_selected,
        })));

        list_col = list_col.push(
            row![
                cb,
                item_btn
            ]
            .spacing(6)
            .align_items(Alignment::Center)
        );
    }

    let scroll = scrollable(list_col)
        .height(Length::Fill);

    let main_col = column![
        search_box,
        toggle_row,
        ordering_row,
        scroll
    ]
    .spacing(10);

    container(main_col)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(10)
        .style(iced::theme::Container::Custom(Box::new(PanelBg)))
        .into()
}
