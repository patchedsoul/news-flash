use crate::util::{GTK_BUILDER_ERROR, GTK_RESOURCE_FILE_ERROR};
use crate::Resources;
use glib::object::{IsA, Object};
use gtk::{prelude::BuilderExtManual, Builder, Widget};
use std::str;

pub struct BuilderHelper {
    builder: Builder,
}

impl BuilderHelper {
    pub fn new(ui_file: &str) -> Self {
        let ui_data = Resources::get(&format!("ui/{}.ui", ui_file)).expect(GTK_RESOURCE_FILE_ERROR);
        let ui_xml = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);
        Self::new_from_xml(ui_xml)
    }

    pub fn new_from_xml(ui_xml: &str) -> Self {
        let builder = Builder::new_from_string(ui_xml);
        BuilderHelper { builder }
    }

    pub fn get<T: IsA<Widget> + IsA<Object>>(&self, name: &str) -> T {
        let widget: T = self.builder.get_object(name).expect(GTK_BUILDER_ERROR);
        widget
    }
}
