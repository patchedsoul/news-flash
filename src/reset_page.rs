use crate::app::Action;
use crate::error_dialog::ErrorDialog;
use crate::util::{BuilderHelper, GtkUtil, Util};
use glib::{clone, translate::ToGlib, Sender};
use gtk::{Button, ButtonExt, InfoBar, InfoBarExt, ResponseType, Stack, StackExt, WidgetExt};
use news_flash::NewsFlashError;
use parking_lot::RwLock;

#[derive(Debug)]
pub struct ResetPage {
    reset_stack: Stack,
    reset_button: Button,
    info_bar: InfoBar,
    error_details_button: Button,
    error_details_signal: RwLock<Option<u64>>,
}

impl ResetPage {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let reset_button = builder.get::<Button>("reset_button");
        let cancel_button = builder.get::<Button>("cancel_button");
        let reset_stack = builder.get::<Stack>("reset_stack");
        let info_bar = builder.get::<InfoBar>("reset_info_bar");
        let error_details_button = builder.get::<Button>("details_button");

        reset_button.connect_clicked(clone!(@weak reset_stack, @strong sender => @default-panic, move |button| {
            reset_stack.set_visible_child_name("reset_spinner");
            button.set_sensitive(false);
            Util::send(&sender, Action::ResetAccount);
        }));

        cancel_button.connect_clicked(clone!(@strong sender => @default-panic, move |_button| {
            Util::send(&sender, Action::ShowContentPage(None));
        }));

        // setup infobar
        info_bar.connect_close(|info_bar| {
            info_bar.set_revealed(false);
        });
        info_bar.connect_response(|info_bar, response| {
            if let ResponseType::Close = response {
                info_bar.set_revealed(false);
            }
        });

        ResetPage {
            reset_stack,
            reset_button,
            info_bar,
            error_details_button,
            error_details_signal: RwLock::new(None),
        }
    }

    pub fn init(&self) {
        self.reset_stack.set_visible_child_name("reset_label");
        self.reset_button.set_sensitive(true);

        GtkUtil::disconnect_signal(*self.error_details_signal.read(), &self.error_details_button);
        self.error_details_signal.write().take();
    }

    pub fn error(&self, error: NewsFlashError) {
        self.init();

        self.error_details_signal.write().replace(
            self.error_details_button
                .connect_clicked(move |button| {
                    let parent = GtkUtil::get_main_window(button)
                        .expect("MainWindow is not a parent of password login error details button.");
                    let _dialog = ErrorDialog::new(&error, &parent);
                })
                .to_glib(),
        );
    }
}
