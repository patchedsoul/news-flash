use crate::Resources;
use crate::util::{GTK_RESOURCE_FILE_ERROR, GTK_BUILDER_ERROR};
use std::str;
use gtk::{Builder, Widget};
use glib::object::{IsA, Object};

pub struct BuilderHelper {
    builder: Builder,
}

impl BuilderHelper {
    pub fn new(ui_file: &str) -> Self {
        let ui_data = Resources::get(&format!("ui/{}.ui", ui_file)).expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);
        let builder = Builder::new_from_string(ui_string);

        BuilderHelper {
            builder
        }
    }

    pub fn get<T: IsA<Widget> + IsA<Object>>(&self, name: &str) -> T {
        let widget: T = self.builder.get_object(name).expect(GTK_BUILDER_ERROR);
        widget
    }
}