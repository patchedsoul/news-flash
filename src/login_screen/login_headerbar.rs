use crate::app::Action;
use crate::util::{BuilderHelper, GtkUtil};
use glib::Sender;
use gtk::{Button, ButtonExt, HeaderBar};

#[derive(Clone, Debug)]
pub struct LoginHeaderbar {
    widget: gtk::HeaderBar,
}

impl LoginHeaderbar {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let headerbar = builder.get::<HeaderBar>("login_headerbar");
        let button = builder.get::<Button>("back_button");
        button.connect_clicked(move |_button| {
            GtkUtil::send(&sender, Action::ShowWelcomePage);
        });

        LoginHeaderbar { widget: headerbar }
    }
}
