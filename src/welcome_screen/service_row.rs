use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use failure::Error;
use gdk::{EventType, NotifyType};
use gtk::{
    self, BinExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, Revealer, RevealerExt, StyleContextExt,
    WidgetExt,
};
use news_flash::models::{PluginIcon, PluginInfo, ServiceLicense, ServicePrice, ServiceType};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ServiceRow {
    row: EventBox,
    arrow_revealer: GtkHandle<Revealer>,
    arrow_event: GtkHandle<EventBox>,
    arrow_image: GtkHandle<Image>,
    info_revealer: GtkHandle<Revealer>,
    show_info: bool,
}

impl ServiceRow {
    pub fn new(info: PluginInfo) -> Result<Self, Error> {
        let builder = BuilderHelper::new("service_row");
        let row = builder.get::<EventBox>("service_row");
        let label = builder.get::<Label>("label");
        label.set_label(&info.name);
        let arrow_revealer = builder.get::<Revealer>("arrow_revealer");
        let info_revealer = builder.get::<Revealer>("info_revealer");

        // Website
        let website_label = builder.get::<Label>("website_label");
        if let Some(website) = info.website {
            let website_string = website.get().to_string();
            website_label.set_markup(&format!("<a href=\"{}\">{}</a>", website_string, website_string));
        } else {
            website_label.set_text("No Website");
        }

        let license_label = builder.get::<Label>("license_label");
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

        let type_label = builder.get::<Label>("type_label");
        let type_string = match info.service_type {
            ServiceType::Local => "Local data only",
            ServiceType::Remote { self_hosted } => {
                if self_hosted {
                    "Synced & self hosted"
                } else {
                    "Synced"
                }
            }
        };
        type_label.set_text(type_string);

        let price_label = builder.get::<Label>("price_label");
        let price_string = match info.service_price {
            ServicePrice::Free => "Free",
            ServicePrice::Paid => "Paid",
            ServicePrice::PaidPremimum => "Free / Paid Premium",
        };
        price_label.set_text(price_string);

        let arrow_event = builder.get::<EventBox>("arrow_event");
        arrow_event.connect_leave_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(0.8);
            gtk::Inhibit(false)
        });
        arrow_event.connect_enter_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(1.0);
            gtk::Inhibit(false)
        });

        let arrow_image = builder.get::<Image>("arrow_image");
        let scale = row.get_style_context().get_scale();

        let image = builder.get::<Image>("icon");
        if let Some(icon) = info.icon {
            let surface = match icon {
                PluginIcon::Vector(icon) => GtkUtil::create_surface_from_bytes(&icon.data, 64, 64, scale)?,
                PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(&icon, scale)?,
            };
            image.set_from_surface(Some(&surface));
        } else {
            // FIXME: default Icon
        }

        let service_row = ServiceRow {
            row,
            arrow_revealer: gtk_handle!(arrow_revealer),
            arrow_event: gtk_handle!(arrow_event),
            arrow_image: gtk_handle!(arrow_image),
            info_revealer: gtk_handle!(info_revealer),
            show_info: false,
        };
        let self_handle = gtk_handle!(service_row.clone());
        service_row.setup_events(self_handle);

        Ok(service_row)
    }

    fn setup_events(&self, handle: GtkHandle<ServiceRow>) {
        let arrow_revealer_1 = self.arrow_revealer.clone();
        let arrow_revealer_2 = self.arrow_revealer.clone();
        let handle_1 = handle.clone();
        self.row.connect_enter_notify_event(move |_widget, crossing| {
            if crossing.get_detail() != NotifyType::Inferior {
                arrow_revealer_1.borrow().set_reveal_child(true);
            }
            Inhibit(false)
        });
        self.row.connect_leave_notify_event(move |_widget, crossing| {
            if crossing.get_detail() != NotifyType::Inferior && !handle_1.borrow().show_info {
                arrow_revealer_2.borrow().set_reveal_child(false);
            }
            Inhibit(false)
        });

        let info_revealer = self.info_revealer.clone();
        self.arrow_event
            .borrow()
            .connect_button_press_event(move |widget, event| {
                if event.get_button() != 1 {
                    return gtk::Inhibit(false);
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return gtk::Inhibit(false),
                }
                let arrow_image = widget.get_child().unwrap();
                let context = arrow_image.get_style_context();
                let expanded = handle.borrow().show_info;
                if !expanded {
                    context.add_class("backward-arrow-expanded");
                    context.remove_class("backward-arrow-collapsed");
                    info_revealer.borrow().set_reveal_child(true);
                } else {
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
