use gtk::{Settings, SettingsExt};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum InternalState {
    Empty,
    Crash,
    View1,
    View2,
}

impl InternalState {
    pub fn to_str(&self) -> Option<&str> {
        match self {
            InternalState::Empty => None,
            InternalState::Crash => None,
            InternalState::View1 => Some("view_1"),
            InternalState::View2 => Some("view_2"),
        }
    }

    pub fn switch(&self) -> Self {
        match self {
            InternalState::View1 => InternalState::View2,
            InternalState::View2 => InternalState::View1,
            InternalState::Empty => InternalState::View1,
            InternalState::Crash => InternalState::View1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ArticleTheme {
    Default,
    Spring,
    Midnight,
    Parchment,
}

impl ArticleTheme {
    pub fn to_str(&self) -> &str {
        match self {
            ArticleTheme::Default => {
                if let Some(settings) = Settings::get_default() {
                    if settings.get_property_gtk_application_prefer_dark_theme() {
                        "theme dark"
                    } else {
                        "theme default"
                    }
                } else {
                    "theme default"
                }
            }
            ArticleTheme::Spring => "theme spring",
            ArticleTheme::Midnight => "theme midnight",
            ArticleTheme::Parchment => "theme parchment",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ArticleTheme::Default => "Default",
            ArticleTheme::Spring => "Spring",
            ArticleTheme::Midnight => "Midnight",
            ArticleTheme::Parchment => "Parchment",
        }
    }
}
