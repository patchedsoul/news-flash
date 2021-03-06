mod article_row;
mod models;
mod single;

use crate::app::Action;
use crate::content_page::ContentHeader;
use crate::content_page::HeaderSelection;
use crate::i18n::{i18n, i18n_f};
use crate::main_window_state::MainWindowState;
use crate::settings::Settings;
use crate::sidebar::models::SidebarSelection;
use crate::util::{BuilderHelper, GtkUtil, Util};
use gdk::RGBA;
use glib::{clone, source::Continue, translate::ToGlib, Sender};
use gtk::{Label, LabelExt, ListBoxExt, ListBoxRowExt, ScrolledWindow, Stack, StackExt, StackTransitionType};
use models::ArticleListChangeSet;
pub use models::{ArticleListArticleModel, ArticleListModel, MarkUpdate, ReadUpdate};
use news_flash::models::{ArticleID, Marked, Read};
use parking_lot::RwLock;
use single::SingleArticleList;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrentList {
    List1,
    List2,
    Empty,
}

pub struct ArticleList {
    sender: Sender<Action>,
    stack: Stack,
    list_1: Arc<RwLock<SingleArticleList>>,
    list_2: Arc<RwLock<SingleArticleList>>,
    list_model: Arc<RwLock<ArticleListModel>>,
    list_activate_signal: Option<usize>,
    local_state: MainWindowState,
    global_state: Arc<RwLock<MainWindowState>>,
    current_list: Arc<RwLock<CurrentList>>,
    settings: Arc<RwLock<Settings>>,
    empty_label: Label,
}

impl ArticleList {
    pub fn new(
        settings: &Arc<RwLock<Settings>>,
        content_header: &Arc<ContentHeader>,
        global_state: &Arc<RwLock<MainWindowState>>,
        sender: Sender<Action>,
    ) -> Self {
        let builder = BuilderHelper::new("article_list");
        let stack = builder.get::<Stack>("article_list_stack");
        let empty_scroll = builder.get::<ScrolledWindow>("empty_scroll");
        let empty_label = builder.get::<Label>("empty_label");

        let list_1 = SingleArticleList::new(sender.clone(), content_header.clone());
        let list_2 = SingleArticleList::new(sender.clone(), content_header.clone());

        let local_state = MainWindowState::new();
        let model = ArticleListModel::new(&settings.read().get_article_list_order());

        stack.add_named(&list_1.widget(), "list_1");
        stack.add_named(&list_2.widget(), "list_2");
        stack.add_named(&empty_scroll, "empty");

        let settings = settings.clone();

        let mut article_list = ArticleList {
            sender,
            stack,
            list_1: Arc::new(RwLock::new(list_1)),
            list_2: Arc::new(RwLock::new(list_2)),
            list_model: Arc::new(RwLock::new(model)),
            list_activate_signal: None,
            local_state,
            global_state: global_state.clone(),
            current_list: Arc::new(RwLock::new(CurrentList::List1)),
            settings,
            empty_label,
        };

        article_list.setup_list_selected_singal();

        article_list
    }

    pub fn widget(&self) -> gtk::Stack {
        self.stack.clone()
    }

    pub fn get_background_color(&self) -> RGBA {
        self.list_1.read().get_background_color()
    }

    pub fn get_relevant_article_count(&self, header_selection: &HeaderSelection) -> usize {
        self.list_model.read().get_relevant_count(header_selection)
    }

    pub fn new_list(&mut self, mut new_list: ArticleListModel) {
        let current_list = match *self.current_list.read() {
            CurrentList::List1 => CurrentList::List2,
            CurrentList::List2 | CurrentList::Empty => CurrentList::List1,
        };
        *self.current_list.write() = current_list;
        let mut empty_model = ArticleListModel::new(&self.settings.read().get_article_list_order());
        let diff = empty_model.generate_diff(&mut new_list);

        self.execute_diff(diff);

        *self.list_model.write() = new_list;

        self.switch_lists();
    }

