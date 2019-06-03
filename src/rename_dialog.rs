use gtk::{Button, ButtonExt, Dialog, DialogExt, Entry, EntryExt, HeaderBar, HeaderBarExt, GtkWindowExt, WidgetExt};
use crate::util::{BuilderHelper};
use crate::sidebar::models::SidebarSelection;

#[derive(Clone, Debug)]
pub struct RenameDialog {
    dialog: Dialog,
    rename_button: Button,
    cancel_button: Button,
    rename_entry: Entry,
}

impl RenameDialog {
    pub fn new(parent: &gtk::ApplicationWindow, item: &SidebarSelection) -> Self {
        let builder = BuilderHelper::new("rename_dialog");
        let header = builder.get::<HeaderBar>("headerbar");
        let rename_button = builder.get::<Button>("rename_button");
        let cancel_button = builder.get::<Button>("cancel_button");
        let rename_entry = builder.get::<Entry>("rename_entry");
        let dialog = builder.get::<Dialog>("rename_dialog");

        match item {
            SidebarSelection::All => {},
            SidebarSelection::Cateogry(_) => header.set_title("Rename Category"),
            SidebarSelection::Feed(_) => header.set_title("Rename Feed"),
            SidebarSelection::Tag(_) => header.set_title("Rename Feed"),
        }

        rename_entry.set_text(match item {
            SidebarSelection::All => "",
            SidebarSelection::Cateogry((_, name)) => name,
            SidebarSelection::Feed((_, name)) => name,
            SidebarSelection::Tag((_, name)) => name,
        });

        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_button| {
            dialog_clone.emit_close();
        });

        dialog.set_transient_for(parent);
        dialog.show_all();

        RenameDialog {
            dialog,
            rename_button,
            cancel_button,
            rename_entry,
        }
    }

    pub fn rename_button(&self) -> Button {
        self.rename_button.clone()
    }

}