use iced::widget::{button, container, text_input};
use iced::{Background, Color};

// Dark Palette Constants
pub const COLOR_PRIMARY_BG_DARK: Color = Color::from_rgb(0.094, 0.094, 0.141); // #181824
pub const COLOR_PANEL_BG_DARK: Color = Color::from_rgb(0.125, 0.125, 0.184); // #20202F
pub const COLOR_TEXT_FG_DARK: Color = Color::from_rgb(0.886, 0.910, 0.941); // #E2E8F0
pub const COLOR_MUTED_TEXT_DARK: Color = Color::from_rgb(0.580, 0.639, 0.722); // #94A3B8
pub const COLOR_ACCENT_DARK: Color = Color::from_rgb(0.388, 0.400, 0.945); // #6366F1
pub const COLOR_BORDER_DARK: Color = Color::from_rgb(0.200, 0.255, 0.333); // #334155

// Light Palette Constants
pub const COLOR_PRIMARY_BG_LIGHT: Color = Color::from_rgb(0.945, 0.961, 0.976); // #F1F5F9 (Clean cool grey/white)
pub const COLOR_PANEL_BG_LIGHT: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF (Pure white card surfaces)
pub const COLOR_TEXT_FG_LIGHT: Color = Color::from_rgb(0.059, 0.090, 0.165); // #0F172A (Deep slate black)
pub const COLOR_MUTED_TEXT_LIGHT: Color = Color::from_rgb(0.392, 0.455, 0.545); // #64748B (Slate gray)
pub const COLOR_ACCENT_LIGHT: Color = Color::from_rgb(0.231, 0.510, 0.965); // #3B82F6 (Vibrant blue)
pub const COLOR_BORDER_LIGHT: Color = Color::from_rgb(0.796, 0.835, 0.882); // #CBD5E1 (Light gray border)

// Common button colors
pub const COLOR_PLAY_NORMAL: Color = Color::from_rgb(0.063, 0.725, 0.506); // #10B981
pub const COLOR_PLAY_RUNNING: Color = Color::from_rgb(0.937, 0.267, 0.267); // #EF4444

pub fn get_text_color(theme: &iced::Theme) -> Color {
    if matches!(theme, iced::Theme::Light) {
        COLOR_TEXT_FG_LIGHT
    } else {
        COLOR_TEXT_FG_DARK
    }
}

pub fn get_muted_text_color(theme: &iced::Theme) -> Color {
    if matches!(theme, iced::Theme::Light) {
        COLOR_MUTED_TEXT_LIGHT
    } else {
        COLOR_MUTED_TEXT_DARK
    }
}

pub struct MainBg;
impl container::StyleSheet for MainBg {
    type Style = iced::Theme;
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        container::Appearance {
            text_color: Some(if is_light { COLOR_TEXT_FG_LIGHT } else { COLOR_TEXT_FG_DARK }),
            background: Some(Background::Color(if is_light { COLOR_PRIMARY_BG_LIGHT } else { COLOR_PRIMARY_BG_DARK })),
            ..Default::default()
        }
    }
}

pub struct PanelBg;
impl container::StyleSheet for PanelBg {
    type Style = iced::Theme;
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        container::Appearance {
            text_color: Some(if is_light { COLOR_TEXT_FG_LIGHT } else { COLOR_TEXT_FG_DARK }),
            background: Some(Background::Color(if is_light { COLOR_PANEL_BG_LIGHT } else { COLOR_PANEL_BG_DARK })),
            border: iced::Border {
                color: if is_light { COLOR_BORDER_LIGHT } else { COLOR_BORDER_DARK },
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
}

pub struct AccentButton;
impl button::StyleSheet for AccentButton {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        button::Appearance {
            background: Some(Background::Color(if is_light { COLOR_ACCENT_LIGHT } else { COLOR_ACCENT_DARK })),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: if is_light { Color::WHITE } else { COLOR_TEXT_FG_DARK },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);
        let is_light = matches!(style, iced::Theme::Light);
        active.background = Some(Background::Color(if is_light {
            Color::from_rgb(0.18, 0.45, 0.88)
        } else {
            Color::from_rgb(0.45, 0.47, 0.98)
        }));
        active
    }
}

pub struct PlayButton;
impl button::StyleSheet for PlayButton {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        button::Appearance {
            background: Some(Background::Color(COLOR_PLAY_NORMAL)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: if is_light { Color::WHITE } else { COLOR_TEXT_FG_DARK },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb(0.10, 0.80, 0.55)));
        active
    }
}

pub struct StopButton;
impl button::StyleSheet for StopButton {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        button::Appearance {
            background: Some(Background::Color(COLOR_PLAY_RUNNING)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: if is_light { Color::WHITE } else { COLOR_TEXT_FG_DARK },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb(0.98, 0.35, 0.35)));
        active
    }
}

pub struct BorderButton;
impl button::StyleSheet for BorderButton {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        button::Appearance {
            background: None,
            border: iced::Border {
                color: if is_light { COLOR_BORDER_LIGHT } else { COLOR_BORDER_DARK },
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: if is_light { COLOR_TEXT_FG_LIGHT } else { COLOR_TEXT_FG_DARK },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);
        let is_light = matches!(style, iced::Theme::Light);
        active.background = Some(Background::Color(if is_light { COLOR_PRIMARY_BG_LIGHT } else { COLOR_PANEL_BG_DARK }));
        active
    }
}

pub struct TextInputStyle;
impl text_input::StyleSheet for TextInputStyle {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        text_input::Appearance {
            background: Background::Color(if is_light { COLOR_PANEL_BG_LIGHT } else { COLOR_PANEL_BG_DARK }),
            border: iced::Border {
                color: if is_light { COLOR_BORDER_LIGHT } else { COLOR_BORDER_DARK },
                width: 1.0,
                radius: 4.0.into(),
            },
            icon_color: if is_light { COLOR_MUTED_TEXT_LIGHT } else { COLOR_MUTED_TEXT_DARK },
        }
    }
    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let is_light = matches!(style, iced::Theme::Light);
        text_input::Appearance {
            background: Background::Color(if is_light { COLOR_PANEL_BG_LIGHT } else { COLOR_PANEL_BG_DARK }),
            border: iced::Border {
                color: if is_light { COLOR_ACCENT_LIGHT } else { COLOR_ACCENT_DARK },
                width: 1.0,
                radius: 4.0.into(),
            },
            icon_color: if is_light { COLOR_TEXT_FG_LIGHT } else { COLOR_TEXT_FG_DARK },
        }
    }
    fn placeholder_color(&self, style: &Self::Style) -> Color {
        let is_light = matches!(style, iced::Theme::Light);
        if is_light { COLOR_MUTED_TEXT_LIGHT } else { COLOR_MUTED_TEXT_DARK }
    }
    fn value_color(&self, style: &Self::Style) -> Color {
        let is_light = matches!(style, iced::Theme::Light);
        if is_light { COLOR_TEXT_FG_LIGHT } else { COLOR_TEXT_FG_DARK }
    }
    fn selection_color(&self, style: &Self::Style) -> Color {
        let is_light = matches!(style, iced::Theme::Light);
        if is_light { COLOR_ACCENT_LIGHT } else { COLOR_ACCENT_DARK }
    }
    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }
    fn disabled_color(&self, style: &Self::Style) -> Color {
        let is_light = matches!(style, iced::Theme::Light);
        if is_light { COLOR_MUTED_TEXT_LIGHT } else { COLOR_MUTED_TEXT_DARK }
    }
}
