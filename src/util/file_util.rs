use super::error::{UtilError, UtilErrorKind};
use failure::ResultExt;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct FileUtil;

impl FileUtil {
    pub fn write_temp_file(file_name: &str, content: &str) -> Result<PathBuf, UtilError> {
        let temp_directory = env::temp_dir();
        let temp_file = temp_directory.join(file_name);

        let mut file = File::create(temp_file.clone()).context(UtilErrorKind::CreateFile)?;
        file.write_all(content.as_bytes()).context(UtilErrorKind::WriteFile)?;

        Ok(temp_file)
    }

    pub fn write_text_file(file_path: &PathBuf, content: &str) -> Result<(), UtilError> {
        let mut file = File::create(file_path).context(UtilErrorKind::CreateFile)?;
        file.write_all(content.as_bytes()).context(UtilErrorKind::WriteFile)?;
        Ok(())
    }

    pub fn read_text_file(file_path: &PathBuf) -> Result<String, UtilError> {
        let mut file = File::open(file_path).context(UtilErrorKind::OpenFile)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).context(UtilErrorKind::ReadFile)?;
        Ok(contents)
    }
}
