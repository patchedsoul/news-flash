mod models;

use failure::Error;
use failure::format_err;
use webkit2gtk::{
    WebContext,
    WebView,
    WebViewExt,
    WebViewExtManual,
    UserContentManager,
    LoadEvent,
};
use gtk::{
    StackExt,
};
use news_flash::models::{
    FatArticle,
    ArticleID,
};
use self::models::{
    InternalView,
    ArticleTheme,
};
use crate::Resources;
use crate::util::DateUtil;
use std::str;



#[derive(Clone, Debug)]
pub struct ArticleView {
    stack: gtk::Stack,
    internal_view: InternalView,
    theme: ArticleTheme,
}

impl ArticleView {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let stack : gtk::Stack = builder.get_object("article_view_stack").ok_or(format_err!("some err"))?;

        let internal_view = InternalView::View1;

        let mut article_view = ArticleView {
            stack: stack,
            internal_view: internal_view,
            theme: ArticleTheme::Default,
        };

        let _ = article_view.switch_view()?;
        Ok(article_view)
    }

    pub fn widget(&self) -> gtk::Stack {
        self.stack.clone()
    }

    pub fn show_article(&mut self, article: FatArticle) -> Result<(), Error> {
        let webview = self.switch_view()?;

        Ok(())
    }

    fn switch_view(&mut self) -> Result<WebView, Error> {
        let webview = Self::new_webview()?;
        self.internal_view = self.internal_view.switch();
        self.stack.add_named(&webview, self.internal_view.to_str());

        Ok(webview)
    }

    fn new_webview() -> Result<WebView, Error> {
        let context = WebContext::get_default().ok_or(format_err!("some err"))?;
        let content_manager = UserContentManager::new();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &content_manager);
        Ok(webview)
    }

    fn build_article(article: FatArticle, feed_name: String) -> Result<String, Error>  {
        let template_data = Resources::get("article_view/article.html").ok_or(format_err!("some err"))?;
        let template_str = str::from_utf8(template_data.as_ref())?;
        let mut template_string = template_str.to_owned();
        template_string.push_str(template_str);

        let mut author_date = String::new();
        let date = DateUtil::format(&article.date);
        if let Some(author) = &article.author {
            author_date.push_str(&format!("posted by: {}, {}", author, date));
        }
        else {
            author_date.push_str(&format!("{}", date));
        }

        if let Some(html) = article.html {
            template_string = template_string.replacen("$HTML", &html, 1);
        }
        
        template_string = template_string.replacen("$AUTHOR", &author_date, 1);

        if let Some(title) = article.title {
            template_string = template_string.replacen("$TITLE", &title, 1);
        }

        // $LARGESIZE


        Ok(template_string)
    }
}