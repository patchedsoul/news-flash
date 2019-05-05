use serde_derive::{Deserialize, Serialize};
use crate::article_view::ArticleTheme;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleViewSettings {
    theme: ArticleTheme,
    allow_select: bool,
    use_system_font: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    font: Option<String>,
}

impl ArticleViewSettings {
    pub fn default() -> Self {
        ArticleViewSettings {
            theme: ArticleTheme::Default,
            allow_select: false,
            use_system_font: true,
            font: None,
        }
    }

    pub fn get_theme(&self) -> ArticleTheme {
        self.theme.clone()
    }

    pub fn get_allow_select(&self) -> bool {
        self.allow_select
    }

    pub fn font(&self) -> Option<String> {
        self.font.clone()
    }
}