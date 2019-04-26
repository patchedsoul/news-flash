use crate::util::BuilderHelper;
use failure::Error;
use gtk::HeaderBar;

#[derive(Clone, Debug)]
pub struct WelcomeHeaderbar {
    widget: gtk::HeaderBar,
}

impl WelcomeHeaderbar {
    pub fn new() -> Result<Self, Error> {
        let builder = BuilderHelper::new("welcome_headerbar");
        let headerbar = builder.get::<HeaderBar>("welcome_headerbar");

        Ok(WelcomeHeaderbar { widget: headerbar })
    }

    pub fn widget(&self) -> gtk::HeaderBar {
        self.widget.clone()
    }
}
