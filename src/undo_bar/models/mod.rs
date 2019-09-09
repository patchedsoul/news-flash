use news_flash::models::{CategoryID, FeedID, TagID};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UndoActionModel {
    DeleteFeed((FeedID, String)),
    DeleteCategory((CategoryID, String)),
    DeleteTag((TagID, String)),
}

impl fmt::Display for UndoActionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UndoActionModel::DeleteFeed((id, label)) => write!(f, "Delete Feed '{}' (id: {})", label, id),
            UndoActionModel::DeleteCategory((id, label)) => write!(f, "Delete Category '{}' (id: {})", label, id),
            UndoActionModel::DeleteTag((id, label)) => write!(f, "Delete Tag '{}' (id: {})", label, id),
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