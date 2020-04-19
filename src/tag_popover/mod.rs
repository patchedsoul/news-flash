mod tag_row;

use crate::app::Action;
use crate::util::{BuilderHelper, GtkUtil, Util};
use gdk::EventType;
use glib::{clone, object::Cast, translate::ToGlib, Sender};
use gtk::{
    Button, ButtonExt, ContainerExt, Inhibit, ListBox, ListBoxExt, ListBoxRow, ListBoxRowExt, Popover, PopoverExt,
    Stack, StackExt, StackTransitionType, WidgetExt,
};
use log::info;
use news_flash::{
    models::{ArticleID, Tag},
    NewsFlash,
};
use parking_lot::RwLock;
use std::sync::Arc;
use tag_row::TagRow;

#[derive(Clone, Debug)]
pub struct TagPopover {
    pub widget: Popover,
    add_button: Button,
    back_button: Button,
    assigned_tag_list: ListBox,
    unassigned_tags_list: ListBox,
    assigned_tags_list_stack: Stack,
    unassigned_tags_list_stack: Stack,
    assigned_tags: Arc<RwLock<Vec<Tag>>>,
    unassigned_tags: Arc<RwLock<Vec<Tag>>>,
    add_button_signal: Option<u64>,
    back_button_signal: Option<u64>,
    popover_close_signal: Option<u64>,
    assigned_click_signal: Option<u64>,
    unassigned_click_signal: Option<u64>,
}

impl TagPopover {
    pub fn new(article_id: &ArticleID, news_flash: &Arc<RwLock<Option<NewsFlash>>>, sender: &Sender<Action>) -> Self {
        let builder = BuilderHelper::new("tag_dialog");
        let popover = builder.get::<Popover>("popover");
        let assigned_tags_list_stack = builder.get::<Stack>("assigned_tags_list_stack");
        let unassigned_tags_list_stack = builder.get::<Stack>("possible_tags_list_stack");
        let main_stack = builder.get::<Stack>("main_stack");
        let add_button = builder.get::<Button>("add_button");
        let back_button = builder.get::<Button>("back_button");
        let unassigned_tags_list = builder.get::<ListBox>("possible_tags_list");
        let assigned_tag_list = builder.get::<ListBox>("assinged_tag_list");

        let assigned_tags: Arc<RwLock<Vec<Tag>>> = Arc::new(RwLock::new(Vec::new()));
        let unassigned_tags: Arc<RwLock<Vec<Tag>>> = Arc::new(RwLock::new(Vec::new()));

        let add_button_signal = Some(
            add_button
                .connect_clicked(clone!(@weak main_stack => move |_button| {
                    main_stack.set_visible_child_full("possible_tags", StackTransitionType::SlideLeft);
                }))
                .to_glib(),
        );

        let back_button_signal = Some(
            back_button
                .connect_clicked(clone!(@weak main_stack => move |_button| {
                    main_stack.set_visible_child_full("assigned_tags", StackTransitionType::SlideRight);
                }))
                .to_glib(),
        );

        let popover_close_signal = Some(
            popover
                .connect_closed(clone!(@weak main_stack => move |_button| {
                    main_stack.set_visible_child_full("assigned_tags", StackTransitionType::None);
                }))
                .to_glib(),
        );

        let assigned_click_signal = Some(
            assigned_tag_list
                .connect_row_activated(|_list, _row| {
                    info!("click");
                })
                .to_glib(),
        );

        let unassigned_click_signal = Some(
            unassigned_tags_list
                .connect_row_activated(clone!(
                    @strong assigned_tags,
                    @strong unassigned_tags,
                    @strong article_id,
                    @strong sender,
                    @weak main_stack,
                    @weak assigned_tags_list_stack,
                    @weak unassigned_tags_list_stack,
                    @weak assigned_tag_list => @default-panic, move |list, row|
                {
                    let index = row.get_index();
                    let tag = unassigned_tags.write().remove(index as usize);

                    Util::send(&sender, Action::TagArticle(article_id.clone(), tag.tag_id.clone()));

                    let tag_row = TagRow::new(&tag, true);
                    Self::setup_tag_row(
                        &article_id,
                        &sender,
                        &tag_row,
                        &assigned_tags,
                        &unassigned_tags,
                        &assigned_tag_list,
                        list,
                        &assigned_tags_list_stack,
                        &unassigned_tags_list_stack);
                    assigned_tag_list.insert(&tag_row.widget, -1);
                    assigned_tag_list.show_all();

                    assigned_tags.write().push(tag);

                    if assigned_tags.read().is_empty() {
                        assigned_tags_list_stack.set_visible_child_name("empty");
                    } else {
                        assigned_tags_list_stack.set_visible_child_name("list");
                    }

                    if unassigned_tags.read().is_empty() {
                        unassigned_tags_list_stack.set_visible_child_name("empty");
                    } else  {
                        unassigned_tags_list_stack.set_visible_child_name("list");
                    }

                    main_stack.set_visible_child_full("assigned_tags", StackTransitionType::SlideRight);
                    list.remove(row);
                }))
                .to_glib(),
        );

        let popover = TagPopover {
            widget: popover,
            add_button,
            back_button,
            assigned_tag_list,
            unassigned_tags_list,
            assigned_tags_list_stack,
            unassigned_tags_list_stack,
            assigned_tags,
            unassigned_tags,
            add_button_signal,
            back_button_signal,
            popover_close_signal,
            assigned_click_signal,
            unassigned_click_signal,
        };
        popover.update(article_id, news_flash, sender);
        popover
    }