    pub fn update(&mut self, mut new_list: ArticleListModel, new_state: &Arc<RwLock<MainWindowState>>) {
        self.stack.set_transition_type(self.calc_transition_type(new_state));

        // check if list model is empty and display a message
        if new_list.len() == 0 {
            self.empty_label.set_label(&self.compose_empty_message(new_state));
            self.stack.set_visible_child_name("empty");
            if let Some(current_list) = self.get_current_list() {
                current_list.write().clear();
                GtkUtil::disconnect_signal(self.list_activate_signal, &current_list.read().list());
            }
            self.list_activate_signal = None;
            *self.current_list.write() = CurrentList::Empty;
            self.local_state = new_state.read().clone();
            return;
        }

        // check if a new list is reqired or current list should be updated
        if self.require_new_list(&new_state) {
            self.new_list(new_list);
            self.local_state = new_state.read().clone();
            return;
        }

        {
            // don't remove selected article during update
            // the `require_new_list` check above already ensures the old and new state are the same
            // so we add the selected model to the new_list so it wont be removed during the update
            if let Some(selected_article_model) = self.get_selected_article_model() {
                if !new_list.contains(&selected_article_model.id) {
                    let _result = new_list.add_model(selected_article_model);
                }
            }

            let old_list = self.list_model.clone();
            let mut old_list = old_list.write();
            let list_diff = old_list.generate_diff(&mut new_list);
            self.execute_diff(list_diff);
        }

        *self.list_model.write() = new_list;
        self.local_state = new_state.read().clone();
    }

    pub fn add_more_articles(&mut self, new_list: ArticleListModel) {
        let list = match *self.current_list.read() {
            CurrentList::List1 => &mut self.list_1,
            CurrentList::List2 => &mut self.list_2,
            CurrentList::Empty => return,
        };

        for model in new_list.models() {
            if self.list_model.write().add_model(model.clone()).is_ok() {
                gtk::idle_add(clone!(
                    @weak self.global_state as global_state,
                    @weak self.list_model as list_model,
                    @strong list,
                    @strong model => @default-panic, move ||
                {
                    list.write().add(&model, -1, &list_model, &global_state);
                    Continue(false)
                }));
            }
        }
    }

    fn execute_diff(&self, diff: Vec<ArticleListChangeSet>) {
        let list = match *self.current_list.read() {
            CurrentList::List1 | CurrentList::Empty => &self.list_1,
            CurrentList::List2 => &self.list_2,
        };

        for diff in diff {
            match diff {
                ArticleListChangeSet::Add(article, pos) => {
                    list.write().add(article, pos, &self.list_model, &self.global_state);
                }
                ArticleListChangeSet::Remove(id) => {
                    list.write().remove(id.clone());
                }
                ArticleListChangeSet::UpdateMarked(id, marked) => {
                    list.write().update_marked(&id, marked);
                }
                ArticleListChangeSet::UpdateRead(id, read) => {
                    list.write().update_read(&id, read);
                }
                ArticleListChangeSet::UpdateDateString(id, date) => {
                    list.write().update_date_string(&id, date);
                }
            }
        }
    }

    fn switch_lists(&mut self) {
        match *self.current_list.read() {
            CurrentList::Empty | CurrentList::List1 => self.stack.set_visible_child_name("list_1"),
            CurrentList::List2 => self.stack.set_visible_child_name("list_2"),
        }

        self.setup_list_selected_singal();

        let old_list = match *self.current_list.read() {
            CurrentList::List1 => Some(self.list_2.clone()),
            CurrentList::List2 => Some(self.list_1.clone()),
            CurrentList::Empty => None,
        };

        gtk::timeout_add(110, move || {
            if let Some(old_list) = &old_list {
                old_list.write().clear();
            }
            Continue(false)
        });
    }

