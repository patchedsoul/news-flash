use crate::util::{BuilderHelper, GtkUtil, Util, RUNTIME_ERROR};
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::future::FutureExt;
use glib::object::Cast;
use gtk::{
    BinExt, Box, BoxExt, Button, ButtonExt, ComboBox, ComboBoxExt, ContainerExt, EditableSignals, Entry, EntryExt,
    GtkListStoreExt, GtkListStoreExtManual, IconSize, Image, ImageExt, Label, LabelExt, ListBox, ListBoxExt,
    ListBoxRow, ListBoxRowExt, ListStore, Orientation, Popover, PopoverExt, Separator, Stack, StackExt,
    StyleContextExt, Type, WidgetExt,
};
use log::error;
use news_flash::models::{Category, CategoryID, FavIcon, Feed, FeedID, Url};
use news_flash::{FeedParserError, ParsedUrl};
use pango::EllipsizeMode;
use parking_lot::RwLock;
use std::rc::Rc;
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
    popover: Popover,
    add_button: Button,
    feed_title_entry: Entry,
    feed_url: Rc<RwLock<Option<Url>>>,
    feed_category: Rc<RwLock<Option<AddCategory>>>,
}

impl AddPopover {
    pub fn new(parent: &Button, categories: Vec<Category>, threadpool: ThreadPool) -> Self {
        let builder = BuilderHelper::new("add_dialog");
        let popover = builder.get::<Popover>("add_pop");
        let url_entry = builder.get::<Entry>("url_entry");
        let parse_button = builder.get::<Button>("parse_button");
        let parse_button_stack = builder.get::<Stack>("parse_button_stack");
        let add_feed_stack = builder.get::<Stack>("add_feed_stack");
        let feed_list = builder.get::<ListBox>("feed_list");
        let select_button = builder.get::<Button>("select_button");
        let select_button_stack = builder.get::<Stack>("select_button_stack");
        let feed_title_entry = builder.get::<Entry>("feed_title_entry");
        let favicon_image = builder.get::<Image>("favicon_image");
        let category_combo = builder.get::<ComboBox>("category_combo");
        let category_entry = builder.get::<Entry>("category_entry");
        let add_button = builder.get::<Button>("add_button");
        let add_button_stack = builder.get::<Stack>("add_button_stack");
        let feed_url: Rc<RwLock<Option<Url>>> = Rc::new(RwLock::new(None));
        let feed_category = Rc::new(RwLock::new(None));

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
        let url_entry_parse_button = parse_button.clone();
        url_entry.connect_changed(move |entry| {
            if let Some(text) = entry.get_text() {
                if text.as_str().is_empty() {
                    url_entry_parse_button.set_sensitive(false);
                } else {
                    url_entry_parse_button.set_sensitive(true);
                }
            } else {
                url_entry_parse_button.set_sensitive(false);
            }

            entry.set_property_secondary_icon_name(None);
            entry.set_property_secondary_icon_tooltip_text(None);
        });

        // hit enter in entry to parse url
        let url_entry_parse_button = parse_button.clone();
        url_entry.connect_activate(move |_entry| {
            if url_entry_parse_button.get_sensitive() {
                url_entry_parse_button.clicked();
            }
        });

        // parse url and switch to feed selection or final page
        let parse_button_add_feed_stack = add_feed_stack.clone();
        let parse_button_feed_list = feed_list.clone();
        let parse_button_feed_title_entry = feed_title_entry.clone();
        let parse_button_favicon_image = favicon_image.clone();
        let parse_button_select_button = select_button.clone();
        let parse_button_select_button_stack = select_button_stack.clone();
        let parse_button_feed_url = feed_url.clone();
        let parse_button_parse_button_stack = parse_button_stack.clone();
        let parse_button_add_button_stack = add_button_stack.clone();
        parse_button.connect_clicked(move |button| {
            if let Some(url_text) = url_entry.get_text() {
                let mut url_text = url_text.as_str().to_owned();
                if !url_text.starts_with("http://") && !url_text.starts_with("https://") {
                    url_text.insert_str(0, "https://");
                }
                if let Ok(url) = Url::parse(&url_text) {
                    // set 'next' button insensitive and show spinner
                    parse_button_parse_button_stack.set_visible_child_name("spinner");
                    button.set_sensitive(false);

                    let parse_button_add_feed_stack = parse_button_add_feed_stack.clone();
                    let parse_button_feed_list = parse_button_feed_list.clone();
                    let parse_button_feed_title_entry = parse_button_feed_title_entry.clone();
                    let parse_button_select_button = parse_button_select_button.clone();
                    let parse_button_select_button_stack = parse_button_select_button_stack.clone();
                    let parse_button_favicon_image = parse_button_favicon_image.clone();
                    let parse_button_feed_url = parse_button_feed_url.clone();
                    let parse_button_parse_button_stack = parse_button_parse_button_stack.clone();
                    let parse_button_add_button_stack = parse_button_add_button_stack.clone();
                    let parse_button = button.clone();
                    let url_entry = url_entry.clone();
                    let parse_button_url = url.clone();

                    let (sender, receiver) = oneshot::channel::<Result<ParsedUrl, FeedParserError>>();

                    let feed_id = FeedID::new(&url_text);
                    let thread_future = async move {
                        let result = Runtime::new().expect(RUNTIME_ERROR).block_on(
                            news_flash::feed_parser::download_and_parse_feed(&url, &feed_id, None, None),
                        );
                        sender.send(result).unwrap();
                    };

                    let parse_button_threadpool = threadpool.clone();
                    let glib_future = receiver.map(move |res| {
                        // parse url
                        match res.unwrap() {
                            Ok(result) => match result {
                                ParsedUrl::MultipleFeeds(feed_vec) => {
                                    // url has multiple feeds: show selection page and list them there
                                    parse_button_add_feed_stack.set_visible_child_name("feed_selection_page");
                                    Self::fill_mupliple_feed_list(
                                        feed_vec,
                                        &parse_button_feed_list,
                                        &parse_button_select_button,
                                        &parse_button_select_button_stack,
                                        &parse_button_add_feed_stack,
                                        &parse_button_feed_title_entry,
                                        &parse_button_favicon_image,
                                        &parse_button_add_button_stack,
                                        &parse_button_feed_url,
                                        parse_button_threadpool,
                                    );
                                }
                                ParsedUrl::SingleFeed(feed) => {
                                    // url has single feed: move to feed page
                                    parse_button_add_feed_stack.set_visible_child_name("feed_page");
                                    Self::fill_feed_page(
                                        feed,
                                        &parse_button_add_button_stack,
                                        &parse_button_feed_title_entry,
                                        &parse_button_favicon_image,
                                        &parse_button_feed_url,
                                        parse_button_threadpool,
                                    );
                                }
                            },
                            Err(error) => {
                                error!("No feed found for url '{}': {}", parse_button_url, error);
                                url_entry.set_property_secondary_icon_name(Some(WARN_ICON));
                                url_entry.set_property_secondary_icon_tooltip_text(Some("No Feed found."));
                            }
                        }

                        // set 'next' buton sensitive again and show label again
                        parse_button_parse_button_stack.set_visible_child_name("text");
                        parse_button.set_sensitive(true);
                    });

                    threadpool.spawn_ok(thread_future);
                    Util::glib_spawn_future(glib_future);
                } else {
                    error!("No valid url: '{}'", url_text);
                    url_entry.set_property_secondary_icon_name(Some(WARN_ICON));
                    url_entry.set_property_secondary_icon_tooltip_text(Some("No valid URL."));
                }
            } else {
                error!("Empty url");
                url_entry.set_property_secondary_icon_name(Some(WARN_ICON));
                url_entry.set_property_secondary_icon_tooltip_text(Some("Empty URL"));
            }
        });

        // make add_button sensitive / insensitive
        let category_entry_add_button = add_button.clone();
        let category_entry_title_entry = feed_title_entry.clone();
        let category_entry_category_combo = category_combo.clone();
        let category_entry_feed_category = feed_category.clone();
        category_entry.connect_changed(move |entry| {
            let sensitive = Self::calc_add_button_sensitive(&category_entry_title_entry, &entry);
            category_entry_add_button.set_sensitive(sensitive);

            let entry_text = entry.get_text().map(|t| t.as_str().to_owned());

            let folder_icon = if category_entry_category_combo.get_active_id().is_some() {
                if let Some(id) = category_entry_category_combo.get_active_id() {
                    let category_id = CategoryID::new(id.as_str());
                    category_entry_feed_category
                        .write()
                        .replace(AddCategory::Existing(category_id));
                }
                None
            } else if entry_text.is_none() {
                category_entry_feed_category.write().take();
                None
            } else if categories.iter().any(|c| Some(c.label.clone()) == entry_text) {
                let category_id = categories
                    .iter()
                    .find(|c| Some(c.label.clone()) == entry_text)
                    .map(|c| c.category_id.clone());

                if let Some(category_id) = category_id {
                    category_entry_feed_category
                        .write()
                        .replace(AddCategory::Existing(category_id));
                }
                None
            } else {
                category_entry_feed_category.write().replace(AddCategory::New(
                    entry_text.expect("entry_text already checked for None"),
                ));
                Some(NEW_CATEGORY_ICON)
            };

            entry.set_property_secondary_icon_name(folder_icon);
        });

        let title_entry_add_button = add_button.clone();
        let title_entry_category_entry = category_entry.clone();
        feed_title_entry.connect_changed(move |entry| {
            let sensitive = Self::calc_add_button_sensitive(&entry, &title_entry_category_entry);
            title_entry_add_button.set_sensitive(sensitive);
        });

        popover.set_relative_to(Some(parent));
        popover.show_all();

        AddPopover {
            popover,
            add_button,
            feed_title_entry,
            feed_url,
            feed_category,
        }
    }

