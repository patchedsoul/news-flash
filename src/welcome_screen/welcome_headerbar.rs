use crate::util::BuilderHelper;
use gtk::HeaderBar;

#[derive(Clone, Debug)]
pub struct WelcomeHeaderbar {
    widget: gtk::HeaderBar,
}

impl WelcomeHeaderbar {
    pub fn new(builder: &BuilderHelper) -> Self {
        let headerbar = builder.get::<HeaderBar>("welcome_headerbar");
        WelcomeHeaderbar { widget: headerbar }
    }
}
