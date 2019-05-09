use gtk::{Dialog, Window, GtkWindowExt};
use glib::{object::IsA};
use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkHandle};

pub struct KeybindingEditor {
    widget: Dialog,
}

impl KeybindingEditor {
    pub fn new<D: IsA<Window> + GtkWindowExt>(settings_dialog: &D, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("keybind_editor");

        let dialog = builder.get::<Dialog>("dialog");
        dialog.set_transient_for(settings_dialog);

        KeybindingEditor {
            widget: dialog,
        }
    }

    pub fn widget(&self) -> Dialog {
        self.widget.clone()
    }
}