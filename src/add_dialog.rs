use crate::app::{Action, App};
use crate::color::ColorRGBA;
use crate::i18n::i18n;
use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkUtil, Util, CHANNEL_ERROR, RUNTIME_ERROR};
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::future::FutureExt;
use glib::{clone, object::Cast, types::Type, Sender};
use gtk::{
    prelude::GtkListStoreExtManual, BinExt, Box, BoxExt, Button, ButtonExt, ColorButton, ColorChooserExt, ComboBox,
    ComboBoxExt, ContainerExt, EditableSignals, Entry, EntryExt, GtkListStoreExt, IconSize, Image, ImageExt, Label,
    LabelExt, ListBox, ListBoxExt, ListBoxRow, ListBoxRowExt, ListStore, Orientation, Popover, PopoverExt, Separator,
    Stack, StackExt, StyleContextExt, Widget, WidgetExt,
};
use log::error;
use news_flash::models::{Category, CategoryID, FavIcon, Feed, FeedID, PluginCapabilities, Url};
use news_flash::{FeedParserError, ParsedUrl};
use pango::EllipsizeMode;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const NEW_CATEGORY_ICON: &str = "folder-new-symbolic";
pub const WARN_ICON: &str = "dialog-warning-symbolic";

#[derive(Clone, Debug)]
pub enum AddCategory {
    New(String),
    Existing(CategoryID),
}

#[derive(Clone, Debug)]
pub struct AddPopover {
    pub feed_add_button: Button,
    pub category_add_button: Button,
    pub tag_add_button: Button,
    color_button: ColorButton,
    popover: Popover,
    feed_title_entry: Entry,
    category_entry: Entry,
    tag_entry: Entry,
    main_stack: Stack,
    feed_list: ListBox,
    select_button: Button,
    select_button_stack: Stack,
    favicon_image: Image,
    parse_button_stack: Stack,
    parse_button: Button,
    url_entry: Entry,
    feed_url: Arc<RwLock<Option<Url>>>,
    feed_category: Arc<RwLock<Option<AddCategory>>>,
}

impl AddPopover {
    pub fn new_for_feed_url(
        sender: &Sender<Action>,
        parent: &Widget,
        categories: Vec<Category>,
        threadpool: &ThreadPool,
        settings: &Arc<RwLock<Settings>>,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
        feed_url: &Url,
    ) -> Self {
        let dialog = Self::new(sender, parent, categories, threadpool.clone(), settings, features);
        dialog.main_stack.set_visible_child_name("spinner");

        Self::parse_feed_url(
            &feed_url,
            settings,
            &threadpool,
            &dialog.main_stack,
            &dialog.feed_list,
            &dialog.feed_title_entry,
            &dialog.select_button,
            &dialog.select_button_stack,
            &dialog.favicon_image,
            &dialog.feed_url,
            &dialog.parse_button_stack,
            &dialog.parse_button,
            &dialog.url_entry,
        );

        dialog
    }

