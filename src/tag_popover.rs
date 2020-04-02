use gtk::{Popover, PopoverExt, Widget, WidgetExt};
use crate::util::{BuilderHelper};
use glib::object::{IsA, Object};
use std::sync::Arc;
use parking_lot::RwLock;
use news_flash::{models::{ArticleID, Tag}, NewsFlash};

#[derive(Clone, Debug)]
pub struct TagPopover {
    pub widget: Popover,
    assigned_tags: Vec<Tag>,
    unassigned_tags: Vec<Tag>,
}

impl TagPopover {
    pub fn new<W: IsA<Object> + IsA<Widget> + WidgetExt + Clone>(parent: &W, article_id: &ArticleID, news_flash: &Arc<RwLock<Option<NewsFlash>>>) -> Self {
        let builder = BuilderHelper::new("tag_dialog");
        let popover = builder.get::<Popover>("popover");

        popover.set_relative_to(Some(parent));
        popover.popup();

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

        TagPopover {
            widget: popover,
            assigned_tags,
            unassigned_tags,
        }
    }
}