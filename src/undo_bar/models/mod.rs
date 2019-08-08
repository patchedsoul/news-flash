use news_flash::models::{CategoryID, FeedID};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UndoActionModel {
    DeleteFeed((FeedID, String)),
    DeleteCategory((CategoryID, String)),
}

impl fmt::Display for UndoActionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UndoActionModel::DeleteFeed((id, label)) => write!(f, "Delete Feed '{}' (id: {})", id, label),
            UndoActionModel::DeleteCategory((id, label)) => write!(f, "Delete Category '{}' (id: {})", id, label),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UndoAction {
    action_model: UndoActionModel,
    timeout: u32,
}

impl UndoAction {
    pub fn new(action: UndoActionModel, timout: u32) -> Self {
        UndoAction {
            action_model: action,
            timeout: timout,
        }
    }

    pub fn get_model(&self) -> &UndoActionModel {
        &self.action_model
    }

    pub fn get_timeout(&self) -> u32 {
        self.timeout
    }
}
