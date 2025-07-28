use crate::internal_prelude::*;
use std::path::{Path, PathBuf};

pub fn list_files_dir(dir: &impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    utils::assert_dir_exists(dir);

    let dir = dir.as_ref();
    let mut res = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            res.push(path.canonicalize()?);
        }
    }

    Ok(res)
}

// traverse a directory recursively and list all files
pub fn list_files_dir_rec(dir: &impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    utils::assert_dir_exists(dir);

    let mut dis = vec![dir.as_ref().to_path_buf()];
    let mut res = Vec::new();
    while let Some(current_dir) = dis.pop() {
        for entry in current_dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                res.push(path);
            } else if path.is_dir() {
                dis.push(path.canonicalize()?);
            }
        }
    }

    Ok(res)
}

mod utils {
    use super::*;

    pub enum PathKind {
        Path,
        Dir,
    }

    pub fn assert_file_exists(path: &impl AsRef<Path>) {
        let path = path.as_ref();
        if !path.exists() {
            panic!("Path does not exist: {:?}", path);
        }
        if !path.is_file() {
            panic!("Expected a file, but found a directory: {:?}", path);
        }
    }

    pub fn assert_dir_exists(path: &impl AsRef<Path>) {
        let path = path.as_ref();
        if !path.exists() {
            panic!("Path does not exist: {:?}", path);
        }
        if !path.is_dir() {
            panic!("Expected a directory, but found a file: {:?}", path);
        }
    }
}
