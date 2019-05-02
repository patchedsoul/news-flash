use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SidebarSettings {
    only_unread: bool,
    only_feeds: bool,
}