use crate::app::Action;
use crate::content_page::ContentPage;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::FeedListDndAction;
use crate::undo_bar::UndoActionModel;
use crate::util::{GtkHandle, GtkUtil, Util};
use gio::{ActionMapExt, SimpleAction};
use glib::futures::FutureExt;
use glib::{Sender, VariantTy};
use gtk::{self, ApplicationWindow};
use log::{error, info, warn};
use news_flash::models::{CategoryID, FeedID};
use news_flash::NewsFlash;

pub struct MainWindowActions;

impl MainWindowActions {
    pub fn setup_delete_selection_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        content_page: &GtkHandle<ContentPage>,
    ) {
        let content_page = content_page.clone();
        let sender = sender.clone();
        let delete_selection_action = SimpleAction::new("delete-selection", None);
        delete_selection_action.connect_activate(move |_action, _data| {
            let selection = content_page.borrow().sidebar_get_selection();
            let undo_action = match selection {
                SidebarSelection::All => {
                    warn!("Trying to delete item while 'All Articles' is selected");
                    None
                }
                SidebarSelection::Feed((feed_id, label)) => Some(UndoActionModel::DeleteFeed((feed_id, label))),
                SidebarSelection::Cateogry((category_id, label)) => {
                    Some(UndoActionModel::DeleteCategory((category_id, label)))
                }
                SidebarSelection::Tag((tag_id, label)) => Some(UndoActionModel::DeleteTag((tag_id, label))),
            };
            if let Some(undo_action) = undo_action {
                Util::send(&sender, Action::UndoableAction(undo_action));
            }
        });
        delete_selection_action.set_enabled(true);
        window.add_action(&delete_selection_action);
    }

    pub fn setup_delete_feed_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let sender = sender.clone();
        let delete_feed_action = SimpleAction::new("delete-feed", VariantTy::new("s").ok());
        delete_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let feed_id = FeedID::new(&data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let (feeds, _mappings) = match news_flash.get_feeds() {
                            Ok(res) => res,
                            Err(error) => {
                                Util::send(&sender, Action::Error("Failed to delete feed.".to_owned(), error));
                                return;
                            }
                        };

                        if let Some(feed) = feeds.iter().find(|f| f.feed_id == feed_id).cloned() {
                            info!("delete feed '{}' (id: {})", feed.label, feed.feed_id);
                            let future = news_flash.remove_feed(&feed).map(|remove_result| {
                                if let Err(error) = remove_result {
                                    Util::send(&sender, Action::Error("Failed to delete feed.".to_owned(), error));
                                }
                            });
                            GtkUtil::block_on_future(future);
                        } else {
                            let message = format!("Failed to delete feed: feed with id '{}' not found.", feed_id);
                            Util::send(&sender, Action::ErrorSimpleMessage(message));
                            error!("feed not found: {}", feed_id);
                        }
                    }
                }
            }
        });
        delete_feed_action.set_enabled(true);
        window.add_action(&delete_feed_action);
    }

    pub fn setup_delete_category_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let sender = sender.clone();
        let delete_feed_action = SimpleAction::new("delete-category", VariantTy::new("s").ok());
        delete_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let category_id = CategoryID::new(&data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let categories = match news_flash.get_categories() {
                            Ok(res) => res,
                            Err(error) => {
                                Util::send(&sender, Action::Error("Failed to delete category.".to_owned(), error));
                                return;
                            }
                        };

                        if let Some(category) = categories.iter().find(|c| c.category_id == category_id).cloned() {
                            info!("delete category '{}' (id: {})", category.label, category.category_id);
                            let future = news_flash.remove_category(&category, true).map(|remove_result| {
                                if let Err(error) = remove_result {
                                    Util::send(&sender, Action::Error("Failed to delete category.".to_owned(), error));
                                }
                            });
                            // FIXME
                            GtkUtil::block_on_future(future);
                        } else {
                            let message = format!(
                                "Failed to delete category: category with id '{}' not found.",
                                category_id
                            );
                            Util::send(&sender, Action::ErrorSimpleMessage(message));
                            error!("category not found: {}", category_id);
                        }
                    }
                }
            }
        });
        delete_feed_action.set_enabled(true);
        window.add_action(&delete_feed_action);
    }

    pub fn setup_move_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let sender = sender.clone();
        let move_action = SimpleAction::new("move", VariantTy::new("s").ok());
        move_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let info: FeedListDndAction = serde_json::from_str(&data).expect("Invalid FeedListDndAction");
                    match info {
                        FeedListDndAction::MoveCategory(category_id, parent_id, _sort_index) => {
                            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                                let future = news_flash.move_category(&category_id, &parent_id).map(|move_result| {
                                    if let Err(error) = move_result {
                                        Util::send(
                                            &sender,
                                            Action::Error("Failed to move category.".to_owned(), error),
                                        );
                                    }
                                });
                                // FIXME
                                GtkUtil::block_on_future(future);
                            }
                        }
                        FeedListDndAction::MoveFeed(feed_id, from_id, to_id, _sort_index) => {
                            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                                let future = news_flash.move_feed(&feed_id, &from_id, &to_id).map(|move_result| {
                                    if let Err(error) = move_result {
                                        Util::send(&sender, Action::Error("Failed to move feed.".to_owned(), error));
                                    }
                                });
                                // FIXME
                                GtkUtil::block_on_future(future);
                            }
                        }
                    }
                    Util::send(&sender, Action::UpdateSidebar);
                }
            }
        });
        move_action.set_enabled(true);
        window.add_action(&move_action);
    }

    pub fn setup_select_next_article_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let next_article_action = SimpleAction::new("next-article", None);
        next_article_action.connect_activate(move |_action, _data| {
            content_page.borrow().select_next_article();
        });
        next_article_action.set_enabled(true);
        window.add_action(&next_article_action);
    }

    pub fn setup_select_prev_article_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let prev_article_action = SimpleAction::new("prev-article", None);
        prev_article_action.connect_activate(move |_action, _data| {
            content_page.borrow().select_prev_article();
        });
        prev_article_action.set_enabled(true);
        window.add_action(&prev_article_action);
    }
}
