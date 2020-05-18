mod error;
mod models;
mod progress_overlay;
mod url_overlay;

use self::error::{ArticleViewError, ArticleViewErrorKind};
pub use self::models::ArticleTheme;
use self::models::InternalState;
use self::progress_overlay::ProgressOverlay;
use self::url_overlay::UrlOverlay;
use crate::app::Action;
use crate::settings::Settings;
use crate::util::{BuilderHelper, DateUtil, FileUtil, GtkUtil, Util, GTK_RESOURCE_FILE_ERROR};
use crate::Resources;
use gdk::{
    enums::key::KP_Add as KP_ADD, enums::key::KP_Subtract as KP_SUBTRACT, enums::key::KP_0, Cursor, CursorType,
    Display, EventMask, ModifierType, ScrollDirection,
};
use gio::{Cancellable, Settings as GSettings, SettingsExt as GSettingsExt};
use glib::{clone, object::Cast, source::Continue, translate::ToGlib, MainLoop, Sender};
use gtk::{
    prelude::WidgetExtManual, Button, ButtonExt, Inhibit, Overlay, OverlayExt, SettingsExt as GtkSettingsExt, Stack,
    StackExt, TickCallbackId, WidgetExt,
};
use log::{error, warn};
use news_flash::models::{Direction, FatArticle, Marked, Read};
use pango::FontDescription;
use parking_lot::RwLock;
use std::str;
use std::sync::Arc;
use url::{Host, Origin};
use webkit2gtk::{
    ContextMenuAction, ContextMenuExt, ContextMenuItemExt, HitTestResultExt, NavigationPolicyDecision,
    NavigationPolicyDecisionExt, PolicyDecisionExt, PolicyDecisionType, Settings as WebkitSettings, SettingsExt,
    URIRequestExt, WebContext, WebView, WebViewExt,
};

const MIDDLE_MOUSE_BUTTON: u32 = 2;
const SCROLL_TRANSITION_DURATION: i64 = 500 * 1000;

#[derive(Clone)]
struct ScrollAnimationProperties {
    pub start_time: Arc<RwLock<Option<i64>>>,
    pub end_time: Arc<RwLock<Option<i64>>>,
    pub scroll_callback_id: Arc<RwLock<Option<TickCallbackId>>>,
    pub transition_start_value: Arc<RwLock<Option<f64>>>,
    pub transition_diff: Arc<RwLock<Option<f64>>>,
}

#[derive(Clone)]
pub struct ArticleView {
    settings: Arc<RwLock<Settings>>,
    sender: Sender<Action>,
    stack: Stack,
    top_overlay: Overlay,
    view_html_button: Button,
    visible_article: Arc<RwLock<Option<FatArticle>>>,
    visible_feed_name: Arc<RwLock<Option<String>>>,
    internal_state: Arc<RwLock<InternalState>>,
    load_changed_signal: Arc<RwLock<Option<u64>>>,
    decide_policy_signal: Arc<RwLock<Option<u64>>>,
    mouse_over_signal: Arc<RwLock<Option<u64>>>,
    scroll_signal: Arc<RwLock<Option<u64>>>,
    key_press_signal: Arc<RwLock<Option<u64>>>,
    ctx_menu_signal: Arc<RwLock<Option<u64>>>,
    load_signal: Arc<RwLock<Option<u64>>>,
    click_signal: Arc<RwLock<Option<u64>>>,
    click_release_signal: Arc<RwLock<Option<u64>>>,
    drag_motion_notify_signal: Arc<RwLock<Option<u64>>>,
    drag_released_motion_signal: Arc<RwLock<Option<u32>>>,
    drag_buffer_update_signal: Arc<RwLock<Option<u32>>>,
    progress_overlay_delay_signal: Arc<RwLock<Option<u32>>>,
    url_overlay_label: Arc<RwLock<UrlOverlay>>,
    progress_overlay_label: Arc<RwLock<ProgressOverlay>>,
    drag_buffer: Arc<RwLock<[f64; 10]>>,
    drag_ongoing: Arc<RwLock<bool>>,
    drag_y_pos: Arc<RwLock<f64>>,
    drag_momentum: Arc<RwLock<f64>>,
    pointer_pos: Arc<RwLock<(f64, f64)>>,
    scroll_animation_data: Arc<ScrollAnimationProperties>,
    web_context: WebContext,
}