    fn fill_feed_page(
        feed: Feed,
        add_button_stack: &Stack,
        title_entry: &Entry,
        favicon_image: &Image,
        feed_url: &Rc<RwLock<Option<Url>>>,
        threadpool: ThreadPool,
    ) {
        title_entry.set_text(&feed.label);
        if let Some(new_feed_url) = &feed.feed_url {
            feed_url.write().replace(new_feed_url.clone());
        } else {
            feed_url.write().take();
        }

        add_button_stack.set_visible_child_name("spinner");

        let (sender, receiver) = oneshot::channel::<Option<FavIcon>>();

        let feed_clone = feed.clone();
        let thread_future = async move {
            let result = Runtime::new()
                .expect(RUNTIME_ERROR)
                .block_on(news_flash::util::favicon_cache::FavIconCache::scrap(&feed_clone));
            sender.send(result).unwrap();
        };

        let scale = GtkUtil::get_scale(favicon_image);
        let favicon_image = favicon_image.clone();
        let add_button_stack = add_button_stack.clone();

        let threadpool_clone = threadpool.clone();

        let glib_future = receiver.map(move |res| {
            if let Some(favicon) = res.unwrap() {
                if let Some(data) = &favicon.data {
                    if let Ok(surface) = GtkUtil::create_surface_from_bytes(data, 64, 64, scale) {
                        favicon_image.set_from_surface(Some(&surface));
                        add_button_stack.set_visible_child_name("text");
                    }
                }
            } else if let Some(icon_url) = feed.icon_url {
                let favicon_image = favicon_image.clone();

                let (sender, receiver) = oneshot::channel::<Option<Vec<u8>>>();

                let thread_future = async move {
                    let mut runtime = Runtime::new().expect(RUNTIME_ERROR);
                    let res = match runtime.block_on(reqwest::get(icon_url.get())) {
                        Ok(response) => match runtime.block_on(response.bytes()) {
                            Ok(bytes) => Some(Vec::from(bytes.as_ref())),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    };
                    sender.send(res).unwrap();
                };

                let glib_future = receiver.map(move |res| {
                    if let Some(byte_vec) = res.unwrap() {
                        if let Ok(surface) = GtkUtil::create_surface_from_bytes(&byte_vec, 64, 64, scale) {
                            favicon_image.set_from_surface(Some(&surface));
                        }
                    }
                    add_button_stack.set_visible_child_name("text");
                });

                threadpool_clone.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            } else {
                add_button_stack.set_visible_child_name("text");
            }
        });

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
        add_button_stack: &Stack,
        feed_url: &Rc<RwLock<Option<Url>>>,
        threadpool: ThreadPool,
    ) {
        let list_select_button = select_button.clone();
        list.connect_row_selected(move |_list, row| {
            list_select_button.set_sensitive(row.is_some());
        });

        let add_feed_stack = stack.clone();
        let title_entry = title_entry.clone();
        let list_clone = list.clone();
        let favicon = favicon.clone();
        let feed_url = feed_url.clone();
        let select_button_stack = select_button_stack.clone();
        let add_button_stack = add_button_stack.clone();
        select_button.connect_clicked(move |button| {
            if let Some(row) = list_clone.get_selected_row() {
                if let Some(name) = row.get_name() {
                    select_button_stack.set_visible_child_name("spinner");
                    button.set_sensitive(false);

                    let url = Url::parse(name.as_str()).expect("should never fail since it comes from 'url.as_str()'");
                    let feed_id = FeedID::new(url.get().as_str());
                    let add_feed_stack = add_feed_stack.clone();
                    let title_entry = title_entry.clone();
                    let favicon = favicon.clone();
                    let feed_url = feed_url.clone();

                    let (sender, receiver) = oneshot::channel::<Option<ParsedUrl>>();

                    let thread_future = async move {
                        let result = Runtime::new()
                            .expect(RUNTIME_ERROR)
                            .block_on(news_flash::feed_parser::download_and_parse_feed(
                                &url, &feed_id, None, None,
                            ))
                            .ok();
                        sender.send(result).unwrap();
                    };

                    let select_button_stack = select_button_stack.clone();
                    let select_button = button.clone();
                    let add_button_stack = add_button_stack.clone();
                    let threadpool_clone = threadpool.clone();
                    let glib_future = receiver.map(move |res| {
                        if let Some(ParsedUrl::SingleFeed(feed)) = res.unwrap() {
                            Self::fill_feed_page(
                                feed,
                                &add_button_stack,
                                &title_entry,
                                &favicon,
                                &feed_url,
                                threadpool_clone,
                            );
                            add_feed_stack.set_visible_child_name("feed_page");
                        } else if let Some(child) = row.get_child() {
                            if let Ok(_box) = child.downcast::<Box>() {
                                if let Some(icon) = _box.get_children().get(1) {
                                    icon.set_visible(true);
                                }
                            }
                        }

                        select_button_stack.set_visible_child_name("text");
                        select_button.set_sensitive(true);
                    });

                    threadpool.spawn_ok(thread_future);
                    Util::glib_spawn_future(glib_future);
                }
            }
        });
        for (title, url) in feed_vec {
            let label = Label::new(Some(&title));
            label.set_size_request(0, 50);
            label.set_ellipsize(EllipsizeMode::End);
            label.set_xalign(0.0);

            let warn_icon = Image::new_from_icon_name(Some(WARN_ICON), IconSize::Button);
            warn_icon.set_tooltip_text(Some("Failed to get Feed."));
            warn_icon.set_no_show_all(true);

            let gtk_box = Box::new(Orientation::Horizontal, 0);
            gtk_box.set_margin_start(20);
            gtk_box.set_margin_end(20);
            gtk_box.pack_start(&label, true, true, 0);
            gtk_box.pack_end(&warn_icon, false, false, 0);

            let row = ListBoxRow::new();

            let row_select_button = select_button.clone();
            row.connect_activate(move |_row| {
                row_select_button.activate();
            });

            row.set_selectable(true);
            row.set_activatable(false);
            row.set_name(url.get().as_str());
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

    fn calc_add_button_sensitive(title_entry: &Entry, category_entry: &Entry) -> bool {
        if let Some(text) = title_entry.get_text() {
            if text.as_str().is_empty() {
                return false;
            }
        }

        if let Some(text) = category_entry.get_text() {
            if text.as_str().is_empty() {
                return false;
            }
        }

        true
    }

    pub fn close(&self) {
        self.popover.popdown()
    }

    pub fn add_button(&self) -> Button {
        self.add_button.clone()
    }

    pub fn get_feed_url(&self) -> Option<Url> {
        self.feed_url.read().clone()
    }

    pub fn get_feed_title(&self) -> Option<String> {
        self.feed_title_entry.get_text().map(|title| title.as_str().to_owned())
    }

    pub fn get_category(&self) -> Option<AddCategory> {
        self.feed_category.read().clone()
    }
}
