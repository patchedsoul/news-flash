use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SidebarSettings {
    only_unread: bool,
    only_feeds: bool,
}

impl SidebarSettings {
    pub fn default() -> Self {
        SidebarSettings {
            only_unread: false,
            only_feeds: false,
        }
    }
}