impl ArticleView {
    pub fn new(settings: &Arc<RwLock<Settings>>, sender: &Sender<Action>) -> Self {
        let builder = BuilderHelper::new("article_view");

        let url_overlay = builder.get::<Overlay>("url_overlay");
        let url_overlay_label = UrlOverlay::new();
        url_overlay.add_overlay(&url_overlay_label.widget());

        let progress_overlay = builder.get::<Overlay>("progress_overlay");
        let progress_overlay_label = ProgressOverlay::new();
        progress_overlay.add_overlay(&progress_overlay_label.widget());

        let visible_article: Arc<RwLock<Option<FatArticle>>> = Arc::new(RwLock::new(None));
        let visible_feed_name: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
        let view_html_button = builder.get::<Button>("view_html_button");
        view_html_button.connect_clicked(
            clone!(@strong visible_article, @strong sender => @default-panic, move |_button| {
                if let Some(article) = visible_article.read().as_ref() {
                    if let Some(html) = &article.html {
                        if let Ok(path) = FileUtil::write_temp_file("crashed_article.html", html) {
                            if let Some(path) = path.to_str() {
                                let uri = format!("file://{}", path);
                                Util::send(&sender, Action::OpenUrlInDefaultBrowser(uri));
                            }
                        }
                    }
                }
            }),
        );

        let web_context = WebContext::new();
        // FIXME: apply appliction wide proxy settings

        let stack = builder.get::<Stack>("article_view_stack");
        stack.set_visible_child_name("empty");
        let view_1 = Self::new_webview(&web_context);
        let view_2 = Self::new_webview(&web_context);
        view_1.load_html("", None);
        view_2.load_html("", None);
        stack.add_named(&view_1, InternalState::View1.to_str().unwrap());
        stack.add_named(&view_2, InternalState::View2.to_str().unwrap());

        let internal_state = InternalState::Empty;
        let settings = settings.clone();

        let article_view = ArticleView {
            settings,
            sender: sender.clone(),
            stack,
            top_overlay: progress_overlay,
            view_html_button,
            visible_article,
            visible_feed_name,
            internal_state: Arc::new(RwLock::new(internal_state)),
            load_changed_signal: Arc::new(RwLock::new(None)),
            decide_policy_signal: Arc::new(RwLock::new(None)),
            mouse_over_signal: Arc::new(RwLock::new(None)),
            scroll_signal: Arc::new(RwLock::new(None)),
            key_press_signal: Arc::new(RwLock::new(None)),
            ctx_menu_signal: Arc::new(RwLock::new(None)),
            load_signal: Arc::new(RwLock::new(None)),
            click_signal: Arc::new(RwLock::new(None)),
            click_release_signal: Arc::new(RwLock::new(None)),
            drag_motion_notify_signal: Arc::new(RwLock::new(None)),
            drag_released_motion_signal: Arc::new(RwLock::new(None)),
            drag_buffer_update_signal: Arc::new(RwLock::new(None)),
            progress_overlay_delay_signal: Arc::new(RwLock::new(None)),
            url_overlay_label: Arc::new(RwLock::new(url_overlay_label)),
            progress_overlay_label: Arc::new(RwLock::new(progress_overlay_label)),
            drag_buffer: Arc::new(RwLock::new([0.0; 10])),
            drag_ongoing: Arc::new(RwLock::new(false)),
            drag_y_pos: Arc::new(RwLock::new(0.0)),
            drag_momentum: Arc::new(RwLock::new(0.0)),
            pointer_pos: Arc::new(RwLock::new((0.0, 0.0))),
            scroll_animation_data: Arc::new(ScrollAnimationProperties {
                start_time: Arc::new(RwLock::new(None)),
                end_time: Arc::new(RwLock::new(None)),
                scroll_callback_id: Arc::new(RwLock::new(None)),
                transition_start_value: Arc::new(RwLock::new(None)),
                transition_diff: Arc::new(RwLock::new(None)),
            }),
            web_context,
        };

        article_view.stack.show_all();
        article_view
    }

    pub fn widget(&self) -> gtk::Overlay {
        self.top_overlay.clone()
    }

    pub fn show_article(&self, article: FatArticle, feed_name: String) {
        let webview = self.switch_view().unwrap();
        let html = self.build_article(&article, &feed_name);
        webview.load_html(&html, Self::get_base_url(&article).as_deref());
        self.visible_article.write().replace(article);
        self.visible_feed_name.write().replace(feed_name);
    }

