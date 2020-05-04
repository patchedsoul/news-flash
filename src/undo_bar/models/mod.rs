use news_flash::models::{CategoryID, FeedID, TagID};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Hash, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum UndoActionModel {
    DeleteFeed(FeedID, String),
    DeleteCategory(CategoryID, String),
    DeleteTag(TagID, String),
}

impl fmt::Display for UndoActionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UndoActionModel::DeleteFeed(id, label) => write!(f, "Delete Feed '{}' (id: {})", label, id),
            UndoActionModel::DeleteCategory(id, label) => write!(f, "Delete Category '{}' (id: {})", label, id),
            UndoActionModel::DeleteTag(id, label) => write!(f, "Delete Tag '{}' (id: {})", label, id),
        }
    }
}

impl PartialEq for UndoActionModel {
    fn eq(&self, other: &Self) -> bool {
        match self {
            UndoActionModel::DeleteFeed(self_id, _self_title) => match other {
                UndoActionModel::DeleteFeed(other_id, __other_title) => self_id == other_id,
                UndoActionModel::DeleteCategory(_other_id, __other_title) => false,
                UndoActionModel::DeleteTag(_other_id, __other_title) => false,
            },
            UndoActionModel::DeleteCategory(self_id, _title) => match other {
                UndoActionModel::DeleteFeed(_other_id, __other_title) => false,
                UndoActionModel::DeleteCategory(other_id, __other_title) => self_id == other_id,
                UndoActionModel::DeleteTag(_other_id, __other_title) => false,
            },
            UndoActionModel::DeleteTag(self_id, _title) => match other {
                UndoActionModel::DeleteFeed(_other_id, __other_title) => false,
                UndoActionModel::DeleteCategory(_other_id, __other_title) => false,
                UndoActionModel::DeleteTag(other_id, __other_title) => self_id == other_id,
            },
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
