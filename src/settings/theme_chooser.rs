use crate::article_view::{ArticleTheme, ArticleView};
use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkHandle};
use chrono::Utc;
use failure::Error;
use glib::object::IsA;
use gtk::{Inhibit, ListBox, ListBoxExt, ListBoxRow, ListBoxRowExt, Popover, PopoverExt, Widget, WidgetExt};
use news_flash::models::{ArticleID, FatArticle, FeedID, Marked, Read};
use webkit2gtk::{WebView, WebViewExt};

pub struct ThemeChooser {
    widget: Popover,
}

impl ThemeChooser {
    pub fn new<D: IsA<Widget>>(parent: &D, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("theme_chooser");

        let pop = builder.get::<Popover>("popover");
        pop.set_relative_to(Some(parent));

        let mut demo_article = FatArticle {
            article_id: ArticleID::new("demo"),
            title: None,
            author: None,
            feed_id: FeedID::new("demo_feed"),
            direction: None,
            date: Utc::now().naive_utc(),
            marked: Marked::Unmarked,
            unread: Read::Unread,
            url: None,
            summary: None,
            html: None,
        };

        Self::prepare_theme_selection(
            &builder,
            settings,
            &mut demo_article,
            ArticleTheme::Default,
            "default",
            "Default",
        )
        .unwrap();
        Self::prepare_theme_selection(
            &builder,
            settings,
            &mut demo_article,
            ArticleTheme::Spring,
            "spring",
            "Spring",
        )
        .unwrap();
        Self::prepare_theme_selection(
            &builder,
            settings,
            &mut demo_article,
            ArticleTheme::Midnight,
            "midnight",
            "Midnight",
        )
        .unwrap();
        Self::prepare_theme_selection(
            &builder,
            settings,
            &mut demo_article,
            ArticleTheme::Parchment,
            "parchment",
            "Parchment",
        )
        .unwrap();

        let pop_clone = pop.clone();
        let settings = settings.clone();
        let theme_list = builder.get::<ListBox>("theme_list");
        theme_list.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "default" == row_name {
                    settings
                        .borrow_mut()
                        .set_article_view_theme(ArticleTheme::Default)
                        .unwrap();
                } else if "spring" == row_name {
                    settings
                        .borrow_mut()
                        .set_article_view_theme(ArticleTheme::Spring)
                        .unwrap();
                } else if "midnight" == row_name {
                    settings
                        .borrow_mut()
                        .set_article_view_theme(ArticleTheme::Midnight)
                        .unwrap();
                } else if "parchment" == row_name {
                    settings
                        .borrow_mut()
                        .set_article_view_theme(ArticleTheme::Parchment)
                        .unwrap();
                }
                pop_clone.popdown();
            }
        });

        ThemeChooser { widget: pop }
    }

    pub fn widget(&self) -> Popover {
        self.widget.clone()
    }

    fn prepare_theme_selection(
        builder: &BuilderHelper,
        settings: &GtkHandle<Settings>,
        article: &mut FatArticle,
        theme: ArticleTheme,
        id: &str,
        name: &str,
    ) -> Result<(), Error> {
        let view = builder.get::<WebView>(&format!("{}_view", id));
        let row = builder.get::<ListBoxRow>(&format!("{}_row", id));
        view.connect_button_press_event(move |_view, _event| {
            row.emit_activate();
            Inhibit(true)
        });
        article.title = Some(name.to_owned());
        let html = ArticleView::build_article_static(
            "theme_preview",
            article,
            "Feed Name",
            settings,
            Some(theme),
            Some(10240),
        )?;
        view.load_html(&html, None);
        Ok(())
    }
}
