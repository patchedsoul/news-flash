use crate::util::{BuilderHelper, GtkUtil};
use gtk::{
    Button, ButtonExt, ComboBoxExt, ComboBoxText, ComboBoxTextExt, ContainerExt, EditableSignals, Entry, EntryExt,
    Image, ImageExt, Label, LabelExt, ListBox, ListBoxExt, ListBoxRow, ListBoxRowExt, Orientation, Popover, PopoverExt,
    Separator, Stack, StackExt, StyleContextExt, WidgetExt,
};
use news_flash::models::{Category, Feed, FeedID, Url};
use news_flash::ParsedUrl;
use pango::EllipsizeMode;

#[derive(Clone, Debug)]
pub struct AddPopover {
    popover: Popover,
}

impl AddPopover {
    pub fn new(parent: &Button, categories: &[Category]) -> Self {
        let builder = BuilderHelper::new("add_dialog");
        let popover = builder.get::<Popover>("add_pop");
        let url_entry = builder.get::<Entry>("url_entry");
        let parse_button = builder.get::<Button>("parse_button");
        let add_feed_stack = builder.get::<Stack>("add_feed_stack");
        let feed_list = builder.get::<ListBox>("feed_list");
        let select_button = builder.get::<Button>("select_button");
        let feed_title_entry = builder.get::<Entry>("feed_title_entry");
        let favicon_image = builder.get::<Image>("favicon_image");
        let category_stack = builder.get::<Stack>("category_stack");
        let add_button = builder.get::<Button>("add_button");
        let category_entry = builder.get::<Entry>("category_entry");
        let category_combo = builder.get::<ComboBoxText>("category_combo");
        let category_type_combo = builder.get::<ComboBoxText>("category_type_combo");

        let category_stack_clone = category_stack.clone();
        category_type_combo.connect_changed(move |combo| {
            if let Some(id) = combo.get_active_id() {
                if id == "new" {
                    category_stack_clone.set_visible_child_name("new_category");
                } else {
                    category_stack_clone.set_visible_child_name("existing_category");
                }
            }
        });

        // setup list of categories to add feed to
        if categories.is_empty() {
            category_combo.set_sensitive(false);
        } else {
            for category in categories {
                category_combo.append(Some(category.category_id.to_str()), &category.label);
            }
            if let Some(first_category) = categories.get(0) {
                category_combo.set_active_id(Some(first_category.category_id.to_str()));
            }
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
        parse_button.connect_clicked(move |_button| {
            if let Some(url_text) = url_entry.get_text() {
                let mut url_text = url_text.as_str().to_owned();
                if !url_text.starts_with("http://") && !url_text.starts_with("https://") {
                    url_text.insert_str(0, "https://");
                }
                if let Ok(url) = Url::parse(&url_text) {
                    let feed_id = FeedID::new(&url_text);
                    if let Ok(result) = news_flash::feed_parser::download_and_parse_feed(&url, &feed_id, None, None) {
                        match result {
                            ParsedUrl::MultipleFeeds(feed_vec) => {
                                parse_button_add_feed_stack.set_visible_child_name("feed_selection_page");
                                Self::fill_mupliple_feed_list(
                                    feed_vec,
                                    &parse_button_feed_list,
                                    &parse_button_select_button,
                                    &parse_button_add_feed_stack,
                                    &parse_button_feed_title_entry,
                                    &parse_button_favicon_image,
                                );
                            }
                            ParsedUrl::SingleFeed(feed) => {
                                parse_button_add_feed_stack.set_visible_child_name("feed_page");
                                Self::fill_feed_page(feed, &parse_button_feed_title_entry, &parse_button_favicon_image);
                            }
                        }
                    } else {
                        // FIXME
                        // downloading or parsing feed failed
                        // indicate in UI
                    }
                } else {
                    // FIXME
                    // failed to parse url
                    // indicate in UI
                }
            }
        });

        // make add_button sensitive / insensitive
        let category_stack_add_button = add_button.clone();
        let category_stack_title_entry = feed_title_entry.clone();
        let category_stack_category_entry = category_entry.clone();
        let category_stack_category_combo = category_combo.clone();
        category_stack.connect_property_visible_child_notify(move |stack| {
            let sensitive = Self::calc_add_button_sensitice(
                &stack,
                &category_stack_title_entry,
                &category_stack_category_entry,
                &category_stack_category_combo,
            );
            category_stack_add_button.set_sensitive(sensitive);
        });

        let category_entry_add_button = add_button.clone();
        let category_entry_title_entry = feed_title_entry.clone();
        let category_entry_category_stack = category_stack.clone();
        let category_entry_category_combo = category_combo.clone();
        category_entry.connect_changed(move |entry| {
            let sensitive = Self::calc_add_button_sensitice(
                &category_entry_category_stack,
                &category_entry_title_entry,
                &entry,
                &category_entry_category_combo,
            );
            category_entry_add_button.set_sensitive(sensitive);
        });

        let category_combo_add_button = add_button.clone();
        let category_combo_title_entry = feed_title_entry.clone();
        let category_combo_category_entry = category_entry.clone();
        let category_combo_category_stack = category_stack.clone();
        category_combo.connect_changed(move |combo| {
            let sensitive = Self::calc_add_button_sensitice(
                &category_combo_category_stack,
                &category_combo_title_entry,
                &category_combo_category_entry,
                &combo,
            );
            category_combo_add_button.set_sensitive(sensitive);
        });

        let title_entry_add_button = add_button.clone();
        let title_entry_category_entry = category_entry.clone();
        let title_entry_category_stack = category_stack.clone();
        let title_entry_category_combo = category_combo.clone();
        feed_title_entry.connect_changed(move |entry| {
            let sensitive = Self::calc_add_button_sensitice(
                &title_entry_category_stack,
                &entry,
                &title_entry_category_entry,
                &title_entry_category_combo,
            );
            title_entry_add_button.set_sensitive(sensitive);
        });

        popover.set_relative_to(Some(parent));
        popover.show_all();

        AddPopover { popover }
    }

    fn fill_feed_page(feed: Feed, title_entry: &Entry, favicon_image: &Image) {
        title_entry.set_text(&feed.label);
        let scale = favicon_image.get_style_context().get_scale();

        if let Some(favicon) = news_flash::util::favicon_cache::FavIconCache::scrap(&feed) {
            if let Some(data) = &favicon.data {
                let surface = GtkUtil::create_surface_from_bytes(data, 32, 32, scale).unwrap();
                favicon_image.set_from_surface(Some(&surface));
            }
        } else if let Some(icon_url) = feed.icon_url {
            if let Ok(mut response) = reqwest::get(icon_url.get()) {
                let mut buf: Vec<u8> = vec![];
                if let Ok(_bytes_written) = response.copy_to(&mut buf) {
                    if let Ok(surface) = GtkUtil::create_surface_from_bytes(&buf, 32, 32, scale) {
                        favicon_image.set_from_surface(Some(&surface));
                    }
                }
            }
        }
    }

    fn fill_mupliple_feed_list(
        feed_vec: Vec<(String, Url)>,
        list: &ListBox,
        select_button: &Button,
        stack: &Stack,
        title_entry: &Entry,
        favicon: &Image,
    ) {
        let list_select_button = select_button.clone();
        list.connect_row_selected(move |_list, row| {
            list_select_button.set_sensitive(row.is_some());
        });

        let add_feed_stack = stack.clone();
        let title_entry = title_entry.clone();
        let list_clone = list.clone();
        let favicon = favicon.clone();
        select_button.connect_clicked(move |_button| {
            if let Some(row) = list_clone.get_selected_row() {
                if let Some(name) = row.get_name() {
                    // should never fail since it comes from `url.as_str()`
                    let url = Url::parse(name.as_str()).unwrap();
                    let feed_id = FeedID::new(url.get().as_str());
                    if let Ok(ParsedUrl::SingleFeed(feed)) =
                        news_flash::feed_parser::download_and_parse_feed(&url, &feed_id, None, None)
                    {
                        Self::fill_feed_page(feed, &title_entry, &favicon);
                        add_feed_stack.set_visible_child_name("feed_page");
                    }
                }
            }
        });
        for (title, url) in feed_vec {
            let label = Label::new(Some(&title));
            label.set_size_request(0, 50);
            label.set_ellipsize(EllipsizeMode::End);
            label.set_xalign(0.0);
            label.set_margin_start(20);
            label.set_margin_end(20);
            let row = ListBoxRow::new();
            row.set_selectable(true);
            row.set_activatable(false);
            row.set_name(url.get().as_str());
            row.add(&label);
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

    fn calc_add_button_sensitice(
        stack: &Stack,
        title_entry: &Entry,
        category_entry: &Entry,
        category_combo: &ComboBoxText,
    ) -> bool {
        if let Some(text) = title_entry.get_text() {
            if text.as_str().is_empty() {
                return false;
            }
        }

        if let Some(child_name) = stack.get_visible_child_name() {
            let child_name = child_name.as_str();
            if child_name == "new_category" {
                if let Some(text) = category_entry.get_text() {
                    if text.as_str().is_empty() {
                        return false;
                    }
                }
            } else if child_name == "existing_category" {
                if category_combo.get_active_id().is_none() {
                    return false;
                }
            }
        }

        true
    }
}
