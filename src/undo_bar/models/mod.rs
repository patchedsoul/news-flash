use news_flash::models::{CategoryID, FeedID};
<<<<<<< HEAD
//use serde_derive::{Deserialize, Serialize};
=======
use serde_derive::{Deserialize, Serialize};
>>>>>>> de322131953ad3023ea39a9aabc55674c6ce4430

#[derive(Clone, Debug)]
pub enum UndoActionType {
    DeleteFeed(FeedID),
    DeleteCategory(CategoryID),
}
<<<<<<< HEAD

#[derive(Clone, Debug)]
pub struct UndoAction {
    action_type: UndoActionType,
    timeout: u32,
}

impl UndoAction {
    pub fn new(action: UndoActionType, timout: u32) -> Self {
        UndoAction {
            action_type: action,
            timeout: timout,
        }
    }

    pub fn get_type(&self) -> &UndoActionType {
        &self.action_type
    }

    pub fn get_timeout(&self) -> u32 {
        self.timeout
    }
}
=======
>>>>>>> de322131953ad3023ea39a9aabc55674c6ce4430