    pub fn redraw_article(&self) {
        if let Some(article) = &*self.visible_article.read() {
            if let Some(feed_name) = &*self.visible_feed_name.read() {
                let html = self.build_article(&article, feed_name);

                let webview = self.switch_view().unwrap();
                webview.load_html(&html, Self::get_base_url(&article).as_deref());
                return;
            }
        }

        warn!("Can't redraw article view. No article is on display.");
    }

    fn get_base_url(article: &FatArticle) -> Option<String> {
        if let Some(url) = &article.url {
            match url.get().origin() {
                Origin::Opaque(_op) => None,
                Origin::Tuple(scheme, host, port) => {
                    let host = match host {
                        Host::Domain(domain) => domain,
                        Host::Ipv4(ipv4) => ipv4.to_string(),
                        Host::Ipv6(ipv6) => ipv6.to_string(),
                    };
                    Some(format!("{}://{}:{}", scheme, host, port))
                }
            }
        } else {
            None
        }
    }

    pub fn get_visible_article(&self) -> Option<FatArticle> {
        (*self.visible_article.read()).clone()
    }

    pub fn update_visible_article(&self, read: Option<Read>, marked: Option<Marked>) {
        if let Some(visible_article) = &mut *self.visible_article.write() {
            if let Some(marked) = marked {
                visible_article.marked = marked;
            }
            if let Some(read) = read {
                visible_article.unread = read;
            }
        }
    }

    pub fn close_article(&self) {
        self.disconnect_old_view();
        self.visible_article.write().take();
        self.visible_feed_name.write().take();
        *self.internal_state.write() = InternalState::Empty;
        self.stack.set_visible_child_name("empty");
    }

    fn switch_view(&self) -> Result<WebView, ArticleViewError> {
        self.disconnect_old_view();
        let old_state = (*self.internal_state.read()).clone();
        *self.internal_state.write() = old_state.switch();
        if let Some(new_name) = self.internal_state.read().to_str() {
            if let Some(webview) = self.stack.get_child_by_name(new_name) {
                let webview = webview.downcast::<WebView>().unwrap();
                self.connect_webview(&webview);
                self.stack.set_visible_child_name(new_name);
                return Ok(webview);
            }
        }

        Err(ArticleViewErrorKind::Unknown)?
    }

