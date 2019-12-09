use super::article_row::ArticleRow;
use super::models::ArticleListArticleModel;
use super::models::ArticleListModel;
use crate::app::Action;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, Util};
use glib::{object::Cast, translate::ToGlib, Sender};
use gtk::{
    AdjustmentExt, ContainerExt, Continue, ListBox, ListBoxExt, ListBoxRowExt, ScrolledWindow, ScrolledWindowExt,
    SettingsExt, TickCallbackId, WidgetExt, WidgetExtManual,
};
use news_flash::models::{
    article::{Marked, Read},
    ArticleID,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const LIST_BOTTOM_THREASHOLD: f64 = 200.0;
const SCROLL_TRANSITION_DURATION: i64 = 500 * 1000;

#[derive(Clone)]
struct ScrollAnimationProperties {
    pub start_time: GtkHandle<Option<i64>>,
    pub end_time: GtkHandle<Option<i64>>,
    pub scroll_callback_id: GtkHandle<Option<TickCallbackId>>,
    pub transition_start_value: GtkHandle<Option<f64>>,
    pub transition_diff: GtkHandle<Option<f64>>,
}

pub struct SingleArticleList {
    sender: Sender<Action>,
    scroll: ScrolledWindow,
    articles: HashMap<ArticleID, GtkHandle<ArticleRow>>,
    list: ListBox,
    select_after_signal: GtkHandle<Option<u32>>,
    scroll_cooldown: GtkHandle<bool>,
    scroll_animation_data: ScrollAnimationProperties,
}

impl SingleArticleList {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = BuilderHelper::new("article_list_single");
        let scroll = builder.get::<ScrolledWindow>("article_list_scroll");
        let list = builder.get::<ListBox>("article_list_box");

        let scroll_cooldown = gtk_handle!(false);

        let cooldown = scroll_cooldown.clone();
        let sender_clone = sender.clone();
        if let Some(vadjustment) = scroll.get_vadjustment() {
            vadjustment.connect_value_changed(move |vadj| {
                let is_on_cooldown = *cooldown.borrow();
                if !is_on_cooldown {
                    let max = vadj.get_upper() - vadj.get_page_size();
                    if max > 0.0 && vadj.get_value() >= (max - LIST_BOTTOM_THREASHOLD) {
                        *cooldown.borrow_mut() = true;
                        let cooldown = cooldown.clone();
                        gtk::timeout_add(800, move || {
                            *cooldown.borrow_mut() = false;
                            Continue(false)
                        });
                        GtkUtil::send(&sender_clone, Action::LoadMoreArticles);
                    }
                }
            });
        }

        SingleArticleList {
            sender,
            scroll,
            articles: HashMap::new(),
            list,
            select_after_signal: gtk_handle!(None),
            scroll_cooldown,
            scroll_animation_data: ScrollAnimationProperties {
                start_time: gtk_handle!(None),
                end_time: gtk_handle!(None),
                scroll_callback_id: gtk_handle!(None),
                transition_start_value: gtk_handle!(None),
                transition_diff: gtk_handle!(None),
            },
        }
    }

    pub fn widget(&self) -> gtk::ScrolledWindow {
        self.scroll.clone()
    }

    pub fn list(&self) -> gtk::ListBox {
        self.list.clone()
    }

    pub fn add(&mut self, article: &ArticleListArticleModel, pos: i32, model: &GtkHandle<ArticleListModel>) {
        let article_row = ArticleRow::new(&article, model, self.sender.clone());
        self.list.insert(&article_row.widget(), pos);
        article_row.widget().show();
        self.articles.insert(article.id.clone(), gtk_handle!(article_row));
    }

    pub fn remove(&mut self, id: ArticleID) {
        if let Some(article_row) = self.articles.get(&id) {
            self.list.remove(&article_row.borrow().widget());
        }
        let _ = self.articles.remove(&id);
    }

    pub fn clear(&mut self) {
        *self.scroll_cooldown.borrow_mut() = true;
        for row in self.list.get_children() {
            let list = self.list.clone();
            gtk::idle_add(move || {
                list.remove(&row);
                Continue(false)
            });
        }
        self.articles.clear();
        if let Some(vadjustment) = self.scroll.get_vadjustment() {
            vadjustment.set_value(0.0);
        }
        *self.scroll_cooldown.borrow_mut() = false;
    }

    pub fn update_marked(&mut self, id: &ArticleID, marked: Marked) {
        if let Some(article_handle) = self.articles.get(id) {
            article_handle.borrow_mut().update_marked(marked);
        }
    }

    pub fn update_read(&mut self, id: &ArticleID, read: Read) {
        if let Some(article_handle) = self.articles.get(id) {
            article_handle.borrow_mut().update_unread(read);
        }
    }

    pub fn get_allocated_row_height(&self, id: &ArticleID) -> Option<i32> {
        self.articles
            .get(id)
            .map(|row| row.borrow().widget().get_allocated_height())
    }

    pub fn select_after(&self, id: &ArticleID, time: u32) {
        if let Some(article_handle) = self.articles.get(id) {
            self.list.select_row(Some(&article_handle.borrow().widget()));
            // FIXME: set as read

            GtkUtil::remove_source(*self.select_after_signal.borrow());
            *self.select_after_signal.borrow_mut() = None;

            let select_after_signal = self.select_after_signal.clone();
            let article_widget = article_handle.borrow().widget();
            *self.select_after_signal.borrow_mut() = Some(
                gtk::timeout_add(time, move || {
                    // FIXME: if search entry is not selected

                    article_widget.activate();

                    *select_after_signal.borrow_mut() = None;
                    Continue(false)
                })
                .to_glib(),
            );
        }
    }

    pub fn animate_scroll_diff(&self, diff: f64) {
        let pos = self.get_scroll_value() + diff;
        self.animate_scroll_absolute(pos)
    }

    pub fn animate_scroll_absolute(&self, pos: f64) {
        let animate = match gtk::Settings::get_default() {
            Some(settings) => settings.get_property_gtk_enable_animations(),
            None => false,
        };

        if !self.widget().get_mapped() || !animate {
            return self.set_scroll_value(pos);
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
                let start_value =
                    Util::some_or_default(*self.scroll_animation_data.transition_start_value.borrow(), 0.0);
                let diff_value = Util::some_or_default(*self.scroll_animation_data.transition_diff.borrow(), 0.0);

                callback_id.remove();
                start_value + diff_value - self.get_scroll_value()
            }
            None => 0.0,
        };

        *self.scroll_animation_data.transition_diff.borrow_mut() = Some(if (pos + 1.0).abs() < 0.001 {
            self.get_scroll_upper() - self.get_scroll_page_size() - self.get_scroll_value()
        } else {
            (pos - self.get_scroll_value()) + leftover_scroll
        });

        *self.scroll_animation_data.transition_start_value.borrow_mut() = Some(self.get_scroll_value());

        let transition_start_value = self.scroll_animation_data.transition_start_value.clone();
        let transition_diff = self.scroll_animation_data.transition_diff.clone();
        let end_time = self.scroll_animation_data.end_time.clone();
        let start_time = self.scroll_animation_data.start_time.clone();
        let callback_id = self.scroll_animation_data.scroll_callback_id.clone();
        *self.scroll_animation_data.scroll_callback_id.borrow_mut() =
            Some(self.scroll.add_tick_callback(move |widget, clock| {
                let scroll = widget
                    .clone()
                    .downcast::<ScrolledWindow>()
                    .expect("Scroll tick not on ScrolledWindow");

                let start_value = Util::some_or_default(*transition_start_value.borrow(), 0.0);
                let diff_value = Util::some_or_default(*transition_diff.borrow(), 0.0);
                let now = clock.get_frame_time();
                let end_time_value = Util::some_or_default(*end_time.borrow(), 0);
                let start_time_value = Util::some_or_default(*start_time.borrow(), 0);

                if !widget.get_mapped() {
                    Self::set_scroll_value_static(&scroll, start_value + diff_value);
                    return Continue(false);
                }

                if end_time.borrow().is_none() {
                    return Continue(false);
                }

                let t = if now < end_time_value {
                    (now - start_time_value) as f64 / (end_time_value - start_time_value) as f64
                } else {
                    1.0
                };

                let t = Util::ease_out_cubic(t);

                Self::set_scroll_value_static(&scroll, start_value + (t * diff_value));

                if Self::get_scroll_value_static(&scroll) <= 0.0 || now >= end_time_value {
                    scroll.queue_draw();
                    *transition_start_value.borrow_mut() = None;
                    *transition_diff.borrow_mut() = None;
                    *start_time.borrow_mut() = None;
                    *end_time.borrow_mut() = None;
                    *callback_id.borrow_mut() = None;
                    return Continue(false);
                }

                Continue(true)
            }));
    }

    fn set_scroll_value(&self, pos: f64) {
        Self::set_scroll_value_static(&self.scroll, pos)
    }

    fn set_scroll_value_static(scroll: &ScrolledWindow, pos: f64) {
        if let Some(vadjustment) = scroll.get_vadjustment() {
            let pos = if (pos + 1.0).abs() < 0.001 {
                vadjustment.get_upper() - vadjustment.get_page_size()
            } else {
                pos
            };
            vadjustment.set_value(pos);
        }
    }

    fn get_scroll_value(&self) -> f64 {
        Self::get_scroll_value_static(&self.scroll)
    }

    fn get_scroll_value_static(scroll: &ScrolledWindow) -> f64 {
        match scroll.get_vadjustment() {
            Some(adj) => adj.get_value(),
            None => 0.0,
        }
    }

    fn get_scroll_upper(&self) -> f64 {
        Self::get_scroll_upper_static(&self.scroll)
    }

    fn get_scroll_upper_static(scroll: &ScrolledWindow) -> f64 {
        match scroll.get_vadjustment() {
            Some(adj) => adj.get_upper(),
            None => 0.0,
        }
    }

    fn get_scroll_page_size(&self) -> f64 {
        Self::get_scroll_page_size_static(&self.scroll)
    }

    fn get_scroll_page_size_static(scroll: &ScrolledWindow) -> f64 {
        match scroll.get_vadjustment() {
            Some(adj) => adj.get_page_size(),
            None => 0.0,
        }
    }

    pub fn get_selected_index(&self) -> Option<i32> {
        self.list.get_selected_row().map(|row| row.get_index())
    }
}
