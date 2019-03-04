use std::rc::Rc;
use std::cell::RefCell;
use gtk::{
    self,
    LabelExt,
    ImageExt,
    WidgetExt,
    RevealerExt,
    BinExt,
    Inhibit,
    StyleContextExt,
};
use gdk::{
    NotifyType,
    EventType,
};
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use news_flash::models::{
    PluginInfo,
    PluginIcon,
    ServiceLicense,
    ServiceType,
    ServicePrice,
};
use crate::gtk_util::GtkUtil;
use crate::main_window::GtkHandle;

#[derive(Clone, Debug)]
pub struct ServiceRow {
    row: gtk::EventBox,
    arrow_revealer: GtkHandle<gtk::Revealer>,
    arrow_event: GtkHandle<gtk::EventBox>,
    arrow_image: GtkHandle<gtk::Image>,
    info_revealer: GtkHandle<gtk::Revealer>,
    show_info: bool,
}

impl ServiceRow {
    pub fn new(info: PluginInfo) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/service_row.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let row : gtk::EventBox = builder.get_object("service_row").ok_or(format_err!("some err"))?;
        let label : gtk::Label = builder.get_object("label").ok_or(format_err!("some err"))?;
        label.set_label(&info.name);
        let arrow_revealer : gtk::Revealer = builder.get_object("arrow_revealer").ok_or(format_err!("some err"))?;
        let info_revealer : gtk::Revealer = builder.get_object("info_revealer").ok_or(format_err!("some err"))?;

        // Website
        let website_label : gtk::Label = builder.get_object("website_label").ok_or(format_err!("some err"))?;
        if let Some(website) = info.website {
            let website_string = website.get().to_string();
            website_label.set_markup(&format!("<a href=\"{}\">{}</a>", website_string, website_string));
        }
        else {
            website_label.set_text("No Website");
        }

        let license_label : gtk::Label = builder.get_object("license_label").ok_or(format_err!("some err"))?;
        let license_string = match info.license_type {
            ServiceLicense::ApacheV2 => "<a href=\"http://www.apache.org/licenses/LICENSE-2.0\">Apache v2</a>",
            ServiceLicense::GPLv2 => "<a href=\"http://www.gnu.de/documents/gpl-2.0.en.html\">GPLv2</a>",
            ServiceLicense::GPlv3 => "<a href=\"http://www.gnu.de/documents/gpl-3.0.en.html\">GPLv3</a>",
            ServiceLicense::MIT => "<a href=\"https://opensource.org/licenses/MIT\">MIT</a>",
            ServiceLicense::LGPLv21 => "<a href=\"http://www.gnu.de/documents/lgpl-2.1.en.html\">LGPLv2.1</a>",
            ServiceLicense::GenericFree => "Unknown Free License",
            ServiceLicense::GenericProprietary => "Proprietary Software",
        };
        license_label.set_markup(license_string);

        let type_label : gtk::Label = builder.get_object("type_label").ok_or(format_err!("some err"))?;
        let type_string = match info.service_type {
            ServiceType::Local => "Local data only",
            ServiceType::Remote{self_hosted} => {
                let mut string = "Synced";
                if self_hosted {
                    string = "Synced & self hosted";
                }
                string
            },
        };
        type_label.set_text(type_string);

        let price_label : gtk::Label = builder.get_object("price_label").ok_or(format_err!("some err"))?;
        let price_string = match info.service_price {
            ServicePrice::Free => "Free",
            ServicePrice::Paid => "Paid",
            ServicePrice::PaidPremimum => "Free / Paid Premium",
        };
        price_label.set_text(price_string);

        let arrow_event : gtk::EventBox = builder.get_object("arrow_event").ok_or(format_err!("some err"))?;
        arrow_event.connect_leave_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(0.8);
            gtk::Inhibit(false)
        });
        arrow_event.connect_enter_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(1.0);
            gtk::Inhibit(false)
        });

        let arrow_image : gtk::Image = builder.get_object("arrow_image").ok_or(format_err!("some err"))?;

        let ctx = row.get_style_context().ok_or(format_err!("some err"))?;
        let scale = ctx.get_scale();

        let image : gtk::Image = builder.get_object("icon").ok_or(format_err!("get icon widget"))?;
        if let Some(icon) = info.icon {
            let surface = match icon {
                PluginIcon::Vector(icon) => GtkUtil::create_surface_from_bytes(&icon.data, 64, 64, scale)?,
                PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(&icon, scale)?,
            };
            image.set_from_surface(&surface);
        }
        else {
            // FIXME: default Icon
        }

        let service_row = ServiceRow {
            row: row,
            arrow_revealer: Rc::new(RefCell::new(arrow_revealer)),
            arrow_event: Rc::new(RefCell::new(arrow_event)),
            arrow_image: Rc::new(RefCell::new(arrow_image)),
            info_revealer: Rc::new(RefCell::new(info_revealer)),
            show_info: false,
        };
        let self_handle = Rc::new(RefCell::new(service_row.clone()));
        service_row.setup_events(self_handle);

        Ok(service_row)
    }

    fn setup_events(&self, handle: GtkHandle<ServiceRow>) {
        let arrow_revealer_1 = self.arrow_revealer.clone();
        let arrow_revealer_2 = self.arrow_revealer.clone();
        let handle_1 = handle.clone();
        self.row.connect_enter_notify_event(move |_widget, crossing| {
            if crossing.get_detail() != NotifyType::Inferior{
                arrow_revealer_1.borrow().set_reveal_child(true);
            }
            Inhibit(false)
        });
        self.row.connect_leave_notify_event(move |_widget, crossing| {
            if crossing.get_detail() != NotifyType::Inferior{
                if !handle_1.borrow().show_info {
                    arrow_revealer_2.borrow().set_reveal_child(false);
                }
            }
            Inhibit(false)
        });

        let info_revealer = self.info_revealer.clone();
        self.arrow_event.borrow().connect_button_press_event(move |widget, event| {
            if event.get_button() != 1 {
                return gtk::Inhibit(false)
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            let arrow_image = widget.get_child().unwrap();
            let context = arrow_image.get_style_context().unwrap();
            let expanded = handle.borrow().show_info;
            if !expanded {
                context.add_class("backward-arrow-expanded");
                context.remove_class("backward-arrow-collapsed");
                info_revealer.borrow().set_reveal_child(true);
            }
            else {
                context.remove_class("backward-arrow-expanded");
                context.add_class("backward-arrow-collapsed");
                info_revealer.borrow().set_reveal_child(false);
            }
            handle.borrow_mut().show_info = !expanded;
            gtk::Inhibit(true)
        });
    }

    pub fn widget(&self) -> gtk::EventBox {
        self.row.clone()
    }
}