    pub fn new(
        sender: &Sender<Action>,
        parent: &Widget,
        categories: Vec<Category>,
        threadpool: ThreadPool,
        settings: &Arc<RwLock<Settings>>,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
    ) -> Self {
        let builder = BuilderHelper::new("add_dialog");
        let popover = builder.get::<Popover>("add_pop");
        let url_entry = builder.get::<Entry>("url_entry");
        let parse_button = builder.get::<Button>("parse_button");
        let parse_button_stack = builder.get::<Stack>("parse_button_stack");
        let main_stack = builder.get::<Stack>("main_stack");
        let feed_list = builder.get::<ListBox>("feed_list");
        let select_button = builder.get::<Button>("select_button");
        let select_button_stack = builder.get::<Stack>("select_button_stack");
        let feed_title_entry = builder.get::<Entry>("feed_title_entry");
        let favicon_image = builder.get::<Image>("favicon_image");
        let category_combo = builder.get::<ComboBox>("category_combo");
        let feed_category_entry = builder.get::<Entry>("feed_category_entry");
        let feed_add_button = builder.get::<Button>("add_button");
        let select_list_box = builder.get::<ListBox>("select_list_box");
        let category_entry = builder.get::<Entry>("category_entry");
        let category_add_button = builder.get::<Button>("add_category_button");
        let tag_entry = builder.get::<Entry>("tag_entry");
        let tag_add_button = builder.get::<Button>("add_tag_button");
        let color_button = builder.get::<ColorButton>("color_button");
        let feed_url: Arc<RwLock<Option<Url>>> = Arc::new(RwLock::new(None));
        let feed_category = Arc::new(RwLock::new(None));
        let feed_row = builder.get::<ListBoxRow>("feed_row");
        let category_row = builder.get::<ListBoxRow>("category_row");
        let tag_row = builder.get::<ListBoxRow>("tag_row");

        // setup list of categories to add feed to
        if !categories.is_empty() {
            let list_store = ListStore::new(&[Type::String, Type::String]);
            for category in &categories {
                let iter = list_store.append();
                list_store.set(&iter, &[0, 1], &[&Some(category.category_id.to_str()), &category.label]);
            }
            category_combo.set_model(Some(&list_store));
        }

        // make parse button sensitive if entry contains text and vice versa
        url_entry.connect_changed(clone!(@weak parse_button => @default-panic, move |entry| {
            if let Some(text) = entry.get_text() {
                if text.as_str().is_empty() {
                    parse_button.set_sensitive(false);
                } else {
                    parse_button.set_sensitive(true);
                }
            } else {
                parse_button.set_sensitive(false);
            }

            entry.set_property_secondary_icon_name(None);
            entry.set_property_secondary_icon_tooltip_text(None);
        }));

        // hit enter in entry to parse url
        url_entry.connect_activate(clone!(@weak parse_button => @default-panic, move |_entry| {
            if parse_button.get_sensitive() {
                parse_button.clicked();
            }
        }));

        // make parse button sensitive if entry contains text and vice versa
        category_entry.connect_changed(clone!(@weak category_add_button => @default-panic, move |entry| {
            if let Some(text) = entry.get_text() {
                if text.as_str().is_empty() {
                    category_add_button.set_sensitive(false);
                } else {
                    category_add_button.set_sensitive(true);
                }
            } else {
                category_add_button.set_sensitive(false);
            }

            entry.set_property_secondary_icon_name(None);
            entry.set_property_secondary_icon_tooltip_text(None);
        }));

        // hit enter in entry to add category
        category_entry.connect_activate(clone!(@weak category_add_button => @default-panic, move |_entry| {
            if category_add_button.get_sensitive() {
                category_add_button.clicked();
            }
        }));

        // make parse button sensitive if entry contains text and vice versa
        tag_entry.connect_changed(clone!(@weak tag_add_button => @default-panic, move |entry| {
            if let Some(text) = entry.get_text() {
                if text.as_str().is_empty() {
                    tag_add_button.set_sensitive(false);
                } else {
                    tag_add_button.set_sensitive(true);
                }
            } else {
                tag_add_button.set_sensitive(false);
            }

            entry.set_property_secondary_icon_name(None);
            entry.set_property_secondary_icon_tooltip_text(None);
        }));

        // hit enter in entry to add category
        tag_entry.connect_activate(clone!(@weak tag_add_button => @default-panic, move |_entry| {
            if tag_add_button.get_sensitive() {
                tag_add_button.clicked();
            }
        }));

        if let Some(features) = features.read().as_ref() {
            feed_row.set_sensitive(features.contains(PluginCapabilities::ADD_REMOVE_FEEDS));
            category_row.set_sensitive(features.contains(PluginCapabilities::MODIFY_CATEGORIES));
            tag_row.set_sensitive(features.contains(PluginCapabilities::SUPPORT_TAGS));
        }

        // parse url and switch to feed selection or final page
        parse_button.connect_clicked(clone!(
            @weak main_stack,
            @weak feed_list,
            @weak feed_title_entry,
            @weak favicon_image,
            @weak select_button,
            @weak select_button_stack,
            @weak parse_button_stack,
            @weak url_entry,
            @strong feed_url,
            @strong settings => @default-panic, move |button|
        {
            if let Some(url_text) = url_entry.get_text() {
                let mut url_text = url_text.as_str().to_owned();
                if !url_text.starts_with("http://") && !url_text.starts_with("https://") {
                    url_text.insert_str(0, "https://");
                }
                if let Ok(url) = Url::parse(&url_text) {
                    // set 'next' button insensitive and show spinner
                    parse_button_stack.set_visible_child_name("spinner");
                    button.set_sensitive(false);

                    Self::parse_feed_url(
                        &url,
                        &settings,
                        &threadpool,
                        &main_stack,
                        &feed_list,
                        &feed_title_entry,
                        &select_button,
                        &select_button_stack,
                        &favicon_image,
                        &feed_url,
                        &parse_button_stack,
                        button,
                        &url_entry);
                } else {
                    error!("No valid url: '{}'", url_text);
                    url_entry.set_property_secondary_icon_name(Some(WARN_ICON));
                    url_entry.set_property_secondary_icon_tooltip_text(Some(&i18n("No valid URL.")));
                }
            } else {
                error!("Empty url");
                url_entry.set_property_secondary_icon_name(Some(WARN_ICON));
                url_entry.set_property_secondary_icon_tooltip_text(Some(&i18n("Empty URL")));
            }
        }));

        // make add_button sensitive / insensitive
        feed_category_entry.connect_changed(clone!(
            @weak feed_add_button,
            @weak feed_title_entry,
            @weak category_combo,
            @strong feed_category => @default-panic, move |entry|
        {
            let sensitive = Self::calc_add_button_sensitive(&feed_title_entry, &entry);
            feed_add_button.set_sensitive(sensitive);

            let entry_text = entry.get_text().map(|t| t.as_str().to_owned());

            let folder_icon = if category_combo.get_active_id().is_some() {
                if let Some(id) = category_combo.get_active_id() {
                    let category_id = CategoryID::new(id.as_str());
                    feed_category
                        .write()
                        .replace(AddCategory::Existing(category_id));
                }
                None
            } else if entry_text.is_none() {
                feed_category.write().take();
                None
            } else if categories.iter().any(|c| Some(c.label.clone()) == entry_text) {
                let category_id = categories
                    .iter()
                    .find(|c| Some(c.label.clone()) == entry_text)
                    .map(|c| c.category_id.clone());

                if let Some(category_id) = category_id {
                    feed_category
                        .write()
                        .replace(AddCategory::Existing(category_id));
                }
                None
            } else {
                feed_category.write().replace(AddCategory::New(
                    entry_text.expect("entry_text already checked for None"),
                ));
                Some(NEW_CATEGORY_ICON)
            };

            entry.set_property_secondary_icon_name(folder_icon);
        }));

        feed_title_entry.connect_changed(
            clone!(@weak feed_add_button, @strong feed_category_entry => @default-panic, move |entry| {
                let sensitive = Self::calc_add_button_sensitive(&entry, &feed_category_entry);
                feed_add_button.set_sensitive(sensitive);
            }),
        );

        select_list_box.connect_row_activated(clone!(@weak main_stack => @default-panic, move |_list, row| {
            let index = row.get_index();
            if index == 0 {
                // Feed
                main_stack.set_visible_child_name("feed_url_page");
            } else if index == 1 {
                // Category
                main_stack.set_visible_child_name("category_page");
            } else if index == 2 {
                // Tag
                main_stack.set_visible_child_name("tag_page");
            }
        }));

        feed_add_button.connect_clicked(clone!(
            @weak popover,
            @weak feed_title_entry,
            @strong feed_category,
            @strong feed_url,
            @strong sender => @default-panic, move |_button|
        {
            let feed_url = match feed_url.read().clone() {
                Some(url) => url,
                None => {
                    error!("Failed to add feed: No valid url");
                    Util::send(&sender, Action::ErrorSimpleMessage("Failed to add feed".into()));
                    return;
                }
            };
            let feed_title = feed_title_entry.get_text().map(|title| title.as_str().to_owned());
            let feed_category = feed_category.read().clone();

            Util::send(&sender, Action::AddFeed((feed_url, feed_title, feed_category)));
            popover.popdown()
        }));
        category_add_button.connect_clicked(clone!(
            @weak popover,
            @weak category_entry,
            @strong sender => @default-panic, move |_button|
        {
            let category_title = category_entry.get_text().map(|text| text.as_str().to_owned());
            if let Some(category_title) = category_title {
                Util::send(&sender, Action::AddCategory(category_title));
                popover.popdown()
            }
        }));
        tag_add_button.connect_clicked(clone!(
            @weak popover,
            @weak tag_entry,
            @weak color_button,
            @strong sender => @default-panic, move |_button|
        {
            let tag_title = tag_entry.get_text().map(|text| text.as_str().to_owned());
            let rgba = color_button.get_rgba();
            let rgba = ColorRGBA::from_normalized(rgba.red, rgba.green, rgba.blue, rgba.alpha);
            let color = rgba.to_string_no_alpha();
            if let Some(tag_title) = tag_title {
                Util::send(&sender, Action::AddTag(color, tag_title));
                popover.popdown()
            }
        }));

        popover.set_relative_to(Some(parent));
        popover.show_all();

        main_stack.set_visible_child_name("start");

        AddPopover {
            feed_add_button,
            category_add_button,
            tag_add_button,
            color_button,
            popover,
            main_stack,
            feed_list,
            feed_title_entry,
            category_entry,
            tag_entry,
            select_button,
            select_button_stack,
            favicon_image,
            parse_button_stack,
            parse_button,
            url_entry,
            feed_url,
            feed_category,
        }
    }

