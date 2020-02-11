mod related_topic_row;
mod search_item_row;

use self::related_topic_row::RelatedTopicRow;
use self::search_item_row::SearchItemRow;
use crate::app::{Action, App};
use crate::settings::Settings;
use crate::util::{BuilderHelper, Util, CHANNEL_ERROR, RUNTIME_ERROR};
use feedly_api::{
    models::{SearchResult, SearchResultItem},
    ApiError, FeedlyApi,
};
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::FutureExt;
use glib::{clone, Sender};
use gtk::{
    ComboBox, ComboBoxExt, ContainerExt, EntryExt, FlowBoxChild, GtkWindowExt, ListBox, ListBoxExt, ListBoxRow,
    Revealer, RevealerExt, SearchEntry, SearchEntryExt, Stack, StackExt, Window,
};
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
        threadpool: ThreadPool,
    ) -> Self {
        let builder = BuilderHelper::new("discover_dialog");
        let dialog = builder.get::<Window>("discover_dialog");
        dialog.set_transient_for(Some(window));

        let main_stack = builder.get::<Stack>("main_stack");
        let search_entry = builder.get::<SearchEntry>("search_entry");
        let language_combo = builder.get::<ComboBox>("language_combo");
        let search_page_stack = builder.get::<Stack>("search_page_stack");
        let search_result_stack = builder.get::<Stack>("search_result_stack");

        let news_card = builder.get::<FlowBoxChild>("news_card");
        let tech_card = builder.get::<FlowBoxChild>("tech_card");
        let science_card = builder.get::<FlowBoxChild>("science_card");
        let culture_card = builder.get::<FlowBoxChild>("culture_card");
        let media_card = builder.get::<FlowBoxChild>("media_card");
        let sports_card = builder.get::<FlowBoxChild>("sports_card");
        let food_card = builder.get::<FlowBoxChild>("food_card");
        let foss_card = builder.get::<FlowBoxChild>("foss_card");

        let search_result_list = builder.get::<ListBox>("search_result_list");
        let related_revealer = builder.get::<Revealer>("topic_revealer");
        let related_list = builder.get::<ListBox>("related_list");
        let current_query: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

        search_entry.connect_search_changed(clone!(
            @strong settings,
            @strong threadpool,
            @strong current_query,
            @weak language_combo,
            @weak search_result_stack,
            @weak search_result_list,
            @weak related_list,
            @weak related_revealer,
            @weak search_page_stack => @default-panic, move |search_entry|
        {
            let query = search_entry.get_buffer().get_text();
            let locale = language_combo.get_active_id().map(|id| id.as_str().to_owned());

            if query.trim() != "" {
                Self::feedly_search(
                    query,
                    locale,
                    &settings,
                    &threadpool,
                    &search_page_stack,
                    &search_result_stack,
                    &search_result_list,
                    &related_list,
                    &related_revealer,
                    &current_query);
            } else {
                Self::clear_list(&search_result_list);
                Self::clear_list(&related_list);
                search_page_stack.set_visible_child_name("featured");
                related_revealer.set_reveal_child(false);
            }
        }));

        language_combo.connect_changed(clone!(
            @strong settings,
            @strong threadpool,
            @strong current_query,
            @weak search_entry,
            @weak search_result_list,
            @weak search_result_stack,
            @weak related_list,
            @weak related_revealer,
            @weak search_page_stack => @default-panic, move |language_combo|
        {
            let query = search_entry.get_buffer().get_text();
            let locale = language_combo.get_active_id().map(|id| id.as_str().to_owned());

            if query.trim() != "" {
                Self::feedly_search(
                    query,
                    locale,
                    &settings,
                    &threadpool,
                    &search_page_stack,
                    &search_result_stack,
                    &search_result_list,
                    &related_list,
                    &related_revealer,
                    &current_query);
            } else {
                Self::clear_list(&search_result_list);
                Self::clear_list(&related_list);
                search_page_stack.set_visible_child_name("featured");
                related_revealer.set_reveal_child(false);
            }
        }));

        DiscoverDialog { widget: dialog }
    }

    fn feedly_search(
        query: String,
        locale: Option<String>,
        settings: &Arc<RwLock<Settings>>,
        threadpool: &ThreadPool,
        search_page_stack: &Stack,
        search_result_stack: &Stack,
        search_result_list: &ListBox,
        related_list: &ListBox,
        related_revealer: &Revealer,
        current_query: &Arc<RwLock<Option<String>>>,
    ) {
        current_query.write().replace(query.clone());
        related_revealer.set_reveal_child(false);
        search_page_stack.set_visible_child_name("search");
        search_result_stack.set_visible_child_name("spinner");
        Self::clear_list(search_result_list);
        Self::clear_list(related_list);
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
            @weak search_result_list,
            @weak related_list,
            @weak related_revealer,
            @weak search_result_stack => @default-panic, move |res|
        {
            if let Ok(res) = res {
                let (query, search_result) = res;

                if Some(query) == *current_query.read() {
                    match search_result {
                        Ok(search_result) => {
                            Self::clear_list(&search_result_list);
                            Self::clear_list(&related_list);

                            let result_count = search_result.results.len();
                            for (i, search_item) in search_result.results.iter().enumerate() {
                                let is_last_row = i + 1 == result_count;
                                let search_item_row = SearchItemRow::new(&search_item, &settings, &threadpool, is_last_row);
                                search_result_list.insert(&search_item_row.widget, -1);
                            }

                            if let Some(related_topics) = search_result.related {
                                related_revealer.set_reveal_child(true);
                                let result_count = related_topics.len();
                                for (i, related_topic) in related_topics.iter().enumerate() {
                                    let is_last_row = i + 1 == result_count;
                                    let related_topic_row = RelatedTopicRow::new(&related_topic, is_last_row);
                                    related_list.insert(&related_topic_row.widget, -1);
                                }
                            } else {
                                related_revealer.set_reveal_child(false);
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
}
