mod models;

use failure::Error;
use failure::format_err;
use webkit2gtk::{
    WebView,
    WebViewExt,
    LoadEvent,
    Settings,
    SettingsExt,
};
use gtk::{
    StackExt,
    WidgetExt,
    ContainerExt,
    Continue,
};
use gdk::{
    EventMask,
};
use news_flash::models::{
    ArticleID,
    FatArticle,
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
    parent_overlay: gtk::Overlay,
    visible_article: Option<ArticleID>,
    internal_view: InternalView,
    theme: ArticleTheme,
}

impl ArticleView {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let overlay : gtk::Overlay = builder.get_object("parent_overlay").ok_or(format_err!("some err"))?;
        let stack : gtk::Stack = builder.get_object("article_view_stack").ok_or(format_err!("some err"))?;
        stack.set_visible_child_name("empty");

        let internal_view = InternalView::Empty;

        let article_view = ArticleView {
            stack: stack,
            parent_overlay: overlay,
            visible_article: None,
            internal_view: internal_view,
            theme: ArticleTheme::Default,
        };

        article_view.stack.show_all();
        Ok(article_view)
    }

    pub fn widget(&self) -> gtk::Overlay {
        self.parent_overlay.clone()
    }

    pub fn show_article(&mut self, article: FatArticle, feed_name: String) -> Result<(), Error> {
        let article_id = article.article_id.clone();
        let webview = self.switch_view()?;
        let html = self.build_article(article, feed_name)?;
        webview.load_html(&html, None);
        self.visible_article = Some(article_id);
        Ok(())
    }

    fn switch_view(&mut self) -> Result<WebView, Error> {
        let old_name = self.internal_view.clone();
        let stack_clone = self.stack.clone();

        let webview = Self::new_webview()?;
        self.internal_view = self.internal_view.switch();
        if let Some(new_name) = self.internal_view.to_str() {
            self.stack.add_named(&webview, new_name);
            self.stack.show_all();
            self.stack.set_visible_child_name(new_name);
        }

        // remove old view after crossfade animation
        gtk::timeout_add(150, move || {
            if let Some(old_name) = old_name.to_str() {
                if let Some(old_view) = stack_clone.get_child_by_name(&old_name) {
                    stack_clone.remove(&old_view);
                }
            }
            Continue(false)
        });

        Ok(webview)
    }

    fn new_webview() -> Result<WebView, Error> {
        let settings = Settings::new();
        settings.set_enable_accelerated_2d_canvas(true);
        settings.set_enable_html5_database(false);
        settings.set_enable_html5_local_storage(false);
        settings.set_enable_java(false);
        settings.set_enable_media_stream(false);
		settings.set_enable_page_cache(false);
		settings.set_enable_plugins(false);
		settings.set_enable_smooth_scrolling(true);
		settings.set_enable_javascript(false);
		settings.set_javascript_can_access_clipboard(false);
		settings.set_javascript_can_open_windows_automatically(false);
		settings.set_media_playback_requires_user_gesture(true);
        settings.set_user_agent_with_application_details("NewsFlash", None);

        let webview = WebView::new_with_settings(&settings);
		webview.set_events(EventMask::POINTER_MOTION_MASK.bits() as i32);
		webview.set_events(EventMask::SCROLL_MASK.bits() as i32);
		webview.set_events(EventMask::BUTTON_PRESS_MASK.bits() as i32);
		webview.set_events(EventMask::BUTTON_RELEASE_MASK.bits() as i32);
		webview.set_events(EventMask::KEY_PRESS_MASK.bits() as i32);
		//webview.load_changed.connect(open_link);
		//webview.context_menu.connect(onContextMenu);
		//webview.mouse_target_changed.connect(onMouseOver);
		//webview.button_press_event.connect(onClick);
		//webview.button_release_event.connect(onRelease);
		//webview.motion_notify_event.connect(onMouseMotion);
		//webview.enter_fullscreen.connect(enterFullscreenVideo);
		//webview.leave_fullscreen.connect(leaveFullscreenVideo);
		//webview.scroll_event.connect(onScroll);
		//webview.key_press_event.connect(onKeyPress);
		//webview.web_process_terminated.connect(onCrash);
		//webview.notify["estimated-load-progress"].connect(printProgress);
		//webview.decide_policy.connect(decidePolicy);
		//webview.set_background_color(m_color);

        Ok(webview)
    }

    fn build_article(&self, article: FatArticle, feed_name: String) -> Result<String, Error>  {
        let template_data = Resources::get("article_view/article.html").ok_or(format_err!("some err"))?;
        let template_str = str::from_utf8(template_data.as_ref())?;
        let mut template_string = template_str.to_owned();
        //template_string.push_str(template_str);

        let css_data = Resources::get("article_view/style.css").ok_or(format_err!("some err"))?;
        let css_string = str::from_utf8(css_data.as_ref())?;

        // FIXME
        let unselectable = true;
        let font_size = 12;
        let font_family = "Cantarell";

        let mut author_date = String::new();
        let date = DateUtil::format(&article.date);
        if let Some(author) = &article.author {
            author_date.push_str(&format!("posted by: {}, {}", author, date));
        }
        else {
            author_date.push_str(&format!("{}", date));
        }

        // $HTML
        if let Some(html) = article.html {
            template_string = template_string.replacen("$HTML", &html, 1);
        }

        // $UNSELECTABLE
        if unselectable {
            template_string = template_string.replacen("$UNSELECTABLE", "unselectable", 1);
        }
        else {
            template_string = template_string.replacen("$UNSELECTABLE", "", 1);
        }
        
        // $AUTHOR / $DATE
        template_string = template_string.replacen("$AUTHOR", &author_date, 1);

        // $SMALLSIZE x2
        let small_size = font_size - 2;
        template_string = template_string.replacen("$SMALLSIZE", &format!("{}", small_size), 2);

        // $TITLE
        if let Some(title) = article.title {
            template_string = template_string.replacen("$TITLE", &title, 1);
        }

        // $LARGESIZE
        let large_size = font_size * 2;
        template_string = template_string.replacen("$LARGESIZE", &format!("{}", large_size), 1);

        // $URL
        if let Some(article_url) = article.url {
            template_string = template_string.replacen("$URL", article_url.get().as_str(), 1);
        }

        // $FEED
        template_string = template_string.replacen("$FEED", &feed_name, 1);

        // $THEME
        template_string = template_string.replacen("$THEME", self.theme.to_str(), 1);

        // $FONTFAMILY
        template_string = template_string.replacen("$FONTFAMILY", font_family, 1);

        // $FONTSIZE
        template_string = template_string.replacen("$FONTSIZE", &format!("{}", font_size), 1);

        // $CSS
        template_string = template_string.replacen("$CSS", &css_string, 1);

        Ok(template_string)
    }
}