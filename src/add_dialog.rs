use crate::util::BuilderHelper;
use gtk::{Button, ButtonExt, Dialog, DialogExt, Entry, EntryExt, GtkWindowExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct AddDilaog {
    dialog: Dialog,
}

impl AddDilaog {
    pub fn new(parent: &gtk::ApplicationWindow) -> Self {
        let builder = BuilderHelper::new("add_dialog");
        let dialog = builder.get::<Dialog>("add_dialog");
        let cancel_button = builder.get::<Button>("cancel_button");

        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_button| {
            dialog_clone.emit_close();
        });

        dialog.set_transient_for(Some(parent));
        dialog.show_all();

        AddDilaog {
            dialog,
        }
    }
}