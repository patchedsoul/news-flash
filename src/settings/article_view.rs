use crate::article_view::ArticleTheme;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleViewSettings {
    pub theme: ArticleTheme,
    pub allow_select: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub font: Option<String>,
}

impl Default for ArticleViewSettings {
    fn default() -> Self {
        ArticleViewSettings {
            theme: ArticleTheme::Default,
            allow_select: false,
            font: None,
        }
    }
}
