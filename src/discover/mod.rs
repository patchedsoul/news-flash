mod search_item_row;

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
    ComboBox, ComboBoxExt, ContainerExt, EntryExt, FlowBoxChild, GtkWindowExt, ListBox, ListBoxExt, Revealer,
    SearchEntry, SearchEntryExt, Stack, StackExt, Window,
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
        let topic_revealer = builder.get::<Revealer>("topic_revealer");
        let related_list = builder.get::<ListBox>("related_list");

        search_entry.connect_search_changed(clone!(
            @strong settings,
            @strong threadpool,
            @weak language_combo,
            @weak search_result_stack,
            @weak search_result_list,
            @weak search_page_stack => @default-panic, move |search_entry|
        {
            let query = search_entry.get_buffer().get_text();
            let locale = language_combo.get_active_id().map(|id| id.as_str().to_owned());

            if query.trim() != "" {
                Self::feedly_search(query, locale, &settings, &threadpool, &search_page_stack, &search_result_stack, &search_result_list);
            } else {
                // FIXME: show message
            }
        }));

        language_combo.connect_changed(clone!(
            @strong settings,
            @strong threadpool,
            @weak search_entry,
            @weak search_result_list,
            @weak search_result_stack,
            @weak search_page_stack => @default-panic, move |language_combo|
        {
            let query = search_entry.get_buffer().get_text();
            let locale = language_combo.get_active_id().map(|id| id.as_str().to_owned());

            if query.trim() != "" {
                Self::feedly_search(query, locale, &settings, &threadpool, &search_page_stack, &search_result_stack, &search_result_list);
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
    ) {
        search_page_stack.set_visible_child_name("search");
        search_result_stack.set_visible_child_name("spinner");
        Self::clear_search_results(search_result_list);
        let count = Some(5);

        let (sender, receiver) = oneshot::channel::<Result<SearchResult, ApiError>>();

        let settings = settings.clone();
        let thread_future = async move {
            let result = Runtime::new()
                .expect(RUNTIME_ERROR)
                .block_on(FeedlyApi::search_feedly_cloud(
                    &App::build_client(&settings),
                    &query,
                    count,
                    locale.as_deref(),
                ));
            sender.send(result).expect(CHANNEL_ERROR);
        };

        let glib_future = receiver.map(clone!(
            @weak search_result_list,
            @weak search_result_stack => @default-panic, move |res|
        {
            match res {
                Ok(Ok(search_result)) => {
                    Self::clear_search_results(&search_result_list);
                    let result_count = search_result.results.len();
                    for (i, search_item) in search_result.results.iter().enumerate() {
                        let is_last_row = i + 1 == result_count;
                        let search_item_row = SearchItemRow::new(&search_item, is_last_row);
                        search_result_list.insert(&search_item_row.widget, -1);
                    }
                },
                Err(e) => {
                    log::error!("Failed to receive search result! {}", e);
                },
                Ok(Err(_)) => {
                    log::error!("Failed to receive search result!");
                },
            }
            search_result_stack.set_visible_child_name("list");
        }));

        threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn clear_search_results(list: &ListBox) {
        for row in list.get_children() {
            list.remove(&row);
        }
    }
}
