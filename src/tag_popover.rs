use gtk::{Popover, PopoverExt, Widget, WidgetExt};
use crate::util::{BuilderHelper};

#[derive(Clone, Debug)]
pub struct TagPopover {
    pub widget: Popover,
}

impl TagPopover {
    pub fn new(parent: &Widget) -> Self {
        let builder = BuilderHelper::new("add_dialog");
        let popover = builder.get::<Popover>("popover");

        popover.set_relative_to(Some(parent));
        popover.show_all();

        TagPopover {
            widget: popover,
        }
    }
}