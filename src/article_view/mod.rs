mod models;
mod progress_overlay;
mod url_overlay;

pub use self::models::ArticleTheme;
use self::models::InternalState;
use self::progress_overlay::ProgressOverlay;
use self::url_overlay::UrlOverlay;
use crate::gtk_handle;
use crate::settings::Settings;
use crate::util::{BuilderHelper, DateUtil, FileUtil, GtkHandle, GtkUtil, Util, GTK_RESOURCE_FILE_ERROR};
use crate::Resources;
use failure::{format_err, Error};
use gdk::{
    enums::key::KP_Add as KP_ADD, enums::key::KP_Subtract as KP_SUBTRACT, enums::key::KP_0, Cursor, CursorType,
    Display, EventMask, ModifierType, ScrollDirection,
};
use gio::{Cancellable, Settings as GSettings, SettingsExt as GSettingsExt};
use glib::{object::Cast, translate::ToGlib, MainLoop};
use gtk::{
    Button, ButtonExt, ContainerExt, Continue, Inhibit, Overlay, OverlayExt, SettingsExt as GtkSettingsExt, Stack,
    StackExt, TickCallbackId, WidgetExt, WidgetExtManual,
};
use log::{error, warn};
use news_flash::models::FatArticle;
use pango::FontDescription;
use std::cell::RefCell;
use std::rc::Rc;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use webkit2gtk::{
    ContextMenuAction, ContextMenuExt, ContextMenuItemExt, HitTestResultExt, LoadEvent, NavigationPolicyDecision,
    NavigationPolicyDecisionExt, PolicyDecisionType, Settings as WebkitSettings, SettingsExt, URIRequestExt, WebView,
    WebViewExt,
};

const MIDDLE_MOUSE_BUTTON: u32 = 2;
const SCROLL_TRANSITION_DURATION: i64 = 500 * 1000;

#[derive(Clone)]
struct ScrollAnimationProperties {
    pub start_time: GtkHandle<Option<i64>>,
    pub end_time: GtkHandle<Option<i64>>,
    pub scroll_callback_id: GtkHandle<Option<TickCallbackId>>,
    pub transition_start_value: GtkHandle<Option<f64>>,
    pub transition_diff: GtkHandle<Option<f64>>,
}

#[derive(Clone)]
pub struct ArticleView {
    settings: GtkHandle<Settings>,
    stack: Stack,
    top_overlay: Overlay,
    view_html_button: Button,
    visible_article: GtkHandle<Option<FatArticle>>,
    visible_feed_name: GtkHandle<Option<String>>,
    internal_state: GtkHandle<InternalState>,
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
    pointer_pos: GtkHandle<(f64, f64)>,
    scroll_animation_data: ScrollAnimationProperties,
}

impl ArticleView {
    pub fn new(settings: &GtkHandle<Settings>) -> Result<Self, Error> {
        let builder = BuilderHelper::new("article_view");

        let url_overlay = builder.get::<Overlay>("url_overlay");
        let url_overlay_label = UrlOverlay::new();
        url_overlay.add_overlay(&url_overlay_label.widget());

        let progress_overlay = builder.get::<Overlay>("progress_overlay");
        let progress_overlay_label = ProgressOverlay::new();
        progress_overlay.add_overlay(&progress_overlay_label.widget());

        let visible_article: GtkHandle<Option<FatArticle>> = gtk_handle!(None);
        let visible_feed_name: GtkHandle<Option<String>> = gtk_handle!(None);
        let visible_article_crash_view = visible_article.clone();
        let view_html_button = builder.get::<Button>("view_html_button");
        view_html_button.connect_clicked(move |_button| {
            if let Some(article) = visible_article_crash_view.borrow().as_ref() {
                if let Some(html) = &article.html {
                    if let Ok(path) = FileUtil::write_temp_file("crashed_article.html", html) {
                        if let Some(default_screen) = gdk::Screen::get_default() {
                            if let Some(path) = path.to_str() {
                                let uri = format!("file://{}", path);
                                if gtk::show_uri(Some(&default_screen), &uri, glib::get_current_time().tv_sec as u32)
                                    .is_err()
                                {
                                    // log smth
                                }
                            }
                        }
                    }
                }
            }
        });

        let stack = builder.get::<Stack>("article_view_stack");
        stack.set_visible_child_name("empty");

        let internal_state = InternalState::Empty;
        let settings = settings.clone();

        let article_view = ArticleView {
            settings,
            stack,
            top_overlay: progress_overlay,
            view_html_button,
            visible_article,
            visible_feed_name,
            internal_state: gtk_handle!(internal_state),
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
            pointer_pos: gtk_handle!((0.0, 0.0)),
            scroll_animation_data: ScrollAnimationProperties {
                start_time: gtk_handle!(None),
                end_time: gtk_handle!(None),
                scroll_callback_id: gtk_handle!(None),
                transition_start_value: gtk_handle!(None),
                transition_diff: gtk_handle!(None),
            },
        };

        article_view.stack.show_all();
        Ok(article_view)
    }