    fn setup_list_selected_singal(&mut self) {
        let list_model = self.list_model.clone();
        let (new_list, old_list) = match *self.current_list.read() {
            CurrentList::List1 => (&self.list_1, &self.list_2),
            CurrentList::List2 => (&self.list_2, &self.list_1),
            CurrentList::Empty => return,
        };
        GtkUtil::disconnect_signal(self.list_activate_signal, &old_list.read().list());
        let activate_signal_id = new_list
            .read()
            .list()
            .connect_row_activated(clone!(
                @strong self.sender as sender,
                @weak self.global_state as global_state,
                @weak self.current_list as current_list,
                @weak self.list_1 as list_1,
                @weak self.list_2 as list_2 => @default-panic, move |_list, row|
            {
                let selected_index = row.get_index();
                let selected_article = list_model.write().calculate_selection(selected_index).cloned();
                if let Some(selected_article) = selected_article {
                    if selected_article.read == Read::Unread && !global_state.read().get_offline() {
                        let update = ReadUpdate {
                            article_id: selected_article.id.clone(),
                            read: Read::Read,
                        };
                        let list = match *current_list.read() {
                            CurrentList::List1 => list_1,
                            CurrentList::List2 => list_2,
                            CurrentList::Empty => return,
                        };
                        Self::set_article_state_static(&selected_article.id, Some(Read::Read), None, &list, &list_model);
                        Util::send(&sender, Action::MarkArticleRead(update));
                    }

                    Util::send(&sender, Action::ShowArticle(selected_article.id));
                }
            }))
            .to_glib();
        self.list_activate_signal = Some(activate_signal_id as usize);
    }

    fn require_new_list(&self, new_state: &RwLock<MainWindowState>) -> bool {
        if self.local_state == *new_state.read()
            && self.settings.read().get_article_list_order() == self.list_model.read().order()
            && *self.current_list.read() != CurrentList::Empty
        {
            return false;
        }
        true
    }

    fn calc_transition_type(&self, new_state: &Arc<RwLock<MainWindowState>>) -> StackTransitionType {
        if self.require_new_list(new_state) {
            match self.local_state.get_header_selection() {
                HeaderSelection::All => match new_state.read().get_header_selection() {
                    HeaderSelection::All => {}
                    HeaderSelection::Unread | HeaderSelection::Marked => return StackTransitionType::SlideLeft,
                },
                HeaderSelection::Unread => match new_state.read().get_header_selection() {
                    HeaderSelection::All => return StackTransitionType::SlideRight,
                    HeaderSelection::Unread => {}
                    HeaderSelection::Marked => return StackTransitionType::SlideLeft,
                },
                HeaderSelection::Marked => match new_state.read().get_header_selection() {
                    HeaderSelection::All | HeaderSelection::Unread => return StackTransitionType::SlideRight,
                    HeaderSelection::Marked => {}
                },
            }
        }
        StackTransitionType::Crossfade
    }

