use crate::util::{BuilderHelper, GtkUtil};
use gtk::{Button, ButtonExt, HeaderBar};

#[derive(Clone, Debug)]
pub struct LoginHeaderbar {
    widget: gtk::HeaderBar,
}

impl LoginHeaderbar {
    pub fn new(builder: &BuilderHelper) -> Self {
        let headerbar = builder.get::<HeaderBar>("login_headerbar");
        let button = builder.get::<Button>("back_button");
        button.connect_clicked(|button| {
            GtkUtil::execute_action(button, "show-welcome-page", None);
        });

        LoginHeaderbar { widget: headerbar }
    }
}
