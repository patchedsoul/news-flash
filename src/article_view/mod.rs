mod models;
mod url_overlay;
mod progress_overlay;

use log::{
    error,
};
use self::url_overlay::UrlOverlay;
use self::progress_overlay::ProgressOverlay;
use failure::{
    Error,
    format_err,
};
use webkit2gtk::{
    WebView,
    WebViewExt,
    LoadEvent,
    Settings,
    SettingsExt,
    HitTestResultExt,
    URIRequestExt,
    PolicyDecisionType,
    NavigationPolicyDecision,
    NavigationPolicyDecisionExt,
    ContextMenuExt,
    ContextMenuItemExt,
    ContextMenuAction,
};
use gtk::{
    StackExt,
    WidgetExt,
    WidgetExtManual,
    ButtonExt,
    ContainerExt,
    Continue,
    OverlayExt,
    Inhibit,
};
use gdk::{
    EventMask,
    ScrollDirection,
    ModifierType,
    Display,
    SeatCapabilities,
    Cursor,
    CursorType,
    enums::key::KP_0,
    enums::key::KP_Add as KP_ADD,
    enums::key::KP_Subtract as KP_SUBTRACT,
};
use glib::{
    translate::{
        ToGlib,
    },
    object::Cast,
    MainLoop,
};
use gio::{
    Cancellable,
};
use news_flash::models::{
    FatArticle,
};
use self::models::{
    InternalState,
    ArticleTheme,
};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::cell::RefCell;
use crate::Resources;
use crate::util::GtkHandle;
use crate::util::FileUtil;
use crate::gtk_handle;
use crate::util::{
    DateUtil,
    GtkUtil,
};
use std::str;

const MIDDLE_MOUSE_BUTTON: u32 = 2;

#[derive(Clone, Debug)]
pub struct ArticleView {
    stack: gtk::Stack,
    top_overlay: gtk::Overlay,
    view_html_button: gtk::Button,
    visible_article: GtkHandle<Option<FatArticle>>,
    internal_state: GtkHandle<InternalState>,
    theme: ArticleTheme,
    load_changed_signal: GtkHandle<Option<u64>>,
    decide_policy_signal: GtkHandle<Option<u64>>,
    mouse_over_signal: GtkHandle<Option<u64>>,
    scroll_signal: GtkHandle<Option<u64>>,
    key_press_signal: GtkHandle<Option<u64>>,
    ctx_menu_signal: GtkHandle<Option<u64>>,
    load_signal: GtkHandle<Option<u64>>,
    click_signal: GtkHandle<Option<u64>>,
    click_release_signal: GtkHandle<Option<u64>>,
    drag_motion_notify_signal: GtkHandle<Option<u64>>,
    drag_released_motion_signal: GtkHandle<Option<u32>>,
    drag_buffer_update_signal: GtkHandle<Option<u32>>,
    progress_overlay_delay_signal: GtkHandle<Option<u32>>,
    url_overlay_label: GtkHandle<UrlOverlay>,
    progress_overlay_label: GtkHandle<ProgressOverlay>,
    drag_buffer: GtkHandle<[f64; 10]>,
    drag_ongoing: GtkHandle<bool>,
    drag_y_pos: GtkHandle<f64>,
    drag_momentum: GtkHandle<f64>,
}