    fn fill_feed_page(
        feed: Feed,
        title_entry: &Entry,
        favicon_image: &Image,
        feed_url: &Arc<RwLock<Option<Url>>>,
        threadpool: ThreadPool,
        settings: &Arc<RwLock<Settings>>,
    ) {
        title_entry.set_text(&feed.label);
        if let Some(new_feed_url) = &feed.feed_url {
            feed_url.write().replace(new_feed_url.clone());
        } else {
            feed_url.write().take();
        }

        let (sender, receiver) = oneshot::channel::<Option<FavIcon>>();

        let feed_clone = feed.clone();
        let settings_clone = settings.clone();
        let thread_future = async move {
            let result =
                Runtime::new()
                    .expect(RUNTIME_ERROR)
                    .block_on(news_flash::util::favicon_cache::FavIconCache::scrap(
                        &feed_clone,
                        &App::build_client(&settings_clone),
                    ));
            sender.send(result).expect(CHANNEL_ERROR);
        };

        let scale = GtkUtil::get_scale(favicon_image);

        let glib_future = receiver.map(clone!(
            @weak favicon_image,
            @strong threadpool,
            @strong settings => @default-panic, move |res|
        {
            if let Some(favicon) = res.expect(CHANNEL_ERROR) {
                if let Some(data) = &favicon.data {
                    if let Ok(surface) = GtkUtil::create_surface_from_bytes(data, 64, 64, scale) {
                        favicon_image.set_from_surface(Some(&surface));
                    }
                }
            } else if let Some(icon_url) = feed.icon_url {
                let favicon_image = favicon_image.clone();

                let (sender, receiver) = oneshot::channel::<Option<Vec<u8>>>();

                let thread_future = async move {
                    let mut runtime = Runtime::new().expect(RUNTIME_ERROR);
                    let res = match runtime.block_on(App::build_client(&settings).get(icon_url.get()).send()) {
                        Ok(response) => match runtime.block_on(response.bytes()) {
                            Ok(bytes) => Some(Vec::from(bytes.as_ref())),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    };
                    sender.send(res).expect(CHANNEL_ERROR);
                };

                let glib_future = receiver.map(move |res| {
                    if let Some(byte_vec) = res.expect(CHANNEL_ERROR) {
                        if let Ok(surface) = GtkUtil::create_surface_from_bytes(&byte_vec, 64, 64, scale) {
                            favicon_image.set_from_surface(Some(&surface));
                        }
                    }
                });

                threadpool.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            }
        }));

        threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn fill_mupliple_feed_list(
        feed_vec: Vec<(String, Url)>,
        list: &ListBox,
        select_button: &Button,
        select_button_stack: &Stack,
        stack: &Stack,
        title_entry: &Entry,
        favicon: &Image,
        feed_url: &Arc<RwLock<Option<Url>>>,
        threadpool: ThreadPool,
        settings: &Arc<RwLock<Settings>>,
    ) {
        list.connect_row_selected(clone!(@weak select_button => @default-panic, move |_list, row| {
            select_button.set_sensitive(row.is_some());
        }));

        select_button.connect_clicked(clone!(
            @weak stack,
            @weak title_entry,
            @weak list,
            @weak favicon,
            @strong feed_url,
            @strong settings,
            @weak select_button_stack => @default-panic, move |button|
        {
            if let Some(row) = list.get_selected_row() {
                if let Some(name) = row.get_widget_name() {
                    select_button_stack.set_visible_child_name("spinner");
                    button.set_sensitive(false);

                    let url = Url::parse(name.as_str()).expect("should never fail since it comes from 'url.as_str()'");
                    let feed_id = FeedID::new(url.get().as_str());

                    let (sender, receiver) = oneshot::channel::<Option<ParsedUrl>>();

                    let settings_clone = settings.clone();
                    let thread_future = async move {
                        let result = Runtime::new()
                            .expect(RUNTIME_ERROR)
                            .block_on(news_flash::feed_parser::download_and_parse_feed(
                                &url, &feed_id, None, None, &App::build_client(&settings_clone),
                            ))
                            .ok();
                        sender.send(result).expect(CHANNEL_ERROR);
                    };

                    let glib_future = receiver.map(clone!(
                        @strong threadpool,
                        @weak button as select_button,
                        @weak select_button_stack,
                        @strong feed_url,
                        @strong settings,
                        @weak favicon,
                        @weak title_entry,
                        @weak stack as main_stack => @default-panic, move |res|
                    {
                        if let Some(ParsedUrl::SingleFeed(feed)) = res.expect(CHANNEL_ERROR) {
                            Self::fill_feed_page(
                                feed,
                                &title_entry,
                                &favicon,
                                &feed_url,
                                threadpool,
                                &settings,
                            );
                            main_stack.set_visible_child_name("feed_page");
                        } else if let Some(child) = row.get_child() {
                            if let Ok(_box) = child.downcast::<Box>() {
                                if let Some(icon) = _box.get_children().get(1) {
                                    icon.set_visible(true);
                                }
                            }
                        }

                        select_button_stack.set_visible_child_name("text");
                        select_button.set_sensitive(true);
                    }));

                    threadpool.spawn_ok(thread_future);
                    Util::glib_spawn_future(glib_future);
                }
            }
        }));
        for (title, url) in feed_vec {
            let label = Label::new(Some(&title));
            label.set_size_request(0, 50);
            label.set_ellipsize(EllipsizeMode::End);
            label.set_xalign(0.0);

            let warn_icon = Image::new_from_icon_name(Some(WARN_ICON), IconSize::Button);
            warn_icon.set_tooltip_text(Some(&i18n("Failed to get Feed.")));
            warn_icon.set_no_show_all(true);

            let gtk_box = Box::new(Orientation::Horizontal, 0);
            gtk_box.set_margin_start(20);
            gtk_box.set_margin_end(20);
            gtk_box.pack_start(&label, true, true, 0);
            gtk_box.pack_end(&warn_icon, false, false, 0);

            let row = ListBoxRow::new();

            row.connect_activate(clone!(@weak select_button => @default-panic, move |_row| {
                select_button.activate();
            }));

            row.set_selectable(true);
            row.set_activatable(false);
            row.set_widget_name(url.get().as_str());
            row.add(&gtk_box);
            row.show_all();
            list.insert(&row, -1);

            let separator = Separator::new(Orientation::Horizontal);
            let separator_row = ListBoxRow::new();
            separator_row.add(&separator);
            separator_row.set_selectable(false);
            separator_row.set_activatable(false);
            separator_row.get_style_context().add_class("separator-row");
            separator_row.show_all();
            list.insert(&separator_row, -1);
        }
        if let Some(last_child) = list.get_children().pop() {
            list.remove(&last_child);
        }
    }

    fn calc_add_button_sensitive(title_entry: &Entry, feed_category_entry: &Entry) -> bool {
        if let Some(text) = title_entry.get_text() {
            if text.as_str().is_empty() {
                return false;
            }
        }

        if let Some(text) = feed_category_entry.get_text() {
            if text.as_str().is_empty() {
                return false;
            }
        }

        true
    }

    fn parse_feed_url(
        url: &Url,
        settings: &Arc<RwLock<Settings>>,
        threadpool: &ThreadPool,
        main_stack: &Stack,
        feed_list: &ListBox,
        feed_title_entry: &Entry,
        select_button: &Button,
        select_button_stack: &Stack,
        favicon_image: &Image,
        feed_url: &Arc<RwLock<Option<Url>>>,
        parse_button_stack: &Stack,
        parse_button: &Button,
        url_entry: &Entry,
    ) {
        let (sender, receiver) = oneshot::channel::<Result<ParsedUrl, FeedParserError>>();

        let feed_id = FeedID::new(url.get().as_str());
        let thread_url = url.clone();
        let settings_clone = settings.clone();
        let thread_future = async move {
            let result =
                Runtime::new()
                    .expect(RUNTIME_ERROR)
                    .block_on(news_flash::feed_parser::download_and_parse_feed(
                        &thread_url,
                        &feed_id,
                        None,
                        None,
                        &App::build_client(&settings_clone),
                    ));
            sender.send(result).expect(CHANNEL_ERROR);
        };

        let parse_button_threadpool = threadpool.clone();
        let glib_future = receiver.map(clone!(
            @weak main_stack,
            @weak feed_list,
            @weak feed_title_entry,
            @weak select_button,
            @weak select_button_stack,
            @weak favicon_image,
            @weak parse_button_stack,
            @weak parse_button,
            @weak url_entry,
            @strong feed_url,
            @strong settings,
            @strong url => @default-panic, move |res|
        {
            // parse url
            match res.expect(CHANNEL_ERROR) {
                Ok(result) => match result {
                    ParsedUrl::MultipleFeeds(feed_vec) => {
                        // url has multiple feeds: show selection page and list them there
                        main_stack.set_visible_child_name("feed_selection_page");
                        Self::fill_mupliple_feed_list(
                            feed_vec,
                            &feed_list,
                            &select_button,
                            &select_button_stack,
                            &main_stack,
                            &feed_title_entry,
                            &favicon_image,
                            &feed_url,
                            parse_button_threadpool,
                            &settings,
                        );
                    }
                    ParsedUrl::SingleFeed(feed) => {
                        // url has single feed: move to feed page
                        main_stack.set_visible_child_name("feed_page");
                        Self::fill_feed_page(
                            feed,
                            &feed_title_entry,
                            &favicon_image,
                            &feed_url,
                            parse_button_threadpool,
                            &settings,
                        );
                    }
                },
                Err(error) => {
                    error!("No feed found for url '{}': {}", url, error);
                    main_stack.set_visible_child_name("feed_url_page");
                    url_entry.set_text(&url.to_string());
                    url_entry.set_property_secondary_icon_name(Some(WARN_ICON));
                    url_entry.set_property_secondary_icon_tooltip_text(Some(&i18n("No Feed found.")));
                }
            }

            // set 'next' buton sensitive again and show label again
            parse_button_stack.set_visible_child_name("text");
            parse_button.set_sensitive(true);
        }));

        threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }
}
