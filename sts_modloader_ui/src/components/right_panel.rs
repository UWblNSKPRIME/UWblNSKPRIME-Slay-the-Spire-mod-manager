use std::collections::HashMap;
use iced::widget::{column, container, row, text, scrollable, vertical_space};
use iced::{Element, Length, Alignment, Color};
use crate::app::{AppState, Message};
use crate::styles::PanelBg;
use sts_modloader_core::{ModInfo, ModSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStatus {
    Satisfied,
    Disabled, // Mod is present but checkbox is unchecked
    Missing,  // Mod is completely absent from local/workshop directory
}

/// Validates dependencies for all selected mods
pub fn validate_mods_dependencies(
    all_mods: &[ModInfo],
) -> HashMap<String, Vec<(String, DependencyStatus)>> {
    let mut status_map = HashMap::new();

    for m in all_mods {
        let mut deps_status = vec![];
        for dep_id in &m.dependencies {
            // ModTheSpire is the loader itself, satisfied implicitly
            if dep_id == "ModTheSpire" {
                deps_status.push((dep_id.clone(), DependencyStatus::Satisfied));
                continue;
            }

            let found_mod = all_mods.iter().find(|x| x.id == *dep_id);
            match found_mod {
                None => {
                    deps_status.push((dep_id.clone(), DependencyStatus::Missing));
                }
                Some(dm) => {
                    if dm.enabled {
                        deps_status.push((dep_id.clone(), DependencyStatus::Satisfied));
                    } else {
                        deps_status.push((dep_id.clone(), DependencyStatus::Disabled));
                    }
                }
            }
        }
        status_map.insert(m.id.clone(), deps_status);
    }

    status_map
}

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let theme = state.active_theme();
    let text_color = crate::styles::get_text_color(&theme);
    let muted_color = crate::styles::get_muted_text_color(&theme);

    let selected_mod = state
        .selected_mod_id
        .as_ref()
        .and_then(|id| state.mods.iter().find(|m| m.id == *id));

    let content = match selected_mod {
        None => column![
            text("No Mod Selected").size(18).style(text_color),
            text("Select a mod from the list to view its details.")
                .size(14)
                .style(muted_color)
        ]
        .spacing(10)
        .align_items(Alignment::Center),
        Some(m) => {
            let title = text(format!("{} (v{})", m.name, m.version))
                .size(22)
                .style(text_color);

            let authors_str = if m.authors.is_empty() {
                "Unknown".to_string()
            } else {
                m.authors.join(", ")
            };
            let authors = text(format!("Authors: {}", authors_str))
                .size(14)
                .style(muted_color);

            let source_str = match m.source {
                ModSource::Local => "Local Folder",
                ModSource::Workshop => "Steam Workshop",
            };
            let source_info = text(format!("Source: {}", source_str))
                .size(14)
                .style(muted_color);

            let path_info = text(format!("Path: {}", m.jar_path.display()))
                .size(11)
                .style(muted_color);

            let sts_version = m.sts_version.as_deref().unwrap_or("Any");
            let mts_version = m.mts_version.as_deref().unwrap_or("Any");
            let versions_row = row![
                text(format!("Target STS: {}", sts_version))
                    .size(13)
                    .style(muted_color),
                text(" | ").size(13).style(muted_color),
                text(format!("MTS Version: {}", mts_version))
                    .size(13)
                    .style(muted_color),
            ]
            .align_items(Alignment::Center);

            let description = text(
                m.description
                    .as_deref()
                    .unwrap_or("No description provided."),
            )
            .size(14)
            .style(text_color);

            // Calculate dependencies validation
            let deps_map = validate_mods_dependencies(&state.mods);
            let deps_statuses = deps_map.get(&m.id);

            let mut deps_col = column![].spacing(6);
            if let Some(deps) = deps_statuses {
                if deps.is_empty() {
                    deps_col = deps_col.push(text("None").size(14).style(muted_color));
                } else {
                    for (dep_id, status) in deps {
                        let (status_text, status_color) = match status {
                            DependencyStatus::Satisfied => {
                                (" [Satisfied]", Color::from_rgb(0.1, 0.75, 0.4))
                            }
                            DependencyStatus::Disabled => {
                                (" [Disabled]", Color::from_rgb(0.9, 0.6, 0.1))
                            }
                            DependencyStatus::Missing => {
                                (" [Missing]", Color::from_rgb(0.9, 0.2, 0.2))
                            }
                        };
                        deps_col = deps_col.push(
                            row![
                                text(format!("- {}", dep_id)).size(14).style(text_color),
                                text(status_text).size(14).style(status_color)
                            ]
                            .spacing(6),
                        );
                    }
                }
            } else {
                deps_col = deps_col.push(text("None").size(14).style(muted_color));
            }

            column![
                title,
                authors,
                source_info,
                versions_row,
                vertical_space().height(10),
                iced::widget::horizontal_rule(1),
                vertical_space().height(10),
                description,
                vertical_space().height(15),
                text("Dependencies:").size(16).style(text_color),
                deps_col,
                vertical_space().height(15),
                iced::widget::horizontal_rule(1),
                vertical_space().height(10),
                path_info,
            ]
            .spacing(8)
        }
    };

    let scroll = scrollable(content).height(Length::Fill);

    container(scroll)
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .padding(15)
        .style(iced::theme::Container::Custom(Box::new(PanelBg)))
        .into()
}
