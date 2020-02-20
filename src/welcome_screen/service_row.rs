use crate::i18n::i18n;
use crate::util::{BuilderHelper, GtkUtil};
use gdk::{EventType, NotifyType};
use glib::clone;
use gtk::{
    self, BinExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, Revealer, RevealerExt, StyleContextExt,
    WidgetExt,
};
use news_flash::models::{PluginIcon, PluginInfo, ServiceLicense, ServicePrice, ServiceType};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ServiceRow {
    row: EventBox,
    arrow_revealer: Arc<RwLock<Revealer>>,
    arrow_event: Arc<RwLock<EventBox>>,
    arrow_image: Arc<RwLock<Image>>,
    info_revealer: Arc<RwLock<Revealer>>,
    show_info: bool,
}

impl ServiceRow {
    pub fn new(info: PluginInfo) -> Self {
        let builder = BuilderHelper::new("service_row");
        let row = builder.get::<EventBox>("service_row");
        let label = builder.get::<Label>("label");
        label.set_label(&info.name);
        let arrow_revealer = builder.get::<Revealer>("arrow_revealer");
        let info_revealer = builder.get::<Revealer>("info_revealer");
        let arrow_event = builder.get::<EventBox>("arrow_event");
        let arrow_image = builder.get::<Image>("arrow_image");
        let image = builder.get::<Image>("icon");

        // Website
        let website_label = builder.get::<Label>("website_label");
        if let Some(website) = info.website {
            let website_string = website.get().to_string();
            website_label.set_markup(&format!("<a href=\"{}\">{}</a>", website_string, website_string));
        } else {
            website_label.set_text(&i18n("No Website"));
        }

        let license_label = builder.get::<Label>("license_label");
        let unknown = i18n("Unknown Free License");
        let sad_panda = i18n("Proprietary Software");
        let license_string = match info.license_type {
            ServiceLicense::ApacheV2 => "<a href=\"http://www.apache.org/licenses/LICENSE-2.0\">Apache v2</a>",
            ServiceLicense::GPLv2 => "<a href=\"http://www.gnu.de/documents/gpl-2.0.en.html\">GPLv2</a>",
            ServiceLicense::GPlv3 => "<a href=\"http://www.gnu.de/documents/gpl-3.0.en.html\">GPLv3</a>",
            ServiceLicense::MIT => "<a href=\"https://opensource.org/licenses/MIT\">MIT</a>",
            ServiceLicense::LGPLv21 => "<a href=\"http://www.gnu.de/documents/lgpl-2.1.en.html\">LGPLv2.1</a>",
            ServiceLicense::GenericFree => &unknown,
            ServiceLicense::GenericProprietary => &sad_panda,
        };
        license_label.set_markup(license_string);

        let type_label = builder.get::<Label>("type_label");
        let type_string = match info.service_type {
            ServiceType::Local => i18n("Local data only"),
            ServiceType::Remote { self_hosted } => {
                if self_hosted {
                    i18n("Synced & self hosted")
                } else {
                    i18n("Synced")
                }
            }
        };
        type_label.set_text(&type_string);

        let price_label = builder.get::<Label>("price_label");
        let price_string = match info.service_price {
            ServicePrice::Free => i18n("Free"),
            ServicePrice::Paid => i18n("Paid"),
            ServicePrice::PaidPremimum => i18n("Free / Paid Premium"),
        };
        price_label.set_text(&price_string);

        arrow_event.connect_leave_notify_event(clone!(@weak arrow_image => @default-panic, move |_widget, _| {
            arrow_image.set_opacity(0.8);
            gtk::Inhibit(false)
        }));
        arrow_event.connect_enter_notify_event(clone!(@weak arrow_image => @default-panic, move |_widget, _| {
            arrow_image.set_opacity(1.0);
            gtk::Inhibit(false)
        }));

        let scale = GtkUtil::get_scale(&row);
        let surface = if let Some(icon) = info.icon {
            match icon {
                PluginIcon::Vector(icon) => GtkUtil::create_surface_from_bytes(&icon.data, 64, 64, scale)
                    .expect("Failed to create surface from service vector icon."),
                PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(&icon, scale)
                    .expect("Failed to create surface from service pixel icon."),
            }
        } else {
            GtkUtil::create_surface_from_icon_name("feed-service-generic", 64, scale)
        };
        image.set_from_surface(Some(&surface));

        let service_row = ServiceRow {
            row,
            arrow_revealer: Arc::new(RwLock::new(arrow_revealer)),
            arrow_event: Arc::new(RwLock::new(arrow_event)),
            arrow_image: Arc::new(RwLock::new(arrow_image)),
            info_revealer: Arc::new(RwLock::new(info_revealer)),
            show_info: false,
        };
        let self_handle = Arc::new(RwLock::new(service_row.clone()));
        service_row.setup_events(self_handle);

        service_row
    }

    fn setup_events(&self, handle: Arc<RwLock<ServiceRow>>) {
        self.row.connect_enter_notify_event(clone!(
            @weak self.arrow_revealer as arrow_revealer => @default-panic, move |_widget, crossing| {
            if crossing.get_detail() != NotifyType::Inferior {
                arrow_revealer.read().set_reveal_child(true);
            }
            Inhibit(false)
        }));

        self.row.connect_leave_notify_event(clone!(
            @weak self.arrow_revealer as arrow_revealer,
            @weak handle => @default-panic, move |_widget, crossing| {
            if crossing.get_detail() != NotifyType::Inferior && !handle.read().show_info {
                arrow_revealer.write().set_reveal_child(false);
            }
            Inhibit(false)
        }));

        self.arrow_event.read().connect_button_press_event(clone!(
            @weak self.info_revealer as info_revealer => @default-panic, move |widget, event|
        {
            if event.get_button() != 1 {
                return gtk::Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            let arrow_image = widget.get_child().expect("arrow_image is not child of arrow_event");
            let context = arrow_image.get_style_context();
            let expanded = handle.read().show_info;
            if !expanded {
                context.add_class("backward-arrow-expanded");
                context.remove_class("backward-arrow-collapsed");
                info_revealer.read().set_reveal_child(true);
            } else {
                context.remove_class("backward-arrow-expanded");
                context.add_class("backward-arrow-collapsed");
                info_revealer.read().set_reveal_child(false);
            }
            handle.write().show_info = !expanded;
            gtk::Inhibit(true)
        }));
    }

    pub fn widget(&self) -> gtk::EventBox {
        self.row.clone()
    }
}
