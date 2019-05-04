use serde_derive::{Deserialize, Serialize};
use crate::article_view::ArticleTheme;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleViewSettings {
    theme: ArticleTheme,
    use_system_font: bool,
    font: Option<String>,
}

impl ArticleViewSettings {
    pub fn default() -> Self {
        ArticleViewSettings {
            theme: ArticleTheme::Default,
            use_system_font: true,
            font: None,
        }
    }
}