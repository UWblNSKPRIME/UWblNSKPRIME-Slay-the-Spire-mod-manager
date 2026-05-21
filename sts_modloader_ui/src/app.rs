use std::path::PathBuf;
use iced::{Application, Command, Element, Theme, Length, Alignment};
use iced::widget::{column, container, row, text, button, scrollable, vertical_space, text_input};
use sts_modloader_core::{AppConfig, ModInfo};
use sts_modloader_fs::runner::LaunchError;
use crate::styles;
use crate::components;

#[derive(Debug, Clone)]
pub enum Message {
    // Application Initialization
    Init,
    ConfigLoaded(Result<AppConfig, String>),
    
    // Slay the Spire Path Management
    AutoDetectPath,
    PathDetected(Option<PathBuf>),
    BrowsePath,
    PathSelected(PathBuf),
    
    // Mod Scan Operations
    ScanMods,
    ScanFinished(Result<Vec<ModInfo>, String>),
    
    // Active Mod Management
    SelectMod(String),        // Highlights a mod for the Right Panel (ID)
    ToggleMod(String),        // Checks/Unchecks a mod in the List (ID)
    ToggleAllMods(bool),      // Enables/Disables all currently filtered mods
    SearchQueryChanged(String),
    
    // Profiles
    SelectProfile(String),
    OpenNewProfileDialog,     // Toggles text input field display
    ProfileNameInput(String),
    CreateNewProfile,
    DeleteActiveProfile,
    
    // Clipboard Export/Import
    ExportMods,
    ImportMods,
    ClipboardImported(Option<String>),
    
    // Launch Operations
    LaunchGame,
    GameExited(Result<(), LaunchError>),
    ToggleDebug(bool),
    
    // Themes selection
    SelectTheme(String),

    // Custom Export Success Modal
    CloseExportModal,
    SaveExportToFile(String),

    // Custom Import Modal
    OpenImportModal,
    CloseImportModal,
    ImportTextInputChanged(String),
    ToggleImportCreateNewProfile(bool),
    ImportProfileNameChanged(String),
    ImportBrowseFile,
    DoImport,
    
    // Modal controls
    DismissError,
}

pub struct AppState {
    pub config: AppConfig,
    pub mods: Vec<ModInfo>,
    
    // UI Filters and Selectors
    pub selected_mod_id: Option<String>,
    pub search_query: String,
    
    // Modals & Overlay States
    pub error_modal: Option<String>,
    pub show_profile_input: bool,
    pub new_profile_name: String,
    pub is_loading: bool,
    pub is_game_running: bool,

    // Export/Import modals
    pub export_success_modlist: Option<String>,
    pub import_modal_open: bool,
    pub import_text_input: String,
    pub import_create_new_profile: bool,
    pub import_profile_name: String,
}

impl AppState {
    pub fn is_setup_required(&self) -> bool {
        self.config.sts_path.is_none()
    }

    pub fn active_theme(&self) -> Theme {
        if self.config.theme.as_deref() == Some("Light") {
            Theme::Light
        } else {
            Theme::Dark
        }
    }

    fn apply_profile_to_mods(&mut self) {
        if let Some(ref profile_name) = self.config.active_profile {
            if let Some(profile) = self.config.profiles.iter().find(|p| p.name == *profile_name) {
                for m in &mut self.mods {
                    m.enabled = profile.enabled_mods.contains(&m.id);
                }
            }
        }
    }

    fn save_mods_to_active_profile(&mut self) {
        if let Some(ref profile_name) = self.config.active_profile {
            if let Some(profile) = self.config.profiles.iter_mut().find(|p| p.name == *profile_name) {
                profile.enabled_mods = self.mods.iter()
                    .filter(|m| m.enabled)
                    .map(|m| m.id.clone())
                    .collect();
                let _ = sts_modloader_core::save_config(&self.config);
            }
        }
    }
}

