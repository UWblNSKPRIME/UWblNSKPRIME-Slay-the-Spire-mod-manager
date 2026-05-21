#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Application, Settings, Size};
use sts_modloader_ui::app::AppState;

#[tokio::main]
async fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = Size::new(1024.0, 600.0);
    settings.window.resizable = true;
    
    #[cfg(target_os = "windows")]
    {
        settings.default_font = iced::Font::with_name("Microsoft YaHei");
    }
    #[cfg(target_os = "macos")]
    {
        settings.default_font = iced::Font::with_name("PingFang SC");
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        settings.default_font = iced::Font::with_name("Noto Sans CJK SC");
    }
    
    AppState::run(settings)
}
