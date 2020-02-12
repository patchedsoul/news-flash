mod related_topic_row;
mod search_item_row;

use self::related_topic_row::RelatedTopicRow;
use self::search_item_row::SearchItemRow;
use crate::app::{Action, App};
use crate::settings::Settings;
use crate::util::{BuilderHelper, Util, CHANNEL_ERROR, RUNTIME_ERROR};
use feedly_api::{models::SearchResult, ApiError, FeedlyApi};
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::FutureExt;
use gdk::EventType;
use glib::{clone, Sender};
use gtk::{
    Button, ButtonExt, ComboBox, ComboBoxExt, ContainerExt, EntryExt, EventBox, FlowBox, FlowBoxExt, GtkWindowExt,
    Image, ListBox, ListBoxExt, Revealer, RevealerExt, SearchEntry, SearchEntryExt, Stack, StackExt, StyleContextExt,
    WidgetExt, Window,
};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct DiscoverDialog {
    pub widget: Window,
}

impl DiscoverDialog {
    pub fn new(
        window: &gtk::ApplicationWindow,
        sender: &Sender<Action>,
        settings: &Arc<RwLock<Settings>>,
        news_flash: &Arc<RwLock<Option<NewsFlash>>>,
        threadpool: ThreadPool,
    ) -> Self {
        let builder = BuilderHelper::new("discover_dialog");
        let dialog = builder.get::<Window>("discover_dialog");
        dialog.set_transient_for(Some(window));

        let search_entry = builder.get::<SearchEntry>("search_entry");
        let language_combo = builder.get::<ComboBox>("language_combo");
        let search_page_stack = builder.get::<Stack>("search_page_stack");
        let search_result_stack = builder.get::<Stack>("search_result_stack");

        Self::setup_featured(&builder, "news", "news", &search_entry);
        Self::setup_featured(&builder, "tech", "tech", &search_entry);
        Self::setup_featured(&builder, "science", "science", &search_entry);
        Self::setup_featured(&builder, "culture", "culture", &search_entry);
        Self::setup_featured(&builder, "media", "media", &search_entry);
        Self::setup_featured(&builder, "sports", "sports", &search_entry);
        Self::setup_featured(&builder, "food", "food", &search_entry);
        Self::setup_featured(&builder, "foss", "open source", &search_entry);

        let search_result_list = builder.get::<ListBox>("search_result_list");
        let related_revealer = builder.get::<Revealer>("topic_revealer");
        let related_box_revealer = builder.get::<Revealer>("topic_box_revealer");
        let related_flow_box = builder.get::<FlowBox>("related_flow_box");
        let current_query: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

        let arrow_event = builder.get::<EventBox>("arrow_event");
        let arrow_image = builder.get::<Image>("arrow_image");
        let arrow_expanded = Arc::new(RwLock::new(false));

        arrow_event.connect_leave_notify_event(clone!(@weak arrow_image => @default-panic, move |_widget, _| {
            arrow_image.set_opacity(0.8);
            gtk::Inhibit(false)
        }));
        arrow_event.connect_enter_notify_event(clone!(@weak arrow_image => @default-panic, move |_widget, _| {
            arrow_image.set_opacity(1.0);
            gtk::Inhibit(false)
        }));
        arrow_event.connect_button_press_event(clone!(
            @strong arrow_expanded,
            @weak related_revealer,
            @weak arrow_image => @default-panic, move |_widget, event|
        {
            if event.get_button() != 1 {
                return gtk::Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }

            let context = arrow_image.get_style_context();
            let expanded = *arrow_expanded.read();
            if !expanded {
                context.add_class("backward-arrow-expanded");
                context.remove_class("backward-arrow-collapsed");
            } else {
                context.remove_class("backward-arrow-expanded");
                context.add_class("backward-arrow-collapsed");
            }
            related_revealer.set_reveal_child(!expanded);
            *arrow_expanded.write() = !expanded;
            gtk::Inhibit(true)
        }));

        search_entry.connect_search_changed(clone!(
            @strong settings,
            @strong threadpool,
            @strong current_query,
            @strong arrow_expanded,
            @strong sender,
            @strong news_flash,
            @weak language_combo,
            @weak arrow_image,
            @weak search_result_stack,
            @weak search_result_list,
            @weak related_flow_box,
            @weak related_revealer,
            @weak related_box_revealer,
            @weak search_page_stack => @default-panic, move |search_entry|
        {
            let query = search_entry.get_buffer().get_text();
            let locale = language_combo.get_active_id().map(|id| id.as_str().to_owned());

            if query.trim() != "" {
                Self::feedly_search(
                    locale,
                    &sender,
                    &news_flash,
                    &settings,
                    &threadpool,
                    &search_entry,
                    &search_page_stack,
                    &search_result_stack,
                    &search_result_list,
                    &related_flow_box,
                    &related_box_revealer,
                    &current_query);
            } else {
                Self::clear_list(&search_result_list);
                Self::clear_flow_box(&related_flow_box);
                search_page_stack.set_visible_child_name("featured");
            }

            related_box_revealer.set_reveal_child(false);
            related_revealer.set_reveal_child(false);
            *arrow_expanded.write() = false;
            let context = arrow_image.get_style_context();
            context.remove_class("backward-arrow-expanded");
            context.remove_class("backward-arrow-collapsed");
        }));

        language_combo.connect_changed(clone!(
            @strong settings,
            @strong threadpool,
            @strong current_query,
            @strong arrow_expanded,
            @strong sender,
            @strong news_flash,
            @weak arrow_image,
            @weak search_entry,
            @weak search_result_list,
            @weak search_result_stack,
            @weak related_flow_box,
            @weak related_revealer,
            @weak related_box_revealer,
            @weak search_page_stack => @default-panic, move |language_combo|
        {
            let query = search_entry.get_buffer().get_text();
            let locale = language_combo.get_active_id().map(|id| id.as_str().to_owned());

            if query.trim() != "" {
                Self::feedly_search(
                    locale,
                    &sender,
                    &news_flash,
                    &settings,
                    &threadpool,
                    &search_entry,
                    &search_page_stack,
                    &search_result_stack,
                    &search_result_list,
                    &related_flow_box,
                    &related_box_revealer,
                    &current_query);
            } else {
                Self::clear_list(&search_result_list);
                Self::clear_flow_box(&related_flow_box);
                search_page_stack.set_visible_child_name("featured");
            }

            related_box_revealer.set_reveal_child(false);
            related_revealer.set_reveal_child(false);
            *arrow_expanded.write() = false;
            let context = arrow_image.get_style_context();
            context.remove_class("backward-arrow-expanded");
            context.remove_class("backward-arrow-collapsed");
        }));

        DiscoverDialog { widget: dialog }
    }