    pub fn disconnect(&self) {
        GtkUtil::disconnect_signal(self.add_button_signal, &self.add_button);
        GtkUtil::disconnect_signal(self.back_button_signal, &self.back_button);
        GtkUtil::disconnect_signal(self.popover_close_signal, &self.widget);
        GtkUtil::disconnect_signal(self.assigned_click_signal, &self.assigned_tag_list);
        GtkUtil::disconnect_signal(self.unassigned_click_signal, &self.unassigned_tags_list);
    }

    pub fn update(&self, article_id: &ArticleID, news_flash: &Arc<RwLock<Option<NewsFlash>>>, sender: &Sender<Action>) {
        self.unassigned_tags.write().clear();
        self.assigned_tags.write().clear();

        for row in &self.assigned_tag_list.get_children() {
            self.assigned_tag_list.remove(row);
        }
        for row in &self.unassigned_tags_list.get_children() {
            self.unassigned_tags_list.remove(row);
        }

        if let Some(news_flash) = news_flash.read().as_ref() {
            if let Ok(mut tags) = news_flash.get_tags_of_article(article_id) {
                self.assigned_tags.write().append(&mut tags);
            }

            if let Ok(tags) = news_flash.get_tags() {
                self.unassigned_tags.write().append(
                    &mut tags
                        .iter()
                        .filter_map(|t| {
                            if self.assigned_tags.read().contains(t) {
                                None
                            } else {
                                Some(t.clone())
                            }
                        })
                        .collect::<Vec<Tag>>(),
                );
            }
        }

        if self.assigned_tags.read().is_empty() {
            self.assigned_tags_list_stack.set_visible_child_name("empty");
        } else {
            self.assigned_tags_list_stack.set_visible_child_name("list");
        }

        if self.unassigned_tags.read().is_empty() {
            self.unassigned_tags_list_stack.set_visible_child_name("empty");
        } else {
            self.unassigned_tags_list_stack.set_visible_child_name("list");
        }

        for unassigned_tag in &(*self.unassigned_tags.read()) {
            self.unassigned_tags_list
                .insert(&TagRow::new(&unassigned_tag, false).widget, -1);
        }

        for assigned_tag in &(*self.assigned_tags.read()) {
            let tag_row = TagRow::new(&assigned_tag, true);
            Self::setup_tag_row(
                &article_id,
                &sender,
                &tag_row,
                &self.assigned_tags,
                &self.unassigned_tags,
                &self.assigned_tag_list,
                &self.unassigned_tags_list,
                &self.assigned_tags_list_stack,
                &self.unassigned_tags_list_stack);
            self.assigned_tag_list
                .insert(&tag_row.widget, -1);
        }

        self.assigned_tag_list.show_all();
        self.unassigned_tags_list.show_all();
    }

    fn setup_tag_row(
        article_id: &ArticleID,
        sender: &Sender<Action>,
        tag_row: &TagRow,
        assigned_tags: &Arc<RwLock<Vec<Tag>>>,
        unassigned_tags: &Arc<RwLock<Vec<Tag>>>,
        assigned_tag_list: &ListBox,
        unassigned_tags_list: &ListBox,
        assigned_tags_list_stack: &Stack,
        unassigned_tags_list_stack: &Stack,
    ) {
        tag_row.eventbox.connect_button_press_event(clone!(
            @strong assigned_tags,
            @strong unassigned_tags,
            @strong article_id,
            @strong sender,
            @weak assigned_tags_list_stack,
            @weak unassigned_tags_list_stack,
            @weak unassigned_tags_list,
            @weak assigned_tag_list => @default-panic, move |widget, event|
        {
            if event.get_event_type() == EventType::ButtonPress {
                if event.get_button() != 1 {
                    return Inhibit(false);
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return Inhibit(false),
                }

                let mut index = -1;
                for row in &assigned_tag_list.get_children() {
                    if widget.is_ancestor(row) {
                        if let Ok(listboxrow) = row.clone().downcast::<ListBoxRow>() {
                            index = listboxrow.get_index();
                            assigned_tag_list.remove(&listboxrow);
                            break;
                        }
                    }
                }

                let tag = assigned_tags.write().remove(index as usize);
                let tag_row = TagRow::new(&tag, false);
                unassigned_tags_list.insert(&tag_row.widget, -1);
                unassigned_tags_list.show_all();

                Util::send(&sender, Action::UntagArticle(article_id.clone(), tag.tag_id.clone()));

                unassigned_tags.write().push(tag);

                if assigned_tags.read().is_empty() {
                    assigned_tags_list_stack.set_visible_child_name("empty");
                } else {
                    assigned_tags_list_stack.set_visible_child_name("list");
                }

                if unassigned_tags.read().is_empty() {
                    unassigned_tags_list_stack.set_visible_child_name("empty");
                } else  {
                    unassigned_tags_list_stack.set_visible_child_name("list");
                }
            }
            Inhibit(false)
        }));
    }
}
