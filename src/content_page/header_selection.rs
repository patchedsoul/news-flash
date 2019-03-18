use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HeaderSelection {
    All,
    Unread,
    Marked,
}