use super::keybindings::Keybindings;
use crate::i18n::i18n;
use crate::util::BuilderHelper;
use gdk::enums::key;
use glib::{clone, object::IsA};
use gtk::{
    Align, Button, ButtonExt, Dialog, DialogExt, GtkWindowExt, Inhibit, Label, LabelExt, ShortcutLabel, Stack,
    StackExt, StyleContextExt, WidgetExt, Window,
};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum KeybindState {
    Enabled(String),
    Disabled,
    Canceled,
    Illegal,
}

#[derive(Debug, Clone)]
pub struct KeybindingEditor {
    widget: Dialog,
    pub keybinding: Arc<RwLock<KeybindState>>,
}

impl KeybindingEditor {
    pub fn new<D: IsA<Window> + GtkWindowExt>(settings_dialog: &D, setting_name: &str) -> Self {
        let keybinding_public: Arc<RwLock<KeybindState>> = Arc::new(RwLock::new(KeybindState::Disabled));
        let keybinding_internal: Arc<RwLock<KeybindState>> = Arc::new(RwLock::new(KeybindState::Disabled));
        let builder = BuilderHelper::new("keybind_editor");
        let set_button = builder.get::<Button>("set_button");
        let cancel_button = builder.get::<Button>("cancel_button");

        let shortcut_meta = builder.get::<Label>("shortcut_label");
        let shortcut_label = ShortcutLabel::new("");
        shortcut_label.set_halign(Align::Center);
        shortcut_label.set_valign(Align::Center);
        shortcut_label.get_style_context().add_class("h2");
        shortcut_label.show();
        let stack = builder.get::<Stack>("stack");
        let dialog = builder.get::<Dialog>("dialog");
        stack.add_named(&shortcut_label, "vis");
        dialog.set_transient_for(Some(settings_dialog));
        dialog.connect_key_press_event(clone!(
            @weak keybinding_internal,
            @weak keybinding_public,
            @weak cancel_button,
            @weak set_button => @default-panic, move |widget, event|
        {
            let keyval = event.get_keyval();
            let modifier = Keybindings::clean_modifier(event.get_state());

            if keyval == key::Escape {
                *keybinding_public.write() = KeybindState::Canceled;
                widget.emit_close();
                return Inhibit(true);
            }

            if keyval == key::BackSpace {
                shortcut_meta.set_label(&i18n("Disable Keybinding"));
                set_button.set_visible(true);
                cancel_button.set_visible(true);
                stack.set_visible_child_name("confirm");
                *keybinding_internal.write() = KeybindState::Disabled;
                return Inhibit(false);
            }

            let internal_shortcut = gtk::accelerator_name(keyval, modifier)
                .expect("Shortcut not convertable. This should never happen!")
                .to_string();

            if Keybindings::parse_keyval(keyval).is_some() {
                set_button.set_visible(true);
                cancel_button.set_visible(true);
                shortcut_label.set_accelerator(&internal_shortcut);
                stack.set_visible_child_name("vis");
                *keybinding_internal.write() = KeybindState::Enabled(internal_shortcut);
            } else {
                set_button.set_visible(false);
                shortcut_meta.set_label(&i18n("Illegal Keybinding"));
                stack.set_visible_child_name("confirm");
                *keybinding_internal.write() = KeybindState::Illegal;
            }

            Inhibit(false)
        }));

        set_button.connect_clicked(
            clone!(@weak dialog, @weak keybinding_public => @default-panic, move |_button| {
                *keybinding_public.write() = (*keybinding_internal.read()).clone();
                dialog.emit_close();
            }),
        );
        cancel_button.connect_clicked(
            clone!(@weak dialog, @weak keybinding_public => @default-panic, move |_button| {
                *keybinding_public.write() = KeybindState::Canceled;
                dialog.emit_close();
            }),
        );

        let label = builder.get::<Label>("instuction_label");
        label.set_label(setting_name);

        KeybindingEditor {
            widget: dialog,
            keybinding: keybinding_public,
        }
    }

    pub fn widget(&self) -> Dialog {
        self.widget.clone()
    }
}
