mod tag_row;

use gtk::{Button, ButtonExt, Popover, PopoverExt, Stack, StackExt, StackTransitionType, ListBox, ListBoxExt, WidgetExt};
use glib::clone;
use crate::util::{BuilderHelper};
use std::sync::Arc;
use parking_lot::RwLock;
use news_flash::{models::{ArticleID, Tag}, NewsFlash};
use tag_row::TagRow;

#[derive(Clone, Debug)]
pub struct TagPopover {
    pub widget: Popover,
    assigned_tags: Vec<Tag>,
    unassigned_tags: Vec<Tag>,
}

impl TagPopover {
    pub fn new(article_id: &ArticleID, news_flash: &Arc<RwLock<Option<NewsFlash>>>) -> Self {
        let builder = BuilderHelper::new("tag_dialog");
        let popover = builder.get::<Popover>("popover");
        let assigned_tags_list_stack  = builder.get::<Stack>("assigned_tags_list_stack");
        let main_stack = builder.get::<Stack>("main_stack");
        let add_button = builder.get::<Button>("add_button");
        let back_button = builder.get::<Button>("back_button");
        let possible_tags_list = builder.get::<ListBox>("possible_tags_list");
        let assinged_tag_list = builder.get::<ListBox>("assinged_tag_list");

        add_button.connect_clicked(clone!(@weak main_stack => move |_button| {
            main_stack.set_visible_child_full("possible_tags", StackTransitionType::SlideLeft);
        }));

        back_button.connect_clicked(clone!(@weak main_stack => move |_button| {
            main_stack.set_visible_child_full("assigned_tags", StackTransitionType::SlideRight);
        }));

        popover.connect_closed(clone!(@weak main_stack => move |_button| {
            main_stack.set_visible_child_full("assigned_tags", StackTransitionType::None);
        }));

        let mut assigned_tags : Vec<Tag> = Vec::new();
        let mut unassigned_tags : Vec<Tag> = Vec::new();

        if let Some(news_flash) = news_flash.read().as_ref() {
            if let Ok(mut tags) = news_flash.get_tags_of_article(article_id) {
                assigned_tags.append(&mut tags);
            }

            if let Ok(tags) = news_flash.get_tags() {
                unassigned_tags.append(&mut tags.iter().filter_map(|t| if assigned_tags.contains(t) { None } else { Some(t.clone()) }).collect::<Vec<Tag>>());
            }
        }

        if assigned_tags.is_empty() {
            assigned_tags_list_stack.set_visible_child_name("empty");
        }

        for unassigned_tag in &unassigned_tags {
            possible_tags_list.insert(&TagRow::new(&unassigned_tag, false).widget, -1);
        }

        for assigned_tag in &assigned_tags {
            assinged_tag_list.insert(&TagRow::new(&assigned_tag, true).widget, -1);
        }

        assinged_tag_list.show_all();
        possible_tags_list.show_all();

        TagPopover {
            widget: popover,
            assigned_tags,
            unassigned_tags,
        }
    }
}