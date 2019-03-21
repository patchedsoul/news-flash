mod single;
mod models;
mod article_row;

use gtk::{
    Builder,
    StackExt,
    StackTransitionType,
    Continue,
};
use single::SingleArticleList;
pub use models::ArticleListModel;
use models::ArticleListChangeSet;
use crate::main_window_state::MainWindowState;
use crate::main_window::MainWindow;
use crate::content_page::HeaderSelection;
use std::str;
use failure::Error;
use failure::format_err;
use crate::Resources;
use std::rc::Rc;
use std::cell::RefCell;
use crate::util::GtkHandle;
use crate::gtk_handle;

pub enum CurrentList {
    List1,
    List2,
}

pub struct ArticleList {
    stack: gtk::Stack,
    list_1: GtkHandle<SingleArticleList>,
    list_2: GtkHandle<SingleArticleList>,
    list_model: GtkHandle<ArticleListModel>,
    window_state: MainWindowState,
    current_list: CurrentList,
}

impl ArticleList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_list.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let stack : gtk::Stack = builder.get_object("article_list_stack").ok_or(format_err!("some err"))?;

        let list_1 = SingleArticleList::new()?;
        let list_2 = SingleArticleList::new()?;

        let window_state = MainWindow::initial_state();
        let model = ArticleListModel::new(&window_state.article_list_order);

        stack.add_named(&list_1.widget(), "list_1");
        stack.add_named(&list_2.widget(), "list_2");

        Ok(ArticleList {
            stack: stack,
            list_1: gtk_handle!(list_1),
            list_2: gtk_handle!(list_2),
            list_model: gtk_handle!(model),
            window_state: window_state,
            current_list: CurrentList::List1,
        })
    }

    pub fn widget(&self) -> gtk::Stack {
        self.stack.clone()
    }

    pub fn new_list(&mut self, mut new_list: ArticleListModel) {
        let current_list = match self.current_list {
            CurrentList::List1 => CurrentList::List2,
            CurrentList::List2 => CurrentList::List1,
        };
        self.current_list = current_list;
        let mut empty_model = ArticleListModel::new(&self.window_state.article_list_order);
        let diff = empty_model.generate_diff(&mut new_list);

        self.execute_diff(diff);
        self.switch_lists();

        self.list_model = gtk_handle!(new_list);
    }

    pub fn update(&mut self, mut new_list: ArticleListModel, new_state: MainWindowState) {
        self.stack.set_transition_type(self.calc_transition_type(&new_state));

        // check if a new list is reqired or current list should be updated
        if self.require_new_list(&new_state) {
            self.new_list(new_list);
            self.window_state = new_state;
            return
        }
        
        let old_list = self.list_model.clone();
        let mut old_list = old_list.borrow_mut();
        let list_diff = old_list.generate_diff(&mut new_list);
        self.execute_diff(list_diff);

        self.list_model = gtk_handle!(new_list);
        self.window_state = new_state;
    }

    fn execute_diff(&mut self, diff: Vec<ArticleListChangeSet>) {
        let list = match self.current_list {
            CurrentList::List1 => &mut self.list_1,
            CurrentList::List2 => &mut self.list_2,
        };

        for diff in diff {
            match diff {
                ArticleListChangeSet::Add(article, pos) => {
                    list.borrow_mut().add(article, pos);
                },
                ArticleListChangeSet::Remove(id) => {
                    list.borrow_mut().remove(id.clone());
                },
                ArticleListChangeSet::UpdateMarked(id, marked) => {
                    list.borrow_mut().update_marked(id.clone(), marked);
                },
                ArticleListChangeSet::UpdateRead(id, read) => {
                    list.borrow_mut().update_read(id.clone(), read);
                },
            }
        }
    }

    fn switch_lists(&self) {
        match self.current_list {
            CurrentList::List1 => self.stack.set_visible_child_name("list_1"),
            CurrentList::List2 => self.stack.set_visible_child_name("list_2"),
        }

        let old_list = match self.current_list {
            CurrentList::List1 => self.list_2.clone(),
            CurrentList::List2 => self.list_1.clone(),
        };

        gtk::timeout_add(100, move || {
            old_list.borrow_mut().clear();
            Continue(false)
        });
    }

    fn require_new_list(&self, new_state: &MainWindowState) -> bool {
        if &self.window_state == new_state {
            return false
        }
        true
    }

    fn calc_transition_type(&self, new_state: &MainWindowState) -> StackTransitionType {
        if self.require_new_list(new_state) {
            match self.window_state.header {
                HeaderSelection::All => {
                    match new_state.header {
                        HeaderSelection::All => {},
                        HeaderSelection::Unread |
                        HeaderSelection::Marked => return StackTransitionType::SlideLeft,
                    }
                },
                HeaderSelection::Unread => {
                    match new_state.header {
                        HeaderSelection::All  => return StackTransitionType::SlideRight,
                        HeaderSelection::Unread => {},
                        HeaderSelection::Marked => return StackTransitionType::SlideLeft,
                    }
                },
                HeaderSelection::Marked => {
                    match new_state.header {
                        HeaderSelection::All |
                        HeaderSelection::Unread => return StackTransitionType::SlideRight,
                        HeaderSelection::Marked => {},
                    }
                },
            }
        }
        StackTransitionType::Crossfade
    }
}