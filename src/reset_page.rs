use crate::util::{BuilderHelper, Util};
use crate::app::Action;
use glib::Sender;
use gtk::{Button, ButtonExt, Stack, StackExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct ResetPage;

impl ResetPage {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let reset_button = builder.get::<Button>("reset_button");
        let cancel_button = builder.get::<Button>("cancel_button");
        let reset_stack = builder.get::<Stack>("reset_stack");

        let reset_sender = sender.clone();
        reset_button.connect_clicked(move |button| {
            reset_stack.set_visible_child_name("reset_spinner");
            button.set_sensitive(false);
            Util::send(&reset_sender, Action::ResetAccount);
        });

        let cancel_sender = sender.clone();
        cancel_button.connect_clicked(move |_button| {
            Util::send(&cancel_sender, Action::ShowContentPage(None));
        });

        ResetPage
    }
}