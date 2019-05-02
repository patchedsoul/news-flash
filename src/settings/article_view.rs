use serde_derive::{Deserialize, Serialize};
use crate::article_view::ArticleTheme;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleViewSettings {
    theme: ArticleTheme,
    font_family: String,
}