mod article_row;
mod models;
mod single;

use crate::content_page::HeaderSelection;
use crate::gtk_handle;
use crate::main_window_state::MainWindowState;
use crate::util::GtkHandle;
use crate::util::GtkUtil;
use crate::Resources;
use failure::Error;
use gio::{ActionExt, ActionMapExt};
use glib::{translate::ToGlib, Variant};
use gtk::{Builder, Continue, ListBoxExt, ListBoxRowExt, StackExt, StackTransitionType};
use models::ArticleListChangeSet;
pub use models::ArticleListModel;
use single::SingleArticleList;
use news_flash::models::Read;
use std::cell::RefCell;
use std::rc::Rc;
use std::str;
use crate::util::{GTK_RESOURCE_FILE_ERROR, GTK_BUILDER_ERROR};

pub enum CurrentList {
    List1,
    List2,
}

pub struct ArticleList {
    stack: gtk::Stack,
    list_1: GtkHandle<SingleArticleList>,
    list_2: GtkHandle<SingleArticleList>,
    list_model: GtkHandle<ArticleListModel>,
    list_select_signal: Option<u64>,
    window_state: MainWindowState,
    current_list: GtkHandle<CurrentList>,
}

impl ArticleList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_list.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);

        let builder = Builder::new_from_string(ui_string);

        let stack: gtk::Stack = builder.get_object("article_list_stack").expect(GTK_BUILDER_ERROR);

        let list_1 = SingleArticleList::new()?;
        let list_2 = SingleArticleList::new()?;

        let window_state = MainWindowState::new();
        let model = ArticleListModel::new(window_state.get_article_list_order());

        stack.add_named(&list_1.widget(), "list_1");
        stack.add_named(&list_2.widget(), "list_2");

        let mut article_list = ArticleList {
            stack,
            list_1: gtk_handle!(list_1),
            list_2: gtk_handle!(list_2),
            list_model: gtk_handle!(model),
            list_select_signal: None,
            window_state,
            current_list: gtk_handle!(CurrentList::List1),
        };

        article_list.setup_list_selected_singal();

        Ok(article_list)
    }

    pub fn widget(&self) -> gtk::Stack {
        self.stack.clone()
    }

    pub fn new_list(&mut self, mut new_list: ArticleListModel) {
        let current_list = match *self.current_list.borrow() {
            CurrentList::List1 => CurrentList::List2,
            CurrentList::List2 => CurrentList::List1,
        };
        *self.current_list.borrow_mut() = current_list;
        let mut empty_model = ArticleListModel::new(self.window_state.get_article_list_order());
        let diff = empty_model.generate_diff(&mut new_list);

        self.execute_diff(diff);

        *self.list_model.borrow_mut() = new_list;

        self.switch_lists();
    }

    pub fn update(&mut self, mut new_list: ArticleListModel, new_state: MainWindowState) {
        self.stack.set_transition_type(self.calc_transition_type(&new_state));

        // check if a new list is reqired or current list should be updated
        if self.require_new_list(&new_state) {
            self.new_list(new_list);
            self.window_state = new_state;
            return;
        }

        {
            let old_list = self.list_model.clone();
            let mut old_list = old_list.borrow_mut();
            let list_diff = old_list.generate_diff(&mut new_list);
            self.execute_diff(list_diff);
        }

        *self.list_model.borrow_mut() = new_list;
        self.window_state = new_state;
    }

    pub fn add_more_articles(&mut self, new_list: ArticleListModel) -> Result<(), Error> {
        let list = match *self.current_list.borrow() {
            CurrentList::List1 => &mut self.list_1,
            CurrentList::List2 => &mut self.list_2,
        };

        for model in new_list.models() {
            self.list_model.borrow_mut().add_model(model.clone())?;
            let model = model.clone();
            let list = list.clone();
            gtk::idle_add(move || {
                list.borrow_mut().add(&model, -1);
                Continue(false)
            });
        }

        Ok(())
    }

    fn execute_diff(&mut self, diff: Vec<ArticleListChangeSet>) {
        let list = match *self.current_list.borrow() {
            CurrentList::List1 => &mut self.list_1,
            CurrentList::List2 => &mut self.list_2,
        };

        for diff in diff {
            match diff {
                ArticleListChangeSet::Add(article, pos) => {
                    list.borrow_mut().add(article, pos);
                }
                ArticleListChangeSet::Remove(id) => {
                    list.borrow_mut().remove(id.clone());
                }
                ArticleListChangeSet::UpdateMarked(id, marked) => {
                    list.borrow_mut().update_marked(&id, marked);
                }
                ArticleListChangeSet::UpdateRead(id, read) => {
                    list.borrow_mut().update_read(&id, read);
                }
            }
        }
    }

    fn switch_lists(&mut self) {
        match *self.current_list.borrow() {
            CurrentList::List1 => self.stack.set_visible_child_name("list_1"),
            CurrentList::List2 => self.stack.set_visible_child_name("list_2"),
        }

        self.setup_list_selected_singal();

        let old_list = match *self.current_list.borrow() {
            CurrentList::List1 => self.list_2.clone(),
            CurrentList::List2 => self.list_1.clone(),
        };

        gtk::timeout_add(110, move || {
            old_list.borrow_mut().clear();
            Continue(false)
        });
    }

    fn setup_list_selected_singal(&mut self) {
        let list_model = self.list_model.clone();
        let (new_list, old_list) = match *self.current_list.borrow() {
            CurrentList::List1 => (&self.list_1, &self.list_2),
            CurrentList::List2 => (&self.list_2, &self.list_1),
        };
        GtkUtil::disconnect_signal(self.list_select_signal, &old_list.borrow().list());
        let current_list = self.current_list.clone();
        let list_1 = self.list_1.clone();
        let list_2 = self.list_2.clone();
        let select_signal_id = new_list
            .borrow()
            .list()
            .connect_row_selected(move |list, row| {
                if let Some(selected_row) = row {
                    let selected_index = selected_row.get_index();
                    let selected_article = list_model.borrow_mut().calculate_selection(selected_index).cloned();
                    if let Some(selected_article) = selected_article {
                        let selected_article_id = selected_article.id.clone();
                        if let Ok(main_window) = GtkUtil::get_main_window(list) {
                            let selected_article_id = Variant::from(&selected_article_id.to_str());
                            if let Some(action) = main_window.lookup_action("show-article") {
                                action.activate(Some(&selected_article_id));
                            }
                            if selected_article.unread == Read::Unread {
                                list_model.borrow_mut().set_read(&selected_article.id, Read::Read);
                                match *current_list.borrow() {
                                    CurrentList::List1 => list_1.borrow_mut().update_read(&selected_article.id, Read::Read),
                                    CurrentList::List2 => list_2.borrow_mut().update_read(&selected_article.id, Read::Read),
                                }
                                if let Some(action) = main_window.lookup_action("mark-article-read") {
                                    action.activate(Some(&selected_article_id));
                                }
                            }
                            
                        }
                    }
                }
            })
            .to_glib();
        self.list_select_signal = Some(select_signal_id);
    }

    fn require_new_list(&self, new_state: &MainWindowState) -> bool {
        if &self.window_state == new_state {
            return false;
        }
        true
    }

    fn calc_transition_type(&self, new_state: &MainWindowState) -> StackTransitionType {
        if self.require_new_list(new_state) {
            match self.window_state.get_header_selection() {
                HeaderSelection::All => match new_state.get_header_selection() {
                    HeaderSelection::All => {}
                    HeaderSelection::Unread | HeaderSelection::Marked => return StackTransitionType::SlideLeft,
                },
                HeaderSelection::Unread => match new_state.get_header_selection() {
                    HeaderSelection::All => return StackTransitionType::SlideRight,
                    HeaderSelection::Unread => {}
                    HeaderSelection::Marked => return StackTransitionType::SlideLeft,
                },
                HeaderSelection::Marked => match new_state.get_header_selection() {
                    HeaderSelection::All | HeaderSelection::Unread => return StackTransitionType::SlideRight,
                    HeaderSelection::Marked => {}
                },
            }
        }
        StackTransitionType::Crossfade
    }
}
