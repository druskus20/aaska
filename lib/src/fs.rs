use chrono::{DateTime, Utc};
use comrak::ComrakOptions;

use crate::internal_prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMeta {
    pub path: PathBuf,
    pub date: DateTime<Utc>,
    pub file_type: FileType,
}

//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct ContentFile {
//    pub path: PathBuf,
//    pub date: DateTime<Utc>,
//    pub file_type: FileType,
//    pub contents: FileContents,
//}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FileType {
    HTML,
    Markdown,
    PlainText,
    Unsupported,
}

impl From<PathBuf> for FileType {
    fn from(path: PathBuf) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => FileType::HTML,
            Some("md") => FileType::Markdown,
            Some("txt") => FileType::PlainText,
            _ => FileType::Unsupported,
        }
    }
}

pub fn list_files_dir(dir: &impl AsRef<Path>) -> Result<Vec<FileMeta>> {
    utils::assert_dir_exists(dir);

    let dir = dir.as_ref();
    let mut res = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            res.push(utils::get_file_meta(&path)?);
        }
    }

    Ok(res)
}

pub fn read_file(path: &impl AsRef<Path>) -> Result<String> {
    utils::assert_file_exists(path);
    let path = path.as_ref();
    std::fs::read_to_string(path).wrap_err_with(|| format!("Failed to read file: {:?}", path))
}

// traverse a directory recursively and list all files
pub fn list_files_dir_rec(dir: &impl AsRef<Path>) -> Result<Vec<FileMeta>> {
    utils::assert_dir_exists(dir);

    let mut dirs = vec![dir.as_ref().to_path_buf()];
    let mut files = Vec::new();
    while let Some(current_dir) = dirs.pop() {
        for entry in current_dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(utils::get_file_meta(&path)?);
            } else if path.is_dir() {
                dirs.push(path.canonicalize()?);
            }
        }
    }

    Ok(files)
}

mod utils {
    use super::*;

    pub fn get_file_meta(path: &impl AsRef<Path>) -> Result<FileMeta> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(eyre!("Path does not exist: {:?}", path));
        }

        let file_type = FileType::from(path.to_path_buf());
        let date: DateTime<Utc> = path
            .metadata()?
            .modified()
            .wrap_err(format!("Failed to get modified date for file: {path:?}"))?
            .into();
        let path = path.to_path_buf().canonicalize()?;

        Ok(FileMeta {
            date,
            file_type,
            path,
        })
    }

    pub fn assert_file_exists(path: &impl AsRef<Path>) {
        let path = path.as_ref();
        if !path.exists() {
            panic!("Path does not exist: {path:?}");
        }
        if !path.is_file() {
            panic!("Expected a file, but found a directory: {path:?}");
        }
    }

    pub fn assert_dir_exists(path: &impl AsRef<Path>) {
        let path = path.as_ref();
        if !path.exists() {
            panic!("Path does not exist: {path:?}");
        }
        if !path.is_dir() {
            panic!("Expected a directory, but found a file: {path:?}");
        }
    }
}