    pub fn widget(&self) -> gtk::Overlay {
        self.top_overlay.clone()
    }

    pub fn show_article(&mut self, article: FatArticle, feed_name: String) -> Result<(), Error> {
        let webview = self.switch_view()?;
        let html = self.build_article(&article, &feed_name)?;
        webview.load_html(&html, None);
        *self.visible_article.borrow_mut() = Some(article);
        *self.visible_feed_name.borrow_mut() = Some(feed_name);
        Ok(())
    }

    pub fn redraw_article(&mut self) -> Result<(), Error> {
        let mut html = String::new();
        let mut success = false;

        if let Some(article) = &*self.visible_article.borrow() {
            if let Some(feed_name) = &*self.visible_feed_name.borrow() {
                html = self.build_article(&article, feed_name)?;
                success = true;
            }
        }

        if !success {
            warn!("Can't redraw article view. No article is on display.");
            return Ok(());
        }

        let webview = self.switch_view()?;
        webview.load_html(&html, None);
        Ok(())
    }

    pub fn close_article(&self) {
        self.remove_old_view(150);
        *self.internal_state.borrow_mut() = InternalState::Empty;
        self.stack.set_visible_child_name("empty");
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

    #[allow(clippy::too_many_arguments)]
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
        *drag_released_motion_signal.borrow_mut() = None;
        *drag_buffer_update_signal.borrow_mut() = None;
        *progress_overlay_delay_signal.borrow_mut() = None;

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
        let settings = WebkitSettings::new();
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
        *self.load_changed_signal.borrow_mut() = Some(
            webview
                .connect_load_changed(|closure_webivew, event| {
                    match event {
                        LoadEvent::Started => {
                            if let Some(uri) = closure_webivew.get_uri() {
                                if "about:blank" != uri {
                                    println!("uri: {}", uri);
                                    if let Some(default_screen) = gdk::Screen::get_default() {
                                        if gtk::show_uri(
                                            Some(&default_screen),
                                            &uri,
                                            glib::get_current_time().tv_sec as u32,
                                        )
                                        .is_err()
                                        {
                                            // log smth
                                        }
                                    }
                                }
                            }
                        }
                        LoadEvent::Redirected => {}
                        LoadEvent::Committed => {}
                        LoadEvent::Finished => {}
                        _ => {}
                    }
                })
                .to_glib(),
        );

        *self.decide_policy_signal.borrow_mut() = Some(
            webview
                .connect_decide_policy(|_closure_webivew, decision, decision_type| {
                    if decision_type == PolicyDecisionType::NewWindowAction {
                        if let Ok(navigation_decision) = decision.clone().downcast::<NavigationPolicyDecision>() {
                            if let Some(frame_name) = navigation_decision.get_frame_name() {
                                if &frame_name == "_blank" {
                                    if let Some(action) = navigation_decision.get_navigation_action() {
                                        if let Some(uri_req) = action.get_request() {
                                            if let Some(uri) = uri_req.get_uri() {
                                                if let Some(default_screen) = gdk::Screen::get_default() {
                                                    if gtk::show_uri(
                                                        Some(&default_screen),
                                                        &uri,
                                                        glib::get_current_time().tv_sec as u32,
                                                    )
                                                    .is_err()
                                                    {
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
                })
                .to_glib(),
        );

        //----------------------------------
        // show url overlay
        //----------------------------------
        let url_overlay_handle = self.url_overlay_label.clone();
        let stack = self.stack.clone();
        let pointer_pos = self.pointer_pos.clone();
        *self.mouse_over_signal.borrow_mut() = Some(
            webview
                .connect_mouse_target_changed(move |_closure_webivew, hit_test, _modifiers| {
                    if hit_test.context_is_link() {
                        if let Some(uri) = hit_test.get_link_uri() {
                            let allocation = stack.get_allocation();
                            let rel_x = pointer_pos.borrow().0 / f64::from(allocation.width);
                            let rel_y = pointer_pos.borrow().1 / f64::from(allocation.height);

                            let align = if rel_x <= 0.5 && rel_y >= 0.85 {
                                gtk::Align::End
                            } else {
                                gtk::Align::Start
                            };

                            url_overlay_handle.borrow().set_url(uri.as_str().to_owned(), align);
                            url_overlay_handle.borrow().reveal(true);
                        }
                    } else {
                        url_overlay_handle.borrow().reveal(false);
                    }
                })
                .to_glib(),
        );

        //----------------------------------
        // zoom with ctrl+scroll
        //----------------------------------
        *self.scroll_signal.borrow_mut() = Some(
            webview
                .connect_scroll_event(|closure_webivew, event| {
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
                            }
                            _ => {}
                        }
                        return Inhibit(true);
                    }
                    Inhibit(false)
                })
                .to_glib(),
        );

        //------------------------------------------------
        // zoom with ctrl+PLUS/MINUS & reset with ctrl+0
        //------------------------------------------------
        *self.key_press_signal.borrow_mut() = Some(
            webview
                .connect_key_press_event(|closure_webivew, event| {
                    if event.get_state().contains(ModifierType::CONTROL_MASK) {
                        let zoom = closure_webivew.get_zoom_level();
                        match event.get_keyval() {
                            KP_0 => closure_webivew.set_zoom_level(1.0),
                            KP_ADD => closure_webivew.set_zoom_level(zoom + 0.25),
                            KP_SUBTRACT => closure_webivew.set_zoom_level(zoom - 0.25),
                            _ => {}
                        }
                        return Inhibit(true);
                    }
                    Inhibit(false)
                })
                .to_glib(),
        );

        //----------------------------------
        // clean up context menu
        //----------------------------------
        *self.ctx_menu_signal.borrow_mut() = Some(
            webview
                .connect_context_menu(|_closure_webivew, ctx_menu, _event, _hit_test| {
                    let menu_items = ctx_menu.get_items();

                    for item in menu_items {
                        if item.is_separator() {
                            ctx_menu.remove(&item);
                            continue;
                        }

                        match item.get_stock_action() {
                            ContextMenuAction::CopyLinkToClipboard
                            | ContextMenuAction::Copy
                            | ContextMenuAction::CopyImageToClipboard
                            | ContextMenuAction::CopyImageUrlToClipboard => {}
                            _ => ctx_menu.remove(&item),
                        }
                    }

                    if ctx_menu.first().is_none() {
                        return true;
                    }

                    false
                })
                .to_glib(),
        );

        //----------------------------------
        // display load progress
        //----------------------------------
        let progress_handle = self.progress_overlay_label.clone();
        let progress_overlay_delay_signal = self.progress_overlay_delay_signal.clone();
        let load_signal = self.load_signal.clone();
        let progress_webview = webview.clone();
        *self.progress_overlay_delay_signal.borrow_mut() = Some(
            gtk::timeout_add(1500, move || {
                *progress_overlay_delay_signal.borrow_mut() = None;
                if (progress_webview.get_estimated_load_progress() - 1.0).abs() < 0.01 {
                    return Continue(false);
                }

                let progress_handle = progress_handle.clone();
                *load_signal.borrow_mut() = Some(
                    progress_webview
                        .connect_property_estimated_load_progress_notify(move |closure_webivew| {
                            let progress = closure_webivew.get_estimated_load_progress();
                            if progress >= 1.0 {
                                progress_handle.borrow().reveal(false);
                                return;
                            }
                            progress_handle.borrow().reveal(true);
                            progress_handle.borrow().set_percentage(progress);
                        })
                        .to_glib(),
                );
                Continue(false)
            })
            .to_glib(),
        );

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
        let scroll_animation_data = self.scroll_animation_data.clone();
        *self.click_signal.borrow_mut() = Some(
            webview
                .connect_button_press_event(move |closure_webview, event| {
                    if event.get_button() == MIDDLE_MOUSE_BUTTON {
                        Self::stop_scroll_animation(&closure_webview, &scroll_animation_data);
                        let (_, y) = event.get_position();
                        *drag_y_pos.borrow_mut() = y;
                        *drag_buffer.borrow_mut() = [y; 10];
                        *drag_ongoing.borrow_mut() = true;

                        if let Some(display) = Display::get_default() {
                            if let Some(seat) = display.get_default_seat() {
                                if let Some(pointer) = seat.get_pointer() {
                                    if let Some(window) = closure_webview.get_window() {
                                        let cursor = Cursor::new_for_display(&display, CursorType::Fleur);

                                        let _grab_status = seat.grab(
                                            &window,
                                            gdk::SeatCapabilities::POINTER,
                                            false,
                                            Some(&cursor),
                                            None,
                                            Some(&mut |_, _| {}),
                                        );

                                        gtk::device_grab_add(&widget, &pointer, false);
                                        let drag_buffer_update = drag_buffer.clone();
                                        let drag_momentum_update = drag_momentum.clone();
                                        let drag_ongoing_update = drag_ongoing.clone();
                                        let drag_y_pos_update = drag_y_pos.clone();
                                        let drag_buffer_update_signal_clone = drag_buffer_update_signal.clone();
                                        *drag_buffer_update_signal.borrow_mut() = Some(
                                            gtk::timeout_add(10, move || {
                                                if !*drag_ongoing_update.borrow() {
                                                    *drag_buffer_update_signal_clone.borrow_mut() = None;
                                                    return Continue(false);
                                                }

                                                for i in (1..10).rev() {
                                                    let value = (*drag_buffer_update.borrow())[i - 1];
                                                    (*drag_buffer_update.borrow_mut())[i] = value;
                                                }

                                                (*drag_buffer_update.borrow_mut())[0] = *drag_y_pos_update.borrow();
                                                *drag_momentum_update.borrow_mut() = (*drag_buffer_update.borrow())[9]
                                                    - (*drag_buffer_update.borrow())[0];
                                                Continue(true)
                                            })
                                            .to_glib(),
                                        );

                                        let drag_y_pos_motion_update = drag_y_pos.clone();
                                        *drag_motion_notify_signal.borrow_mut() = Some(
                                            closure_webview
                                                .connect_motion_notify_event(move |view, event| {
                                                    let (_, y) = event.get_position();
                                                    let scroll = *drag_y_pos_motion_update.borrow() - y;
                                                    *drag_y_pos_motion_update.borrow_mut() = y;
                                                    if let Ok(scroll_pos) = Self::get_scroll_pos_static(view) {
                                                        Self::set_scroll_pos_static(view, scroll_pos + scroll).unwrap();
                                                    }
                                                    Inhibit(false)
                                                })
                                                .to_glib(),
                                        );
                                    }
                                }
                            }
                        }
                        return Inhibit(true);
                    }
                    Inhibit(false)
                })
                .to_glib(),
        );

        let drag_motion_notify_signal = self.drag_motion_notify_signal.clone();
        let drag_released_motion_signal = self.drag_released_motion_signal.clone();
        let drag_ongoing = self.drag_ongoing.clone();
        let drag_momentum = self.drag_momentum.clone();
        let widget = self.top_overlay.clone();
        *self.click_release_signal.borrow_mut() = Some(
            webview
                .connect_button_release_event(move |closure_webivew, event| {
                    if event.get_button() == MIDDLE_MOUSE_BUTTON {
                        GtkUtil::disconnect_signal(*drag_motion_notify_signal.borrow(), closure_webivew);
                        *drag_ongoing.borrow_mut() = false;

                        let drag_momentum = drag_momentum.clone();
                        let drag_released_motion_signal_clone = drag_released_motion_signal.clone();
                        let view = closure_webivew.clone();
                        *drag_released_motion_signal.borrow_mut() = Some(
                            gtk::timeout_add(20, move || {
                                *drag_momentum.borrow_mut() /= 1.2;
                                let allocation = view.get_allocation();

                                let page_size = f64::from(view.get_allocated_height());
                                let adjust_value = page_size * *drag_momentum.borrow() / f64::from(allocation.height);
                                let old_adjust = f64::from(Self::get_scroll_pos_static(&view).unwrap());
                                let upper =
                                    f64::from(Self::get_scroll_upper_static(&view).unwrap()) * view.get_zoom_level();

                                if (old_adjust + adjust_value) > (upper - page_size)
                                    || (old_adjust + adjust_value) < 0.0
                                {
                                    *drag_momentum.borrow_mut() = 0.0;
                                }

                                let new_scroll_pos = f64::min(old_adjust + adjust_value, upper - page_size);
                                Self::set_scroll_pos_static(&view, new_scroll_pos).unwrap();

                                if drag_momentum.borrow().abs() < 1.0 {
                                    *drag_released_motion_signal_clone.borrow_mut() = None;
                                    return Continue(false);
                                }

                                Continue(true)
                            })
                            .to_glib(),
                        );

                        if let Some(display) = Display::get_default() {
                            if let Some(seat) = display.get_default_seat() {
                                if let Some(pointer) = seat.get_pointer() {
                                    gtk::device_grab_remove(&widget, &pointer);
                                    seat.ungrab();
                                }
                            }
                        }

                        return Inhibit(true);
                    }
                    Inhibit(false)
                })
                .to_glib(),
        );

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

        let pointer_pos = self.pointer_pos.clone();
        webview.connect_motion_notify_event(move |_closure_webivew, event| {
            *pointer_pos.borrow_mut() = event.get_position();
            Inhibit(false)
        });

        // webview.enter_fullscreen.connect(enterFullscreenVideo);
        // webview.leave_fullscreen.connect(leaveFullscreenVideo);

        Ok(webview)
    }

    fn build_article(&self, article: &FatArticle, feed_name: &str) -> Result<String, Error> {
        Self::build_article_static("article", article, feed_name, &self.settings, None, None)
    }

    pub fn build_article_static(
        file_name: &str,
        article: &FatArticle,
        feed_name: &str,
        settings: &GtkHandle<Settings>,
        theme_override: Option<ArticleTheme>,
        font_size_override: Option<i32>,
    ) -> Result<String, Error> {
        let template_data = Resources::get(&format!("article_view/{}.html", file_name)).expect(GTK_RESOURCE_FILE_ERROR);
        let template_str = str::from_utf8(template_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);
        let mut template_string = template_str.to_owned();

        let css_data = Resources::get("article_view/style.css").expect(GTK_RESOURCE_FILE_ERROR);
        let css_string = str::from_utf8(css_data.as_ref())?;

        // A list of fonts we should try to use in order of preference
        // We will pass all of these to CSS in order
        let mut font_options: Vec<String> = Vec::new();
        let mut font_families: Vec<String> = Vec::new();
        let mut font_size: Option<i32> = None;

        // Try to use the configured font if it exists
        if let Some(font_setting) = settings.borrow().get_article_view_font() {
            font_options.push(font_setting);
        }

        // If there is no configured font, or it's broken, use the system default font
        if let Some(font_system) = GSettings::new("org.gnome.desktop.interface").get_string("document-font-name") {
            font_options.push(font_system.to_string());
        }

        // Backup if the system font is broken too
        font_options.push("sans".to_owned());

        for font in font_options {
            let desc = FontDescription::from_string(&font);
            if let Some(family) = desc.get_family() {
                font_families.push(family.to_string());
            }
            if font_size.is_none() {
                if desc.get_size() > 0 {
                    font_size = Some(desc.get_size());
                }
            }
        }

        // if font size configured use it, otherwise use 12 as default
        let font_size = match font_size_override {
            Some(fsize_override) => fsize_override,
            None => match font_size {
                Some(size) => size,
                None => 12,
            },
        };

        let font_size = font_size / pango::SCALE;
        let font_family = font_families.join(", ");

        let mut author_date = String::new();
        let date = DateUtil::format(&article.date);
        if let Some(author) = &article.author {
            author_date.push_str(&format!("posted by: {}, {}", author, date));
        } else {
            author_date.push_str(&date.to_string());
        }

        // $HTML
        if let Some(html) = &article.html {
            template_string = template_string.replacen("$HTML", html, 1);
        }

        // $UNSELECTABLE
        if settings.borrow().get_article_view_allow_select() {
            template_string = template_string.replacen("$UNSELECTABLE", "", 1);
        } else {
            template_string = template_string.replacen("$UNSELECTABLE", "unselectable", 1);
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
        template_string = template_string.replacen("$FEED", feed_name, 1);

        // $THEME
        let theme = if let Some(theme_override) = &theme_override {
            theme_override.to_str().to_owned()
        } else {
            settings.borrow().get_article_view_theme().to_str().to_owned()
        };
        template_string = template_string.replacen("$THEME", &theme, 1);

        // $FONTFAMILY
        template_string = template_string.replacen("$FONTFAMILY", &font_family, 1);

        // $FONTSIZE
        template_string = template_string.replacen("$FONTSIZE", &format!("{}", font_size), 1);

        // $CSS
        template_string = template_string.replacen("$CSS", &css_string, 1);

        Ok(template_string)
    }

    fn set_scroll_pos_static(view: &WebView, pos: f64) -> Result<(), Error> {
        let cancellable: Option<&Cancellable> = None;
        view.run_javascript(&format!("window.scrollTo(0,{});", pos), cancellable, |res| match res {
            Ok(_) => {}
            Err(_) => error!("Setting scroll pos failed"),
        });
        Ok(())
    }

    fn get_scroll_pos_static(view: &WebView) -> Result<f64, Error> {
        Self::webview_js_get_f64(view, "window.scrollY")
    }

    fn get_scroll_window_height_static(view: &WebView) -> Result<f64, Error> {
        Self::webview_js_get_f64(view, "window.innerHeight")
    }

    fn get_scroll_upper_static(view: &WebView) -> Result<f64, Error> {
        Self::webview_js_get_f64(
            view,
            "Math.max (
            document.body.scrollHeight,
            document.body.offsetHeight,
            document.documentElement.clientHeight,
            document.documentElement.scrollHeight,
            document.documentElement.offsetHeight
        )",
        )
    }

    fn webview_js_get_f64(view: &WebView, java_script: &str) -> Result<f64, Error> {
        let wait_loop = Arc::new(MainLoop::new(None, false));
        let callback_wait_loop = wait_loop.clone();
        let value: Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(None));
        let callback_value = value.clone();
        let cancellable: Option<&Cancellable> = None;
        view.run_javascript(java_script, cancellable, move |res| {
            match res {
                Ok(result) => {
                    let context = result.get_global_context().unwrap();
                    let value = result.get_value().unwrap();
                    *callback_value.lock().unwrap() = value.to_number(&context);
                }
                Err(_) => error!("Getting scroll pos failed"),
            }
            callback_wait_loop.quit();
        });

        wait_loop.run();
        if let Ok(pos) = value.lock() {
            if let Some(pos) = *pos {
                return Ok(pos);
            }
        }
        Err(format_err!("some err"))
    }

    fn set_scroll_abs(&self, scroll: f64) -> Result<(), Error> {
        let view_name = (*self.internal_state.borrow()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    Self::set_scroll_pos_static(&view, scroll)?;
                    return Ok(());
                }
            }
        }
        Err(format_err!("some err"))
    }

    fn get_scroll_abs(&self) -> Result<f64, Error> {
        let view_name = (*self.internal_state.borrow()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    return Self::get_scroll_pos_static(&view);
                }
            }
        }
        Err(format_err!("some err"))
    }

    fn get_scroll_window_height(&self) -> Result<f64, Error> {
        let view_name = (*self.internal_state.borrow()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    return Self::get_scroll_window_height_static(&view);
                }
            }
        }
        Err(format_err!("some err"))
    }

    fn get_scroll_upper(&self) -> Result<f64, Error> {
        let view_name = (*self.internal_state.borrow()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    return Self::get_scroll_upper_static(&view);
                }
            }
        }
        Err(format_err!("some err"))
    }

    pub fn animate_scroll_diff(&self, diff: f64) -> Result<(), Error> {
        let pos = self.get_scroll_abs()?;
        let upper = self.get_scroll_upper()?;
        let window_height = self.get_scroll_window_height()?;

        if pos <= 0.0 && diff.is_sign_negative() {
            return Ok(());
        } else if pos >= (upper - window_height) && diff.is_sign_positive() {
            return Ok(());
        }

        self.animate_scroll_absolute(pos + diff, pos)
    }

    pub fn animate_scroll_absolute(&self, pos: f64, current_pos: f64) -> Result<(), Error> {
        let animate = match gtk::Settings::get_default() {
            Some(settings) => settings.get_property_gtk_enable_animations(),
            None => false,
        };

        if !self.widget().get_mapped() || !animate {
            return self.set_scroll_abs(pos);
        }

        *self.scroll_animation_data.start_time.borrow_mut() =
            self.widget().get_frame_clock().map(|clock| clock.get_frame_time());
        *self.scroll_animation_data.end_time.borrow_mut() = self
            .widget()
            .get_frame_clock()
            .map(|clock| clock.get_frame_time() + SCROLL_TRANSITION_DURATION);

        let callback_id = self.scroll_animation_data.scroll_callback_id.replace(None);
        let leftover_scroll = match callback_id {
            Some(callback_id) => {
                callback_id.remove();
                let start_value =
                    Util::some_or_default(*self.scroll_animation_data.transition_start_value.borrow(), 0.0);
                let diff_value = Util::some_or_default(*self.scroll_animation_data.transition_diff.borrow(), 0.0);
                start_value + diff_value - current_pos
            }
            None => 0.0,
        };

        *self.scroll_animation_data.transition_diff.borrow_mut() = Some(if pos == -1.0 {
            self.get_scroll_upper()? - self.get_scroll_window_height()? - current_pos
        } else {
            (pos - current_pos) + leftover_scroll
        });

        *self.scroll_animation_data.transition_start_value.borrow_mut() = Some(current_pos);

        let view_name = (*self.internal_state.borrow())
            .to_str()
            .map(|s| s.to_owned())
            .ok_or(format_err!("some err"))?;
        let view = self
            .stack
            .get_child_by_name(&view_name)
            .ok_or(format_err!("some err"))?;

        let scroll_animation_data = self.scroll_animation_data.clone();
        *self.scroll_animation_data.scroll_callback_id.borrow_mut() =
            Some(view.add_tick_callback(move |widget, clock| {
                let view = widget
                    .clone()
                    .downcast::<WebView>()
                    .expect("Scroll tick not on WebView");

                let start_value = Util::some_or_default(*scroll_animation_data.transition_start_value.borrow(), 0.0);
                let diff_value = Util::some_or_default(*scroll_animation_data.transition_diff.borrow(), 0.0);
                let now = clock.get_frame_time();
                let end_time_value = Util::some_or_default(*scroll_animation_data.end_time.borrow(), 0);
                let start_time_value = Util::some_or_default(*scroll_animation_data.start_time.borrow(), 0);

                if !widget.get_mapped() {
                    Self::set_scroll_pos_static(&view, start_value + diff_value).unwrap();
                    return Continue(false);
                }

                if scroll_animation_data.end_time.borrow().is_none() {
                    return Continue(false);
                }

                let t = if now < end_time_value {
                    (now - start_time_value) as f64 / (end_time_value - start_time_value) as f64
                } else {
                    1.0
                };

                let t = Util::ease_out_cubic(t);

                Self::set_scroll_pos_static(&view, start_value + (t * diff_value)).unwrap();

                let pos = Self::get_scroll_pos_static(&view).unwrap();
                let upper = Self::get_scroll_upper_static(&view).unwrap();
                if pos <= 0.0 || pos >= upper || now >= end_time_value {
                    Self::stop_scroll_animation(&view, &scroll_animation_data);
                    return Continue(false);
                }

                Continue(true)
            }));

        Ok(())
    }

    fn stop_scroll_animation(view: &WebView, properties: &ScrollAnimationProperties) {
        if let Some(callback_id) = properties.scroll_callback_id.replace(None) {
            callback_id.remove();
        }
        view.queue_draw();
        *properties.transition_start_value.borrow_mut() = None;
        *properties.transition_diff.borrow_mut() = None;
        *properties.start_time.borrow_mut() = None;
        *properties.end_time.borrow_mut() = None;
    }
}
