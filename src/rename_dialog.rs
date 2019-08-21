use crate::sidebar::models::SidebarSelection;
use crate::util::BuilderHelper;
use gtk::{Button, Dialog, DialogExt, Entry, EntryExt, GtkWindowExt, HeaderBar, HeaderBarExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct RenameDialog {
    dialog: Dialog,
    rename_button: Button,
    rename_entry: Entry,
}

impl RenameDialog {
    pub fn new(parent: &gtk::ApplicationWindow, item: &SidebarSelection) -> Self {
        let builder = BuilderHelper::new("rename_dialog");
        let header = builder.get::<HeaderBar>("headerbar");
        let rename_button = builder.get::<Button>("rename_button");
        let rename_entry = builder.get::<Entry>("rename_entry");
        let dialog = builder.get::<Dialog>("rename_dialog");

        match item {
            SidebarSelection::All => {}
            SidebarSelection::Cateogry(_) => header.set_title(Some("Rename Category")),
            SidebarSelection::Feed(_) => header.set_title(Some("Rename Feed")),
            SidebarSelection::Tag(_) => header.set_title(Some("Rename Feed")),
        }

        rename_entry.set_text(match item {
            SidebarSelection::All => "",
            SidebarSelection::Cateogry((_, name)) => name,
            SidebarSelection::Feed((_, name)) => name,
            SidebarSelection::Tag((_, name)) => name,
        });

        dialog.set_transient_for(Some(parent));
        dialog.show_all();

        RenameDialog {
            dialog,
            rename_button,
            rename_entry,
        }
    }

    pub fn rename_button(&self) -> Button {
        self.rename_button.clone()
    }

    pub fn new_label(&self) -> Option<String> {
        self.rename_entry.get_text().map(|label| label.to_owned())
    }

    pub fn close(&self) {
        self.dialog.emit_close()
    }
}