    fn feedly_search(
        locale: Option<String>,
        global_sender: &Sender<Action>,
        news_flash: &Arc<RwLock<Option<NewsFlash>>>,
        settings: &Arc<RwLock<Settings>>,
        threadpool: &ThreadPool,
        search_entry: &SearchEntry,
        search_page_stack: &Stack,
        search_result_stack: &Stack,
        search_result_list: &ListBox,
        related_flow_box: &FlowBox,
        related_box_revealer: &Revealer,
        current_query: &Arc<RwLock<Option<String>>>,
    ) {
        let query = search_entry.get_buffer().get_text();
        current_query.write().replace(query.clone());
        search_page_stack.set_visible_child_name("search");
        search_result_stack.set_visible_child_name("spinner");
        Self::clear_list(search_result_list);
        Self::clear_flow_box(related_flow_box);
        let count = Some(5);

        let (sender, receiver) = oneshot::channel::<(String, Result<SearchResult, ApiError>)>();

        let settings_clone = settings.clone();
        let thread_future = async move {
            let result = Runtime::new()
                .expect(RUNTIME_ERROR)
                .block_on(FeedlyApi::search_feedly_cloud(
                    &App::build_client(&settings_clone),
                    &query,
                    count,
                    locale.as_deref(),
                ));
            sender.send((query, result)).expect(CHANNEL_ERROR);
        };

        let glib_future = receiver.map(clone!(
            @strong current_query,
            @strong settings,
            @strong threadpool,
            @strong global_sender,
            @strong news_flash,
            @weak search_result_list,
            @weak search_entry,
            @weak related_flow_box,
            @weak related_box_revealer,
            @weak search_result_stack => @default-panic, move |res|
        {
            if let Ok(res) = res {
                let (query, search_result) = res;

                if Some(query) == *current_query.read() {
                    match search_result {
                        Ok(search_result) => {
                            Self::clear_list(&search_result_list);
                            Self::clear_flow_box(&related_flow_box);

                            let result_count = search_result.results.len();
                            for (i, search_item) in search_result.results.iter().enumerate() {
                                if search_item.title.is_none() {
                                    // dont show items without title
                                    continue;
                                }
                                let is_last_row = i + 1 == result_count;
                                let search_item_row = SearchItemRow::new(&search_item, &settings, &threadpool, &global_sender, &news_flash, is_last_row);
                                search_result_list.insert(&search_item_row.widget, -1);
                            }

                            if let Some(related_topics) = search_result.related {
                                related_box_revealer.set_reveal_child(true);
                                for related_topic in related_topics {
                                    let related_topic_row = RelatedTopicRow::new(&related_topic, &search_entry);
                                    related_flow_box.insert(&related_topic_row.widget, -1);
                                }
                            } else {
                                related_box_revealer.set_reveal_child(false);
                            }
                        },
                        Err(e) => {
                            log::error!("Feedly search query failed: '{}'", e);
                        },
                    }
                    search_result_stack.set_visible_child_name("list");
                    current_query.write().take();
                }
            } else {
                log::error!("Failed to receive search result!");
            }
        }));

        threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn clear_list(list: &ListBox) {
        for row in list.get_children() {
            list.remove(&row);
        }
    }

    fn clear_flow_box(list: &FlowBox) {
        for row in list.get_children() {
            list.remove(&row);
        }
    }

    fn setup_featured(builder: &BuilderHelper, widget_name: &str, topic_name: &str, search_entry: &SearchEntry) {
        let button = builder.get::<Button>(&format!("{}_card_button", widget_name));
        let topic_name_string = topic_name.to_owned();
        button.connect_clicked(clone!(
            @weak search_entry => @default-panic, move |_button|
        {
            search_entry.set_text(&format!("#{}", topic_name_string));
        }));
    }
}
