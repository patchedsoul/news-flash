use crate::settings::Settings;
use crate::article_view::{ArticleView, ArticleTheme};
use crate::util::{BuilderHelper, GtkHandle};
use gtk::{Dialog, DialogExt, ListBox, ListBoxExt, GtkWindowExt, Window, WidgetExt};
use glib::{object::IsA};
use webkit2gtk::{WebView, WebViewExt};
use news_flash::models::{ArticleID, FeedID, FatArticle, Read, Marked};
use chrono::Utc;

pub struct ThemeChooser {
    widget: Dialog,
}

impl ThemeChooser {
    pub fn new<D: IsA<Window> + GtkWindowExt>(settings_dialog: &D, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("theme_chooser");

        let dialog = builder.get::<Dialog>("dialog");
        dialog.set_transient_for(settings_dialog);

        let mut demo_article = FatArticle {
            article_id: ArticleID::new("demo"),
            title: None,
            author: Some("Author".to_owned()),
            feed_id: FeedID::new("demo_feed"),
            direction: None,
            date: Utc::now().naive_utc(),
            marked: Marked::Unmarked,
            unread: Read::Unread,
            url: None,
            summary: None,
            html: Some("test123".to_owned()),
        };

        let default_view = builder.get::<WebView>("default_view");
        demo_article.title = Some("Default".to_owned());
        let default_html = ArticleView::build_article_static(&demo_article, "Feed Name", settings, Some(ArticleTheme::Default), Some(7000)).unwrap();
        default_view.load_html(&default_html, None);

        let spring_view = builder.get::<WebView>("spring_view");
        demo_article.title = Some("Spring".to_owned());
        let spring_html = ArticleView::build_article_static(&demo_article, "Feed Name", settings, Some(ArticleTheme::Spring), Some(7000)).unwrap();
        spring_view.load_html(&spring_html, None);

        let midnight_view = builder.get::<WebView>("midnight_view");
        demo_article.title = Some("Midnight".to_owned());
        let midnight_html = ArticleView::build_article_static(&demo_article, "Feed Name", settings, Some(ArticleTheme::Midnight), Some(7000)).unwrap();
        midnight_view.load_html(&midnight_html, None);
        
        let parchment_view = builder.get::<WebView>("parchment_view");
        demo_article.title = Some("Parchment".to_owned());
        let parchment_html = ArticleView::build_article_static(&demo_article, "Feed Name", settings, Some(ArticleTheme::Parchment), Some(7000)).unwrap();
        parchment_view.load_html(&parchment_html, None);

        let dialog_clone = dialog.clone();
        let settings = settings.clone();
        let theme_list = builder.get::<ListBox>("theme_list");
        theme_list.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "default" == row_name {
                    settings.borrow_mut().set_article_view_theme(ArticleTheme::Default).unwrap();
                } else if "spring" == row_name {
                    settings.borrow_mut().set_article_view_theme(ArticleTheme::Spring).unwrap();
                } else if "midnight" == row_name {
                    settings.borrow_mut().set_article_view_theme(ArticleTheme::Midnight).unwrap();
                } else if "parchment" == row_name {
                    settings.borrow_mut().set_article_view_theme(ArticleTheme::Parchment).unwrap();
                }
                dialog_clone.emit_close();
            }
        });

        ThemeChooser {
            widget: dialog,
        }
    }

    pub fn widget(&self) -> Dialog {
        self.widget.clone()
    }
}