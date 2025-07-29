use chrono::{DateTime, Utc};

use crate::internal_prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentFile {
    pub path: PathBuf,
    pub date: Option<DateTime<Utc>>,
    pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub fn list_files_dir(dir: &impl AsRef<Path>) -> Result<Vec<ContentFile>> {
    utils::assert_dir_exists(dir);

    let dir = dir.as_ref();
    let mut res = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            res.push(utils::get_file_data(&path)?);
        }
    }

    Ok(res)
}

// traverse a directory recursively and list all files
pub fn list_files_dir_rec(dir: &impl AsRef<Path>) -> Result<Vec<ContentFile>> {
    utils::assert_dir_exists(dir);

    let mut dis = vec![dir.as_ref().to_path_buf()];
    let mut res = Vec::new();
    while let Some(current_dir) = dis.pop() {
        for entry in current_dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                res.push(utils::get_file_data(&path)?);
            } else if path.is_dir() {
                dis.push(path.canonicalize()?);
            }
        }
    }

    Ok(res)
}

mod utils {
    use super::*;

    pub fn get_file_data(path: &impl AsRef<Path>) -> Result<ContentFile> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(eyre!("Path does not exist: {:?}", path));
        }

        let file_type = FileType::from(path.to_path_buf());
        let date = path.metadata()?.modified().ok().map(|d| d.into());

        Ok(ContentFile {
            path: path.to_path_buf().canonicalize()?,
            date,
            file_type,
        })
    }

    pub enum PathKind {
        Path,
        Dir,
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
