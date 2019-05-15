mod builder_helper;
mod date_util;
mod error;
mod file_util;
mod gtk_util;

pub use builder_helper::BuilderHelper;
pub use date_util::DateUtil;
pub use file_util::FileUtil;
pub use gtk_util::GtkHandle;
pub use gtk_util::GtkHandleMap;
pub use gtk_util::GtkUtil;
pub use gtk_util::GTK_BUILDER_ERROR;
pub use gtk_util::GTK_CSS_ERROR;
pub use gtk_util::GTK_RESOURCE_FILE_ERROR;


pub struct Util;

impl Util {
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
}