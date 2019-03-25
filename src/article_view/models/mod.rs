
#[derive(Clone, Debug)]
pub enum InternalView {
    View1,
    View2,
}

impl InternalView {
    pub fn to_str(&self) -> &str {
        match self {
            InternalView::View1 => "view_1",
            InternalView::View2 => "view_2",
        }
    }

    pub fn switch(&self) -> Self {
        match self {
            InternalView::View1 => InternalView::View2,
            InternalView::View2 => InternalView::View1,
        }
    }
}


#[derive(Clone, Debug)]
pub enum ArticleTheme {
    Default,
    Spring,
    Midnight,
    Parchment,
}

impl ArticleTheme {
    pub fn to_str(&self) -> &str {
        match self {
            ArticleTheme::Default => "theme default",
            ArticleTheme::Spring => "theme spring",
            ArticleTheme::Midnight => "theme midnight",
            ArticleTheme::Parchment => "theme parchment",
        }
    }
}