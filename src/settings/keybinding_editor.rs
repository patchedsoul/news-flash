use gtk::{Inhibit, Button, ButtonExt, Dialog, DialogExt, Label, LabelExt, Window, GtkWindowExt, WidgetExt, Stack, StackExt};
use glib::{object::IsA};
use gdk::enums::key;
use crate::util::{BuilderHelper, GtkHandle};
use crate::gtk_handle;
use std::rc::Rc;
use std::cell::RefCell;
use super::keybindings::Keybindings;

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
    pub keybinding: GtkHandle<KeybindState>,
}

impl KeybindingEditor {
    pub fn new<D: IsA<Window> + GtkWindowExt>(settings_dialog: &D, setting_name: &str) -> Self {
        let keybinding_public : GtkHandle<KeybindState> = gtk_handle!(KeybindState::Disabled);
        let keybinding_internal : GtkHandle<KeybindState> = gtk_handle!(KeybindState::Disabled);
        let builder = BuilderHelper::new("keybind_editor");
        let set_button = builder.get::<Button>("set_button");
        let cancel_button = builder.get::<Button>("cancel_button");

        let shortcut_label = builder.get::<Label>("shortcut_label");
        let stack = builder.get::<Stack>("stack");
        let keybinding_internal_clone = keybinding_internal.clone();
        let keybinding_public_clone = keybinding_public.clone();
        let set_button_clone = set_button.clone();
        let cancel_button_clone = cancel_button.clone();
        let dialog = builder.get::<Dialog>("dialog");
        dialog.set_transient_for(settings_dialog);
        dialog.connect_key_press_event(move |widget, event| {
            let keyval = event.get_keyval();
            let modifier = Keybindings::clean_modifier(&event.get_state());

            stack.set_visible_child_name("confirm");

            if keyval == key::Escape {
                *keybinding_public_clone.borrow_mut() = KeybindState::Canceled;
                widget.emit_close();
                return Inhibit(true)
            }

            if keyval == key::BackSpace {
                shortcut_label.set_label("Disable Keybinding");
                set_button_clone.set_visible(true);
                cancel_button_clone.set_visible(true);
                *keybinding_internal_clone.borrow_mut() = KeybindState::Disabled;
                return Inhibit(false)
            }

            let printable_shortcut = Keybindings::parse_shortcut(keyval, &modifier);
            let internal_shortcut = gtk::accelerator_name(keyval, modifier)
                .expect("Shortcut not convertable. This should never happen!")
                .to_string();
            
            if let Some(printable_shortcut) = printable_shortcut {
                shortcut_label.set_label(&printable_shortcut);
                set_button_clone.set_visible(true);
                cancel_button_clone.set_visible(true);
                *keybinding_internal_clone.borrow_mut() = KeybindState::Enabled(internal_shortcut);
            } else {
                set_button_clone.set_visible(false);
                shortcut_label.set_label("Illegal Keybinding");
                *keybinding_internal_clone.borrow_mut() = KeybindState::Illegal;
            }
            
            Inhibit(false)
        });

        let dialog_clone_set = dialog.clone();
        let keybinding_public_set = keybinding_public.clone();
        set_button.connect_clicked(move |_button| {
            *keybinding_public_set.borrow_mut() = (*keybinding_internal.borrow()).clone();
            dialog_clone_set.emit_close();
        });
        let dialog_clone_cancel = dialog.clone();
        let keybinding_public_cancel = keybinding_public.clone();
        cancel_button.connect_clicked(move |_button| {
            *keybinding_public_cancel.borrow_mut() = KeybindState::Canceled;
            dialog_clone_cancel.emit_close();
        });

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