    fn compose_empty_message(&self, new_state: &RwLock<MainWindowState>) -> String {
        match new_state.read().get_sidebar_selection() {
            SidebarSelection::All => match new_state.read().get_header_selection() {
                HeaderSelection::All => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No articles that fit \"{}\"", &[&search]),
                    None => i18n("No articles"),
                },
                HeaderSelection::Unread => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No unread articles that fit \"{}\"", &[&search]),
                    None => i18n("No unread articles"),
                },
                HeaderSelection::Marked => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No starred articles that fit \"{}\"", &[&search]),
                    None => i18n("No starred articles"),
                },
            },
            SidebarSelection::Category(_id, title) => match new_state.read().get_header_selection() {
                HeaderSelection::All => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No articles that fit \"{}\" in category \"{}\"", &[&search, &title]),
                    None => i18n_f("No articles in category \"{}\"", &[&title]),
                },
                HeaderSelection::Unread => match new_state.read().get_search_term() {
                    Some(search) => i18n_f(
                        "No unread articles that fit \"{}\" in category \"{}\"",
                        &[&search, &title],
                    ),
                    None => i18n_f("No unread articles in category \"{}\"", &[&title]),
                },
                HeaderSelection::Marked => match new_state.read().get_search_term() {
                    Some(search) => i18n_f(
                        "No starred articles that fit \"{}\" in category \"{}\"",
                        &[&search, &title],
                    ),
                    None => i18n_f("No starred articles in category \"{}\"", &[&title]),
                },
            },
            SidebarSelection::Feed(_id, _parent_id, title) => match new_state.read().get_header_selection() {
                HeaderSelection::All => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No articles that fit \"{}\" in feed \"{}\"", &[&search, &title]),
                    None => i18n_f("No articles in feed \"{}\"", &[&title]),
                },
                HeaderSelection::Unread => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No unread articles that fit \"{}\" in feed \"{}\"", &[&search, &title]),
                    None => i18n_f("No unread articles in feed \"{}\"", &[&title]),
                },
                HeaderSelection::Marked => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No starred articles that fit \"{}\" in feed \"{}\"", &[&search, &title]),
                    None => i18n_f("No starred articles in feed \"{}\"", &[&title]),
                },
            },
            SidebarSelection::Tag(_id, title) => match new_state.read().get_header_selection() {
                HeaderSelection::All => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No articles that fit \"{}\" in tag \"{}\"", &[&search, &title]),
                    None => i18n_f("No articles in tag \"{}\"", &[&title]),
                },
                HeaderSelection::Unread => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No unread articles that fit \"{}\" in tag \"{}\"", &[&search, &title]),
                    None => i18n_f("No unread articles in tag \"{}\"", &[&title]),
                },
                HeaderSelection::Marked => match new_state.read().get_search_term() {
                    Some(search) => i18n_f("No starred articles that fit \"{}\" in tag \"{}\"", &[&search, &title]),
                    None => i18n_f("No starred articles in tag \"{}\"", &[&title]),
                },
            },
        }
    }

    fn get_current_list(&self) -> Option<Arc<RwLock<SingleArticleList>>> {
        match *self.current_list.read() {
            CurrentList::List1 => Some(self.list_1.clone()),
            CurrentList::List2 => Some(self.list_2.clone()),
            CurrentList::Empty => None,
        }
    }

    pub fn select_next_article(&self) {
        self.select_article(1)
    }

    pub fn select_prev_article(&self) {
        self.select_article(-1)
    }

    fn select_article(&self, direction: i32) {
        if let Some(current_list) = self.get_current_list() {
            let selected_index = current_list.read().get_selected_index();
            if let Some(selected_index) = selected_index {
                let selected_row = self.list_model.write().calculate_selection(selected_index).cloned();
                let next_row = self
                    .list_model
                    .write()
                    .calculate_selection(selected_index + direction)
                    .cloned();

                if let Some(selected_row) = selected_row {
                    if let Some(next_row) = next_row {
                        current_list.read().select_after(&next_row.id, 300);
                        if let Some(height) = current_list.read().get_allocated_row_height(&selected_row.id) {
                            current_list.read().animate_scroll_diff(f64::from(direction * height));
                        }
                    }
                }
            } else {
                let first_row = self.list_model.write().first().cloned();

                if let Some(first_row) = first_row {
                    current_list.read().select_after(&first_row.id, 300);
                    current_list.read().animate_scroll_absolute(0.0);
                }
            }
        }
    }

    pub fn get_selected_article_model(&self) -> Option<ArticleListArticleModel> {
        if let Some(current_list) = self.get_current_list() {
            let selected_index = current_list.read().get_selected_index();
            if let Some(selected_index) = selected_index {
                let selected_row = self.list_model.write().calculate_selection(selected_index).cloned();

                if let Some(selected_row) = selected_row {
                    return Some(selected_row);
                }
            }
        }

        None
    }

    pub fn set_article_row_state(&self, article_id: &ArticleID, read: Option<Read>, marked: Option<Marked>) {
        if let Some(current_list) = self.get_current_list() {
            Self::set_article_state_static(article_id, read, marked, &current_list, &self.list_model);
        }
    }

    fn set_article_state_static(
        article_id: &ArticleID,
        read: Option<Read>,
        marked: Option<Marked>,
        current_list: &Arc<RwLock<SingleArticleList>>,
        model: &Arc<RwLock<ArticleListModel>>,
    ) {
        current_list.read().set_article_row_state(article_id, read, marked);
        if let Some(read) = read {
            model.write().set_read(article_id, read);
        }
        if let Some(marked) = marked {
            model.write().set_marked(article_id, marked);
        }
    }
}
