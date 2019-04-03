mod models;
mod url_overlay;

use self::url_overlay::UrlOverlay;
use failure::Error;
use failure::format_err;
use webkit2gtk::{
    WebView,
    WebViewExt,
    LoadEvent,
    Settings,
    SettingsExt,
    HitTestResultExt,
};
use gtk::{
    StackExt,
    WidgetExt,
    ContainerExt,
    Continue,
    OverlayExt,
    Inhibit,
};
use gdk::{
    EventMask,
    ScrollDirection,
    ModifierType,
    enums::key::KP_0,
    enums::key::KP_Add as KP_ADD,
    enums::key::KP_Subtract as KP_SUBTRACT,
};
use glib::{
    translate::ToGlib,
};
use news_flash::models::{
    ArticleID,
    FatArticle,
};
use self::models::{
    InternalView,
    ArticleTheme,
};
use std::rc::Rc;
use std::cell::RefCell;
use crate::Resources;
use crate::util::GtkHandle;
use crate::gtk_handle;
use crate::util::{
    DateUtil,
    GtkUtil,
};
use std::str;



#[derive(Clone, Debug)]
pub struct ArticleView {
    stack: gtk::Stack,
    parent_overlay: gtk::Overlay,
    visible_article: Option<ArticleID>,
    internal_view: InternalView,
    theme: ArticleTheme,
    load_changed_signal: Option<u64>,
    mouse_over_signal: Option<u64>,
    scroll_signal: Option<u64>,
    key_press_signal: Option<u64>,
    url_overlay: GtkHandle<UrlOverlay>,
}

impl ArticleView {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let overlay : gtk::Overlay = builder.get_object("url_overlay").ok_or(format_err!("some err"))?;
        let url_overlay = UrlOverlay::new()?;
        overlay.add_overlay(&url_overlay.widget());
        let stack : gtk::Stack = builder.get_object("article_view_stack").ok_or(format_err!("some err"))?;
        stack.set_visible_child_name("empty");

        let internal_view = InternalView::Empty;

        let article_view = ArticleView {
            stack: stack,
            parent_overlay: overlay,
            visible_article: None,
            internal_view: internal_view,
            theme: ArticleTheme::Default,
            load_changed_signal: None,
            mouse_over_signal: None,
            scroll_signal: None,
            key_press_signal: None,
            url_overlay: gtk_handle!(url_overlay),
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
        // disconnect signals
        if let Some(old_name) = old_name.to_str() {
            if let Some(old_webview) = self.stack.get_child_by_name(old_name) {
                GtkUtil::disconnect_signal(self.load_changed_signal, &old_webview);
                GtkUtil::disconnect_signal(self.mouse_over_signal, &old_webview);
                GtkUtil::disconnect_signal(self.scroll_signal, &old_webview);
                GtkUtil::disconnect_signal(self.key_press_signal, &old_webview);
                self.load_changed_signal = None;
                self.mouse_over_signal = None;
                self.scroll_signal = None;
                self.key_press_signal = None;
            }
        }
        let stack_clone = self.stack.clone();

        let webview = self.new_webview()?;
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

    fn new_webview(&mut self) -> Result<WebView, Error> {
        let settings = Settings::new();
        settings.set_enable_accelerated_2d_canvas(true);
        settings.set_enable_html5_database(false);
        settings.set_enable_html5_local_storage(false);
        settings.set_enable_java(false);
        settings.set_enable_media_stream(false);
		settings.set_enable_page_cache(false);
		settings.set_enable_plugins(false);
		settings.set_enable_smooth_scrolling(false);
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

        //----------------------------------
        // open link in external browser
        //----------------------------------
		self.load_changed_signal = Some(webview.connect_load_changed(|closure_webivew, event| {
            match event {
                LoadEvent::Started => {
                    if let Some(uri) = closure_webivew.get_uri() {
                        if let Some(default_screen) = gdk::Screen::get_default() {
                            if let Err(_) = gtk::show_uri(&default_screen, &uri, glib::get_current_time().tv_sec as u32) {
                                // log smth
                            }
                        }
                    }
                },
                LoadEvent::Redirected => {},
                LoadEvent::Committed => {},
                LoadEvent::Finished => {},
                _ => {},
            }
        }).to_glib());

        //----------------------------------
        // show url overlay
        //----------------------------------
        let url_overlay_handle = self.url_overlay.clone();
		self.mouse_over_signal = Some(webview.connect_mouse_target_changed(move |_closure_webivew, hit_test, _modifiers| {
            if hit_test.context_is_link() {
                if let Some(uri) = hit_test.get_link_uri() {
                    let mut align = gtk::Align::Start;
                    let rel_x = 0.0; // FIXME
                    let rel_y = 0.0; // FIXME

                    if rel_x <= 0.5 && rel_y >= 0.85 {
                        align = gtk::Align::End;
                    }

                    url_overlay_handle.borrow().set_url(uri, align);
                    url_overlay_handle.borrow_mut().reveal(true);
                }
            }
            else {
                url_overlay_handle.borrow_mut().reveal(false);
            }
        }).to_glib());

        //----------------------------------
        // zoom with ctrl+scroll
        //----------------------------------
        self.scroll_signal = Some(webview.connect_scroll_event(|closure_webivew, event| {
            if event.get_state().contains(ModifierType::CONTROL_MASK) {
                let zoom = closure_webivew.get_zoom_level();
                match event.get_direction() {
                    ScrollDirection::Up => closure_webivew.set_zoom_level(zoom - 0.25),
                    ScrollDirection::Down => closure_webivew.set_zoom_level(zoom + 0.25),
                    ScrollDirection::Smooth => {
                        let (_, y_delta) = event.get_delta();
                        let (_, y_root) = event.get_root();
                        let diff = 10.0 * (y_delta / y_root);
                        closure_webivew.set_zoom_level(zoom - diff);
                    },
                    _ => {},
                }
                return Inhibit(true)
            }
            Inhibit(false)
        }).to_glib());

        //------------------------------------------------
        // zoom with ctrl+PLUS/MINUS & reset with ctrl+0
        //------------------------------------------------
        self.key_press_signal = Some(webview.connect_key_press_event(|closure_webivew, event| {
            if event.get_state().contains(ModifierType::CONTROL_MASK) {
                let zoom = closure_webivew.get_zoom_level();
                println!("KP_0: {}", KP_0);
                println!("keyval: {}", event.get_keyval());
                match event.get_keyval() {
                    KP_0 => closure_webivew.set_zoom_level(1.0),
                    KP_ADD => closure_webivew.set_zoom_level(zoom + 0.25),
                    KP_SUBTRACT => closure_webivew.set_zoom_level(zoom - 0.25),
                    _ => {},
                }
                return Inhibit(true)
            }
            Inhibit(false)
        }).to_glib());


        //webview.context_menu.connect(onContextMenu);
		//webview.button_press_event.connect(onClick);
		//webview.button_release_event.connect(onRelease);
		//webview.motion_notify_event.connect(onMouseMotion);
		//webview.enter_fullscreen.connect(enterFullscreenVideo);
		//webview.leave_fullscreen.connect(leaveFullscreenVideo);
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