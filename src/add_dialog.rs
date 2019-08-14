use crate::util::BuilderHelper;
use gtk::{Dialog, GtkWindowExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct AddDilaog {
    dialog: Dialog,
}

impl AddDilaog {
    pub fn new(parent: &gtk::ApplicationWindow) -> Self {
        let builder = BuilderHelper::new("add_dialog");
        let dialog = builder.get::<Dialog>("add_dialog");

        dialog.set_transient_for(Some(parent));
        dialog.show_all();

        AddDilaog { dialog }
    }
}