    fn disconnect_old_view(&self) {
        Self::disconnect_old_view_static(
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
    fn disconnect_old_view_static(
        progress_overlay_label: &Arc<RwLock<ProgressOverlay>>,
        old_state: &Arc<RwLock<InternalState>>,
        stack: &gtk::Stack,
        load_changed_signal: &Arc<RwLock<Option<u64>>>,
        decide_policy_signal: &Arc<RwLock<Option<u64>>>,
        mouse_over_signal: &Arc<RwLock<Option<u64>>>,
        scroll_signal: &Arc<RwLock<Option<u64>>>,
        key_press_signal: &Arc<RwLock<Option<u64>>>,
        ctx_menu_signal: &Arc<RwLock<Option<u64>>>,
        load_signal: &Arc<RwLock<Option<u64>>>,
        click_signal: &Arc<RwLock<Option<u64>>>,
        click_release_signal: &Arc<RwLock<Option<u64>>>,
        drag_released_motion_signal: &Arc<RwLock<Option<u32>>>,
        drag_buffer_update_signal: &Arc<RwLock<Option<u32>>>,
        progress_overlay_delay_signal: &Arc<RwLock<Option<u32>>>,
    ) {
        let old_state = (*old_state.read()).clone();
        progress_overlay_label.read().reveal(false);

        GtkUtil::remove_source(*drag_released_motion_signal.read());
        GtkUtil::remove_source(*drag_buffer_update_signal.read());
        GtkUtil::remove_source(*progress_overlay_delay_signal.read());
        drag_released_motion_signal.write().take();
        drag_buffer_update_signal.write().take();
        progress_overlay_delay_signal.write().take();

        // disconnect signals
        if let Some(old_state) = old_state.to_str() {
            if let Some(old_webview) = stack.get_child_by_name(old_state) {
                GtkUtil::disconnect_signal(*load_changed_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*decide_policy_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*mouse_over_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*scroll_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*key_press_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*ctx_menu_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*load_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*click_signal.read(), &old_webview);
                GtkUtil::disconnect_signal(*click_release_signal.read(), &old_webview);
                load_changed_signal.write().take();
                decide_policy_signal.write().take();
                mouse_over_signal.write().take();
                scroll_signal.write().take();
                key_press_signal.write().take();
                ctx_menu_signal.write().take();
                load_signal.write().take();
                click_signal.write().take();
                click_release_signal.write().take();
            }
        }

        if let Some(old_state) = old_state.to_str() {
            if let Some(old_view) = stack.get_child_by_name(&old_state) {
                if let Ok(webview) = old_view.downcast::<WebView>() {
                    webview.load_html("", None);
                }
            }
        }
    }

    fn new_webview(ctx: &WebContext) -> WebView {
        let settings = WebkitSettings::new();
        // settings.set_enable_accelerated_2d_canvas(true);
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

        let webview = WebView::new_with_context(ctx);
        webview.set_settings(&settings);
        webview.set_events(EventMask::POINTER_MOTION_MASK);
        webview.set_events(EventMask::SCROLL_MASK);
        webview.set_events(EventMask::BUTTON_PRESS_MASK);
        webview.set_events(EventMask::BUTTON_RELEASE_MASK);
        webview.set_events(EventMask::KEY_PRESS_MASK);
        webview
    }

    fn connect_webview(&self, webview: &WebView) {
        //----------------------------------
        // open link in external browser
        //----------------------------------
        let policy_sender = self.sender.clone();
        self.decide_policy_signal.write().replace(
            webview
                .connect_decide_policy(move |_closure_webivew, decision, decision_type| {
                    if decision_type == PolicyDecisionType::NewWindowAction {
                        if let Ok(navigation_decision) = decision.clone().downcast::<NavigationPolicyDecision>() {
                            if let Some(frame_name) = navigation_decision.get_frame_name() {
                                if &frame_name == "_blank" {
                                    if let Some(action) = navigation_decision.get_navigation_action() {
                                        if let Some(uri_req) = action.get_request() {
                                            if let Some(uri) = uri_req.get_uri() {
                                                Util::send(
                                                    &policy_sender,
                                                    Action::OpenUrlInDefaultBrowser(uri.as_str().into()),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        decision.ignore();
                        return true;
                    } else if decision_type == PolicyDecisionType::NavigationAction {
                        if let Ok(navigation_decision) = decision.clone().downcast::<NavigationPolicyDecision>() {
                            if let Some(action) = navigation_decision.get_navigation_action() {
                                if let Some(request) = action.get_request() {
                                    if let Some(uri) = request.get_uri() {
                                        if action.is_user_gesture() {
                                            decision.ignore();
                                            Util::send(
                                                &policy_sender,
                                                Action::OpenUrlInDefaultBrowser(uri.as_str().into()),
                                            );
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
        self.mouse_over_signal.write().replace(
            webview
                .connect_mouse_target_changed(clone!(
                    @weak self.url_overlay_label as url_overlay_label,
                    @weak self.pointer_pos as pointer_pos,
                    @weak self.stack as stack => @default-panic, move |_closure_webivew, hit_test, _modifiers|
                {
                    if hit_test.context_is_link() {
                        if let Some(uri) = hit_test.get_link_uri() {
                            let allocation = stack.get_allocation();
                            let rel_x = pointer_pos.read().0 / f64::from(allocation.width);
                            let rel_y = pointer_pos.read().1 / f64::from(allocation.height);

                            let align = if rel_x <= 0.5 && rel_y >= 0.85 {
                                gtk::Align::End
                            } else {
                                gtk::Align::Start
                            };

                            url_overlay_label.read().set_url(uri.as_str().to_owned(), align);
                            url_overlay_label.read().reveal(true);
                        }
                    } else {
                        url_overlay_label.read().reveal(false);
                    }
                }))
                .to_glib(),
        );

        //----------------------------------
        // zoom with ctrl+scroll
        //----------------------------------
        self.scroll_signal.write().replace(
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
        self.key_press_signal.write().replace(
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
                    }
                    Inhibit(true)
                })
                .to_glib(),
        );

        //----------------------------------
        // clean up context menu
        //----------------------------------
        self.ctx_menu_signal.write().replace(
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
                            | ContextMenuAction::CopyImageUrlToClipboard
                            | ContextMenuAction::DownloadImageToDisk => {}
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
        self.progress_overlay_delay_signal.write().replace(
            gtk::timeout_add(1500, clone!(
                @weak webview,
                @weak self.progress_overlay_label as progress_handle,
                @weak self.progress_overlay_delay_signal as progress_overlay_delay_signal,
                @weak self.load_signal as load_signal => @default-panic, move ||
            {
                progress_overlay_delay_signal.write().take();
                if (webview.get_estimated_load_progress() - 1.0).abs() < 0.01 {
                    return Continue(false);
                }

                load_signal.write().replace(
                    webview
                        .connect_property_estimated_load_progress_notify(clone!(@weak progress_handle => @default-panic, move |closure_webivew| {
                            let progress = closure_webivew.get_estimated_load_progress();
                            if progress >= 1.0 {
                                progress_handle.read().reveal(false);
                                return;
                            }
                            progress_handle.read().reveal(true);
                            progress_handle.read().set_percentage(progress);
                        }))
                        .to_glib(),
                );
                Continue(false)
            }))
            .to_glib(),
        );

        //----------------------------------
        // drag page
        //----------------------------------
        self.click_signal.write().replace(
            webview
                .connect_button_press_event(clone!(
                    @weak self.top_overlay as widget,
                    @weak self.drag_ongoing as drag_ongoing,
                    @weak self.drag_y_pos as drag_y_pos,
                    @weak self.drag_momentum as drag_momentum,
                    @weak self.drag_motion_notify_signal as drag_motion_notify_signal,
                    @weak self.drag_buffer_update_signal as drag_buffer_update_signal,
                    @weak self.scroll_animation_data as scroll_animation_data,
                    @weak self.drag_buffer as drag_buffer => @default-panic, move |closure_webview, event|
                {
                    if event.get_button() == MIDDLE_MOUSE_BUTTON {
                        Self::stop_scroll_animation(&closure_webview, &scroll_animation_data);
                        let (_, y) = event.get_position();
                        *drag_y_pos.write() = y;
                        *drag_buffer.write() = [y; 10];
                        *drag_ongoing.write() = true;

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
                                        drag_buffer_update_signal.write().replace(
                                            gtk::timeout_add(10, clone!(
                                                @weak drag_ongoing,
                                                @weak drag_y_pos,
                                                @weak drag_momentum,
                                                @weak drag_buffer_update_signal,
                                                @weak drag_buffer => @default-panic, move ||
                                            {
                                                if !*drag_ongoing.read() {
                                                    drag_buffer_update_signal.write().take();
                                                    return Continue(false);
                                                }

                                                for i in (1..10).rev() {
                                                    let value = (*drag_buffer.read())[i - 1];
                                                    (*drag_buffer.write())[i] = value;
                                                }

                                                (*drag_buffer.write())[0] = *drag_y_pos.read();
                                                *drag_momentum.write() =
                                                    (*drag_buffer.read())[9] - (*drag_buffer.read())[0];
                                                Continue(true)
                                            }))
                                            .to_glib(),
                                        );

                                        drag_motion_notify_signal.write().replace(
                                            closure_webview
                                                .connect_motion_notify_event(clone!(@weak drag_y_pos => @default-panic, move |view, event| {
                                                    let (_, y) = event.get_position();
                                                    let scroll = *drag_y_pos.read() - y;
                                                    *drag_y_pos.write() = y;
                                                    let scroll_pos = Self::get_scroll_pos_static(view);
                                                    Self::set_scroll_pos_static(view, scroll_pos + scroll);
                                                    Inhibit(false)
                                                }))
                                                .to_glib(),
                                        );
                                    }
                                }
                            }
                        }
                        return Inhibit(true);
                    }
                    Inhibit(false)
                }))
                .to_glib(),
        );

        self.click_release_signal.write().replace(
            webview
                .connect_button_release_event(clone!(
                    @weak self.drag_ongoing as drag_ongoing,
                    @weak self.drag_momentum as drag_momentum,
                    @weak self.drag_motion_notify_signal as drag_motion_notify_signal,
                    @weak self.drag_released_motion_signal as drag_released_motion_signal,
                    @weak self.top_overlay as widget => @default-panic, move |view, event|
                {
                    if event.get_button() == MIDDLE_MOUSE_BUTTON {
                        GtkUtil::disconnect_signal(*drag_motion_notify_signal.read(), view);
                        *drag_ongoing.write() = false;

                        drag_released_motion_signal.write().replace(
                            gtk::timeout_add(20, clone!(
                                @weak view,
                                @weak drag_released_motion_signal,
                                @weak drag_momentum => @default-panic, move ||
                            {
                                *drag_momentum.write() /= 1.2;
                                let allocation = view.get_allocation();

                                let page_size = f64::from(view.get_allocated_height());
                                let adjust_value = page_size * *drag_momentum.read() / f64::from(allocation.height);
                                let old_adjust = Self::get_scroll_pos_static(&view);
                                let upper = Self::get_scroll_upper_static(&view) * view.get_zoom_level();

                                if (old_adjust + adjust_value) > (upper - page_size)
                                    || (old_adjust + adjust_value) < 0.0
                                {
                                    *drag_momentum.write() = 0.0;
                                }

                                let new_scroll_pos = f64::min(old_adjust + adjust_value, upper - page_size);
                                Self::set_scroll_pos_static(&view, new_scroll_pos);

                                if drag_momentum.read().abs() < 1.0 {
                                    drag_released_motion_signal.write().take();
                                    return Continue(false);
                                }

                                Continue(true)
                            }))
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
                }))
                .to_glib(),
        );

        //----------------------------------
        // crash view
        //----------------------------------
        webview.connect_web_process_crashed(clone!(
            @weak self.progress_overlay_delay_signal as progress_overlay_delay_signal,
            @weak self.drag_buffer_update_signal as drag_buffer_update_signal,
            @weak self.drag_released_motion_signal as drag_released_motion_signal,
            @weak self.click_release_signal as click_release_signal,
            @weak self.click_signal as click_signal,
            @weak self.load_signal as load_signal,
            @weak self.ctx_menu_signal as ctx_menu_signal,
            @weak self.key_press_signal as key_press_signal,
            @weak self.scroll_signal as scroll_signal,
            @weak self.mouse_over_signal as mouse_over_signal,
            @weak self.decide_policy_signal as decide_policy_signal,
            @weak self.load_changed_signal as load_changed_signal,
            @weak self.internal_state as internal_state,
            @weak self.progress_overlay_label as progress_overlay_label,
            @weak self.stack as stack => @default-panic, move |_closure_webivew|
        {
            Self::disconnect_old_view_static(
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
            *internal_state.write() = InternalState::Crash;
            false
        }));

        webview.connect_motion_notify_event(
            clone!(@weak self.pointer_pos as pointer_pos => @default-panic, move |_closure_webivew, event| {
                *pointer_pos.write() = event.get_position();
                Inhibit(false)
            }),
        );

        // webview.enter_fullscreen.connect(enterFullscreenVideo);
        // webview.leave_fullscreen.connect(leaveFullscreenVideo);
    }

    fn build_article(&self, article: &FatArticle, feed_name: &str) -> String {
        Self::build_article_static("article", article, feed_name, &self.settings, None, None)
    }

    pub fn build_article_static(
        file_name: &str,
        article: &FatArticle,
        feed_name: &str,
        settings: &Arc<RwLock<Settings>>,
        theme_override: Option<ArticleTheme>,
        font_size_override: Option<i32>,
    ) -> String {
        let template_data = Resources::get(&format!("article_view/{}.html", file_name)).expect(GTK_RESOURCE_FILE_ERROR);
        let template_str = str::from_utf8(template_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);
        let mut template_string = template_str.to_owned();

        let css_data = Resources::get("article_view/style.css").expect(GTK_RESOURCE_FILE_ERROR);
        let css_string = str::from_utf8(css_data.as_ref()).expect("Failed to load CSS from resources");
        let direction_string = match &article.direction {
            Some(direction) => match direction {
                Direction::LeftToRight => "ltr",
                Direction::RightToLeft => "rtl",
            },
            None => "ltr",
        };

        // A list of fonts we should try to use in order of preference
        // We will pass all of these to CSS in order
        let mut font_options: Vec<String> = Vec::new();
        let mut font_families: Vec<String> = Vec::new();
        let mut font_size: Option<i32> = None;

        // Try to use the configured font if it exists
        if let Some(font_setting) = settings.read().get_article_view_font() {
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
            if font_size.is_none() && desc.get_size() > 0 {
                font_size = Some(desc.get_size());
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
        if settings.read().get_article_view_allow_select() {
            template_string = template_string.replacen("$UNSELECTABLE", "", 2);
        } else {
            template_string = template_string.replacen("$UNSELECTABLE", "unselectable", 2);
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
            theme_override
                .to_str(settings.read().get_prefer_dark_theme())
                .to_owned()
        } else {
            settings
                .read()
                .get_article_view_theme()
                .to_str(settings.read().get_prefer_dark_theme())
                .to_owned()
        };
        template_string = template_string.replacen("$THEME", &theme, 1);

        // $FONTFAMILY
        template_string = template_string.replacen("$FONTFAMILY", &font_family, 1);

        // $FONTSIZE
        template_string = template_string.replacen("$FONTSIZE", &format!("{}", font_size), 1);

        // $CSS
        template_string = template_string.replacen("$CSS", &css_string, 1);

        // $DIRECTION
        template_string = template_string.replacen("$DIRECTION", &direction_string, 1);

        template_string
    }

    fn set_scroll_pos_static(view: &WebView, pos: f64) {
        let cancellable: Option<&Cancellable> = None;
        view.run_javascript(&format!("window.scrollTo(0,{});", pos), cancellable, |res| match res {
            Ok(_) => {}
            Err(_) => error!("Setting scroll pos failed"),
        });
    }

    fn get_scroll_pos_static(view: &WebView) -> f64 {
        Self::webview_js_get_f64(view, "window.scrollY").expect("Failed to get scroll position from webview.")
    }

    fn get_scroll_window_height_static(view: &WebView) -> f64 {
        Self::webview_js_get_f64(view, "window.innerHeight").expect("Failed to get window height from webview.")
    }

    fn get_scroll_upper_static(view: &WebView) -> f64 {
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
        .expect("Failed to get upper limit from webview.")
    }

    fn webview_js_get_f64(view: &WebView, java_script: &str) -> Result<f64, ArticleViewError> {
        let wait_loop = Arc::new(MainLoop::new(None, false));
        let value: Arc<RwLock<Option<f64>>> = Arc::new(RwLock::new(None));
        let cancellable: Option<&Cancellable> = None;
        view.run_javascript(
            java_script,
            cancellable,
            clone!(@weak wait_loop, @weak value => @default-panic, move |res| {
                match res {
                    Ok(result) => {
                        let context = result.get_global_context().expect("Failed to get webkit js context.");
                        let new_value = result.get_value().expect("Failed to get value from js result.");
                        *value.write() = new_value.to_number(&context);
                    }
                    Err(_) => error!("Getting scroll pos failed"),
                }
                wait_loop.quit();
            }),
        );

        wait_loop.run();

        if let Some(pos) = *value.clone().read() {
            return Ok(pos);
        } else {
            return Err(ArticleViewErrorKind::NoValueFromJS.into());
        }
    }

    fn set_scroll_abs(&self, scroll: f64) -> Result<(), ArticleViewError> {
        let view_name = (*self.internal_state.read()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    Self::set_scroll_pos_static(&view, scroll);
                    Ok(())
                } else {
                    Err(ArticleViewErrorKind::InvalidActiveWebView.into())
                }
            } else {
                Err(ArticleViewErrorKind::InvalidActiveWebView.into())
            }
        } else {
            Err(ArticleViewErrorKind::NoActiveWebView.into())
        }
    }

    fn get_scroll_abs(&self) -> Result<f64, ArticleViewError> {
        let view_name = (*self.internal_state.read()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    Ok(Self::get_scroll_pos_static(&view))
                } else {
                    Err(ArticleViewErrorKind::InvalidActiveWebView.into())
                }
            } else {
                Err(ArticleViewErrorKind::InvalidActiveWebView.into())
            }
        } else {
            Err(ArticleViewErrorKind::NoActiveWebView.into())
        }
    }

    fn get_scroll_window_height(&self) -> Result<f64, ArticleViewError> {
        let view_name = (*self.internal_state.read()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    Ok(Self::get_scroll_window_height_static(&view))
                } else {
                    Err(ArticleViewErrorKind::InvalidActiveWebView.into())
                }
            } else {
                Err(ArticleViewErrorKind::InvalidActiveWebView.into())
            }
        } else {
            Err(ArticleViewErrorKind::NoActiveWebView.into())
        }
    }

    fn get_scroll_upper(&self) -> Result<f64, ArticleViewError> {
        let view_name = (*self.internal_state.read()).to_str().map(|s| s.to_owned());
        if let Some(view_name) = view_name {
            if let Some(view) = self.stack.get_child_by_name(&view_name) {
                if let Ok(view) = view.downcast::<WebView>() {
                    Ok(Self::get_scroll_upper_static(&view))
                } else {
                    Err(ArticleViewErrorKind::InvalidActiveWebView.into())
                }
            } else {
                Err(ArticleViewErrorKind::InvalidActiveWebView.into())
            }
        } else {
            Err(ArticleViewErrorKind::NoActiveWebView.into())
        }
    }

    pub fn animate_scroll_diff(&self, diff: f64) -> Result<(), ArticleViewError> {
        let pos = self.get_scroll_abs()?;
        let upper = self.get_scroll_upper()?;
        let window_height = self.get_scroll_window_height()?;

        if (pos <= 0.0 && diff.is_sign_negative()) || (pos >= (upper - window_height) && diff.is_sign_positive()) {
            return Ok(());
        }

        self.animate_scroll_absolute(pos + diff, pos)
    }

    pub fn animate_scroll_absolute(&self, pos: f64, current_pos: f64) -> Result<(), ArticleViewError> {
        let animate = match gtk::Settings::get_default() {
            Some(settings) => settings.get_property_gtk_enable_animations(),
            None => false,
        };

        if !self.widget().get_mapped() || !animate {
            return self.set_scroll_abs(pos);
        }

        *self.scroll_animation_data.start_time.write() =
            self.widget().get_frame_clock().map(|clock| clock.get_frame_time());
        *self.scroll_animation_data.end_time.write() = self
            .widget()
            .get_frame_clock()
            .map(|clock| clock.get_frame_time() + SCROLL_TRANSITION_DURATION);

        let callback_id = self.scroll_animation_data.scroll_callback_id.write().take();
        let leftover_scroll = match callback_id {
            Some(callback_id) => {
                callback_id.remove();
                let start_value = Util::some_or_default(*self.scroll_animation_data.transition_start_value.read(), 0.0);
                let diff_value = Util::some_or_default(*self.scroll_animation_data.transition_diff.read(), 0.0);
                start_value + diff_value - current_pos
            }
            None => 0.0,
        };

        self.scroll_animation_data
            .transition_diff
            .write()
            .replace(if (pos + 1.0).abs() < 0.001 {
                self.get_scroll_upper()? - self.get_scroll_window_height()? - current_pos
            } else {
                (pos - current_pos) + leftover_scroll
            });

        self.scroll_animation_data
            .transition_start_value
            .write()
            .replace(current_pos);

        let view_name = (*self.internal_state.read())
            .to_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| ArticleViewErrorKind::NoActiveWebView)?;
        let view = self
            .stack
            .get_child_by_name(&view_name)
            .ok_or_else(|| ArticleViewErrorKind::InvalidActiveWebView)?;

        self.scroll_animation_data
            .scroll_callback_id
            .write()
            .replace(view.add_tick_callback(clone!(
                @weak self.scroll_animation_data as scroll_animation_data => @default-panic, move |widget, clock|
            {
                let view = widget
                    .clone()
                    .downcast::<WebView>()
                    .expect("Scroll tick not on WebView");

                let start_value = Util::some_or_default(*scroll_animation_data.transition_start_value.read(), 0.0);
                let diff_value = Util::some_or_default(*scroll_animation_data.transition_diff.read(), 0.0);
                let now = clock.get_frame_time();
                let end_time_value = Util::some_or_default(*scroll_animation_data.end_time.read(), 0);
                let start_time_value = Util::some_or_default(*scroll_animation_data.start_time.read(), 0);

                if !widget.get_mapped() {
                    Self::set_scroll_pos_static(&view, start_value + diff_value);
                    return Continue(false);
                }

                if scroll_animation_data.end_time.read().is_none() {
                    return Continue(false);
                }

                let t = if now < end_time_value {
                    (now - start_time_value) as f64 / (end_time_value - start_time_value) as f64
                } else {
                    1.0
                };

                let t = Util::ease_out_cubic(t);

                Self::set_scroll_pos_static(&view, start_value + (t * diff_value));

                let pos = Self::get_scroll_pos_static(&view);
                let upper = Self::get_scroll_upper_static(&view);
                if pos <= 0.0 || pos >= upper || now >= end_time_value {
                    Self::stop_scroll_animation(&view, &scroll_animation_data);
                    return Continue(false);
                }

                Continue(true)
            })));

        Ok(())
    }

    fn stop_scroll_animation(view: &WebView, properties: &ScrollAnimationProperties) {
        if let Some(callback_id) = properties.scroll_callback_id.write().take() {
            callback_id.remove();
        }
        view.queue_draw();
        properties.transition_start_value.write().take();
        properties.transition_diff.write().take();
        properties.start_time.write().take();
        properties.end_time.write().take();
    }
}
