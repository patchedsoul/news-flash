mod builder_helper;
pub mod constants;
mod date_util;
mod error;
mod file_util;
mod gtk_util;
mod stopwatch;

pub use builder_helper::BuilderHelper;
pub use date_util::DateUtil;
pub use file_util::FileUtil;
pub use gtk_util::GtkUtil;
pub use gtk_util::GTK_BUILDER_ERROR;
pub use gtk_util::GTK_CSS_ERROR;
pub use gtk_util::GTK_RESOURCE_FILE_ERROR;
pub use stopwatch::StopWatch;

use self::error::{UtilError, UtilErrorKind};
use crate::app::Action;
use crate::settings::{ProxyModel, ProxyProtocoll};
use failure::ResultExt;
use gio::{Cancellable, ProxyResolver, ProxyResolverExt};
use glib::Sender;
use lazy_static::lazy_static;
use news_flash::models::{Category, CategoryID, Feed, FeedID, FeedMapping};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::future::Future;

pub const CHANNEL_ERROR: &str = "Error sending message via glib channel";
pub const RUNTIME_ERROR: &str = "Error creating tokio runtime";

lazy_static! {
    pub static ref NEWSFLASH_UNCATEGORIZED: CategoryID = CategoryID::new("NEWSFLASH_UNCATEGORIZED");
}

pub struct Util;

impl Util {
    #[allow(dead_code)]
    pub fn serialize_and_save<T: Serialize>(object: &T, path: &str) -> Result<String, UtilError> {
        let data = serde_json::to_string_pretty(object).context(UtilErrorKind::Serde)?;
        fs::write(path, &data).context(UtilErrorKind::WriteFile)?;
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn load_and_deserialize<T: DeserializeOwned>(path: &str) -> Result<T, UtilError> {
        let json_data = fs::read_to_string(path).context(UtilErrorKind::OpenFile)?;
        let object: T = serde_json::from_str(&json_data).context(UtilErrorKind::Serde)?;
        Ok(object)
    }

    pub fn send(sender: &Sender<Action>, action: Action) {
        sender.send(action).expect(CHANNEL_ERROR);
    }

    pub fn glib_spawn_future<F: Future<Output = ()> + 'static>(future: F) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(future);
    }

    pub fn some_or_default<T>(option: Option<T>, default: T) -> T {
        match option {
            Some(value) => value,
            None => default,
        }
    }

    pub fn ease_out_cubic(p: f64) -> f64 {
        let p = p - 1.0;
        p * p * p + 1.0
    }

    pub fn calculate_item_count_for_category(
        category_id: &CategoryID,
        categories: &[Category],
        feed_mappings: &[FeedMapping],
        item_count_map: &HashMap<FeedID, i64>,
        pending_deleted_feeds: &HashSet<&FeedID>,
        pending_deleted_categories: &HashSet<&CategoryID>,
    ) -> i64 {
        let mut count = 0;

        count += feed_mappings
            .iter()
            .filter_map(|m| {
                if &m.category_id == category_id {
                    if pending_deleted_feeds.contains(&m.feed_id) {
                        return None;
                    }

                    item_count_map.get(&m.feed_id)
                } else {
                    None
                }
            })
            .sum::<i64>();

        count += categories
            .iter()
            .filter_map(|c| {
                if &c.parent_id == category_id {
                    if pending_deleted_categories.contains(&c.category_id) {
                        return None;
                    }

                    Some(Self::calculate_item_count_for_category(
                        &c.category_id,
                        categories,
                        feed_mappings,
                        item_count_map,
                        pending_deleted_feeds,
                        pending_deleted_categories,
                    ))
                } else {
                    None
                }
            })
            .sum::<i64>();

        count
    }

    pub fn create_mappings_for_uncategorized_feeds(feeds: &[Feed], mappings: &[FeedMapping]) -> Vec<FeedMapping> {
        let mut uncategorized_mappings = Vec::new();
        for feed in feeds {
            if !mappings.iter().any(|m| m.feed_id == feed.feed_id) {
                uncategorized_mappings.push(FeedMapping {
                    feed_id: feed.feed_id.clone(),
                    category_id: NEWSFLASH_UNCATEGORIZED.clone(),
                });
            }
        }

        uncategorized_mappings
    }

    pub fn discover_gnome_proxy() -> Vec<ProxyModel> {
        let mut proxy_vec = Vec::new();

        if let Some(proxy_resolver) = ProxyResolver::get_default() {
            let cancellable: Option<&Cancellable> = None;
            if let Ok(proxy_list) = proxy_resolver.lookup("http://example.com/", cancellable) {
                for http_proxy in proxy_list {
                    if http_proxy != "direct://" {
                        let url = http_proxy.as_str().to_owned();
                        log::info!("HTTP proxy: '{}'", url);
                        proxy_vec.push(ProxyModel {
                            protocoll: ProxyProtocoll::HTTP,
                            url,
                            user: None,
                            password: None,
                        });
                    }
                }
            }

            if let Ok(proxy_list) = proxy_resolver.lookup("https://example.com/", cancellable) {
                for https_proxy in proxy_list {
                    if https_proxy != "direct://" {
                        let url = https_proxy.as_str().to_owned();
                        log::info!("HTTPS proxy: '{}'", url);
                        proxy_vec.push(ProxyModel {
                            protocoll: ProxyProtocoll::HTTPS,
                            url,
                            user: None,
                            password: None,
                        });
                    }
                }
            }
        }

        proxy_vec
    }
}
