use crate::add_dialog::{AddCategory, AddPopover};
use crate::app::Action;
use crate::article_view::ArticleView;
use crate::content_page::ContentPage;
use crate::gtk_handle;
use crate::rename_dialog::RenameDialog;
use crate::settings::Settings;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::FeedListDndAction;
use crate::undo_bar::UndoActionModel;
use crate::util::{FileUtil, GtkHandle, GtkUtil};
use gio::{ActionMapExt, SimpleAction};
use glib::futures::FutureExt;
use glib::{Sender, VariantTy};
use gtk::{
    self, ApplicationWindow, ButtonExt, DialogExt, FileChooserAction, FileChooserDialog, FileChooserExt, FileFilter,
    ResponseType,
};
use log::{error, info, warn};
use news_flash::models::{CategoryID, FeedID};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MainWindowActions;

impl MainWindowActions {
    pub fn setup_add_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content: &GtkHandle<ContentPage>,
    ) {
        let news_flash_handle = news_flash.clone();
        let sender = sender.clone();
        let add_button = content.borrow().sidebar_get_add_button();
        let add_action = SimpleAction::new("add-feed", None);
        add_action.connect_activate(move |_action, _data| {
            if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
                let news_flash_handle = news_flash_handle.clone();
                let error_message = "Failed to add feed".to_owned();

                let categories = match news_flash.get_categories() {
                    Ok(categories) => categories,
                    Err(error) => {
                        error!("{}", error_message);
                        GtkUtil::send(&sender, Action::Error(error_message.clone(), error));
                        return;
                    }
                };
                let dialog = AddPopover::new(&add_button, categories);
                let sender = sender.clone();
                dialog.add_button().connect_clicked(move |_button| {
                    let feed_url = match dialog.get_feed_url() {
                        Some(url) => url,
                        None => {
                            error!("{}: No valid url", error_message);
                            GtkUtil::send(&sender, Action::ErrorSimpleMessage(error_message.clone()));
                            return;
                        }
                    };
                    let feed_title = dialog.get_feed_title();
                    let feed_category = dialog.get_category();

                    if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
                        let category_id = match feed_category {
                            AddCategory::New(category_title) => {
                                let add_category_future = news_flash.add_category(&category_title, None, None);
                                let category = match GtkUtil::block_on_future(add_category_future) {
                                    Ok(category) => category,
                                    Err(error) => {
                                        error!("{}: Can't add Category", error_message);
                                        GtkUtil::send(&sender, Action::Error(error_message.clone(), error));
                                        return;
                                    }
                                };
                                Some(category.category_id)
                            }
                            AddCategory::Existing(category_id) => Some(category_id),
                            AddCategory::None => None,
                        };

                        let add_feed_future =
                            news_flash
                                .add_feed(&feed_url, feed_title, category_id)
                                .map(|result| match result {
                                    Ok(_) => {}
                                    Err(error) => {
                                        error!("{}: Can't add Feed", error_message);
                                        GtkUtil::send(&sender, Action::Error(error_message.clone(), error));
                                    }
                                });
                        GtkUtil::block_on_future(add_feed_future);
                    } else {
                        error!("{}: Can't borrow NewsFlash", error_message);
                        GtkUtil::send(&sender, Action::ErrorSimpleMessage(error_message.clone()));
                    }
                });
            }
        });
        add_action.set_enabled(true);
        window.add_action(&add_action);
    }

    pub fn setup_rename_feed_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let sender = sender.clone();
        let main_window = window.clone();
        let rename_feed_action = SimpleAction::new("rename-feed", VariantTy::new("s").ok());
        rename_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let feed_id = FeedID::new(&data);
                    let dialog_news_flash = news_flash.clone();
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let (feeds, _mappings) = match news_flash.get_feeds() {
                            Ok(result) => result,
                            Err(error) => {
                                let message = "Failed to laod list of feeds.".to_owned();
                                GtkUtil::send(&sender, Action::Error(message, error));
                                return;
                            }
                        };

                        let feed = match feeds.iter().find(|f| f.feed_id == feed_id).cloned() {
                            Some(feed) => feed,
                            None => {
                                let message = format!("Failed to find feed '{}'", feed_id);
                                GtkUtil::send(&sender, Action::ErrorSimpleMessage(message));
                                return;
                            }
                        };

                        let dialog =
                            RenameDialog::new(&main_window, &SidebarSelection::Feed((feed_id, feed.label.clone())));
                        let rename_button = dialog.rename_button();
                        let dialog_handle = gtk_handle!(dialog);
                        let sender = sender.clone();
                        rename_button.connect_clicked(move |_button| {
                            if let Some(news_flash) = dialog_news_flash.borrow_mut().as_mut() {
                                let new_label = match dialog_handle.borrow().new_label() {
                                    Some(label) => label,
                                    None => {
                                        GtkUtil::send(
                                            &sender,
                                            Action::ErrorSimpleMessage("No valid title to rename feed.".to_owned()),
                                        );
                                        dialog_handle.borrow().close();
                                        return;
                                    }
                                };

                                let future = news_flash.rename_feed(&feed, &new_label).map(|result| {
                                    if let Err(error) = result {
                                        GtkUtil::send(
                                            &sender,
                                            Action::Error("Failed to rename feed.".to_owned(), error),
                                        );
                                    }
                                });
                                GtkUtil::block_on_future(future);

                                dialog_handle.borrow().close();
                            }

                            GtkUtil::send(&sender, Action::UpdateArticleList);
                            GtkUtil::send(&sender, Action::UpdateSidebar);
                        });
                    }
                }
            }
        });
        rename_feed_action.set_enabled(true);
        window.add_action(&rename_feed_action);
    }

    pub fn setup_rename_category_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let sender = sender.clone();
        let main_window = window.clone();
        let rename_category_action = SimpleAction::new("rename-category", VariantTy::new("s").ok());
        rename_category_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let category_id = CategoryID::new(&data);
                    let dialog_news_flash = news_flash.clone();
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let categories = match news_flash.get_categories() {
                            Ok(categories) => categories,
                            Err(error) => {
                                let message = "Failed to load list of categories.".to_owned();
                                GtkUtil::send(&sender, Action::Error(message, error));
                                return;
                            }
                        };

                        let category = match categories.iter().find(|c| c.category_id == category_id).cloned() {
                            Some(category) => category,
                            None => {
                                let message = format!("Failed to find category '{}'", category_id);
                                GtkUtil::send(&sender, Action::ErrorSimpleMessage(message));
                                return;
                            }
                        };

                        let dialog = RenameDialog::new(
                            &main_window,
                            &SidebarSelection::Cateogry((category_id, category.label.clone())),
                        );

                        let rename_button = dialog.rename_button();
                        let dialog_handle = gtk_handle!(dialog);
                        let sender = sender.clone();
                        rename_button.connect_clicked(move |_button| {
                            if let Some(news_flash) = dialog_news_flash.borrow_mut().as_mut() {
                                let new_label = match dialog_handle.borrow().new_label() {
                                    Some(label) => label,
                                    None => {
                                        GtkUtil::send(
                                            &sender,
                                            Action::ErrorSimpleMessage("No valid title to rename category.".to_owned()),
                                        );
                                        return;
                                    }
                                };
                                let future = news_flash.rename_category(&category, &new_label).map(|result| {
                                    if let Err(error) = result {
                                        GtkUtil::send(
                                            &sender,
                                            Action::Error("Failed to rename category.".to_owned(), error),
                                        );
                                    }
                                });
                                GtkUtil::block_on_future(future);
                                dialog_handle.borrow().close();
                            }

                            GtkUtil::send(&sender, Action::UpdateSidebar);
                        });
                    }
                }
            }
        });
        rename_category_action.set_enabled(true);
        window.add_action(&rename_category_action);
    }

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
                GtkUtil::send(&sender, Action::UndoableAction(undo_action));
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
                                GtkUtil::send(&sender, Action::Error("Failed to delete feed.".to_owned(), error));
                                return;
                            }
                        };

                        if let Some(feed) = feeds.iter().find(|f| f.feed_id == feed_id).cloned() {
                            info!("delete feed '{}' (id: {})", feed.label, feed.feed_id);
                            let future = news_flash.remove_feed(&feed).map(|remove_result| {
                                if let Err(error) = remove_result {
                                    GtkUtil::send(&sender, Action::Error("Failed to delete feed.".to_owned(), error));
                                }
                            });
                            GtkUtil::block_on_future(future);
                        } else {
                            let message = format!("Failed to delete feed: feed with id '{}' not found.", feed_id);
                            GtkUtil::send(&sender, Action::ErrorSimpleMessage(message));
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
                                GtkUtil::send(&sender, Action::Error("Failed to delete category.".to_owned(), error));
                                return;
                            }
                        };

                        if let Some(category) = categories.iter().find(|c| c.category_id == category_id).cloned() {
                            info!("delete category '{}' (id: {})", category.label, category.category_id);
                            let future = news_flash.remove_category(&category, true).map(|remove_result| {
                                if let Err(error) = remove_result {
                                    GtkUtil::send(
                                        &sender,
                                        Action::Error("Failed to delete category.".to_owned(), error),
                                    );
                                }
                            });
                            // FIXME
                            GtkUtil::block_on_future(future);
                        } else {
                            let message = format!(
                                "Failed to delete category: category with id '{}' not found.",
                                category_id
                            );
                            GtkUtil::send(&sender, Action::ErrorSimpleMessage(message));
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
                                        GtkUtil::send(
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
                                        GtkUtil::send(&sender, Action::Error("Failed to move feed.".to_owned(), error));
                                    }
                                });
                                // FIXME
                                GtkUtil::block_on_future(future);
                            }
                        }
                    }
                    GtkUtil::send(&sender, Action::UpdateSidebar);
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

    pub fn setup_export_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let main_window = window.clone();
        let sender = sender.clone();
        let news_flash = news_flash.clone();
        let export_action = SimpleAction::new("export", None);
        export_action.connect_activate(move |_action, _data| {
            let dialog = FileChooserDialog::with_buttons(
                Some("Export OPML"),
                Some(&main_window),
                FileChooserAction::Save,
                &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
            );

            let filter = FileFilter::new();
            filter.add_pattern("*.OPML");
            filter.add_pattern("*.opml");
            filter.add_mime_type("application/xml");
            filter.add_mime_type("text/xml");
            filter.add_mime_type("text/x-opml");
            filter.set_name(Some("OPML"));
            dialog.add_filter(&filter);
            dialog.set_filter(&filter);
            dialog.set_current_name("NewsFlash.OPML");

            if let ResponseType::Ok = dialog.run() {
                if let Some(news_flash) = news_flash.borrow().as_ref() {
                    let opml = match news_flash.export_opml() {
                        Ok(opml) => opml,
                        Err(error) => {
                            GtkUtil::send(&sender, Action::Error("Failed to get OPML data.".to_owned(), error));
                            return;
                        }
                    };
                    if let Some(filename) = dialog.get_filename() {
                        if FileUtil::write_text_file(&filename, &opml).is_err() {
                            GtkUtil::send(
                                &sender,
                                Action::ErrorSimpleMessage("Failed to write OPML data to disc.".to_owned()),
                            );
                        }
                    }
                }
            }

            dialog.emit_close();
        });
        export_action.set_enabled(true);
        window.add_action(&export_action);
    }

    pub fn setup_export_article_action(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content_page: &GtkHandle<ContentPage>,
        settings: &Rc<RwLock<Settings>>,
    ) {
        let main_window = window.clone();
        let sender = sender.clone();
        let news_flash = news_flash.clone();
        let content_page = content_page.clone();
        let settings = settings.clone();
        let export_article_action = SimpleAction::new("export-article", None);
        export_article_action.connect_activate(move |_action, _data| {
            if let Some(article) = content_page.borrow().article_view_visible_article() {
                let dialog = FileChooserDialog::with_buttons(
                    Some("Export Article"),
                    Some(&main_window),
                    FileChooserAction::Save,
                    &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
                );

                let filter = FileFilter::new();
                filter.add_pattern("*.html");
                filter.add_mime_type("text/html");
                filter.set_name(Some("HTML"));
                dialog.add_filter(&filter);
                dialog.set_filter(&filter);
                if let Some(title) = &article.title {
                    dialog.set_current_name(&format!("{}.html", title));
                } else {
                    dialog.set_current_name("Article.html");
                }

                if let ResponseType::Ok = dialog.run() {
                    if let Some(news_flash) = news_flash.borrow().as_ref() {
                        let sender = sender.clone();
                        let settings = settings.clone();
                        let dialog_clone = dialog.clone();
                        let future =
                            news_flash
                                .article_download_images(&article.article_id)
                                .map(move |article_result| {
                                    let article = match article_result {
                                        Ok(article) => article,
                                        Err(error) => {
                                            GtkUtil::send(
                                                &sender,
                                                Action::Error("Failed to downlaod article images.".to_owned(), error),
                                            );
                                            return;
                                        }
                                    };

                                    let (feeds, _) = match news_flash.get_feeds() {
                                        Ok(opml) => opml,
                                        Err(error) => {
                                            GtkUtil::send(
                                                &sender,
                                                Action::Error("Failed to load feeds from db.".to_owned(), error),
                                            );
                                            return;
                                        }
                                    };
                                    let feed = match feeds.iter().find(|&f| f.feed_id == article.feed_id) {
                                        Some(feed) => feed,
                                        None => {
                                            GtkUtil::send(
                                                &sender,
                                                Action::ErrorSimpleMessage("Failed to find specific feed.".to_owned()),
                                            );
                                            return;
                                        }
                                    };
                                    if let Some(filename) = dialog_clone.get_filename() {
                                        let html = ArticleView::build_article_static(
                                            "article",
                                            &article,
                                            &feed.label,
                                            &settings,
                                            None,
                                            None,
                                        );
                                        if FileUtil::write_text_file(&filename, &html).is_err() {
                                            GtkUtil::send(
                                                &sender,
                                                Action::ErrorSimpleMessage(
                                                    "Failed to write OPML data to disc.".to_owned(),
                                                ),
                                            );
                                        }
                                    }
                                });
                        //FIXME
                        GtkUtil::block_on_future(future);
                    }
                }

                dialog.emit_close();
            }
        });
        export_article_action.set_enabled(true);
        window.add_action(&export_article_action);
    }
}
