use serde_derive::{Deserialize, Serialize};
use crate::article_view::ArticleTheme;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleViewSettings {
    pub theme: ArticleTheme,
    pub allow_select: bool,
    pub use_system_font: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub font: Option<String>,
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
}