impl ArticleView {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);

        let url_overlay : gtk::Overlay = builder.get_object("url_overlay").ok_or(format_err!("some err"))?;
        let url_overlay_label = UrlOverlay::new()?;
        url_overlay.add_overlay(&url_overlay_label.widget());

        let progress_overlay : gtk::Overlay = builder.get_object("progress_overlay").ok_or(format_err!("some err"))?;
        let progress_overlay_label = ProgressOverlay::new()?;
        progress_overlay.add_overlay(&progress_overlay_label.widget());

        let visible_article : GtkHandle<Option<FatArticle>> = gtk_handle!(None);
        let visible_article_crash_view = visible_article.clone();
        let view_html_button : gtk::Button = builder.get_object("view_html_button").ok_or(format_err!("some err"))?;
        view_html_button.connect_clicked(move |_button| {
            if let Some(article) = visible_article_crash_view.borrow().as_ref() {
                if let Some(html) = &article.html {
                    if let Ok(path) = FileUtil::write_temp_file("crashed_article.html", html) {
                        if let Some(default_screen) = gdk::Screen::get_default() {
                            if let Some(path) = path.to_str() {
                                let uri = format!("file://{}", path);
                                if let Err(_) = gtk::show_uri(&default_screen, &uri, glib::get_current_time().tv_sec as u32) {
                                    // log smth
                                }
                            }
                        }
                    }
                }
            }
        });

        let stack : gtk::Stack = builder.get_object("article_view_stack").ok_or(format_err!("some err"))?;
        stack.set_visible_child_name("empty");

        let internal_state = InternalState::Empty;

        let article_view = ArticleView {
            stack: stack,
            top_overlay: progress_overlay,
            view_html_button: view_html_button,
            visible_article: visible_article,
            internal_state: gtk_handle!(internal_state),
            theme: ArticleTheme::Default,
            load_changed_signal: gtk_handle!(None),
            decide_policy_signal: gtk_handle!(None),
            mouse_over_signal: gtk_handle!(None),
            scroll_signal: gtk_handle!(None),
            key_press_signal: gtk_handle!(None),
            ctx_menu_signal: gtk_handle!(None),
            load_signal: gtk_handle!(None),
            click_signal: gtk_handle!(None),
            click_release_signal: gtk_handle!(None),
            drag_motion_notify_signal: gtk_handle!(None),
            drag_released_motion_signal: gtk_handle!(None),
            drag_buffer_update_signal: gtk_handle!(None),
            progress_overlay_delay_signal: gtk_handle!(None),
            url_overlay_label: gtk_handle!(url_overlay_label),
            progress_overlay_label: gtk_handle!(progress_overlay_label),
            drag_buffer: gtk_handle!([0.0; 10]),
            drag_ongoing: gtk_handle!(false),
            drag_y_pos: gtk_handle!(0.0),
            drag_momentum: gtk_handle!(0.0),
        };

        article_view.stack.show_all();
        Ok(article_view)
    }

    pub fn widget(&self) -> gtk::Overlay {
        self.top_overlay.clone()
    }

    pub fn show_article(&mut self, article: FatArticle, feed_name: String) -> Result<(), Error> {
        let webview = self.switch_view()?;
        let html = self.build_article(&article, feed_name)?;
        webview.load_html(&html, None);
        *self.visible_article.borrow_mut() = Some(article);
        Ok(())
    }

    fn switch_view(&mut self) -> Result<WebView, Error> {
        self.remove_old_view(150);

        let webview = self.new_webview()?;
        let old_state = (*self.internal_state.borrow()).clone();
        *self.internal_state.borrow_mut() = old_state.switch();
        if let Some(new_name) = self.internal_state.borrow().to_str() {
            self.stack.add_named(&webview, new_name);
            self.stack.show_all();
            self.stack.set_visible_child_name(new_name);
        }

        Ok(webview)
    }

    fn remove_old_view(&self, timeout: u32) {
        Self::remove_old_view_static(
            timeout,
            &self.progress_overlay_label,
            &self.internal_state,
            &self.stack,
            &self.load_changed_signal,
            &self.decide_policy_signal,
            &self.mouse_over_signal,
            &self.scroll_signal,
            &self.key_press_signal,
            &self.ctx_menu_signal,
            &self.load_signal,
            &self.click_signal,
            &self.click_release_signal,
            &self.drag_released_motion_signal,
            &self.drag_buffer_update_signal,
            &self.progress_overlay_delay_signal,
        );
    }

    fn remove_old_view_static(
        timeout: u32,
        progress_overlay_label: &GtkHandle<ProgressOverlay>,
        old_state: &GtkHandle<InternalState>,
        stack: &gtk::Stack,
        load_changed_signal: &GtkHandle<Option<u64>>,
        decide_policy_signal: &GtkHandle<Option<u64>>,
        mouse_over_signal: &GtkHandle<Option<u64>>,
        scroll_signal: &GtkHandle<Option<u64>>,
        key_press_signal: &GtkHandle<Option<u64>>,
        ctx_menu_signal: &GtkHandle<Option<u64>>,
        load_signal: &GtkHandle<Option<u64>>,
        click_signal: &GtkHandle<Option<u64>>,
        click_release_signal: &GtkHandle<Option<u64>>,
        drag_released_motion_signal: &GtkHandle<Option<u32>>,
        drag_buffer_update_signal: &GtkHandle<Option<u32>>,
        progress_overlay_delay_signal: &GtkHandle<Option<u32>>,
    ) {
        let old_state = (*old_state.borrow()).clone();
        progress_overlay_label.borrow().reveal(false);

        GtkUtil::remove_source(*drag_released_motion_signal.borrow());
        GtkUtil::remove_source(*drag_buffer_update_signal.borrow());
        GtkUtil::remove_source(*progress_overlay_delay_signal.borrow());

        // disconnect signals
        if let Some(old_state) = old_state.to_str() {
            if let Some(old_webview) = stack.get_child_by_name(old_state) {
                GtkUtil::disconnect_signal_handle(load_changed_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(decide_policy_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(mouse_over_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(scroll_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(key_press_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(ctx_menu_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(load_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(click_signal, &old_webview);
                GtkUtil::disconnect_signal_handle(click_release_signal, &old_webview);
                *load_changed_signal.borrow_mut() = None;
                *decide_policy_signal.borrow_mut() = None;
                *mouse_over_signal.borrow_mut() = None;
                *scroll_signal.borrow_mut() = None;
                *key_press_signal.borrow_mut() = None;
                *ctx_menu_signal.borrow_mut() = None;
                *load_signal.borrow_mut() = None;
                *click_signal.borrow_mut() = None;
                *click_release_signal.borrow_mut() = None;
            }
        }

        // remove old view after timeout
        let stack_clone = stack.clone();
        gtk::timeout_add(timeout, move || {
            if let Some(old_state) = old_state.to_str() {
                if let Some(old_view) = stack_clone.get_child_by_name(&old_state) {
                    stack_clone.remove(&old_view);
                }
            }
            Continue(false)
        });
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
		settings.set_enable_javascript(true);
		settings.set_javascript_can_access_clipboard(false);
		settings.set_javascript_can_open_windows_automatically(false);
		settings.set_media_playback_requires_user_gesture(true);
        settings.set_user_agent_with_application_details(Some("NewsFlash"), None);

        let webview = WebView::new_with_settings(&settings);
		webview.set_events(EventMask::POINTER_MOTION_MASK);
		webview.set_events(EventMask::SCROLL_MASK);
		webview.set_events(EventMask::BUTTON_PRESS_MASK);
		webview.set_events(EventMask::BUTTON_RELEASE_MASK);
		webview.set_events(EventMask::KEY_PRESS_MASK);

        //----------------------------------
        // open link in external browser
        //----------------------------------
		*self.load_changed_signal.borrow_mut() = Some(webview.connect_load_changed(|closure_webivew, event| {
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

        *self.decide_policy_signal.borrow_mut() = Some(webview.connect_decide_policy(|_closure_webivew, decision, decision_type| {
            if decision_type == PolicyDecisionType::NewWindowAction {
                if let Ok(navigation_decision) = decision.clone().downcast::<NavigationPolicyDecision>() {
                    if let Some(frame_name) = navigation_decision.get_frame_name() {
                        if &frame_name == "_blank" {
                            if let Some(action) = navigation_decision.get_navigation_action() {
                                if let Some(uri_req) = action.get_request() {
                                    if let Some(uri) = uri_req.get_uri() {
                                        if let Some(default_screen) = gdk::Screen::get_default() {
                                            if let Err(_) = gtk::show_uri(&default_screen, &uri, glib::get_current_time().tv_sec as u32) {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            false
        }).to_glib());

        //----------------------------------
        // show url overlay
        //----------------------------------
        let url_overlay_handle = self.url_overlay_label.clone();
		*self.mouse_over_signal.borrow_mut() = Some(webview.connect_mouse_target_changed(move |_closure_webivew, hit_test, _modifiers| {
            if hit_test.context_is_link() {
                if let Some(uri) = hit_test.get_link_uri() {
                    let mut align = gtk::Align::Start;
                    let rel_x = 0.0; // FIXME
                    let rel_y = 0.0; // FIXME

                    if rel_x <= 0.5 && rel_y >= 0.85 {
                        align = gtk::Align::End;
                    }

                    url_overlay_handle.borrow().set_url(uri.as_str().to_owned(), align);
                    url_overlay_handle.borrow().reveal(true);
                }
            }
            else {
                url_overlay_handle.borrow().reveal(false);
            }
        }).to_glib());

        //----------------------------------
        // zoom with ctrl+scroll
        //----------------------------------
        *self.scroll_signal.borrow_mut() = Some(webview.connect_scroll_event(|closure_webivew, event| {
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
        *self.key_press_signal.borrow_mut() = Some(webview.connect_key_press_event(|closure_webivew, event| {
            if event.get_state().contains(ModifierType::CONTROL_MASK) {
                let zoom = closure_webivew.get_zoom_level();
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


        //----------------------------------
        // clean up context menu
        //----------------------------------
        *self.ctx_menu_signal.borrow_mut() = Some(webview.connect_context_menu(|_closure_webivew, ctx_menu, _event, _hit_test| {
            let menu_items = ctx_menu.get_items();

            for item in menu_items {
                if item.is_separator() {
                    ctx_menu.remove(&item);
                    continue
                }

                match item.get_stock_action() {
                    ContextMenuAction::CopyLinkToClipboard |
                    ContextMenuAction::Copy |
                    ContextMenuAction::CopyImageToClipboard |
                    ContextMenuAction::CopyImageUrlToClipboard => {},
                    _ => ctx_menu.remove(&item),
                }
            }

            if ctx_menu.first().is_none() {
                return true
            }

            false
        }).to_glib());

        //----------------------------------
        // display load progress
        //----------------------------------
        let progress_handle = self.progress_overlay_label.clone();
        let progress_overlay_delay_signal = self.progress_overlay_delay_signal.clone();
        let load_signal = self.load_signal.clone();
        let progress_webview = webview.clone();
        *self.progress_overlay_delay_signal.borrow_mut() = Some(gtk::timeout_add(1500, move || {
            *progress_overlay_delay_signal.borrow_mut() = None;
            if progress_webview.get_estimated_load_progress() == 1.0 {
                return Continue(false)
            }

            let progress_handle = progress_handle.clone();
            *load_signal.borrow_mut() = Some(progress_webview.connect_property_estimated_load_progress_notify(move |closure_webivew| {
                let progress = closure_webivew.get_estimated_load_progress();
                if progress >= 1.0 {
                    progress_handle.borrow().reveal(false);
                    return;
                }
                progress_handle.borrow().reveal(true);
                progress_handle.borrow().set_percentage(progress);
            }).to_glib());
            Continue(false)
        }).to_glib());

        //----------------------------------
        // drag page
        //----------------------------------
        let drag_buffer = self.drag_buffer.clone();
        let drag_ongoing = self.drag_ongoing.clone();
        let widget = self.top_overlay.clone();
        let drag_y_pos = self.drag_y_pos.clone();
        let drag_momentum = self.drag_momentum.clone();
        let drag_motion_notify_signal = self.drag_motion_notify_signal.clone();
        let drag_buffer_update_signal = self.drag_buffer_update_signal.clone();
		*self.click_signal.borrow_mut() = Some(webview.connect_button_press_event(move |closure_webivew, event| {
            if event.get_button() == MIDDLE_MOUSE_BUTTON {
                let (_, y) = event.get_position();
                *drag_y_pos.borrow_mut() = y;
                *drag_buffer.borrow_mut() = [y; 10];
                *drag_ongoing.borrow_mut() = true;

                if let Some(display) = Display::get_default() {
                    if let Some(seat) = display.get_default_seat() {
                        if let Some(pointer) = seat.get_pointer() {
                            if let Some(window) = closure_webivew.get_window() {

                                let cursor = Cursor::new_for_display(&display, CursorType::Fleur);
                                // let _grab_status = seat.grab(
                                //     &window,
                                //     SeatCapabilities::POINTER,
                                //     false,
                                //     Some(&cursor),
                                //     None,
                                //     None,
                                // );

                                gtk::device_grab_add(&widget, &pointer, false);
                                let drag_buffer_update = drag_buffer.clone();
                                let drag_momentum_update = drag_momentum.clone();
                                let drag_ongoing_update = drag_ongoing.clone();
                                let drag_y_pos_update = drag_y_pos.clone();
                                let drag_buffer_update_signal_clone = drag_buffer_update_signal.clone();
                                *drag_buffer_update_signal.borrow_mut() = Some(gtk::timeout_add(10, move || {
                                    if !*drag_ongoing_update.borrow() {
                                        *drag_buffer_update_signal_clone.borrow_mut() = None;
                                        return Continue(false)
                                    }

                                    for i in (1..10).rev() {
                                        let value = (*drag_buffer_update.borrow())[i-1];
                                        (*drag_buffer_update.borrow_mut())[i] = value;
                                    }

                                    (*drag_buffer_update.borrow_mut())[0] = *drag_y_pos_update.borrow();
                                    *drag_momentum_update.borrow_mut() = (*drag_buffer_update.borrow())[9] - (*drag_buffer_update.borrow())[0];
                                    Continue(true)
                                }).to_glib());

                                let drag_y_pos_motion_update = drag_y_pos.clone();
                                *drag_motion_notify_signal.borrow_mut() = Some(closure_webivew.connect_motion_notify_event(move |view, event| {
                                    let (_, y) = event.get_position();
                                    let scroll = *drag_y_pos_motion_update.borrow() - y;
                                    *drag_y_pos_motion_update.borrow_mut() = y;
                                    if let Ok(scroll_pos) = Self::get_scroll_pos(view) {
                                        Self::set_scroll_pos(view, scroll_pos + scroll as i32).unwrap();
                                    }
                                    Inhibit(false)
                                }).to_glib());
                            }
                        }
                    }
                }
                return Inhibit(true)
            }
            Inhibit(false)
        }).to_glib());

        let drag_motion_notify_signal = self.drag_motion_notify_signal.clone();
        let drag_released_motion_signal = self.drag_released_motion_signal.clone();
        let drag_ongoing = self.drag_ongoing.clone();
        let drag_momentum = self.drag_momentum.clone();
        let widget = self.top_overlay.clone();
		*self.click_release_signal.borrow_mut() = Some(webview.connect_button_release_event(move |closure_webivew, event| {
            if event.get_button() == MIDDLE_MOUSE_BUTTON {
                GtkUtil::disconnect_signal(*drag_motion_notify_signal.borrow(), closure_webivew);
                *drag_ongoing.borrow_mut() = false;

                let drag_momentum = drag_momentum.clone();
                let drag_released_motion_signal_clone = drag_released_motion_signal.clone();
                let view = closure_webivew.clone();
                *drag_released_motion_signal.borrow_mut() = Some(gtk::timeout_add(20, move || {
                    *drag_momentum.borrow_mut() /= 1.2;
                    let allocation = view.get_allocation();

                    let page_size = view.get_allocated_height() as f64;
                    let adjust_value = page_size * *drag_momentum.borrow() / allocation.height as f64;
                    let old_adjust = Self::get_scroll_pos(&view).unwrap() as f64;
                    let upper = Self::get_scroll_upper(&view).unwrap() as f64 * view.get_zoom_level();

                    if (old_adjust + adjust_value) > (upper - page_size) || (old_adjust + adjust_value) < 0.0 {
                        *drag_momentum.borrow_mut() = 0.0;
                    }

                    let new_scroll_pos = f64::min(old_adjust + adjust_value, upper - page_size);
                    Self::set_scroll_pos(&view, new_scroll_pos as i32).unwrap();

                    if drag_momentum.borrow().abs() < 1.0 {
                        *drag_released_motion_signal_clone.borrow_mut() = None;
                        return Continue(false)
                    }

                    Continue(true)
                }).to_glib());

                if let Some(display) = Display::get_default() {
                    if let Some(seat) = display.get_default_seat() {
                        if let Some(pointer) = seat.get_pointer() {
                            gtk::device_grab_remove(&widget, &pointer);
                            seat.ungrab();
                        }
                    }
                }

                return Inhibit(true)
            }
            Inhibit(false)
        }).to_glib());

        //----------------------------------
        // crash view
        //----------------------------------
        let stack = self.stack.clone();
        let progress_overlay_label = self.progress_overlay_label.clone();
        let internal_state = self.internal_state.clone();
        let load_changed_signal = self.load_changed_signal.clone();
        let decide_policy_signal = self.decide_policy_signal.clone();
        let mouse_over_signal = self.mouse_over_signal.clone();
        let scroll_signal = self.scroll_signal.clone();
        let key_press_signal = self.key_press_signal.clone();
        let ctx_menu_signal = self.ctx_menu_signal.clone();
        let load_signal = self.load_signal.clone();
        let click_signal = self.click_signal.clone();
        let click_release_signal = self.click_release_signal.clone();
        let drag_released_motion_signal = self.drag_released_motion_signal.clone();
        let drag_buffer_update_signal = self.drag_buffer_update_signal.clone();
        let progress_overlay_delay_signal = self.progress_overlay_delay_signal.clone();
        webview.connect_web_process_crashed(move |_closure_webivew| {
            Self::remove_old_view_static(
                150,
                &progress_overlay_label,
                &internal_state,
                &stack,
                &load_changed_signal,
                &decide_policy_signal,
                &mouse_over_signal,
                &scroll_signal,
                &key_press_signal,
                &ctx_menu_signal,
                &load_signal,
                &click_signal,
                &click_release_signal,
                &drag_released_motion_signal,
                &drag_buffer_update_signal,
                &progress_overlay_delay_signal,
            );
            stack.set_visible_child_name("crash");
            *internal_state.borrow_mut() = InternalState::Crash;
            false
        });

		// webview.motion_notify_event.connect(onMouseMotion);
		// webview.enter_fullscreen.connect(enterFullscreenVideo);
		// webview.leave_fullscreen.connect(leaveFullscreenVideo);
		// webview.set_background_color(m_color);

        Ok(webview)
    }

    fn build_article(&self, article: &FatArticle, feed_name: String) -> Result<String, Error>  {
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
        if let Some(html) = &article.html {
            template_string = template_string.replacen("$HTML", html, 1);
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
        if let Some(title) = &article.title {
            template_string = template_string.replacen("$TITLE", title, 1);
        }

        // $LARGESIZE
        let large_size = font_size * 2;
        template_string = template_string.replacen("$LARGESIZE", &format!("{}", large_size), 1);

        // $URL
        if let Some(article_url) = &article.url {
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

    fn set_scroll_pos(view: &WebView, pos: i32) -> Result<(), Error> {
        let cancellable : Option<&Cancellable> = None;
        view.run_javascript(
            &format!("window.scrollTo(0,{});", pos),
            cancellable,
            |res| {
                match res {
                    Ok(_) => {},
                    Err(_) => error!("Setting scroll pos failed"),
                }
            }
        );
        Ok(())
    }

    fn get_scroll_pos(view: &WebView) -> Result<i32, Error> {
        Self::webview_js_get_i32(view, "window.scrollY")
    }

    fn get_scroll_upper(view: &WebView) -> Result<i32, Error> {
        Self::webview_js_get_i32(view, "Math.max (
            document.body.scrollHeight,
            document.body.offsetHeight,
            document.documentElement.clientHeight,
            document.documentElement.scrollHeight,
            document.documentElement.offsetHeight
        )")
    }

    fn webview_js_get_i32(view: &WebView, java_script: &str) -> Result<i32, Error> {
        let wait_loop = Arc::new(MainLoop::new(None, false));
        let callback_wait_loop = wait_loop.clone();
        let value : Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(None));
        let callback_value = value.clone();
        let cancellable : Option<&Cancellable> = None;
        view.run_javascript(java_script, cancellable, move |res| {
                match res {
                    Ok(result) => {
                        let context = result.get_global_context().unwrap();
                        let value = result.get_value().unwrap();
                        *callback_value.lock().unwrap() = value.to_number(&context);
                    },
                    Err(_) => error!("Getting scroll pos failed"),
                }
                callback_wait_loop.quit();
            }
        );

        wait_loop.run();
        if let Ok(pos) = value.lock() {
            if let Some(pos) = *pos {
                return Ok(pos as i32)
            }
        }
        Err(format_err!("some err"))
    }
}