impl Application for AppState {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = AppState {
            config: AppConfig::default(),
            mods: vec![],
            selected_mod_id: None,
            search_query: String::new(),
            error_modal: None,
            show_profile_input: false,
            new_profile_name: String::new(),
            is_loading: false,
            is_game_running: false,
            export_success_modlist: None,
            import_modal_open: false,
            import_text_input: String::new(),
            import_create_new_profile: false,
            import_profile_name: String::new(),
        };
        (app, Command::perform(async {}, |_| Message::Init))
    }

    fn title(&self) -> String {
        "Slay the Spire Mod Loader".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Init => {
                self.is_loading = true;
                Command::perform(
                    async { sts_modloader_core::load_config().map_err(|e| e.to_string()) },
                    Message::ConfigLoaded,
                )
            }
            Message::ConfigLoaded(res) => {
                self.is_loading = false;
                match res {
                    Ok(config) => {
                        self.config = config;
                        // Ensure there is at least a Default profile
                        if self.config.profiles.is_empty() {
                            self.config.profiles.push(sts_modloader_core::config::Profile {
                                name: "Default".to_string(),
                                enabled_mods: vec![],
                            });
                            self.config.active_profile = Some("Default".to_string());
                        }
                        if self.config.sts_path.is_some() {
                            return Command::perform(async {}, |_| Message::ScanMods);
                        }
                    }
                    Err(e) => {
                        self.error_modal = Some(e);
                    }
                }
                Command::none()
            }
            Message::AutoDetectPath => {
                self.is_loading = true;
                Command::perform(
                    async { sts_modloader_fs::steam::auto_detect_sts_path() },
                    Message::PathDetected,
                )
            }
            Message::PathDetected(opt_path) => {
                self.is_loading = false;
                if let Some(path) = opt_path {
                    self.config.sts_path = Some(path);
                    let _ = sts_modloader_core::save_config(&self.config);
                    Command::perform(async {}, |_| Message::ScanMods)
                } else {
                    self.error_modal = Some("Could not automatically detect Slay the Spire path. Please browse manually.".to_string());
                    Command::none()
                }
            }
            Message::BrowsePath => {
                Command::perform(
                    async {
                        tokio::task::spawn_blocking(|| {
                            rfd::FileDialog::new()
                                .set_title("Select Slay the Spire Installation Directory")
                                .pick_folder()
                        })
                        .await
                        .ok()
                        .flatten()
                    },
                    |opt_path| {
                        if let Some(path) = opt_path {
                            Message::PathSelected(path)
                        } else {
                            Message::DismissError // No-op message
                        }
                    },
                )
            }
            Message::PathSelected(path) => {
                if path.join("desktop-1.0.jar").exists() {
                    self.config.sts_path = Some(path);
                    let _ = sts_modloader_core::save_config(&self.config);
                    Command::perform(async {}, |_| Message::ScanMods)
                } else {
                    self.error_modal = Some("Selected directory does not appear to be Slay the Spire installation folder (desktop-1.0.jar is missing).".to_string());
                    Command::none()
                }
            }
            Message::ScanMods => {
                if let Some(ref sts_path) = self.config.sts_path {
                    let path = sts_path.clone();
                    self.is_loading = true;
                    Command::perform(
                        async move { sts_modloader_fs::scanner::scan_mods(&path).await },
                        Message::ScanFinished,
                    )
                } else {
                    Command::none()
                }
            }
            Message::ScanFinished(res) => {
                self.is_loading = false;
                match res {
                    Ok(mods) => {
                        self.mods = mods;
                        self.apply_profile_to_mods();
                        if self.selected_mod_id.is_none() && !self.mods.is_empty() {
                            self.selected_mod_id = Some(self.mods[0].id.clone());
                        }
                    }
                    Err(e) => {
                        self.error_modal = Some(e);
                    }
                }
                Command::none()
            }
            Message::SelectMod(id) => {
                self.selected_mod_id = Some(id);
                Command::none()
            }
            Message::ToggleMod(id) => {
                if let Some(m) = self.mods.iter_mut().find(|x| x.id == id) {
                    m.enabled = !m.enabled;
                }
                self.save_mods_to_active_profile();
                Command::none()
            }
            Message::ToggleAllMods(enable) => {
                let query = self.search_query.to_lowercase();
                for m in &mut self.mods {
                    let matches_search = m.name.to_lowercase().contains(&query) || m.id.to_lowercase().contains(&query);
                    if matches_search {
                        m.enabled = enable;
                    }
                }
                self.save_mods_to_active_profile();
                Command::none()
            }
            Message::SearchQueryChanged(q) => {
                self.search_query = q;
                Command::none()
            }
            Message::SelectProfile(name) => {
                self.config.active_profile = Some(name);
                self.apply_profile_to_mods();
                let _ = sts_modloader_core::save_config(&self.config);
                Command::none()
            }
            Message::OpenNewProfileDialog => {
                self.show_profile_input = !self.show_profile_input;
                Command::none()
            }
            Message::ProfileNameInput(name) => {
                self.new_profile_name = name;
                Command::none()
            }
            Message::CreateNewProfile => {
                let name = self.new_profile_name.clone();
                let enabled_ids: Vec<String> = self
                    .mods
                    .iter()
                    .filter(|m| m.enabled)
                    .map(|m| m.id.clone())
                    .collect();
                match sts_modloader_profile::manager::create_profile(
                    &mut self.config,
                    name,
                    enabled_ids,
                ) {
                    Ok(_) => {
                        self.show_profile_input = false;
                        self.new_profile_name.clear();
                        let _ = sts_modloader_core::save_config(&self.config);
                        self.apply_profile_to_mods();
                    }
                    Err(e) => {
                        self.error_modal = Some(e.to_string());
                    }
                }
                Command::none()
            }
            Message::DeleteActiveProfile => {
                if let Some(active) = self.config.active_profile.clone() {
                    match sts_modloader_profile::manager::delete_profile(&mut self.config, &active) {
                        Ok(_) => {
                            let _ = sts_modloader_core::save_config(&self.config);
                            self.apply_profile_to_mods();
                        }
                        Err(e) => {
                            self.error_modal = Some(e.to_string());
                        }
                    }
                }
                Command::none()
            }
            Message::LaunchGame => {
                if let Some(ref sts_path) = self.config.sts_path {
                    let enabled_mods: Vec<String> = self
                        .mods
                        .iter()
                        .filter(|m| m.enabled)
                        .map(|m| m.id.clone())
                        .collect();
                    let debug_mode = self.config.debug_mode;
                    let sts_path = sts_path.clone();
                    self.is_game_running = true;
                    Command::perform(
                        async move {
                            sts_modloader_fs::runner::launch_game(
                                &sts_path,
                                &enabled_mods,
                                debug_mode,
                            )
                            .await
                        },
                        Message::GameExited,
                    )
                } else {
                    Command::none()
                }
            }
            Message::GameExited(result) => {
                self.is_game_running = false;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        self.error_modal = Some(e.to_string());
                    }
                }
                Command::none()
            }
            Message::ToggleDebug(val) => {
                self.config.debug_mode = val;
                let _ = sts_modloader_core::save_config(&self.config);
                Command::none()
            }
            Message::ExportMods => {
                let enabled_ids: Vec<String> = self
                    .mods
                    .iter()
                    .filter(|m| m.enabled)
                    .map(|m| m.id.clone())
                    .collect();
                let csv = enabled_ids.join(",");
                self.export_success_modlist = Some(csv.clone());
                iced::clipboard::write(csv)
            }
            Message::ImportMods => {
                iced::clipboard::read(Message::ClipboardImported)
            }
            Message::ClipboardImported(content) => {
                if let Some(text) = content {
                    let imported_tokens: std::collections::HashSet<String> = text
                        .split(|c| c == ',' || c == ';' || c == '\n' || c == '\r')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                    for m in &mut self.mods {
                        m.enabled = imported_tokens.contains(&m.id.to_lowercase())
                            || imported_tokens.contains(&m.name.to_lowercase());
                    }
                    self.save_mods_to_active_profile();
                }
                Command::none()
            }
            Message::SelectTheme(theme_name) => {
                self.config.theme = Some(theme_name);
                let _ = sts_modloader_core::save_config(&self.config);
                Command::none()
            }
            Message::CloseExportModal => {
                self.export_success_modlist = None;
                Command::none()
            }
            Message::SaveExportToFile(content) => {
                Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Save Modlist")
                                .add_filter("Text Files", &["txt"])
                                .save_file()
                            {
                                let _ = std::fs::write(path, content);
                            }
                        })
                        .await
                        .ok();
                    },
                    |_| Message::CloseExportModal
                )
            }
            Message::OpenImportModal => {
                self.import_modal_open = true;
                self.import_text_input.clear();
                self.import_create_new_profile = false;
                self.import_profile_name.clear();
                Command::none()
            }
            Message::CloseImportModal => {
                self.import_modal_open = false;
                Command::none()
            }
            Message::ImportTextInputChanged(text) => {
                self.import_text_input = text;
                Command::none()
            }
            Message::ToggleImportCreateNewProfile(val) => {
                self.import_create_new_profile = val;
                Command::none()
            }
            Message::ImportProfileNameChanged(name) => {
                self.import_profile_name = name;
                Command::none()
            }
            Message::ImportBrowseFile => {
                Command::perform(
                    async {
                        tokio::task::spawn_blocking(|| {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select Modlist File")
                                .add_filter("Text Files", &["txt"])
                                .pick_file()
                            {
                                std::fs::read_to_string(path).ok()
                            } else {
                                None
                            }
                        })
                        .await
                        .ok()
                        .flatten()
                    },
                    |opt_content| {
                        if let Some(content) = opt_content {
                            Message::ImportTextInputChanged(content)
                        } else {
                            Message::DismissError // No-op
                        }
                    }
                )
            }
            Message::DoImport => {
                let imported_tokens: std::collections::HashSet<String> = self.import_text_input
                    .split(|c| c == ',' || c == ';' || c == '\n' || c == '\r')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();

                if self.import_create_new_profile {
                    let name = self.import_profile_name.trim().to_string();
                    if name.is_empty() {
                        self.error_modal = Some("Profile name cannot be empty.".to_string());
                        return Command::none();
                    }

                    let enabled_ids: Vec<String> = self.mods.iter()
                        .filter(|m| imported_tokens.contains(&m.id.to_lowercase()) || imported_tokens.contains(&m.name.to_lowercase()))
                        .map(|m| m.id.clone())
                        .collect();

                    match sts_modloader_profile::manager::create_profile(
                        &mut self.config,
                        name,
                        enabled_ids,
                    ) {
                        Ok(_) => {
                            self.import_modal_open = false;
                            let _ = sts_modloader_core::save_config(&self.config);
                            self.apply_profile_to_mods();
                        }
                        Err(e) => {
                            self.error_modal = Some(e.to_string());
                        }
                    }
                } else {
                    for m in &mut self.mods {
                        m.enabled = imported_tokens.contains(&m.id.to_lowercase())
                            || imported_tokens.contains(&m.name.to_lowercase());
                    }
                    self.save_mods_to_active_profile();
                    self.import_modal_open = false;
                }
                Command::none()
            }
            Message::DismissError => {
                self.error_modal = None;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let theme = self.theme();
        let text_color = styles::get_text_color(&theme);
        let muted_color = styles::get_muted_text_color(&theme);

        let main_content = if self.is_setup_required() {
            components::setup_screen::view()
        } else {
            let controls = components::control_block::view(self);
            
            let panels = row![
                components::left_panel::view(self),
                components::right_panel::view(self)
            ]
            .spacing(15)
            .height(Length::Fill);

            let bottom = components::bottom_bar::view(self);

            column![
                controls,
                panels,
                bottom
            ]
            .spacing(15)
            .padding(15)
            .into()
        };

        let final_view = if let Some(error) = &self.error_modal {
            let modal_card = container(
                column![
                    text("Error Occurred")
                        .size(20)
                        .style(iced::Color::from_rgb(0.9, 0.3, 0.3)),
                    vertical_space().height(10),
                    scrollable(
                        text(error)
                            .size(14)
                            .style(text_color)
                    )
                    .height(Length::Shrink),
                    vertical_space().height(20),
                    button(text("Dismiss").horizontal_alignment(iced::alignment::Horizontal::Center))
                        .padding(8)
                        .width(100)
                        .on_press(Message::DismissError)
                        .style(iced::theme::Button::Custom(Box::new(styles::StopButton)))
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(500)
            .max_height(400)
            .padding(20)
            .style(iced::theme::Container::Custom(Box::new(styles::PanelBg)));

            container(modal_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(styles::MainBg)))
                .into()
        } else if let Some(export_content) = &self.export_success_modlist {
            let title_row = row![
                text("Success!").size(20).style(text_color),
                iced::widget::horizontal_space().width(Length::Fill),
                button(text("X").size(14))
                    .padding(4)
                    .on_press(Message::CloseExportModal)
                    .style(iced::theme::Button::Custom(Box::new(styles::BorderButton)))
            ]
            .align_items(Alignment::Center);

            let info_icon = text("?")
                .size(28)
                .style(iced::Color::from_rgb(0.23, 0.51, 0.96));

            let modal_card = container(
                column![
                    title_row,
                    vertical_space().height(15),
                    row![
                        info_icon,
                        iced::widget::horizontal_space().width(15),
                        text("Modlist key has been copied to clipboard!").size(14).style(text_color)
                    ]
                    .align_items(Alignment::Center),
                    vertical_space().height(25),
                    row![
                        button(text("Save to file").horizontal_alignment(iced::alignment::Horizontal::Center))
                            .padding(8)
                            .width(120)
                            .on_press(Message::SaveExportToFile(export_content.clone()))
                            .style(iced::theme::Button::Custom(Box::new(styles::BorderButton))),
                        iced::widget::horizontal_space().width(10),
                        button(text("Close").horizontal_alignment(iced::alignment::Horizontal::Center))
                            .padding(8)
                            .width(120)
                            .on_press(Message::CloseExportModal)
                            .style(iced::theme::Button::Custom(Box::new(styles::AccentButton)))
                    ]
                    .align_items(Alignment::Center)
                ]
                .spacing(10)
            )
            .width(420)
            .padding(20)
            .style(iced::theme::Container::Custom(Box::new(styles::PanelBg)));

            container(modal_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(styles::MainBg)))
                .into()
        } else if self.import_modal_open {
            let title_row = row![
                text("📁 Import").size(20).style(text_color),
                iced::widget::horizontal_space().width(Length::Fill),
                button(text("X").size(14))
                    .padding(4)
                    .on_press(Message::CloseImportModal)
                    .style(iced::theme::Button::Custom(Box::new(styles::BorderButton)))
            ]
            .align_items(Alignment::Center);

            let label = text("Enter modlist key:").size(14).style(muted_color);
            let key_input = text_input("Paste modlist key here...", &self.import_text_input)
                .on_input(Message::ImportTextInputChanged)
                .padding(10)
                .style(iced::theme::TextInput::Custom(Box::new(styles::TextInputStyle)));

            let or_label = text("OR").size(12).style(muted_color);
            let browse_btn = button(text("Import from file").horizontal_alignment(iced::alignment::Horizontal::Center))
                .padding(8)
                .width(Length::Fill)
                .on_press(Message::ImportBrowseFile)
                .style(iced::theme::Button::Custom(Box::new(styles::BorderButton)));

            let create_profile_checkbox = iced::widget::checkbox("Create as new profile", self.import_create_new_profile)
                .on_toggle(Message::ToggleImportCreateNewProfile)
                .text_size(14);

            let mut import_content = column![
                title_row,
                vertical_space().height(10),
                label,
                key_input,
                vertical_space().height(10),
                or_label,
                browse_btn,
                vertical_space().height(10),
                create_profile_checkbox,
            ]
            .spacing(10);

            if self.import_create_new_profile {
                let name_input = text_input("Profile name...", &self.import_profile_name)
                    .on_input(Message::ImportProfileNameChanged)
                    .padding(8)
                    .style(iced::theme::TextInput::Custom(Box::new(styles::TextInputStyle)));
                import_content = import_content.push(name_input);
            }

            let import_btn = button(text("Import").horizontal_alignment(iced::alignment::Horizontal::Center))
                .padding(8)
                .width(120)
                .on_press(Message::DoImport)
                .style(iced::theme::Button::Custom(Box::new(styles::AccentButton)));

            import_content = import_content.push(vertical_space().height(15));
            import_content = import_content.push(
                row![
                    iced::widget::horizontal_space().width(Length::Fill),
                    import_btn,
                    iced::widget::horizontal_space().width(Length::Fill)
                ]
            );

            let modal_card = container(import_content)
                .width(500)
                .padding(20)
                .style(iced::theme::Container::Custom(Box::new(styles::PanelBg)));

            container(modal_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(styles::MainBg)))
                .into()
        } else {
            main_content
        };

        container(final_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(styles::MainBg)))
            .into()
    }

    fn theme(&self) -> Self::Theme {
        self.active_theme()
    }
}
